use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};

#[cfg(feature = "experiment_lua_worker")]
use mlua::prelude::*;
#[cfg(feature = "experiment_lua_worker")]
use std::rc::Rc;

pub const DEFAULT_ORDERING: Ordering = Ordering::SeqCst;

pub struct AtomicInstant {
    base: Instant,
    offset: AtomicUsize,
}

impl AtomicInstant {
    /// Create a new AtomicInstant with the given base time.
    pub fn new(base: Instant) -> AtomicInstant {
        AtomicInstant {
            base,
            offset: AtomicUsize::new(0),
        }
    }

    /// Load the current time from the AtomicInstant.
    pub fn load(&self, order: Ordering) -> Instant {
        let offset_nanos = self.offset.load(order) as u64;
        let secs = offset_nanos / 1_000_000_000;
        let subsec_nanos = (offset_nanos % 1_000_000_000) as u32;
        let offset = Duration::new(secs, subsec_nanos);
        self.base + offset
    }

    /// Store the given time in the AtomicInstant.
    pub fn store(&self, val: Instant, order: Ordering) {
        let offset = val - self.base;
        let offset_nanos = offset.as_secs() * 1_000_000_000 + offset.subsec_nanos() as u64;
        self.offset.store(offset_nanos as usize, order);
    }
}

// NOTE: Because the mlua crate is not Sync, we can use tokio spawn_local to run the Lua VM in an async context
// but pinned to a single thread
//
// This is highly experimental

#[cfg(feature = "experiment_lua_worker")]
pub struct LuaWorkerManager {
    workers: dashmap::DashMap<u64, LuaWorker>,
    max_workers: u64,
}

#[cfg(feature = "experiment_lua_worker")]
impl LuaWorkerManager {
    /// Spawns a new LuaWorkerManager
    pub fn new(max_workers: u64) -> Self {
        LuaWorkerManager {
            workers: dashmap::DashMap::new(),
            max_workers
        }
    }

    /// Spawn all workers
    /// 
    /// NOTE: Calling mat prevent deadlocks
    pub fn spawn_all(&self) {
        for i in 0..self.max_workers {
            self.spawn_worker(i);
        }
    }

    /// Spawns a new LuaWorker given thread number to spawn
    pub fn spawn_worker(&self, tid: u64) {
        let mut worker = LuaWorker::new();
        worker.spawn();
        self.workers.insert(tid, worker);
    }

    /// Make a lua worker request given guild id
    /// 
    /// The thread number is calculated by the guild id modulo the number of workers
    pub async fn make_request(&self, guild_id: serenity::all::GuildId, request: LuaWorkerRequest) -> Result<LuaWorkerResponse, base_data::Error> {
        let tid = guild_id.get() % self.max_workers;
        match self.workers.get(&tid) {
            Some(worker) => {
                if let Some(ref thread) = worker.thread {
                    if thread.is_finished() {
                        self.spawn_worker(tid);
                        tokio::time::sleep(Duration::from_secs(1)).await;
                    }
                }

                let (tx, rx) = tokio::sync::oneshot::channel();

                let request = LuaWorkerFullRequest {
                    responder: tx,
                    request
                };
        
                worker.request_queue.tx.send(request).await.unwrap();
        
                rx.await.map_err(|e| format!("Failed to receive response: {}", e).into())        
            },
            None => {
                self.spawn_worker(tid);

                // Wait for 1 second to avoid deadlock
                tokio::time::sleep(Duration::from_secs(1)).await;
                
                let worker = match self.workers.get(&tid) {
                    Some(worker) => worker,
                    None => return Err("Failed to spawn worker".into())
                };

                let (tx, rx) = tokio::sync::oneshot::channel();

                let request = LuaWorkerFullRequest {
                    responder: tx,
                    request
                };
        
                worker.request_queue.tx.send(request).await.unwrap();
        
                rx.await.map_err(|e| format!("Failed to receive response: {}", e).into())        
            }
        }
    }
}

#[cfg(feature = "experiment_lua_worker")]
/// Wrapper around async-channels channel
pub struct AsyncChannel<T> {
    pub tx: async_channel::Sender<T>,
    pub rx: async_channel::Receiver<T>,
}

#[cfg(feature = "experiment_lua_worker")]
impl<T> AsyncChannel<T> {
    pub fn new() -> Self {
        let (tx, rx) = async_channel::unbounded();
        Self { tx, rx }
    }
}

#[cfg(feature = "experiment_lua_worker")]
pub struct ThreadLocalCache<K: core::hash::Hash + std::cmp::Eq + std::clone::Clone, V> {
    cache: dashmap::DashMap<K, ThreadLocalCacheEntry<V>>,
    time_to_idle: Duration,
}

#[cfg(feature = "experiment_lua_worker")]
struct ThreadLocalCacheEntry<V> {
    value: Rc<V>,
    last_access: Instant,
}

#[cfg(feature = "experiment_lua_worker")]
impl<K: core::hash::Hash + std::cmp::Eq + std::clone::Clone, V> ThreadLocalCache<K, V> {
    pub fn new(time_to_idle: Duration) -> Self {
        Self {
            cache: dashmap::DashMap::new(),
            time_to_idle,
        }
    }

    pub async fn get(&self, key: K) -> Option<std::rc::Rc<V>> {
        let now = Instant::now();
        let entry = self.cache.get(&key);

        let (new_entry, remove) = match entry {
            Some(ref entry) => {
                if now.duration_since(entry.value().last_access) > self.time_to_idle {
                    (None, true)
                } else {
                    let entry = entry.value();
                    (Some(ThreadLocalCacheEntry {
                        value: entry.value.clone(),
                        last_access: now,
                    }), false)
                }
            }
            None => (None, false),
        };

        drop(entry); // Avoid deadlock in dashmap.get

        if remove {
            self.cache.remove(&key);
        }

        new_entry.map(|e| e.value)
    }

    pub async fn insert(&self, key: K, value: V) -> Rc<V> {
        let now = Instant::now();
        let value = std::rc::Rc::new(value);
        self.cache.insert(key, ThreadLocalCacheEntry { value: value.clone(), last_access: now });

        value
    }

    pub async fn remove(&self, key: K) {
        self.cache.remove(&key);
    }

    pub async fn clear(&self) {
        self.cache.clear();
    }

    pub async fn len(&self) -> usize {
        self.cache.len()
    }

    pub async fn loop_and_clear_expired(&self, stopper: async_channel::Receiver<()>) {
        loop {
            tokio::select! {
                _ = stopper.recv() => {
                    break;
                }
                else => {
                    tokio::time::sleep(Duration::from_millis(500)).await;
                    let now = Instant::now();
                    let mut to_remove = Vec::new();
                    for r in self.cache.iter() {
                        let k = r.key().clone();
                        let v = r.value();
                        if now.duration_since(v.last_access) > self.time_to_idle {
                            to_remove.push(k);
                        }
                    }
        
                    for k in to_remove {
                        self.cache.remove(&k);
                    }        
                }
            };
        }
    }
}

#[cfg(feature = "experiment_lua_worker")]
pub struct LuaWorker {
    /// A handle that allows stopping the VM inside its tokio localset
    pub stopper: AsyncChannel<()>,
    request_queue: AsyncChannel<LuaWorkerFullRequest>,
    thread: Option<std::thread::JoinHandle<()>>,
}

#[cfg(feature = "experiment_lua_worker")]
impl Drop for LuaWorker {
    fn drop(&mut self) {
        self.stopper.tx.try_send(()).unwrap();
        log::info!("Dropping LuaWorker");
    }
}

#[cfg(feature = "experiment_lua_worker")]
impl LuaWorker {
    /// Compiles a Lua script in the Lua VM 
    pub async fn compile(
        vm: &Lua,
        template: &str,
    ) -> Result<(), base_data::Error> {
        vm
            .load(template)
            .eval_async()
            .await?;

        Ok(())
    }
    /// Executes a Lua script in the Lua VM
    pub async fn exec(
        vm: &Lua,
        template: &str,
        args: Box<dyn erased_serde::Serialize + Send>,
    ) -> Result<serde_json::Value, base_data::Error> {
        let f: LuaFunction = vm
            .load(template)
            .eval_async()
            .await?;

        log::info!("exec (done creating function f)");

        let args = vm
            .to_value(&args)?;

        let v: LuaValue = f
            .call_async(args)
            .await?;

        let v = serde_json::to_value(v)?;

        Ok(v)
    }

    pub fn new() -> Self {
        Self {
            stopper: AsyncChannel::new(),
            request_queue: AsyncChannel::new(),
            thread: None,
        }
    }

    pub fn spawn(&mut self) {
        log::debug!("Spawning LuaWorker");

        let stopper = self.stopper.rx.clone();
        let request_queue = self.request_queue.rx.clone();
        self.thread = Some(std::thread::Builder::new().name("lua-worker".to_string()).spawn(move || {
            let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

            let local_set = tokio::task::LocalSet::new();
            local_set.spawn_local(async move {
                let stopper = stopper.clone();
                let request_queue = request_queue.clone();
                let mut current_tasks: Vec<tokio::task::JoinHandle<()>> = Vec::new();
                let handling_guilds = Rc::<ThreadLocalCache<serenity::all::GuildId, super::ArLuaNonSend>>::new(ThreadLocalCache::new(super::MAX_TEMPLATE_LIFETIME));

                // Spawn a tokio task to expire dashmap entries with time_to_idle MAX_TEMPLATE_LIFETIME
                let _sl = handling_guilds.clone();
                let _st = stopper.clone();
                tokio::task::spawn_local(async move {
                    let handling_guilds = _sl.clone();
                    let stopper = _st.clone();
                    handling_guilds.loop_and_clear_expired(stopper).await;
                });

                loop {
                    let handling_guilds = handling_guilds.clone();
                    tokio::select! {
                        _ = stopper.recv() => {
                            for task in current_tasks.iter() {
                                task.abort();
                            }

                            // Close stopper
                            stopper.close();
                            break;
                        }
                        msg = request_queue.recv() => {
                            log::info!("Received request");
                            /*let msg = match msg {
                                Ok(msg) => msg,
                                Err(_) => {
                                    for task in current_tasks.iter() {
                                        task.abort();
                                    }
                                    stopper.close();
                                    break;
                                },
                            };*/
                            let msg = msg.unwrap();
                            let jh = tokio::task::spawn_local(async move {
                                log::debug!("Executing request");

                                let handling_guilds = handling_guilds.clone();

                                match msg.request {
                                    LuaWorkerRequest::Template { guild_id, template, args } => {
                                        let vm = match handling_guilds.get(guild_id).await {
                                            Some(vm) => vm.clone(),
                                            None => {
                                                let vm = match super::create_lua_vm_nonsend().await {
                                                    Ok(vm) => vm,
                                                    Err(e) => {
                                                        let _ = msg.responder.send(LuaWorkerResponse::Err(e.to_string()));
                                                        return;
                                                    }
                                                };
                                                handling_guilds.insert(guild_id, vm).await
                                            },
                                        };

                                        log::debug!("Executing template");

                                        match Self::exec(&vm.vm, &template, args).await {
                                            Ok(v) => {
                                                log::debug!("Compiled template, returning to sender");
                                                let _ = msg.responder.send(LuaWorkerResponse::Ok(v));
                                            },
                                            Err(e) => {
                                                log::debug!("Compiled template, returning to sender");
                                                let _ = msg.responder.send(LuaWorkerResponse::Err(e.to_string()));
                                            }
                                        }
                                    },
                                    LuaWorkerRequest::Compile { guild_id, template } => {
                                        let vm = match handling_guilds.get(guild_id).await {
                                            Some(vm) => vm.clone(),
                                            None => {
                                                let vm = match super::create_lua_vm_nonsend().await {
                                                    Ok(vm) => vm,
                                                    Err(e) => {
                                                        let _ = msg.responder.send(LuaWorkerResponse::Err(e.to_string()));
                                                        return;
                                                    }
                                                };
                                                handling_guilds.insert(guild_id, vm).await
                                            },
                                        };

                                        match Self::compile(&vm.vm, &template).await {
                                            Ok(()) => {
                                                log::debug!("Compiled template, returning to sender");
                                                let _ = msg.responder.send(LuaWorkerResponse::Ok(serde_json::Value::Null)); // No need to send anything
                                            },
                                            Err(e) => {
                                                log::debug!("Compiled template, returning to sender");
                                                let _ = msg.responder.send(LuaWorkerResponse::Err(e.to_string()));
                                            }
                                        }
                                    } 
                                };
                            });
                            current_tasks.push(jh);
                        }
                    }
                }
            });

            rt.block_on(local_set);
        }).unwrap());
    }
}

#[cfg(feature = "experiment_lua_worker")]
struct LuaWorkerFullRequest {
    responder: tokio::sync::oneshot::Sender<LuaWorkerResponse>,
    request: LuaWorkerRequest,
}

#[cfg(feature = "experiment_lua_worker")]
pub enum LuaWorkerRequest {
    /// Compiles a Lua template
    Compile {
        guild_id: serenity::all::GuildId,
        template: String,
    },
    /// Execute a Lua template
    Template {
        guild_id: serenity::all::GuildId,
        template: String,
        args: Box<dyn erased_serde::Serialize + Send>,
    }
}

pub type LuaWorkerResponse = Result<serde_json::Value, String>;