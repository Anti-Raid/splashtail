use ar_settings::types::{
    HookContext, SettingCreator, SettingDeleter, SettingUpdater, SettingView, SettingsError,
};
use mlua::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Page {
    pub title: String,
    pub description: String,
    pub settings: Vec<ar_settings::types::Setting>,
}

#[derive(FromLua, Clone)]
pub struct CreatePageSetting {
    pub setting: ar_settings::types::Setting,
}

pub struct LuaSettingExecutor {
    pub template_name: String,
    pub name: String,
}

#[async_trait::async_trait]
impl SettingView for LuaSettingExecutor {
    async fn view<'a>(
        &self,
        context: HookContext<'a>,
        filters: indexmap::IndexMap<String, splashcore_rs::value::Value>,
    ) -> Result<Vec<indexmap::IndexMap<String, splashcore_rs::value::Value>>, SettingsError> {
        let result: Vec<indexmap::IndexMap<String, splashcore_rs::value::Value>> = crate::execute(
            context.guild_id,
            crate::Template::Named(self.template_name.clone()),
            context.data.pool.clone(),
            context.data.serenity_context.clone(),
            context.data.reqwest.clone(),
            crate::event::Event::new_boxed(
                "(Anti-Raid) View Setting".to_string(),
                "Settings/View".to_string(),
                self.name.clone(),
                filters,
                false,
            ),
        )
        .await
        .map_err(|e| SettingsError::Generic {
            message: e.to_string(),
            src: "LuaSettingExecutor".to_string(),
            typ: "internal".to_string(),
        })?;

        Ok(result)
    }
}

#[async_trait::async_trait]
impl SettingCreator for LuaSettingExecutor {
    async fn create<'a>(
        &self,
        context: HookContext<'a>,
        state: indexmap::IndexMap<String, splashcore_rs::value::Value>,
    ) -> Result<indexmap::IndexMap<String, splashcore_rs::value::Value>, SettingsError> {
        let result: indexmap::IndexMap<String, splashcore_rs::value::Value> = crate::execute(
            context.guild_id,
            crate::Template::Named(self.template_name.clone()),
            context.data.pool.clone(),
            context.data.serenity_context.clone(),
            context.data.reqwest.clone(),
            crate::event::Event::new_boxed(
                "(Anti-Raid) Create Setting".to_string(),
                "Settings/Create".to_string(),
                self.name.clone(),
                state,
                false,
            ),
        )
        .await
        .map_err(|e| SettingsError::Generic {
            message: e.to_string(),
            src: "LuaSettingExecutor".to_string(),
            typ: "internal".to_string(),
        })?;

        Ok(result)
    }
}

#[async_trait::async_trait]
impl SettingUpdater for LuaSettingExecutor {
    async fn update<'a>(
        &self,
        context: HookContext<'a>,
        state: indexmap::IndexMap<String, splashcore_rs::value::Value>,
    ) -> Result<indexmap::IndexMap<String, splashcore_rs::value::Value>, SettingsError> {
        let result: indexmap::IndexMap<String, splashcore_rs::value::Value> = crate::execute(
            context.guild_id,
            crate::Template::Named(self.template_name.clone()),
            context.data.pool.clone(),
            context.data.serenity_context.clone(),
            context.data.reqwest.clone(),
            crate::event::Event::new_boxed(
                "(Anti-Raid) Update Setting".to_string(),
                "Settings/Update".to_string(),
                self.name.clone(),
                state,
                false,
            ),
        )
        .await
        .map_err(|e| SettingsError::Generic {
            message: e.to_string(),
            src: "LuaSettingExecutor".to_string(),
            typ: "internal".to_string(),
        })?;

        Ok(result)
    }
}

/*
   /// Deletes the setting
   async fn delete<'a>(
       &self,
       context: HookContext<'a>,
       pkey: splashcore_rs::value::Value,
   ) -> Result<(), SettingsError>;
*/

#[async_trait::async_trait]
impl SettingDeleter for LuaSettingExecutor {
    /// Deletes the setting
    async fn delete<'a>(
        &self,
        context: HookContext<'a>,
        pkey: splashcore_rs::value::Value,
    ) -> Result<(), SettingsError> {
        let result: indexmap::IndexMap<String, splashcore_rs::value::Value> = crate::execute(
            context.guild_id,
            crate::Template::Named(self.template_name.clone()),
            context.data.pool.clone(),
            context.data.serenity_context.clone(),
            context.data.reqwest.clone(),
            crate::event::Event::new_boxed(
                "(Anti-Raid) Delete Setting".to_string(),
                "Settings/Delete".to_string(),
                self.name.clone(),
                pkey,
                false,
            ),
        )
        .await
        .map_err(|e| SettingsError::Generic {
            message: e.to_string(),
            src: "LuaSettingExecutor".to_string(),
            typ: "internal".to_string(),
        })?;

        Ok(())
    }
}

impl LuaUserData for Page {
    fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
        // Go to the next item in the stream
        methods.add_async_method_mut(
            "add_setting",
            |lua, mut this, setting: CreatePageSetting| async move {
                // Create new setting executor
                Ok(())
            },
        ); // Implement the method
    }
}

pub fn init_plugin(lua: &Lua) -> LuaResult<LuaTable> {
    let module = lua.create_table()?;

    module.set_readonly(true); // Block any attempt to modify this table

    Ok(module)
}
