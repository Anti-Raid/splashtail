use std::sync::{
    atomic::{AtomicUsize, Ordering},
};
use std::time::{Duration, Instant};

#[cfg(feature = "experiment_lua_worker")]
use tokio::sync::Mutex;
#[cfg(feature = "experiment_lua_worker")]
use mlua::prelude::*;

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
pub struct LuaWorker {
    /// A handle that allows stopping the VM inside its tokio localset
    ///
    /// This is wrapped in an option to allow destroying the handle when the LuaWorker is dropped
    pub tx_stop: Option<tokio::sync::oneshot::Sender<()>>,
    /// A channel used for sending requests to the VM
    pub tx_msg_recv: tokio::sync::broadcast::Sender<LuaWorkerRequest>,
    /// A channel that can be used to listen for a response from the VM
    pub rx_msg_resp: tokio::sync::broadcast::Receiver<LuaWorkerResponse>,
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

    // Spawn a new LuaWorker thread
    pub async fn spawn(&self) {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        std::thread::spawn(move || {
            let local = tokio::task::LocalSet::new();

            let _ = local.spawn_local(async move {
                let vms = std::rc::Rc::new(mini_moka::unsync::Cache::<serenity::all::GuildId, super::ArLua>::builder()
                    .time_to_idle(super::MAX_TEMPLATE_LIFETIME)
                    .build());

                let mut rx_stop = rx_stop;
                let mut rx_msg_recv = rx_msg_recv;
                let mut tx_msg_resp = tx_msg_resp;

                loop {
                    let tx_msg_resp = tx_msg_resp.clone();

                    tokio::select! {
                        /*_ = rx_stop => {
                            break;
                        },*/
                        Some(msg) = rx_msg_recv.recv() => {
                            tokio::task::spawn_local(async move {
                                let vms = vms.clone();
                                let lua = match vms.get(msg.guild_id) {
                                    Some(vm) => Ok(vm),
                                    None => {
                                        let vm = match super::create_lua_vm().await {
                                            Ok(vm) => vm,
                                            Err(e) => {
                                                let _ = tx_msg_resp.send(LuaWorkerResponse::Err(e.to_string())).await;
                                                return;
                                            }
                                        };
                                        vms.insert(msg.guild_id, vm.clone());
                                        Ok(vm)
                                    }
                                };                            


                                let res = LuaWorker::exec(&lua, &msg.template, msg.args).await;
                                let _ = tx_msg_resp.send(match res {
                                    Ok(v) => LuaWorkerResponse::Ok(v),
                                    Err(e) => LuaWorkerResponse::Err(e.to_string()),
                                }).await;
                            });
                        }
                    }
                }
            });

            rt.block_on(local);
        });
    }

    // Spawns a new LuaWorker with the given Lua VM
    pub fn new(lua: Lua) -> Self {
        let (tx_stop, rx_stop) = tokio::sync::oneshot::channel();
        let (tx_msg_recv, rx_msg_recv) = tokio::sync::mpsc::channel(32);
        let (tx_msg_resp, rx_msg_resp) = tokio::sync::mpsc::channel(32);

        let worker = LuaWorker {
            tx_stop: Some(tx_stop),
            tx_msg_recv,
            rx_msg_resp,
        };

        worker
    }
}

#[cfg(feature = "experiment_lua_worker")]
impl Drop for LuaWorker {
    fn drop(&mut self) {
        if let Some(sender) = self.tx_stop.take() {
            let _ = sender.send(());
        }
    }
}

#[cfg(feature = "experiment_lua_worker")]
pub struct LuaWorkerRequest {
    pub guild_id: serenity::all::GuildId,
    pub template: String,
    pub args: serde_json::Value,
}

#[cfg(feature = "experiment_lua_worker")]
pub enum LuaWorkerResponse {
    Ok(serde_json::Value),
    Err(String),
}
