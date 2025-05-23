# Templating Architecture

## Distribution of VMs

A *vm distribution strategy* defines how guild VMs should be distributed out. There are two strategies right now: thread per guild and threadpool. With thread per guild, each guild is given its own thread and with threadpool, a set of guilds share a thread. Thread per guild is considered legacy and internally uses thread pool itself now (it used to be its own strategy). A third strategy of process pooling is being considered for addition in the future but more work is needed in the sandwich daemon layer (and other layers) for process pooling to become a reality. When either strategy encounters a panic, a per thread panic hook is triggered/fired to clean up all current guild templatingrt handles so future dispatches to a guild start a fresh new guild vm on a different (non-panicking) thread.

The actual contents of a vm distribution strategy is simple, do any setup needed (to init guild state struct, tokio runtime and tis_ref, which is the khronos runtime manager). Then run a loop listening on a action channel for LuaVmActions. When a action is recieved with its associated callback channel, simply call the handle method on LuaVmAction which will handle the execution layer. For example, the below code shows how thread per guild vm distribution strategy was implemented prior to its merge with threadpool:

```rust
use super::client::ArLua;
use super::client::{ArLuaHandle, LuaVmResult, VMS};
use super::core::create_guild_state;
use crate::templatingrt::vm_manager::client::LuaVmAction;
use crate::templatingrt::vm_manager::core::configure_runtime_manager;
use crate::templatingrt::MAX_VM_THREAD_STACK_SIZE;
use serenity::all::GuildId;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::{Ordering, AtomicUsize};

pub static NUM_THREADS: AtomicUsize = AtomicUsize::new(0);

#[allow(dead_code)]
pub async fn create_lua_vm(
    guild_id: GuildId,
    pool: sqlx::PgPool,
    serenity_context: serenity::all::Context,
    reqwest_client: reqwest::Client,
    object_store: Arc<silverpelt::objectstore::ObjectStore>
) -> Result<ArLua, silverpelt::Error> {
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<(
        LuaVmAction,
        tokio::sync::oneshot::Sender<Vec<(String, LuaVmResult)>>,
    )>();

    std::thread::Builder::new()
        .name(format!("lua-vm-{}", guild_id))
        .stack_size(MAX_VM_THREAD_STACK_SIZE)
        .spawn(move || {
            NUM_THREADS.fetch_add(1, Ordering::SeqCst);

            super::perthreadpanichook::set_hook(Box::new(move |_| {
                NUM_THREADS.fetch_sub(1, Ordering::SeqCst);
                super::remove_vm(guild_id);
            }));

            let gs = Rc::new(
                create_guild_state(guild_id, pool, serenity_context, reqwest_client, object_store)
                    .expect("Failed to create Lua VM userdata"),
            );

            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("Failed to create tokio runtime");

            let local = tokio::task::LocalSet::new();

            local.block_on(&rt, async {
                let tis_ref = match configure_runtime_manager() {
                    Ok(tis) => tis,
                    Err(e) => {
                        log::error!("Failed to configure Lua VM: {}", e);
                        panic!("Failed to configure Lua VM");
                    }
                };

                tis_ref.set_on_broken(Box::new(move || {
                    super::remove_vm(guild_id);
                }));

                while let Some((action, callback)) = rx.recv().await {
                    let tis_ref = tis_ref.clone();
                    let gs = gs.clone();

                    local.spawn_local(async move {
                        action.handle(tis_ref, gs, callback).await
                    });
                }
            });
        })?;

    Ok(ArLua::ThreadPerGuild(PerThreadLuaHandle { handle: tx }))
}

#[derive(Clone)]
pub struct PerThreadLuaHandle {
    #[allow(clippy::type_complexity)]
    /// The thread handle for the Lua VM
    pub handle: tokio::sync::mpsc::UnboundedSender<(
        LuaVmAction,
        tokio::sync::oneshot::Sender<Vec<(String, LuaVmResult)>>,
    )>,
}

impl ArLuaHandle for PerThreadLuaHandle {
    fn send_action(
        &self,
        action: LuaVmAction,
        callback: tokio::sync::oneshot::Sender<Vec<(String, LuaVmResult)>>,
    ) -> Result<(), khronos_runtime::Error> {
        self.handle.send((action, callback))?;
        Ok(())
    }
}
```

## VM Notes

Each lua vm is owned by a single struct called a KhronosRuntime (which is provided by the ``khronos`` runtime). It is considered a logic bug to attempt to clone the lua vm handle. Instead, a WeakLua (weak ref to lua vm handle) should be used with the resulting lua vm handle being held for as little time as possible in all uses.