use mlua::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LuaPermissionResult {
    /// The raw result of the permission check
    pub result: permissions::types::PermissionResult,
    /// Whether the permission result represents a success or a failure
    pub is_ok: bool,
    /// The code of the permission result
    pub code: String,
    /// The markdown representation of the permission result
    pub markdown: String,
}

impl LuaPermissionResult {
    pub fn new(result: permissions::types::PermissionResult) -> Self {
        Self {
            is_ok: result.is_ok(),
            code: result.code().to_string(),
            markdown: result.to_markdown(),
            result,
        }
    }
}

impl Default for LuaPermissionResult {
    fn default() -> Self {
        Self {
            is_ok: true,
            code: "Ok".to_string(),
            markdown: "".to_string(),
            result: permissions::types::PermissionResult::Ok {},
        }
    }
}

pub fn plugin_docs() -> templating_docgen::Plugin {
    templating_docgen::Plugin::default()
        .name("@antiraid/permissions")
        .description("Utilities for handling permission checks.")
        .type_mut(
            "PermissionResult",
            "PermissionResult is an internal type containing the status of a permission check in AntiRaid. The exact contents are undocumented as of now",
            |t| {
                t
            },
        )
        .type_mut(
            "LuaPermissionResult",
            "LuaPermissionResult is a type containing the status of a permission check in AntiRaid with prior parsing done for Lua.",
            |t| {
                t
                .example(std::sync::Arc::new(LuaPermissionResult::default()))
                .field("result", |f| f.typ("PermissionResult").description("The raw/underlying result of the permission check."))
                .field("is_ok", |f| f.typ("bool").description("Whether the permission check was successful."))
                .field("code", |f| f.typ("string").description("The code of the permission check."))
                .field("markdown", |f| f.typ("string").description("The markdown representation of the permission check."))
            },
        )
        .type_mut(
            "PermissionCheck",
            "PermissionCheck is a type containing the permissions to check for a user.",
            |t| {
                t
                .example(std::sync::Arc::new(permissions::types::PermissionCheck::default()))
                .field("kittycat_perms", |f| f.typ("{Permission}").description("The kittycat permissions needed to run the command."))
                .field("native_perms", |f| f.typ("{string}").description("The native permissions needed to run the command."))
                .field("inner_and", |f| f.typ("bool").description("Whether or not the perms are ANDed (all needed) or OR'd (at least one)"))
            },
        )
        .type_mut(
            "Permission",
            "Permission is the primitive permission type used by AntiRaid. See https://github.com/InfinityBotList/kittycat for more information",
            |t| {
                t
                .example(std::sync::Arc::new(kittycat::perms::Permission::from_string("moderation.ban")))
                .field("namespace", |f| f.typ("string").description("The namespace of the permission."))
                .field("perm", |f| f.typ("string").description("The permission bit on the namespace."))
                .field("negator", |f| f.typ("bool").description("Whether the permission is a negator permission or not"))
            },
        )
        .method_mut("permission_from_string", |m| {
            m.description("Returns a Permission object from a string.")
            .parameter("perm_string", |p| {
                p.typ("string").description("The string to parse into a Permission object.")
            })
            .return_("permission", |r| {
                r.typ("Permission").description("The parsed Permission object.")
            })
        })
        .method_mut("permission_to_string", |m| {
            m.description("Returns a string from a Permission object.")
            .parameter("permission", |p| {
                p.typ("Permission").description("The Permission object to parse into a string.")
            })
            .return_("perm_string", |r| {
                r.typ("string").description("The parsed string.")
            })
        })
        .method_mut("has_perm", |m| {
            m.description("Checks if a list of permissions in Permission object form contains a specific permission.")
            .parameter("permissions", |p| {
                p.typ("{Permission}").description("The list of permissions")
            })
            .parameter("permission", |p| {
                p.typ("Permission").description("The permission to check for.")
            })
            .return_("has_perm", |r| {
                r.typ("bool").description("Whether the permission is present in the list of permissions as per kittycat rules.")
            })
        })
        .method_mut("has_perm_str", |m| {
            m.description("Checks if a list of permissions in canonical string form contains a specific permission.")
            .parameter("permissions", |p| {
                p.typ("{string}").description("The list of permissions")
            })
            .parameter("permission", |p| {
                p.typ("string").description("The permission to check for.")
            })
            .return_("has_perm", |r| {
                r.typ("bool").description("Whether the permission is present in the list of permissions as per kittycat rules.")
            })
        })
        .method_mut("check_perms", |m| {
            m.description("Checks if a permission check passes.")
            .parameter("check", |p| {
                p.typ("PermissionCheck").description("The permission check to evaluate.")
            })
            .parameter("member_native_perms", |p| {
                p.typ("Permissions").description("The native permissions of the member.")
            })
            .parameter("member_kittycat_perms", |p| {
                p.typ("{Permission}").description("The kittycat permissions of the member.")
            })
            .return_("result", |r| {
                r.typ("LuaPermissionResult").description("The result of the permission check.")
            })
        })
}

pub fn init_plugin(lua: &Lua) -> LuaResult<LuaTable> {
    let module = lua.create_table()?;

    module.set(
        "permission_from_string",
        lua.create_function(|lua, (perm_string,): (String,)| {
            let ps = kittycat::perms::Permission::from_string(&perm_string);
            lua.to_value(&ps)
        })?,
    )?;

    module.set(
        "permission_to_string",
        lua.create_function(|lua, (permission,): (LuaValue,)| {
            let perm: kittycat::perms::Permission = lua.from_value(permission)?;
            Ok(perm.to_string())
        })?,
    )?;

    module.set(
        "has_perm",
        lua.create_function(|lua, (permissions, permission): (LuaValue, LuaValue)| {
            let perm: kittycat::perms::Permission = lua.from_value(permission)?;
            let perms: Vec<kittycat::perms::Permission> = lua.from_value(permissions)?;
            Ok(kittycat::perms::has_perm(&perms, &perm))
        })?,
    )?;

    module.set(
        "has_perm_str",
        lua.create_function(|_, (permissions, permission): (Vec<String>, String)| {
            Ok(kittycat::perms::has_perm_str(&permissions, &permission))
        })?,
    )?;

    module.set(
        "check_perms",
        lua.create_function(
            |lua,
             (check, member_native_perms, member_kittycat_perms): (
                LuaValue,
                LuaValue,
                LuaValue,
            )| {
                let check: permissions::types::PermissionCheck = lua.from_value(check)?;
                let member_native_perms: serenity::all::Permissions =
                    lua.from_value(member_native_perms)?;
                let member_kittycat_perms: Vec<kittycat::perms::Permission> =
                    lua.from_value(member_kittycat_perms)?;
                lua.to_value(&LuaPermissionResult::new(permissions::check_perms(
                    &check,
                    member_native_perms,
                    &member_kittycat_perms,
                )))
            },
        )?,
    )?;

    module.set_readonly(true); // Block any attempt to modify this table

    Ok(module)
}
