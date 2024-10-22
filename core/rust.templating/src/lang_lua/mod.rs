mod perthreadpanichook;
mod plugins;
pub(crate) mod state;

use crate::atomicinstant;
use mlua::prelude::*;
use moka::future::Cache;
use serenity::all::GuildId;
use std::panic::PanicHookInfo;
use std::sync::Arc;
use std::sync::LazyLock;

static VMS: LazyLock<Cache<GuildId, ArLua>> =
    LazyLock::new(|| Cache::builder().time_to_idle(MAX_TEMPLATE_LIFETIME).build());

pub const MAX_TEMPLATE_MEMORY_USAGE: usize = 1024 * 1024 * 3; // 3MB maximum memory
pub const MAX_VM_THREAD_STACK_SIZE: usize = 1024 * 1024 * 4; // 4MB maximum memory
pub const MAX_TEMPLATE_LIFETIME: std::time::Duration = std::time::Duration::from_secs(60 * 15); // 15 minutes maximum lifetime
pub const MAX_TEMPLATES_EXECUTION_TIME: std::time::Duration = std::time::Duration::from_secs(30); // 30 seconds maximum execution time

struct LoadLuaTemplate {
    template: String,
    pragma: crate::TemplatePragma,
    args: Option<serde_json::Value>,
    callback: tokio::sync::oneshot::Sender<Result<serde_json::Value, LuaError>>,
}

#[derive(Clone)]
struct ArLua {
    /// The Lua VM. The VM is wrapped in an async aware Mutex to ensure it is safe to use across await points
    #[allow(dead_code)]
    vm: Lua,
    /// The last execution time of the Lua VM
    last_execution_time: Arc<atomicinstant::AtomicInstant>,
    /// The thread handle for the Lua VM
    thread_handle: (
        std::thread::Thread,
        tokio::sync::mpsc::UnboundedSender<LoadLuaTemplate>,
    ),
    /// Is the VM broken/needs to be remade
    broken: Arc<std::sync::atomic::AtomicBool>,
}

/// Create a new Lua VM complete with sandboxing and modules pre-loaded
///
/// Note that callers should instead call the render_template functions
///
/// As such, this function is private and should not be used outside of this module
async fn create_lua_vm(
    guild_id: GuildId,
    pool: sqlx::PgPool,
    cache_http: botox::cache::CacheHttpImpl,
    reqwest_client: reqwest::Client,
) -> LuaResult<ArLua> {
    let lua = Lua::new_with(
        LuaStdLib::ALL_SAFE,
        LuaOptions::new().catch_rust_panics(true),
    )?;

    let compiler = mlua::Compiler::new()
        .set_optimization_level(2)
        .set_type_info_level(1);
    lua.set_compiler(compiler);

    // Prelude code providing some basic functions directly to the Lua VM
    lua.load(
        r#"
        -- Override print function with function that appends to __stack.stdout table
        -- We do this by executing a lua script
        _G.print = function(...)
            local args = {...}

            if not _G.__stack then
                error("No __stack found")
            end

            if not _G.__stack.stdout then
                _G.__stack.stdout = {}
            end

            if #args == 0 then
                table.insert(__stack.stdout, "nil")
            end

            local str = ""
            for i = 1, #args do
                str = str .. tostring(args[i])
            end
            table.insert(__stack.stdout, str)
        end

        -- Set AntiRaid version
        _G.ANTIRAID_VER = "1"

        -- To allow locking down _G, we need to create a table to store user data (__stack)
        -- Note: this becomes read-write later and is the ONLY global variable that is read-write
        _G.__stack = {}
    "#,
    )
    .set_name("prelude")
    .exec()?;

    lua.sandbox(true)?; // We explicitly want globals to be shared across all scripts in this VM
    lua.set_memory_limit(MAX_TEMPLATE_MEMORY_USAGE)?;

    // Make __stack read-write
    let stack = lua.globals().get::<LuaTable>("__stack")?;
    stack.set_readonly(false);

    // Override require function for plugin support and increased security
    lua.globals()
        .set("require", lua.create_function(plugins::require)?)?;

    let last_execution_time =
        Arc::new(atomicinstant::AtomicInstant::new(std::time::Instant::now()));

    let last_execution_time_interrupt_ref = last_execution_time.clone();

    // Create an interrupt to limit the execution time of a template
    lua.set_interrupt(move |_| {
        if last_execution_time_interrupt_ref
            .load(std::sync::atomic::Ordering::Acquire)
            .elapsed()
            >= MAX_TEMPLATES_EXECUTION_TIME
        {
            return Ok(LuaVmState::Yield);
        }
        Ok(LuaVmState::Continue)
    });

    // Set lua user data
    // TODO: Use guild id to find any custom constraints
    let user_data = state::LuaUserData {
        pool,
        guild_id,
        cache_http,
        reqwest_client,
        kv_constraints: state::LuaKVConstraints::default(),
        per_template: scc::HashMap::new(),
        kv_ratelimits: Arc::new(
            state::LuaKvRatelimit::new().map_err(|e| LuaError::external(e.to_string()))?,
        ),
        actions_ratelimits: Arc::new(
            state::LuaActionsRatelimit::new().map_err(|e| LuaError::external(e.to_string()))?,
        ),
    };

    lua.set_app_data(user_data);

    let lua_ref = lua.clone();

    // Used to mark a thread as broken and needing to be remade
    let broken = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let broken_ref = broken.clone();
    // Create thread handle for async execution
    //
    // This both avoids locking and allows running multiple scripts concurrently
    let thread_handle: (
        std::thread::Thread,
        tokio::sync::mpsc::UnboundedSender<LoadLuaTemplate>,
    ) = {
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<LoadLuaTemplate>();

        let thread = std::thread::Builder::new()
            .name(format!("lua-vm-{}", guild_id))
            .stack_size(MAX_VM_THREAD_STACK_SIZE)
            .spawn(move || {
                let rt = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .unwrap();

                let lua_ref = lua_ref.clone();
                let broken_ref = broken_ref.clone();
                rt.block_on(async {
                    // Catch panics
                    fn panic_catcher(
                        guild_id: GuildId,
                        broken_ref: Arc<std::sync::atomic::AtomicBool>,
                    ) -> Box<dyn Fn(&PanicHookInfo<'_>) + 'static + Sync + Send>
                    {
                        Box::new(move |_| {
                            log::error!("Lua thread panicked: {}", guild_id);
                            broken_ref.store(true, std::sync::atomic::Ordering::Release);
                        })
                    }

                    perthreadpanichook::set_hook(panic_catcher(guild_id, broken_ref));

                    while let Some(template) = rx.recv().await {
                        let args = template.args;
                        let callback = template.callback;
                        let pragma = template.pragma;
                        let template = template.template;
                        let lua_ref = lua_ref.clone();

                        rt.spawn(async move {
                            let token = match state::add_template(&lua_ref, pragma) {
                                Ok(token) => token,
                                Err(e) => {
                                    let _ = callback.send(Err(LuaError::external(e.to_string())));
                                    return;
                                }
                            };

                            let f: LuaFunction = match lua_ref
                                .load(&template)
                                .set_name("script")
                                .set_mode(mlua::ChunkMode::Text) // Ensure auto-detection never selects binary mode
                                .eval_async()
                                .await
                            {
                                Ok(f) => f,
                                Err(e) => {
                                    let _ = callback.send(Err(e));
                                    if let Err(e) = state::remove_template(&lua_ref, &token) {
                                        log::error!("Could not remove template: {}", e);
                                    };
                                    return;
                                }
                            };

                            match args {
                                Some(args) => {
                                    let args = match lua_ref.to_value(&args) {
                                        Ok(args) => args,
                                        Err(e) => {
                                            let _ = callback.send(Err(LuaError::external(e)));
                                            if let Err(e) = state::remove_template(&lua_ref, &token)
                                            {
                                                log::error!("Could not remove template: {}", e);
                                            };
                                            return;
                                        }
                                    };

                                    let v: LuaValue =
                                        match f.call_async((args, token.clone())).await {
                                            Ok(v) => v,
                                            Err(e) => {
                                                let _ = callback.send(Err(e));
                                                if let Err(e) =
                                                    state::remove_template(&lua_ref, &token)
                                                {
                                                    log::error!("Could not remove template: {}", e);
                                                };
                                                return;
                                            }
                                        };

                                    let _v: Result<serde_json::Value, LuaError> = lua_ref
                                        .from_value(v)
                                        .map_err(|e| LuaError::external(e.to_string()));

                                    let _ = callback.send(_v);

                                    if let Err(e) = state::remove_template(&lua_ref, &token) {
                                        log::error!("Could not remove template: {}", e);
                                    };
                                }
                                None => {
                                    // Just compiling etc. return Null
                                    let _ = callback.send(Ok(serde_json::Value::Null));
                                }
                            };
                        });
                    }
                });
            })?;

        let thread_handle = thread.thread().clone();

        (thread_handle, tx)
    };

    let ar_lua = ArLua {
        vm: lua,
        last_execution_time,
        thread_handle,
        broken,
    };

    Ok(ar_lua)
}

/// Get a Lua VM for a guild
///
/// This function will either return an existing Lua VM for the guild or create a new one if it does not exist
async fn get_lua_vm(
    guild_id: GuildId,
    pool: sqlx::PgPool,
    cache_http: botox::cache::CacheHttpImpl,
    reqwest_client: reqwest::Client,
) -> LuaResult<ArLua> {
    match VMS.get(&guild_id).await {
        Some(vm) => {
            if vm.broken.load(std::sync::atomic::Ordering::Acquire) {
                let vm = create_lua_vm(guild_id, pool, cache_http, reqwest_client).await?;
                VMS.insert(guild_id, vm.clone()).await;
                return Ok(vm);
            }
            Ok(vm.clone())
        }
        None => {
            let vm = create_lua_vm(guild_id, pool, cache_http, reqwest_client).await?;
            VMS.insert(guild_id, vm.clone()).await;
            Ok(vm)
        }
    }
}

/// Compiles a template
pub async fn parse(
    cache_http: botox::cache::CacheHttpImpl,
    reqwest_client: reqwest::Client,
    guild_id: serenity::all::GuildId,
    pragma: crate::TemplatePragma,
    template: &str,
    pool: sqlx::PgPool,
) -> LuaResult<()> {
    let lua = get_lua_vm(guild_id, pool, cache_http, reqwest_client).await?;

    // Update last execution time.
    lua.last_execution_time.store(
        std::time::Instant::now(),
        std::sync::atomic::Ordering::Release,
    );

    let (tx, rx) = tokio::sync::oneshot::channel();

    lua.thread_handle
        .1
        .send(LoadLuaTemplate {
            pragma,
            template: template.to_string(),
            args: None,
            callback: tx,
        })
        .map_err(|e| LuaError::external(format!("Could not send data to Lua thread: {}", e)))?;

    tokio::select! {
        _ = tokio::time::sleep(MAX_TEMPLATES_EXECUTION_TIME) => {
            return Err(LuaError::external("Template took too long to compile"));
        }
        _ = rx => {}
    }

    Ok(())
}

/// Render a template
pub async fn render_template<Request: serde::Serialize, Response: serde::de::DeserializeOwned>(
    cache_http: botox::cache::CacheHttpImpl,
    reqwest_client: reqwest::Client,
    guild_id: GuildId,
    pragma: crate::TemplatePragma,
    template: &str,
    pool: sqlx::PgPool,
    args: Request,
) -> LuaResult<Response> {
    let lua = get_lua_vm(guild_id, pool, cache_http, reqwest_client).await?;

    let args = serde_json::to_value(&args).map_err(|e| LuaError::external(e.to_string()))?;

    // Update last execution time.
    lua.last_execution_time.store(
        std::time::Instant::now(),
        std::sync::atomic::Ordering::Release,
    );

    let (tx, rx) = tokio::sync::oneshot::channel();

    lua.thread_handle
        .1
        .send(LoadLuaTemplate {
            pragma,
            template: template.to_string(),
            args: Some(args),
            callback: tx,
        })
        .map_err(|e| LuaError::external(format!("Could not send data to Lua thread: {}", e)))?;

    tokio::select! {
        _ = tokio::time::sleep(MAX_TEMPLATES_EXECUTION_TIME) => {
            Err(LuaError::external("Template took too long to compile"))
        }
        value = rx => {
            let Ok(value) = value else {
                return Err(LuaError::external("Could not receive data from Lua thread"));
            };

            // Check for an error
            if let Ok(serde_json::Value::Object(ref map)) = value {
                if let Some(value) = map.get("__error") {
                    return Err(LuaError::external(value.to_string()));
                }
            }

            let v: Response = serde_json::from_value(value?)
                .map_err(|e| LuaError::external(e.to_string()))?;

            Ok(v)
        }
    }
}
