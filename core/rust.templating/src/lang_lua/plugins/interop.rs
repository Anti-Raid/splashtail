use crate::lang_lua::state;
use mlua::prelude::*;

pub fn plugin_docs() -> templating_docgen::Plugin {
    templating_docgen::Plugin::default()
        .name("@antiraid/interop")
        .description("This plugin allows interoperability with AntiRaid and controlled interaction with the low-levels of AntiRaid templating subsystem.")
        .type_mut(
            "null",
            "`null` is a special value that represents nothing. It is often used in AntiRaid instead of `nil` due to issues regarding existence etc. `null` is not equal to `nil` but is also an opaque type.",
            |t| {
                t
            },
        )
        .type_mut(
            "array_metatable",
            "`array_metatable` is a special metatable that is used to represent arrays across the Lua-AntiRaid templating subsystem boundary. This metatable must be set on all arrays over this boundary and is required to ensure AntiRaid knows the value you're sending it is actually an array and not an arbitrary Luau table.",
            |t| {
                t
            },
        )
        .method_mut("array_metatable", |m| {
            m.description("Returns the array metatable.")
            .return_("array_metatable", |r| {
                r.typ("table").description("The array metatable.")
            })
        })
        .method_mut("null", |m| {
            m.description("Returns the null value.")
            .return_("null", |r| {
                r.typ("null").description("The null value.")
            })
        })
        .method_mut("memusage", |m| {
            m.description("Returns the current memory usage of the Lua VM.")
            .return_("memory_usage", |r| {
                r.typ("f64").description("The current memory usage, in bytes, of the Lua VM.")
            })
        })
        .method_mut("guild_id", |m| {
            m.description("Returns the current guild ID of the Lua VM.")
            .return_("guild_id", |r| {
                r.typ("string").description("The current guild ID.")
            })
        })
        .type_mut(
            "TemplatePragma",
            "`TemplatePragma` contains the pragma of the template. Note that the list of fields below in non-exhaustive as templates can define extra fields on the pragma as well",
            |t| {
                t
                .example(std::sync::Arc::new(crate::TemplatePragma::default()))
                .field("lang", |f| {
                    f.typ("string").description("The language of the template.")
                })
                .field("allowed_caps", |f| {
                    f.typ("{string}").description("The allowed capabilities provided to the template.")
                })
            },
        )
        .type_mut(
            "TemplateData",
            "`TemplateData` is a struct that represents the data associated with a template token. It is used to store the path and pragma of a template token.",
            |t| {
                t
                .example(std::sync::Arc::new(crate::lang_lua::state::TemplateData {
                    path: "test".to_string(),
                    pragma: crate::TemplatePragma::default(),
                    template: crate::Template::Named("foo".to_string()),
                }))
                .field("path", |f| {
                    f.typ("string").description("The path of the template token.")
                })
                .field("pragma", |f| {
                    f.typ("TemplatePragma").description("The pragma of the template.")
                })
            },
        )
        .method_mut("gettemplatedata", |m| {
            m.description("Returns the data associated with a template token.")
            .parameter("token", |p| {
                p.typ("string").description("The token of the template to retrieve data for.")
            })
            .return_("data", |r| {
                r.typ("TemplateData?").description("The data associated with the template token, or `null` if no data is found.")
            })
        })
        .method_mut("current_user", |m| {
            m.description("Returns the current user of the Lua VM.")
            .return_("user", |r| {
                r
                .typ("Serenity.User")
                .description("Returns AntiRaid's discord user object.")
            })
        })
}

pub fn init_plugin(lua: &Lua) -> LuaResult<LuaTable> {
    let module = lua.create_table()?;

    // Null
    module.set("null", lua.null())?;

    // Array metatable
    module.set("array_metatable", lua.array_metatable())?;

    module.set(
        "memusage",
        lua.create_function(|lua, _: ()| Ok(lua.used_memory()))?,
    )?;

    module.set(
        "guild_id",
        lua.create_function(|lua, _: ()| {
            let Some(data) = lua.app_data_ref::<state::LuaUserData>() else {
                return Err(LuaError::external("No app data found"));
            };

            Ok(data.guild_id.to_string())
        })?,
    )?;

    module.set(
        "gettemplatedata",
        lua.create_function(|lua, token: String| {
            let Some(data) = lua.app_data_ref::<state::LuaUserData>() else {
                return Err(LuaError::external("No app data found"));
            };

            match data.per_template.read(&token, |_, x| x.clone()) {
                Some(data) => {
                    let v = lua.to_value(&data)?;
                    Ok(v)
                }
                None => Ok(lua.null()),
            }
        })?,
    )?;

    module.set(
        "current_user",
        lua.create_function(|lua, _: ()| {
            let Some(data) = lua.app_data_ref::<state::LuaUserData>() else {
                return Err(LuaError::external("No app data found"));
            };

            let v = lua.to_value(&data.serenity_context.cache.current_user().clone())?;
            Ok(v)
        })?,
    )?;

    module.set_readonly(true); // Block any attempt to modify this table

    Ok(module)
}
