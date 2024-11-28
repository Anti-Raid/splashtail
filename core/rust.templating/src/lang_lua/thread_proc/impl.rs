use crate::{
    handle_event, ArLuaThreadInnerState, LuaVmAction, LuaVmResult, MAX_VM_THREAD_STACK_SIZE,
};
use serenity::all::GuildId;
use std::{panic::PanicHookInfo, sync::Arc};

pub fn lua_thread_impl(
    thread_inner_state: Arc<ArLuaThreadInnerState>,
    guild_id: GuildId,
) -> Result<
    tokio::sync::mpsc::UnboundedSender<(LuaVmAction, tokio::sync::oneshot::Sender<LuaVmResult>)>,
    std::io::Error,
> {
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<(
        LuaVmAction,
        tokio::sync::oneshot::Sender<LuaVmResult>,
    )>();

    std::thread::Builder::new()
        .name(format!("lua-vm-{}", guild_id))
        .stack_size(MAX_VM_THREAD_STACK_SIZE)
        .spawn(move || {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();

            let local = tokio::task::LocalSet::new();

            let tis_ref = thread_inner_state.clone();
            local.block_on(&rt, async {
                // Catch panics
                fn panic_catcher(
                    guild_id: GuildId,
                    broken_ref: Arc<std::sync::atomic::AtomicBool>,
                ) -> Box<dyn Fn(&PanicHookInfo<'_>) + 'static + Sync + Send> {
                    Box::new(move |_| {
                        log::error!("Lua thread panicked: {}", guild_id);
                        broken_ref.store(true, std::sync::atomic::Ordering::Release);
                    })
                }

                super::perthreadpanichook::set_hook(panic_catcher(
                    guild_id,
                    tis_ref.broken.clone(),
                ));

                while let Some((action, callback)) = rx.recv().await {
                    let tis_ref = tis_ref.clone();
                    local.spawn_local(async move {
                        let result = handle_event(action, &tis_ref).await;

                        let _ = callback.send(result);
                    });
                }
            });
        })?;

    Ok(tx)
}
