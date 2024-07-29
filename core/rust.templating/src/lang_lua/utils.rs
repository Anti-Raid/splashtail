use mlua::prelude::*;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use std::time::{Duration, Instant};
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
pub struct LuaWorker {
    /// The inner lua vm
    pub lua: Mutex<Lua>,
    /// A handle that allows stopping the VM inside its tokio localset
    ///
    /// This is wrapped in an option to allow destroying the handle when the LuaWorker is dropped
    pub tx_stop: Option<tokio::sync::oneshot::Sender<()>>,
    /// A channel used for sending requests to the VM
    pub rx: tokio::sync::mpsc::Sender<Arc<LuaWorkerRequest>>,
}

impl Drop for LuaWorker {
    fn drop(&mut self) {
        if let Some(sender) = self.tx_stop.take() {
            let _ = sender.send(());
        }
    }
}

pub struct LuaWorkerRequest {
    pub template: String,
    pub args: serde_json::Value,
}
