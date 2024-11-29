pub mod ctx;
pub mod event;
pub mod primitives_docs;
pub mod samples;
pub(crate) mod state;

mod plugins;
pub use plugins::PLUGINS;

mod handler;
pub use handler::handle_event;

use crate::atomicinstant;
use crate::{MAX_TEMPLATES_EXECUTION_TIME, MAX_TEMPLATE_LIFETIME, MAX_TEMPLATE_MEMORY_USAGE};
use mlua::prelude::*;
use moka::future::Cache;
use serenity::all::GuildId;
use std::hash::Hash;
use std::hash::Hasher;
use std::sync::Arc;
use std::sync::LazyLock;

#[cfg(feature = "thread_proc")]
mod thread_proc;

static VMS: LazyLock<Cache<GuildId, ArLua>> =
    LazyLock::new(|| Cache::builder().time_to_idle(MAX_TEMPLATE_LIFETIME).build());

#[derive(serde::Serialize, serde::Deserialize)]
#[allow(dead_code)]
pub enum LuaVmAction {
    /// Execute a template
    Exec {
        content: String,
        template: crate::Template,
        pragma: crate::TemplatePragma,
        event: event::Event,
    },
    /// Stop the Lua VM entirely
    Stop {},
    /// Returns the memory usage of the Lua VM
    GetMemoryUsage {},
    /// Set the memory limit of the Lua VM
    SetMemoryLimit { limit: usize },
}

pub enum LuaVmResult {
    Ok { result_val: serde_json::Value },
    LuaError { err: String },
    VmBroken {},
}

pub type BytecodeCache = scc::HashMap<crate::Template, (Vec<u8>, u64)>;

/// ArLua provides a handle to a Lua VM
///
/// Note that the Lua VM is not directly exposed due to thread safety issues
#[derive(Clone)]
struct ArLua {
    /// The last execution time of the Lua VM
    last_execution_time: Arc<atomicinstant::AtomicInstant>,
    /// The thread handle for the Lua VM
    handle: tokio::sync::mpsc::UnboundedSender<(
        LuaVmAction,
        tokio::sync::oneshot::Sender<LuaVmResult>,
    )>,
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

pub struct ArLuaThreadInnerState {
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
    serenity_context: serenity::all::Context,
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
        _G.require = function() error("Not yet set") end
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
        .set("require", lua.create_async_function(plugins::require)?)?;

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

    let compiler = Arc::new(compiler);

    let bytecode_cache: Arc<BytecodeCache> = Arc::new(scc::HashMap::new());

    // Set lua user data
    let user_data = state::LuaUserData {
        pool,
        guild_id,
        shard_messenger: shard_messenger_for_guild(&serenity_context, guild_id)
            .await
            .map_err(|e| LuaError::external(e.to_string()))?,
        serenity_context,
        reqwest_client,
        kv_constraints: state::LuaKVConstraints::default(),
        kv_ratelimits: Arc::new(
            state::LuaRatelimits::new_kv_rl().map_err(|e| LuaError::external(e.to_string()))?,
        ),
        actions_ratelimits: Arc::new(
            state::LuaRatelimits::new_action_rl().map_err(|e| LuaError::external(e.to_string()))?,
        ),
        sting_ratelimits: Arc::new(
            state::LuaRatelimits::new_stings_rl().map_err(|e| LuaError::external(e.to_string()))?,
        ),
        last_execution_time: last_execution_time.clone(),
        vm_bytecode_cache: bytecode_cache.clone(),
        compiler: compiler.clone(),
    };

    lua.set_app_data(user_data);

    let broken = Arc::new(std::sync::atomic::AtomicBool::new(false));

    let thread_inner_state = Arc::new(ArLuaThreadInnerState {
        lua: lua.clone(),
        bytecode_cache: bytecode_cache.clone(),
        compiler: compiler.clone(),
        broken: broken.clone(),
    });

    let ar_lua = ArLua {
        last_execution_time,
        #[cfg(feature = "thread_proc")]
        handle: thread_proc::lua_thread_impl(thread_inner_state.clone(), guild_id)?,
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
    serenity_context: serenity::all::Context,
    reqwest_client: reqwest::Client,
) -> LuaResult<ArLua> {
    match VMS.get(&guild_id).await {
        Some(vm) => {
            if vm.broken.load(std::sync::atomic::Ordering::Acquire) {
                let vm = create_lua_vm(guild_id, pool, serenity_context, reqwest_client).await?;
                VMS.insert(guild_id, vm.clone()).await;
                return Ok(vm);
            }
            Ok(vm.clone())
        }
        None => {
            let vm = create_lua_vm(guild_id, pool, serenity_context, reqwest_client).await?;
            VMS.insert(guild_id, vm.clone()).await;
            Ok(vm)
        }
    }
}

#[derive(Clone)]
pub struct ParseCompileState {
    pub serenity_context: serenity::all::Context,
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
pub async fn render_template<Response: serde::de::DeserializeOwned>(
    event: event::Event,
    state: ParseCompileState,
) -> Result<Response, silverpelt::Error> {
    let state = ParseCompileState {
        template_content: unravel_function_expression(state.template_content),
        ..state
    };

    let lua = get_lua_vm(
        state.guild_id,
        state.pool,
        state.serenity_context,
        state.reqwest_client,
    )
    .await?;

    // Update last execution time.
    lua.last_execution_time.store(
        std::time::Instant::now(),
        std::sync::atomic::Ordering::Release,
    );

    let (tx, rx) = tokio::sync::oneshot::channel();

    lua.handle
        .send((
            LuaVmAction::Exec {
                template: state.template,
                content: state.template_content,
                pragma: state.pragma,
                event,
            },
            tx,
        ))
        .map_err(|e| LuaError::external(format!("Could not send data to Lua thread: {}", e)))?;

    tokio::select! {
        _ = tokio::time::sleep(MAX_TEMPLATES_EXECUTION_TIME) => {
            Err("Template took too long to compile".into())
        }
        value = rx => {
            let Ok(value) = value else {
                return Err("Could not receive data from Lua thread".into());
            };
            match value {
                LuaVmResult::Ok { result_val: value }=> {
                    // Check for __error
                    if let serde_json::Value::Object(ref map) = value {
                        if let Some(value) = map.get("__error") {
                            return Err(value.to_string().into());
                        }
                    }

                    let v: Response = serde_json::from_value(value)
                        .map_err(|e| e.to_string())?;

                    Ok(v)
                }
                LuaVmResult::LuaError { err } => Err(err.into()),
                LuaVmResult::VmBroken {} => {
                    // Rerun render_template
                    return Err("Lua VM is broken".into());
                },
            }
        }
    }
}

async fn shard_messenger_for_guild(
    serenity_context: &serenity::all::Context,
    guild_id: serenity::all::GuildId,
) -> Result<serenity::all::ShardMessenger, crate::Error> {
    let data = serenity_context.data::<silverpelt::data::Data>();

    let guild_shard_count = data.props.shard_count().await?;
    let guild_shard_count =
        std::num::NonZeroU16::new(guild_shard_count).ok_or("No shards available")?;
    let guild_shard_id = serenity::all::utils::shard_id(guild_id, guild_shard_count);
    let guild_shard_id = serenity::all::ShardId(guild_shard_id);

    if serenity_context.shard_id != guild_shard_id {
        return data.props.shard_messenger(guild_shard_id).await;
    }

    Ok(serenity_context.shard.clone())
}
