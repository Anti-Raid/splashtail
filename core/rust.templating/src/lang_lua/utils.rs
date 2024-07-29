use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

#[cfg(feature = "experiment_lua_worker")]
use mlua::prelude::*;
#[cfg(feature = "experiment_lua_worker")]
use tokio::sync::Mutex;

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

pub struct LuaWorkerManager {
    pub workers: dashmap::DashMap<usize, LuaWorker>,
}

impl LuaWorkerManager {
    /// Spawns a new LuaWorkerManager
    pub fn new() -> Self {
        let manager = LuaWorkerManager {
            workers: dashmap::DashMap::new(),
        };

        manager
    }

    /// Spawns a new LuaWorker given thread number to spawn
    fn spawn_worker(&self, tid: usize) {}
}

/// Wrapper around async-channels channel
pub struct AsyncChannel<T> {
    pub tx: async_channel::Sender<T>,
    pub rx: async_channel::Receiver<T>,
}

impl<T> AsyncChannel<T> {
    pub fn new() -> Self {
        let (tx, rx) = async_channel::unbounded();
        Self { tx, rx }
    }
}

#[cfg(feature = "experiment_lua_worker")]
pub struct LuaWorker {
    /// A handle that allows stopping the VM inside its tokio localset
    pub stopper: AsyncChannel<()>,
    pub request_queue: AsyncChannel<LuaWorkerRequest>,
    pub response_queue: AsyncChannel<LuaWorkerResponse>,
    pub open: Arc<AtomicBool>,
}

impl Drop for LuaWorker {
    fn drop(&mut self) {
        self.stopper.tx.try_send(()).unwrap();
    }
}

#[cfg(feature = "experiment_lua_worker")]
impl LuaWorker {
    // Executes a Lua script in the Lua VM
    pub async fn exec(
        vm: &Lua,
        template: &str,
        args: serde_json::Value,
    ) -> Result<serde_json::Value, base_data::Error> {
        let f: LuaFunction = vm
            .load(template)
            .eval_async()
            .await
            .map_err(|e| LuaError::external(e.to_string()))?;

        let _args = vm
            .create_table()
            .map_err(|e| LuaError::external(e.to_string()))?;

        let args = vm
            .to_value(&args)
            .map_err(|e| LuaError::external(e.to_string()))?;

        _args
            .set("args", args)
            .map_err(|e| LuaError::external(e.to_string()))?;

        let v: LuaValue = f
            .call_async(_args)
            .await
            .map_err(|e| LuaError::external(e.to_string()))?;

        let v = serde_json::to_value(v).map_err(|e| LuaError::external(e.to_string()))?;

        Ok(v)
    }

    pub fn new() -> Self {
        Self {
            stopper: AsyncChannel::new(),
            request_queue: AsyncChannel::new(),
            response_queue: AsyncChannel::new(),
            open: Arc::new(AtomicBool::new(true)),
        }
    }

    pub fn spawn(&self) {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        let stopper = self.stopper.rx.clone();
        let request_queue = self.request_queue.rx.clone();
        let response_queue = self.response_queue.tx.clone();
        let open = self.open.clone();
        std::thread::spawn(move || {
            rt.block_on(async move {
                let stopper = stopper.clone();
                let request_queue = request_queue.clone();
                let open = open.clone();
                let mut current_tasks: Vec<tokio::task::JoinHandle<()>> = Vec::new();
                let handling_guilds = std::rc::Rc::new(
                    tokio::sync::Mutex::new(
                    mini_moka::unsync::Cache::<serenity::all::GuildId, super::ArLuaNonSend>::builder()
                        .time_to_idle(super::MAX_TEMPLATE_LIFETIME).build()
                    )
                );

                loop {
                    let handling_guilds = handling_guilds.clone();
                    let response_queue = response_queue.clone();
                    tokio::select! {
                        _ = stopper.recv() => {
                            for task in current_tasks.iter() {
                                task.abort();
                            }

                            open.store(false, DEFAULT_ORDERING);

                            break;
                        }
                        msg = request_queue.recv() => {
                            let msg = match msg {
                                Ok(msg) => msg,
                                Err(_) => {
                                    for task in current_tasks.iter() {
                                        task.abort();
                                    }
                                    open.store(false, DEFAULT_ORDERING);
                                    break;
                                },
                            };
                            let jh = tokio::task::spawn_local(async move {
                                let handling_guilds = handling_guilds.clone();
                                let response_queue = response_queue.clone();

                                match msg {
                                    LuaWorkerRequest::Template { guild_id, template, args } => {
                                        // Get a lock
                                        let mut vmg = handling_guilds.lock().await;
                                        let vm = match (*vmg).get(&guild_id) {
                                            Some(vm) => vm.clone(),
                                            None => {
                                                let vm = match super::create_lua_vm_nonsend().await {
                                                    Ok(vm) => vm,
                                                    Err(e) => {
                                                        let _ = response_queue.send(LuaWorkerResponse::Err(e.to_string())).await;
                                                        return;
                                                    }
                                                };
                                                (*vmg).insert(guild_id, vm.clone());
                                                vm
                                            },
                                        };

                                        drop(vmg);

                                        match Self::exec(&vm.vm, &template, args).await {
                                            Ok(v) => {
                                                let _ = response_queue.send(LuaWorkerResponse::Ok(v)).await;
                                            },
                                            Err(e) => {
                                                let _ = response_queue.send(LuaWorkerResponse::Err(e.to_string())).await;
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
        });
    }
}

#[cfg(feature = "experiment_lua_worker")]
pub enum LuaWorkerRequest {
    /// Execute a Lua template
    Template {
        guild_id: serenity::all::GuildId,
        template: String,
        args: serde_json::Value,
    }
}

#[cfg(feature = "experiment_lua_worker")]
pub enum LuaWorkerResponse {
    Ok(serde_json::Value),
    Err(String),
}
