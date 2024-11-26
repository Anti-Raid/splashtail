use crate::lang_lua::state;
use ar_settings::types::{
    HookContext, SettingCreator, SettingDeleter, SettingUpdater, SettingView, SettingsError,
};
use mlua::prelude::*;
use std::sync::Arc;

pub struct CreatePage {
    pub page_id: String,
    pub guild_id: serenity::all::GuildId,
    pub title: String,
    pub description: String,
    pub template: crate::Template,
    pub settings: Vec<ar_settings::types::Setting>,
    pub is_created: bool,
}

#[derive(FromLua, Clone)]
pub struct CreatePageSetting {
    pub setting: ar_settings::types::Setting,
    pub operations: Vec<ar_settings::types::OperationType>,
}

#[derive(Clone)]
pub struct LuaSettingExecutor {
    /// The template to execute
    pub template: crate::Template,

    /// The ID of the setting
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
            self.template.clone(),
            context.data.pool.clone(),
            context.data.serenity_context.clone(),
            context.data.reqwest.clone(),
            crate::event::Event::new_normal(
                "(Anti-Raid) View Setting".to_string(),
                "Settings/View".to_string(),
                self.name.clone(),
                serde_json::to_value(filters).map_err(|e| SettingsError::Generic {
                    message: e.to_string(),
                    src: "LuaSettingExecutor".to_string(),
                    typ: "internal".to_string(),
                })?,
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
            self.template.clone(),
            context.data.pool.clone(),
            context.data.serenity_context.clone(),
            context.data.reqwest.clone(),
            crate::event::Event::new_normal(
                "(Anti-Raid) Create Setting".to_string(),
                "Settings/Create".to_string(),
                self.name.clone(),
                serde_json::to_value(state).map_err(|e| SettingsError::Generic {
                    message: e.to_string(),
                    src: "LuaSettingExecutor".to_string(),
                    typ: "internal".to_string(),
                })?,
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
            self.template.clone(),
            context.data.pool.clone(),
            context.data.serenity_context.clone(),
            context.data.reqwest.clone(),
            crate::event::Event::new_normal(
                "(Anti-Raid) Update Setting".to_string(),
                "Settings/Update".to_string(),
                self.name.clone(),
                serde_json::to_value(state).map_err(|e| SettingsError::Generic {
                    message: e.to_string(),
                    src: "LuaSettingExecutor".to_string(),
                    typ: "internal".to_string(),
                })?,
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
impl SettingDeleter for LuaSettingExecutor {
    /// Deletes the setting
    async fn delete<'a>(
        &self,
        context: HookContext<'a>,
        pkey: splashcore_rs::value::Value,
    ) -> Result<(), SettingsError> {
        let _: () = crate::execute(
            context.guild_id,
            self.template.clone(),
            context.data.pool.clone(),
            context.data.serenity_context.clone(),
            context.data.reqwest.clone(),
            crate::event::Event::new_normal(
                "(Anti-Raid) Delete Setting".to_string(),
                "Settings/Delete".to_string(),
                self.name.clone(),
                pkey.to_json(),
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

impl LuaUserData for CreatePage {
    fn add_fields<F: LuaUserDataFields<Self>>(fields: &mut F) {
        // Page ID (Read/Write with restrictions)
        fields.add_field_method_get("page_id", |lua, this| {
            let page_id = lua.to_value(&this.page_id)?;
            Ok(page_id)
        });

        fields.add_field_method_set("page_id", |_lua, this, value: String| {
            if this.is_created {
                return Err(LuaError::runtime("Page is already created"));
            }

            if value.len() > crate::core::page::MAX_PAGE_ID_LENGTH {
                return Err(LuaError::runtime("Page ID is too long"));
            }

            if value.contains(' ')
                || value.contains('\n')
                || value.contains('\0')
                || value.contains('\r')
                || value.contains('\t')
            {
                return Err(LuaError::runtime(
                    "Page ID cannot contain spaces, newlines, or null characters",
                ));
            }

            // Ensure Page ID is fully ASCII
            if !value.is_ascii() {
                return Err(LuaError::runtime("Page ID must be ASCII"));
            }

            if !this.settings.is_empty() {
                return Err(LuaError::runtime(
                    "Cannot change page ID after settings are added",
                ));
            }

            this.page_id = value;
            Ok(())
        });

        // Title (Read/Write)
        fields.add_field_method_get("title", |lua, this| {
            let title = lua.to_value(&this.title)?;
            Ok(title)
        });

        fields.add_field_method_set("title", |_lua, this, value: String| {
            if this.is_created {
                return Err(LuaError::runtime("Page is already created"));
            }
            this.title = value;
            Ok(())
        });

        // Description (Read/Write)
        fields.add_field_method_get("description", |lua, this| {
            let description = lua.to_value(&this.description)?;
            Ok(description)
        });

        fields.add_field_method_set("description", |_lua, this, value: String| {
            if this.is_created {
                return Err(LuaError::runtime("Page is already created"));
            }

            this.description = value;
            Ok(())
        });

        // Settings (Read only)
        fields.add_field_method_get("settings", |lua, this| {
            let settings = lua.to_value(&this.settings)?;
            Ok(settings)
        });

        // Is created (Read only)
        fields.add_field_method_get("is_created", |lua, this| {
            let is_created = lua.to_value(&this.is_created)?;
            Ok(is_created)
        });

        // Template (Read only)
        fields.add_field_method_get("template", |lua, this| {
            let template = lua.to_value(&this.template)?;
            Ok(template)
        });
    }

    fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
        // Adds a setting to the page
        methods.add_async_method_mut(
            "add_setting",
            |_lua, mut this, setting: CreatePageSetting| async move {
                if this.is_created {
                    return Err(LuaError::runtime("Page is already created"));
                }

                let settings_executor = LuaSettingExecutor {
                    template: this.template.clone(),
                    name: setting.setting.id.clone(),
                };

                let ops = setting.operations;
                let mut setting = setting.setting;

                let sops = ar_settings::types::SettingOperations {
                    view: {
                        if ops.contains(&ar_settings::types::OperationType::View) {
                            Some(Arc::new(settings_executor.clone()))
                        } else {
                            None
                        }
                    },
                    create: {
                        if ops.contains(&ar_settings::types::OperationType::Create) {
                            Some(Arc::new(settings_executor.clone()))
                        } else {
                            None
                        }
                    },
                    update: {
                        if ops.contains(&ar_settings::types::OperationType::Update) {
                            Some(Arc::new(settings_executor.clone()))
                        } else {
                            None
                        }
                    },
                    delete: {
                        if ops.contains(&ar_settings::types::OperationType::Delete) {
                            Some(Arc::new(settings_executor.clone()))
                        } else {
                            None
                        }
                    },
                };

                setting.operations = sops;
                setting.id = format!("{}:{}", this.page_id, setting.id);

                this.settings.push(setting);
                Ok(())
            },
        ); // Implement the method

        // Creates the page
        methods.add_async_method_mut("create", |_lua, mut this, _: ()| async move {
            if this.is_created {
                return Err(LuaError::runtime("Page is already created"));
            }

            // Create the page
            let page = crate::Page {
                page_id: this.page_id.clone(),
                title: this.title.clone(),
                description: this.description.clone(),
                template: this.template.clone(),
                settings: this.settings.clone(),
            };

            // Add the page to the cache
            crate::cache::add_page(this.guild_id, page)
                .await
                .map_err(|e| LuaError::external(e.to_string()))?;

            this.is_created = true;
            Ok(())
        });

        // Removes the page (by page ID)
        methods.add_async_method_mut("remove", |_lua, mut this, _: ()| async move {
            if !this.is_created {
                return Err(LuaError::runtime("Page is not created"));
            }

            crate::cache::remove_page(this.guild_id, this.page_id.clone())
                .await
                .map_err(|e| LuaError::external(e.to_string()))?;

            this.is_created = false;
            Ok(())
        });

        // Pulls out a page (by page ID) and populates the user data with it
        //
        // Note that the CreatePage being modified is overwritten with the page data of the pulled page
        methods.add_async_method_mut("pull", |_lua, mut this, _: ()| async move {
            let page = crate::cache::take_page(this.guild_id, this.page_id.clone())
                .await
                .map_err(|e| LuaError::external(e.to_string()))?;

            *this = CreatePage {
                page_id: page.page_id,
                guild_id: this.guild_id,
                title: page.title,
                description: page.description,
                template: page.template,
                settings: page.settings,
                is_created: true,
            };

            Ok(())
        });
    }
}

pub fn init_plugin(lua: &Lua) -> LuaResult<LuaTable> {
    let module = lua.create_table()?;

    module.set(
        "new",
        lua.create_function(|lua, (token,): (String,)| {
            let Some(data) = lua.app_data_ref::<state::LuaUserData>() else {
                return Err(LuaError::external("No app data found"));
            };

            let template_data = data
                .per_template
                .get(&token)
                .ok_or_else(|| LuaError::external("Template not found"))?;

            let page = CreatePage {
                page_id: sqlx::types::Uuid::new_v4().to_string(),
                guild_id: data.guild_id,
                template: template_data.template.clone(),
                title: template_data.path.clone(),
                description: "Missing description".to_string(),
                settings: vec![],
                is_created: false,
            };

            Ok(page)
        })?,
    )?;

    module.set_readonly(true); // Block any attempt to modify this table

    Ok(module)
}
