mod perthreadpanichook;
mod plugins;
pub(crate) mod state;

use crate::atomicinstant;
use mlua::prelude::*;
use moka::future::Cache;
use serenity::all::GuildId;
use std::hash::Hash;
use std::hash::Hasher;
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
    content: String,
    template: crate::Template,
    pragma: crate::TemplatePragma,
    args: serde_json::Value,
    callback: tokio::sync::oneshot::Sender<Result<serde_json::Value, LuaError>>,
}

pub type BytecodeCache = scc::HashMap<crate::Template, (Vec<u8>, u64)>;

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
    #[allow(dead_code)]
    /// The compiler for the Lua VM
    compiler: Arc<mlua::Compiler>,
    #[allow(dead_code)]
    /// The bytecode cache maps template to (bytecode, source hash)
    ///
    /// If source hash does not match expected source hash (the template changed), the template is recompiled
    bytecode_cache: Arc<BytecodeCache>,
}

struct ArLuaThreadInnerState {
    lua: Lua,
    bytecode_cache: Arc<BytecodeCache>,
    compiler: Arc<mlua::Compiler>,
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
    lua.set_compiler(compiler.clone());

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

    let bytecode_cache: Arc<BytecodeCache> = Arc::new(scc::HashMap::new());
    let broken = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let compiler = Arc::new(compiler);

    let thread_inner_state = Arc::new(ArLuaThreadInnerState {
        lua: lua.clone(),
        bytecode_cache: bytecode_cache.clone(),
        compiler: compiler.clone(),
        broken: broken.clone(),
    });

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

                let tis_ref = thread_inner_state.clone();

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

                    perthreadpanichook::set_hook(panic_catcher(guild_id, tis_ref.broken.clone()));

                    while let Some(template) = rx.recv().await {
                        let args = template.args;
                        let callback = template.callback;
                        let pragma = template.pragma;
                        let template_content = template.content;
                        let template = template.template;

                        let tis_ref = tis_ref.clone();

                        rt.spawn(async move {
                            // Check bytecode cache first, compile template if not found
                            let template_bytecode = match resolve_template_to_bytecode(
                                template_content,
                                template.clone(),
                                &tis_ref.bytecode_cache,
                                &tis_ref.compiler,
                            )
                            .await
                            {
                                Ok(bytecode) => bytecode,
                                Err(e) => {
                                    let _ = callback.send(Err(e));
                                    return;
                                }
                            };

                            let token = match state::add_template(&tis_ref.lua, pragma) {
                                Ok(token) => token,
                                Err(e) => {
                                    let _ = callback.send(Err(LuaError::external(e)));
                                    return;
                                }
                            };

                            let args = match tis_ref.lua.to_value(&args) {
                                Ok(args) => args,
                                Err(e) => {
                                    let _ = callback.send(Err(LuaError::external(e)));
                                    if let Err(e) = state::remove_template(&tis_ref.lua, &token) {
                                        log::error!("Could not remove template: {}", e);
                                    };
                                    return;
                                }
                            };

                            let v: LuaValue = match tis_ref
                                .lua
                                .load(&template_bytecode)
                                .set_name("script")
                                .set_mode(mlua::ChunkMode::Binary) // Ensure auto-detection never selects binary mode
                                .call_async((args, token.clone()))
                                .await
                            {
                                Ok(f) => f,
                                Err(e) => {
                                    let _ = callback.send(Err(e));
                                    if let Err(e) = state::remove_template(&tis_ref.lua, &token) {
                                        log::error!("Could not remove template: {}", e);
                                    };
                                    return;
                                }
                            };

                            let _v: Result<serde_json::Value, LuaError> = tis_ref
                                .lua
                                .from_value(v)
                                .map_err(|e| LuaError::external(e.to_string()));

                            let _ = callback.send(_v);

                            if let Err(e) = state::remove_template(&tis_ref.lua, &token) {
                                log::error!("Could not remove template: {}", e);
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
        compiler,
        bytecode_cache,
    };

    Ok(ar_lua)
}

/// Helper method to fetch a template from bytecode or compile it if it doesnt exist in bytecode cache
pub(crate) async fn resolve_template_to_bytecode(
    template_content: String,
    template: crate::Template,
    bytecode_cache_ref: &BytecodeCache,
    compiler_ref: &mlua::Compiler,
) -> Result<Vec<u8>, LuaError> {
    // Check if the source hash matches the expected source hash
    let mut hasher = std::hash::DefaultHasher::new();
    template_content.hash(&mut hasher);
    let cur_hash = hasher.finish();

    let existing_bycode = bytecode_cache_ref.read(&template, |_, v| {
        if v.1 == cur_hash {
            Some(v.0.clone())
        } else {
            None
        }
    });

    println!("Resolving template to bytecode: {:?}", template_content);

    if let Some(Some(bytecode)) = existing_bycode {
        return Ok(bytecode);
    }

    let bytecode = compiler_ref.compile(&template_content)?;

    let _ = bytecode_cache_ref
        .insert_async(template, (bytecode.clone(), cur_hash))
        .await;

    Ok(bytecode)
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

pub(crate) struct ParseCompileState {
    pub cache_http: botox::cache::CacheHttpImpl,
    pub reqwest_client: reqwest::Client,
    pub guild_id: GuildId,
    pub template: crate::Template,
    pub pragma: crate::TemplatePragma,
    pub template_content: String,
    pub pool: sqlx::PgPool,
}

// If the code in question is a function expression starting with `function`, we need to unravel it
fn unravel_function_expression(template_content: String) -> String {
    let template_content = template_content.trim().to_string();
    if template_content.starts_with("function") && template_content.ends_with("end") {
        let mut lines = template_content.lines().collect::<Vec<&str>>();
        lines.remove(0);
        lines.pop();
        let uw = lines.join("\n");

        format!(
            "
local args, token = ...
{}
        ",
            uw
        )
    } else {
        template_content
    }
}

/// Render a template
pub async fn render_template<Request: serde::Serialize, Response: serde::de::DeserializeOwned>(
    args: Request,
    state: ParseCompileState,
) -> LuaResult<Response> {
    let state = ParseCompileState {
        template_content: unravel_function_expression(state.template_content),
        ..state
    };

    let lua = get_lua_vm(
        state.guild_id,
        state.pool,
        state.cache_http,
        state.reqwest_client,
    )
    .await?;

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
            template: state.template,
            content: state.template_content,
            pragma: state.pragma,
            args,
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
