pub mod plugins;
mod state;
mod utils; // Private utils like AtomicInstant

use mlua::prelude::*;
use moka::future::Cache;
use serenity::all::GuildId;
use std::sync::Arc;
use std::sync::LazyLock;

static VMS: LazyLock<Cache<GuildId, ArLua>> =
    LazyLock::new(|| Cache::builder().time_to_idle(MAX_TEMPLATE_LIFETIME).build());

pub const MAX_TEMPLATE_MEMORY_USAGE: usize = 1024 * 1024 * 3; // 3MB maximum memory
pub const MAX_VM_THREAD_STACK_SIZE: usize = 1024 * 1024 * 4; // 4MB maximum memory
pub const MAX_TEMPLATE_LIFETIME: std::time::Duration = std::time::Duration::from_secs(60 * 5); // 5 minutes maximum lifetime
pub const MAX_TEMPLATES_EXECUTION_TIME: std::time::Duration = std::time::Duration::from_secs(5); // 5 seconds maximum execution time

pub struct LoadLuaTemplate {
    pub template: String,
    pub args: Option<serde_json::Value>,
    pub callback: tokio::sync::oneshot::Sender<Result<serde_json::Value, LuaError>>,
}

#[derive(Clone)]
pub struct ArLua {
    /// The Lua VM. The VM is wrapped in an async aware Mutex to ensure it is safe to use across await points
    #[allow(dead_code)]
    pub vm: Lua,
    /// The last execution time of the Lua VM
    pub last_execution_time: Arc<utils::AtomicInstant>,
    /// The thread handle for the Lua VM
    pub thread_handle: (
        std::thread::Thread,
        tokio::sync::mpsc::UnboundedSender<LoadLuaTemplate>,
    ),
}

/// Create a new Lua VM complete with sandboxing and modules pre-loaded
///
/// Note that callers should instead call the render_message_template/render_permissions_template functions
///
/// As such, this function is private and should not be used outside of this module
async fn create_lua_vm(guild_id: GuildId, pool: sqlx::PgPool) -> LuaResult<ArLua> {
    let lua = Lua::new_with(
        LuaStdLib::ALL_SAFE,
        LuaOptions::new().catch_rust_panics(true),
    )?;

    // Override print function with function that appends to __stack.stdout table
    // We do this by executing a lua script
    lua.load(
        r#"
        _G.print = function(...)
            local args = {...}
            local str = ""
            for i = 1, #args do
                str = str .. tostring(args[i]) .. "\t"
            end
            __stack.stdout = __stack.stdout or {}
            table.insert(__stack.stdout, str)
        end
    "#,
    )
    .exec()?;

    lua.sandbox(true)?; // We explicitly want globals to be shared across all scripts in this VM
    lua.set_memory_limit(MAX_TEMPLATE_MEMORY_USAGE)?;

    // To allow locking down _G, we need to create a table to store user data (__stack)
    lua.globals().set("__stack", lua.create_table()?)?;

    // First copy existing require function to registry
    lua.set_named_registry_value(
        "_lua_require",
        lua.globals().get::<_, LuaFunction>("require")?,
    )?;

    // Then override require
    lua.globals()
        .set("require", lua.create_function(plugins::require)?)?;

    let last_execution_time = Arc::new(utils::AtomicInstant::new(std::time::Instant::now()));

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
        kv_constraints: state::LuaKVConstraints::default(),
    };

    lua.set_app_data(user_data);

    let lua_ref = lua.clone();

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
                rt.block_on(async {
                    while let Some(template) = rx.recv().await {
                        let args = template.args;
                        let callback = template.callback;
                        let template = template.template;
                        let lua_ref = lua_ref.clone();

                        rt.spawn(async move {
                            let f: LuaFunction = match lua_ref.load(&template).eval_async().await {
                                Ok(f) => f,
                                Err(e) => {
                                    let _ = callback.send(Err(e));
                                    return;
                                }
                            };

                            match args {
                                Some(args) => {
                                    let args = match lua_ref.to_value(&args) {
                                        Ok(args) => args,
                                        Err(e) => {
                                            let _ = callback
                                                .send(Err(LuaError::external(e.to_string())));
                                            return;
                                        }
                                    };

                                    let v: LuaValue = f.call_async(args).await.unwrap();

                                    let _v: Result<serde_json::Value, LuaError> = lua_ref
                                        .from_value(v)
                                        .map_err(|e| LuaError::external(e.to_string()));

                                    let _ = callback.send(_v);
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
    };

    Ok(ar_lua)
}

/// Get a Lua VM for a guild
///
/// This function will either return an existing Lua VM for the guild or create a new one if it does not exist
async fn get_lua_vm(guild_id: GuildId, pool: sqlx::PgPool) -> LuaResult<ArLua> {
    match VMS.get(&guild_id).await {
        Some(vm) => Ok(vm.clone()),
        None => {
            let vm = create_lua_vm(guild_id, pool).await?;
            VMS.insert(guild_id, vm.clone()).await;
            Ok(vm)
        }
    }
}

/// Compiles a template
pub async fn compile_template(
    guild_id: serenity::all::GuildId,
    template: &str,
    pool: sqlx::PgPool,
) -> LuaResult<()> {
    let lua = get_lua_vm(guild_id, pool).await?;

    // Update last execution time.
    lua.last_execution_time.store(
        std::time::Instant::now(),
        std::sync::atomic::Ordering::Release,
    );

    let (tx, rx) = tokio::sync::oneshot::channel();

    lua.thread_handle
        .1
        .send(LoadLuaTemplate {
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
    guild_id: GuildId,
    template: &str,
    pool: sqlx::PgPool,
    args: Request,
) -> LuaResult<Response> {
    let lua = get_lua_vm(guild_id, pool).await?;

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

            let v: Response = serde_json::from_value(value?)
                .map_err(|e| LuaError::external(e.to_string()))?;

            Ok(v)
        }
    }
}

#[cfg(test)]
mod test {
    use mlua::prelude::*;
    use rand::Rng;
    use serenity::all::GuildId;

    #[tokio::test]
    async fn lua_test() {
        let pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(3)
            .connect(&config::CONFIG.meta.postgres_url)
            .await
            .expect("Could not initialize connection");

        let mut vms = Vec::new();

        for i in 0..100000 {
            println!("{}", i);

            let lua = super::create_lua_vm(GuildId::new(1), pool.clone())
                .await
                .unwrap();
            lua.vm
                .load("require \"@antiraid/builtins\" ")
                .exec()
                .unwrap();
            lua.vm.load("require \"os\" ").exec().unwrap();
            vms.push(std::rc::Rc::new(lua));
        }
    }

    #[tokio::test]
    async fn multilua_test() {
        let vm1 = Lua::new();
        vm1.sandbox(true).unwrap();
        let vm2 = Lua::new();
        vm2.sandbox(true).unwrap();

        // To allow locking down _G, we need to create a table to store user data (__stack)
        vm1.globals()
            .set("__stack", vm1.create_table().unwrap())
            .unwrap();
        vm2.globals()
            .set("__stack", vm2.create_table().unwrap())
            .unwrap();

        // Set a global variable
        let f: LuaFunction = vm1
            .load(
                r#"
            -- My first function
            function(args)
                for k, v in pairs(args) do
                    print(k, v)
                end
                __stack.obs = "some text value"
                return args["a"]
            end
        "#,
            )
            .eval_async()
            .await
            .unwrap();

        let luav = vm1.create_table().unwrap();
        luav.set("a", 1).unwrap();

        let res: i32 = f.call_async(luav).await.unwrap();

        assert_eq!(res, 1);

        // _G.obj must persist
        let f: LuaFunction = vm1
            .load(
                r#"
        -- If __stack.obs is set, return arg1
        function(args)
            for k, v in pairs(args) do
                print(k, v)
            end

            if __stack.obs == "some text value" then
                return args["arg1"]
            end

            return 3
        end
    "#,
            )
            .eval_async()
            .await
            .unwrap();

        let res: i32 = f
            .call_async({
                let t = vm1.create_table().unwrap();
                t.set("arg1", 5).unwrap();
                t
            })
            .await
            .unwrap();

        assert_eq!(res, 5);

        // But _G.obs must not be set in vm2
        // _G.obj must persist
        let f: LuaFunction = vm2
            .load(
                r#"
        -- If __stack.obs is set, return arg1
        function(args)
            for k, v in pairs(args) do
                print(k, v)
            end

            if __stack.obs == "some text value" then
                return args["arg1"]
            end

            return 3
        end
    "#,
            )
            .eval_async()
            .await
            .unwrap();

        let res: i32 = f
            .call_async({
                let t = vm2.create_table().unwrap();
                t.set("arg1", 5).unwrap();
                t
            })
            .await
            .unwrap();

        assert_eq!(res, 3);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_async_multi() {
        use mlua::{Lua, Result};
        use std::time::Duration;

        async fn sleep(_: &Lua, i: u64) -> Result<()> {
            // Get a random number between 0 and 1000
            let ms = rand::thread_rng().gen_range(0..1000);
            tokio::time::sleep(Duration::from_millis(ms)).await;
            println!("Slept for {ms} ms {}", i);
            Ok(())
        }

        let lua = Lua::new();

        lua.globals()
            .set("sleep", lua.create_async_function(sleep).unwrap())
            .unwrap();

        let mut tasks = tokio::task::JoinSet::new();
        let mut i = 100;
        loop {
            let lua = lua.clone();
            tasks.spawn(async move {
                lua.load(format!("sleep({i})")).exec_async().await.unwrap();
            });
            i -= 1;

            if i == 0 {
                break;
            }
        }

        #[allow(clippy::redundant_pattern_matching)]
        while let Some(_) = tasks.join_next().await {}
    }
}
