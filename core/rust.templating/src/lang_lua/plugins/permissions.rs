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

pub fn init_plugin(lua: &Lua) -> LuaResult<LuaTable> {
    let module = lua.create_table()?;

    module.set(
        "new_permission_check",
        lua.create_function(|lua, ()| {
            let pc = permissions::types::PermissionCheck::default();
            lua.to_value(&pc)
        })?,
    )?;

    module.set(
        "new_permission_checks",
        lua.create_function(|lua, ()| {
            let pc = permissions::types::PermissionChecks::default();
            lua.to_value(&pc)
        })?,
    )?;

    module.set(
        "new_permission",
        lua.create_function(|lua, (namespace, perm, negator): (String, String, bool)| {
            let perm = kittycat::perms::Permission {
                namespace,
                perm,
                negator,
            };
            lua.to_value(&perm)
        })?,
    )?;

    module.set(
        "new_permission_from_string",
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
        "check_perms_single",
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
                lua.to_value(&LuaPermissionResult::new(permissions::check_perms_single(
                    &check,
                    member_native_perms,
                    &member_kittycat_perms,
                )))
            },
        )?,
    )?;

    module.set(
        "eval_checks",
        lua.create_function(
            |lua,
             (checks, member_native_perms, member_kittycat_perms): (
                LuaValue,
                LuaValue,
                LuaValue,
            )| {
                let checks: Vec<permissions::types::PermissionCheck> = lua.from_value(checks)?;
                let member_native_perms: serenity::all::Permissions =
                    lua.from_value(member_native_perms)?;
                let member_kittycat_perms: Vec<kittycat::perms::Permission> =
                    lua.from_value(member_kittycat_perms)?;
                lua.to_value(&LuaPermissionResult::new(permissions::eval_checks(
                    &checks,
                    member_native_perms,
                    member_kittycat_perms,
                )))
            },
        )?,
    )?;

    module.set_readonly(true); // Block any attempt to modify this table

    Ok(module)
}
