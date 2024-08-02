pub mod plugins;
mod utils; // Private utils like AtomicInstant

use mlua::prelude::*;
use moka::future::Cache;
use once_cell::sync::Lazy;
use serenity::all::GuildId;
use std::sync::Arc;

static VMS: Lazy<Cache<GuildId, ArLua>> =
    Lazy::new(|| Cache::builder().time_to_idle(MAX_TEMPLATE_LIFETIME).build());

pub const MAX_TEMPLATE_MEMORY_USAGE: usize = 1024 * 1024 * 3; // 3MB maximum memory
pub const MAX_TEMPLATE_LIFETIME: std::time::Duration = std::time::Duration::from_secs(60 * 5); // 5 minutes maximum lifetime
pub const MAX_TEMPLATES_EXECUTION_TIME: std::time::Duration = std::time::Duration::from_secs(5); // 5 seconds maximum execution time

pub struct ArLuaExecutionState {
    /// The last time this Lua VM was executed
    pub last_exec: utils::AtomicInstant,
}

#[derive(Clone)]
pub struct ArLua {
    /// The Lua VM
    pub vm: Lua,
    /// The execution state of the Lua VM
    pub state: Arc<ArLuaExecutionState>,
}

/// Create a new Lua VM complete with sandboxing and modules pre-loaded
///
/// Note that callers should instead call the render_message_template/render_permissions_template functions
///
/// As such, this function is private and should not be used outside of this module
async fn create_lua_vm() -> LuaResult<ArLua> {
    let lua = Lua::new_with(
        LuaStdLib::ALL_SAFE,
        LuaOptions::new().catch_rust_panics(true),
    )?;
    lua.sandbox(true)?; // We explicitly want globals to be shared across all scripts in this VM
    lua.set_memory_limit(MAX_TEMPLATE_MEMORY_USAGE)?;

    // To allow locking down _G, we need to create a table to store user data (__stack)
    lua.globals().set("__stack", lua.create_table()?)?;

    // Disable print function, templates should not be able to access stdout
    // TODO: Offer a custom print function that logs to a channel
    lua.globals().set("print", LuaValue::Nil)?;

    // Add the builtins module to global scope
    //
    // Note that we do not register every plugin here to give more memory to the templates themselves
    let builtins_module = plugins::builtins(&lua)?;

    lua.globals().set("__ar_builtins", builtins_module)?;

    let state: Arc<ArLuaExecutionState> = Arc::new(ArLuaExecutionState {
        last_exec: utils::AtomicInstant::new(std::time::Instant::now()),
    });

    let state_interrupt_ref = state.clone();

    // Create an interrupt to limit the execution time of a template
    lua.set_interrupt(move |_| {
        if state_interrupt_ref
            .last_exec
            .load(std::sync::atomic::Ordering::Acquire)
            .elapsed()
            >= MAX_TEMPLATES_EXECUTION_TIME
        {
            return Ok(LuaVmState::Yield);
        }
        Ok(LuaVmState::Continue)
    });

    let ar_lua = ArLua { vm: lua, state };

    Ok(ar_lua)
}

/// Get a Lua VM for a guild
///
/// This function will either return an existing Lua VM for the guild or create a new one if it does not exist
async fn get_lua_vm(guild_id: GuildId) -> LuaResult<ArLua> {
    match VMS.get(&guild_id).await {
        Some(vm) => {
            vm.state.last_exec.store(
                std::time::Instant::now(),
                std::sync::atomic::Ordering::Release,
            ); // Update the last execution time
            Ok(vm.clone())
        }
        None => {
            let vm = create_lua_vm().await?;
            VMS.insert(guild_id, vm.clone()).await;
            Ok(vm)
        }
    }
}

/// Compiles a template
pub async fn compile_template(guild_id: serenity::all::GuildId, template: &str) -> LuaResult<()> {
    let lua = get_lua_vm(guild_id).await?;

    let _: LuaFunction = lua.vm.load(template).eval_async().await?;

    Ok(())
}

/// Render a message template
pub async fn render_message_template(
    guild_id: GuildId,
    template: &str,
    args: crate::core::MessageTemplateContext,
) -> LuaResult<plugins::message::Message> {
    let lua = get_lua_vm(guild_id).await?;

    let args = lua.vm.to_value(&args)?;

    let f: LuaFunction = lua.vm.load(template).eval_async().await?;

    let v: LuaValue = f.call_async(args).await?;

    let json_v = serde_json::to_value(v).map_err(|e| LuaError::external(e.to_string()))?;
    let v: plugins::message::Message =
        serde_json::from_value(json_v).map_err(|e| LuaError::external(e.to_string()))?;

    Ok(v)
}

/// Render a message template
pub async fn render_permissions_template(
    guild_id: GuildId,
    template: &str,
    args: crate::core::PermissionTemplateContext,
) -> LuaResult<splashcore_rs::types::silverpelt::PermissionResult> {
    let lua = get_lua_vm(guild_id).await?;

    let args = lua.vm.to_value(&args)?;

    let f: LuaFunction = lua.vm.load(template).eval_async().await?;

    let v: LuaValue = f.call_async(args).await?;

    let json_v = serde_json::to_value(v).map_err(|e| LuaError::external(e.to_string()))?;
    let v: splashcore_rs::types::silverpelt::PermissionResult =
        serde_json::from_value(json_v).map_err(|e| LuaError::external(e.to_string()))?;

    Ok(v)
}

#[cfg(test)]
mod test {
    use mlua::prelude::*;

    fn test_and_return_luavm() -> LuaResult<Lua> {
        let lua = Lua::new();
        lua.sandbox(true)?;
        lua.set_memory_limit(super::MAX_TEMPLATE_MEMORY_USAGE)?;
        lua.globals().set("require", LuaValue::Nil)?;

        let map_table = lua.create_table()?;
        map_table.set(1, "one")?;
        map_table.set("two", 2)?;

        lua.globals().set("map_table", map_table)?;

        lua.load("for k,v in pairs(map_table) do end").exec()?;

        Ok(lua)
    }

    #[tokio::test]
    async fn lua_test() {
        let mut vms = Vec::new();

        for i in 0..100000 {
            println!("{}", i);

            let lua = test_and_return_luavm().unwrap();
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
}
