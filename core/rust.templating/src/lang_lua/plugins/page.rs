use crate::lang_lua::state;
use ar_settings::types::{
    HookContext, OperationType, SettingCreator, SettingDeleter, SettingUpdater, SettingView, SettingsError
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
            crate::event::Event::new(
                "(Anti-Raid) View Setting".to_string(),
                "Settings/View".to_string(),
                self.name.clone(),
                crate::event::ArcOrNormal::Normal(
                    serde_json::to_value(filters).map_err(|e| SettingsError::Generic {
                        message: e.to_string(),
                        src: "LuaSettingExecutor".to_string(),
                        typ: "internal".to_string(),
                    })?
                ),
                false,
                Some(context.author.to_string()),
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
            crate::event::Event::new(
                "(Anti-Raid) Create Setting".to_string(),
                "Settings/Create".to_string(),
                self.name.clone(),
                crate::event::ArcOrNormal::Normal(
                    serde_json::to_value(state).map_err(|e| SettingsError::Generic {
                        message: e.to_string(),
                        src: "LuaSettingExecutor".to_string(),
                        typ: "internal".to_string(),
                    })?
                ),
                false,
                Some(context.author.to_string()),
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
            crate::event::Event::new(
                "(Anti-Raid) Update Setting".to_string(),
                "Settings/Update".to_string(),
                self.name.clone(),
                crate::event::ArcOrNormal::Normal(
                    serde_json::to_value(state).map_err(|e| SettingsError::Generic {
                        message: e.to_string(),
                        src: "LuaSettingExecutor".to_string(),
                        typ: "internal".to_string(),
                    })?)
                ,
                false,
                Some(context.author.to_string()),
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
            crate::event::Event::new(
                "(Anti-Raid) Delete Setting".to_string(),
                "Settings/Delete".to_string(),
                self.name.clone(),
                crate::event::ArcOrNormal::Normal(pkey.to_json()),
                false,
                Some(context.author.to_string()),
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

pub fn plugin_docs() -> templating_docgen::Plugin {
    templating_docgen::Plugin::default()
        .name("@antiraid/page")
        .description("Create a page dedicated to your template on a server.")
        .method_mut("create_page", |mut m| {
            m.parameter("token", |p| {
                p.typ("TemplateContext")
                    .description("The token of the template to use.")
            })
            .return_("create_page", |r| {
                r.typ("CreatePage").description("An empty created page.")
            })
        })
        .enum_mut("Setting.Column.InnerColumnType", "The inner column type of the value", |e| e)
        .enum_mut("Setting.Column.ColumnType", "The type of a setting column", |e| {
            e.variant("Scalar", |v| {
                v.description("A scalar column type.")
                    .field("inner", |f| {
                        f.typ("Setting.Column.InnerColumnType")
                            .description("The inner type of the column.")
                    })
            })
            .variant("Array", |v| {
                v.description("An array column type.")
                    .field("inner", |f| {
                        f.typ("Setting.Column.InnerColumnType")
                            .description("The array type of the column.")
                    })
            })
        })
        .type_mut("Setting.Column", "A setting column", |t| {
            t.example(Arc::new(ar_settings::common_columns::created_at()))
            .field("id", |f| {
                f.typ("string").description("The ID of the column.")
            })
            .field("name", |f| {
                f.typ("string").description("The name of the column.")
            })
            .field("description", |f| {
                f.typ("string").description("The description of the column.")
            })
            .field("column_type", |f| {
                f.typ("Setting.Column.ColumnType").description("The type of the column.")
            })
            .field("nullable", |f| {
                f.typ("bool").description("Whether the column can be null.")
            })
            .field("suggestions", |f| {
                f.typ("Setting.Column.ColumnSuggestion").description("The suggestions for the column.")
            })
            .field("secret", |f| {
                f.typ("bool").description("Whether the column is secret.")
            })
            .field("ignored_for", |f| {
                f.typ("{OperationType}").description("The operations that the column is ignored for [read-only]. It is *not guaranteed* that ignored field are sent to the template.")
            })
        })
        .type_mut("Setting", "A setting", |t| {
            t.example(Arc::new(ar_settings::types::Setting {
                id: "setting_id".to_string(),
                name: "Setting Name".to_string(),
                description: "Setting Description".to_string(),
                operations: ar_settings::types::SettingOperations {
                    view: None,
                    create: None,
                    update: None,
                    delete: None,
                },
                primary_key: "id".to_string(),
                title_template: "{col1} - {col2}".to_string(),
                columns: vec![
                    ar_settings::types::Column {
                        id: "col1".to_string(),
                        name: "Column 1".to_string(),
                        description: "Column 1 desc".to_string(),
                        column_type: ar_settings::types::ColumnType::new_scalar(
                            ar_settings::types::InnerColumnType::String {
                                min_length: Some(120),
                                max_length: Some(120),
                                allowed_values: vec!["allowed_value".to_string()],
                                kind: ar_settings::types::InnerColumnTypeStringKind::Normal {},
                            },
                        ),
                        nullable: false,
                        suggestions: ar_settings::types::ColumnSuggestion::Static { suggestions: vec!["suggestion".to_string()] },
                        secret: false,
                        ignored_for: vec![OperationType::Create], 
                    },
                    ar_settings::types::Column {
                        id: "col2".to_string(),
                        name: "Column 2".to_string(),
                        description: "Column 2 desc".to_string(),
                        column_type: ar_settings::types::ColumnType::new_array(
                            ar_settings::types::InnerColumnType::String {
                                min_length: Some(120),
                                max_length: Some(120),
                                allowed_values: vec!["allowed_value".to_string()],
                                kind: ar_settings::types::InnerColumnTypeStringKind::Token {
                                    default_length: 10,
                                },
                            },
                        ),
                        nullable: false,
                        suggestions: ar_settings::types::ColumnSuggestion::Static { suggestions: vec!["suggestion".to_string()] },
                        secret: false,
                        ignored_for: vec![OperationType::View], 
                    },
                    ar_settings::types::Column {
                        id: "col3".to_string(),
                        name: "Column 3".to_string(),
                        description: "Column 3 desc".to_string(),
                        column_type: ar_settings::types::ColumnType::new_array(
                            ar_settings::types::InnerColumnType::String {
                                min_length: Some(120),
                                max_length: Some(120),
                                allowed_values: vec!["allowed_value".to_string()],
                                kind: ar_settings::types::InnerColumnTypeStringKind::Textarea {
                                    ctx: "anything".to_string(),
                                },
                            },
                        ),
                        nullable: false,
                        suggestions: ar_settings::types::ColumnSuggestion::Static { suggestions: vec!["suggestion".to_string()] },
                        secret: false,
                        ignored_for: vec![OperationType::Create], 
                    },
                    ar_settings::types::Column {
                        id: "col4".to_string(),
                        name: "Column 4".to_string(),
                        description: "Column 4 desc".to_string(),
                        column_type: ar_settings::types::ColumnType::new_array(
                            ar_settings::types::InnerColumnType::String {
                                min_length: Some(120),
                                max_length: Some(120),
                                allowed_values: vec!["allowed_value".to_string()],
                                kind: ar_settings::types::InnerColumnTypeStringKind::TemplateRef {},
                            },
                        ),
                        nullable: false,
                        suggestions: ar_settings::types::ColumnSuggestion::Static { suggestions: vec!["suggestion".to_string()] },
                        secret: false,
                        ignored_for: vec![OperationType::Create], 
                    },
                    ar_settings::types::Column {
                        id: "col5".to_string(),
                        name: "Column 5".to_string(),
                        description: "Column 5 desc".to_string(),
                        column_type: ar_settings::types::ColumnType::new_array(
                            ar_settings::types::InnerColumnType::String {
                                min_length: Some(120),
                                max_length: Some(120),
                                allowed_values: vec!["allowed_value".to_string()],
                                kind: ar_settings::types::InnerColumnTypeStringKind::KittycatPermission {},
                            },
                        ),
                        nullable: false,
                        suggestions: ar_settings::types::ColumnSuggestion::Static { suggestions: vec!["suggestion".to_string()] },
                        secret: false,
                        ignored_for: vec![OperationType::Create], 
                    },
                    ar_settings::types::Column {
                        id: "col6".to_string(),
                        name: "Column 6".to_string(),
                        description: "Column 6 desc".to_string(),
                        column_type: ar_settings::types::ColumnType::new_array(
                            ar_settings::types::InnerColumnType::String {
                                min_length: Some(120),
                                max_length: Some(120),
                                allowed_values: vec!["allowed_value".to_string()],
                                kind: ar_settings::types::InnerColumnTypeStringKind::User {},
                            },
                        ),
                        nullable: false,
                        suggestions: ar_settings::types::ColumnSuggestion::Static { suggestions: vec!["suggestion".to_string()] },
                        secret: false,
                        ignored_for: vec![OperationType::Create], 
                    },
                    ar_settings::types::Column {
                        id: "col7".to_string(),
                        name: "Column 7".to_string(),
                        description: "Column 7 desc".to_string(),
                        column_type: ar_settings::types::ColumnType::new_array(
                            ar_settings::types::InnerColumnType::String {
                                min_length: Some(120),
                                max_length: Some(120),
                                allowed_values: vec!["allowed_value".to_string()],
                                kind: ar_settings::types::InnerColumnTypeStringKind::Role {},
                            },
                        ),
                        nullable: false,
                        suggestions: ar_settings::types::ColumnSuggestion::Static { suggestions: vec!["suggestion".to_string()] },
                        secret: false,
                        ignored_for: vec![OperationType::Create], 
                    },
                    ar_settings::types::Column {
                        id: "col8".to_string(),
                        name: "Column 8".to_string(),
                        description: "Column 8 desc".to_string(),
                        column_type: ar_settings::types::ColumnType::new_array(
                            ar_settings::types::InnerColumnType::String {
                                min_length: Some(120),
                                max_length: Some(120),
                                allowed_values: vec!["allowed_value".to_string()],
                                kind: ar_settings::types::InnerColumnTypeStringKind::Channel {
                                    allowed_channel_types: vec![serenity::all::ChannelType::Text, serenity::all::ChannelType::Voice],
                                    needed_bot_permissions: serenity::all::Permissions::SEND_MESSAGES,
                                },
                            },
                        ),
                        nullable: false,
                        suggestions: ar_settings::types::ColumnSuggestion::Static { suggestions: vec!["suggestion".to_string()] },
                        secret: false,
                        ignored_for: vec![OperationType::Update], 
                    },
                    ar_settings::types::Column {
                        id: "col9".to_string(),
                        name: "Column 9".to_string(),
                        description: "Column 9 desc".to_string(),
                        column_type: ar_settings::types::ColumnType::new_scalar(
                            ar_settings::types::InnerColumnType::Integer {}
                        ),
                                                nullable: false,
                        suggestions: ar_settings::types::ColumnSuggestion::Static { suggestions: vec!["suggestion".to_string()] },
                        secret: false,
                        ignored_for: vec![OperationType::Update], 
                    },
                    ar_settings::types::Column {
                        id: "col10".to_string(),
                        name: "Column 10".to_string(),
                        description: "Column 10 desc".to_string(),
                        column_type: ar_settings::types::ColumnType::new_scalar(
                            ar_settings::types::InnerColumnType::Boolean {}
                        ),
                                                nullable: false,
                        suggestions: ar_settings::types::ColumnSuggestion::Static { suggestions: vec!["suggestion".to_string()] },
                        secret: false,
                        ignored_for: vec![OperationType::Update], 
                    },
                    ar_settings::common_columns::created_at(),
                ].into(),
            }))
            .field("id", |f| {
                f.typ("string").description("The ID of the setting.")
            })
            .field("name", |f| {
                f.typ("string").description("The name of the setting.")
            })
            .field("description", |f| {
                f.typ("string").description("The description of the setting.")
            })
            .field("operations", |f| {
                f.typ("{OperationType}").description("The operations that can be performed on the setting. **Note that when using ``add_settings``, you must pass this as the second argument to settings and ignore this field.**")
            })
            .field("primary_key", |f| {
                f.typ("string").description("The primary key of the setting that UNIQUELY identifies the row. When ``Delete`` is called, the value of this is what will be sent in the event. On ``Update``, this key MUST also exist (otherwise, the template MUST error out)")
            })
            .field("title_template", |f| {
                f.typ("string").description("The template for the title of each row for the setting. This is a string that can contain placeholders for columns. The placeholders are in the form of ``{column_id}``. For example, if you have a column with ID ``col1`` and another with ID ``col2``, you can have a title template of ``{col1} - {col2}`` etc..")
            })
            .field("columns", |f| {
                f.typ("{Setting.Column}").description("The columns of the setting.")
            })
        })
        .type_mut("CreatePageSetting", "A table containing a setting for a page", |mut t| {
            t.field("setting", |f| {
                f.typ("Setting").description("The setting to add to the page.")
            })
            .field("operations", |f| {
                f.typ("{string}").description("The operations to perform on the setting. Elements of the array can be either `View`, `Create`, `Update` or `Delete`.")
            })
        })
        .type_mut("CreatePage", "An intermediary structure for creating a page for a template", |mut t| {
            t.field("page_id", |f| {
                f.typ("string").description("The ID of the page. This field **can be updated ONLY if the page is not created yet with no current settings.** The ID must not contain spaces, newlines, null characters, or tabs.")
            })
            .field("title", |f| {
                f.typ("string").description("The title of the page. This field **can be updated ONLY if the page is not created yet.**")
            })
            .field("description", |f| {
                f.typ("string").description("The description of the page. This field **can be updated ONLY if the page is not created yet.**")
            })
            .field("settings", |f| {
                f.typ("table").description("The settings of the page. **This field is read-only.**")
            })
            .field("is_created", |f| {
                f.typ("bool").description("Whether the page is created. **This field is read-only.**")
            })
            .field("template", |f| {
                f.typ("Template").description("The template of the page. **This field is read-only.**")
            })
            .method_mut("add_setting", |mut m| {
                m.parameter("setting", |p| {
                    p.typ("CreatePageSetting").description("The setting to add to the page.")
                })
                .return_("ret", |r| {
                    r.typ("nil")
                })
            })
        })
}

pub fn init_plugin(lua: &Lua) -> LuaResult<LuaTable> {
    let module = lua.create_table()?;

    module.set(
        "create_page",
        lua.create_function(|lua, (token,): (crate::TemplateContextRef,)| {
            let Some(data) = lua.app_data_ref::<state::LuaUserData>() else {
                return Err(LuaError::external("No app data found"));
            };

            let page = CreatePage {
                page_id: sqlx::types::Uuid::new_v4().to_string(),
                guild_id: data.guild_id,
                template: token.template_data.template.clone(),
                title: token.template_data.path.clone(),
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
