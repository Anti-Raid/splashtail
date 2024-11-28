use crate::lang_lua::state;
use mlua::prelude::*;
use std::sync::Arc;

/// An sting executor is used to execute actions related to stings from Lua
/// templates
pub struct StingExecutor {
    template_data: Arc<state::TemplateData>,
    guild_id: serenity::all::GuildId,
    pool: sqlx::PgPool,
    serenity_context: serenity::all::Context,
    ratelimits: Arc<state::LuaRatelimits>,
}

impl StingExecutor {
    pub fn check_action(&self, action: String) -> Result<(), crate::Error> {
        if !self
            .template_data
            .pragma
            .allowed_caps
            .contains(&format!("sting:{}", action))
        {
            return Err("Sting operation not allowed in this template context".into());
        }

        self.ratelimits.check(&action)?;

        Ok(())
    }
}

impl LuaUserData for StingExecutor {
    fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
        methods.add_async_method("list", |lua, this, page: usize| async move {
            this.check_action("list".to_string())
                .map_err(LuaError::external)?;

            let stings = silverpelt::stings::Sting::list(&this.pool, this.guild_id, page)
                .await
                .map_err(LuaError::external)?;

            let v = lua.to_value(&stings)?;

            Ok(v)
        });

        methods.add_async_method("create", |lua, this, data: LuaValue| async move {
            let sting = lua.from_value::<silverpelt::stings::StingCreate>(data)?;

            this.check_action("create".to_string())
                .map_err(LuaError::external)?;

            if sting.guild_id != this.guild_id {
                return Err(LuaError::external("Guild ID mismatch"));
            }

            let sting = sting
                .create_and_dispatch_returning_id(this.serenity_context.clone(), &this.pool)
                .await
                .map_err(LuaError::external)?;

            Ok(sting.to_string())
        });

        methods.add_async_method("delete", |lua, this, id: LuaValue| async move {
            let id = lua.from_value::<sqlx::types::Uuid>(id)?;

            this.check_action("delete".to_string())
                .map_err(LuaError::external)?;

            silverpelt::stings::Sting::delete_by_id(&this.pool, this.guild_id, id)
                .await
                .map_err(LuaError::external)?;

            Ok(())
        });
    }
}

pub fn init_plugin(lua: &Lua) -> LuaResult<LuaTable> {
    let module = lua.create_table()?;

    module.set(
        "new",
        lua.create_function(|lua, (token,): (crate::TemplateContextRef,)| {
            let Some(data) = lua.app_data_ref::<state::LuaUserData>() else {
                return Err(LuaError::external("No app data found"));
            };

            let executor = StingExecutor {
                template_data: token.template_data.clone(),
                guild_id: data.guild_id,
                serenity_context: data.serenity_context.clone(),
                ratelimits: data.sting_ratelimits.clone(),
                pool: data.pool.clone(),
            };

            Ok(executor)
        })?,
    )?;

    module.set_readonly(true); // Block any attempt to modify this table

    Ok(module)
}
