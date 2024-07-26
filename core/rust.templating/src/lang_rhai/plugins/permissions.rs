use rhai::plugin::*;
use splashcore_rs::permissions::check_perms_single;

#[export_module]
pub mod plugin {
    /// PermissionResult structure
    pub type PermissionResult = splashcore_rs::types::silverpelt::PermissionResult;

    /// Kittycat permission
    pub type KittycatPermission = kittycat::perms::Permission;

    /// Returns the permission as a string
    #[rhai_fn(get = "as_string")]
    pub fn get_kittycat_permission_as_string(perm: &mut KittycatPermission) -> String {
        perm.to_string()
    }

    /// Returns the namespace bit of the permission
    #[rhai_fn(get = "namespace")]
    pub fn get_kittycat_permission_namespace(perm: &mut KittycatPermission) -> String {
        perm.namespace.to_string()
    }

    /// Returns the permission bit of the permission
    #[rhai_fn(get = "perm")]
    pub fn get_kittycat_permission_permission(perm: &mut KittycatPermission) -> String {
        perm.perm.to_string()
    }

    /// Returns whether the permission is negated
    #[rhai_fn(get = "negator")]
    pub fn get_kittycat_permission_negator(perm: &mut KittycatPermission) -> bool {
        perm.negator
    }

    /// Returns whether a set of permissions has the permission string
    #[rhai_fn(name = "has_kittycat_permission", return_raw)]
    pub fn has_kittycat_permission(
        perms: rhai::Array,
        perm: &str,
    ) -> Result<bool, Box<EvalAltResult>> {
        let mut fperms = Vec::new();

        for perm in perms {
            let perm = KittycatPermission::from(perm.into_string()?);

            fperms.push(perm);
        }

        Ok(kittycat::perms::has_perm(
            &fperms,
            &KittycatPermission::from(perm),
        ))
    }

    /// Creates a new kittycat permission from a string
    #[rhai_fn(name = "string_to_kittycat_permission")]
    pub fn string_to_kittycat_permission(perm: &str) -> KittycatPermission {
        KittycatPermission::from(perm)
    }

    /// Returns if a Pe`rmissionResult is a Ok or not
    #[rhai_fn(name = "is_ok", global, pure)]
    pub fn is_permission_result_ok(perm_res: &mut PermissionResult) -> bool {
        perm_res.is_ok()
    }

    /// Creates a new permission result from a object map
    #[rhai_fn(name = "map_to_permission_result")]
    pub fn map_to_permission_result(map: Dynamic) -> PermissionResult {
        let perm_res = rhai::serde::from_dynamic(&map);

        match perm_res {
            Ok(perm_res) => perm_res,
            Err(e) => PermissionResult::GenericError {
                error: format!("Failed to deserialize object map: {}", e),
            },
        }
    }

    /// Creates a new permission result from a JSON string
    #[rhai_fn(name = "json_to_permission_result")]
    pub fn json_to_permission_result(json: &str) -> PermissionResult {
        let perm_res = serde_json::from_str(json);

        match perm_res {
            Ok(perm_res) => perm_res,
            Err(e) => PermissionResult::GenericError {
                error: format!("Failed to deserialize JSON string: {}", e),
            },
        }
    }

    /// Permission check
    pub type PermissionCheck = splashcore_rs::types::silverpelt::PermissionCheck;

    /// Creates a new permission check
    #[rhai_fn(name = "create_permission_check", return_raw)]
    pub fn create_permission_check(
        kittycat_perms: rhai::Array,
        native_perms: rhai::Array,
        outer_and: bool,
        inner_and: bool,
    ) -> Result<PermissionCheck, Box<EvalAltResult>> {
        let mut fperms = Vec::new();

        for perm in kittycat_perms {
            let perm = perm
                .into_string()
                .map_err(|v| format!("Failed to convert permission to string: {}", v))?;

            fperms.push(perm);
        }

        let mut fnative_perms: Vec<serenity::all::Permissions> = Vec::new();

        for perm in native_perms {
            let p: serenity::all::Permissions = rhai::serde::from_dynamic(&perm)
                .map_err(|e| format!("Failed to deserialize native permissions: {}", e))?;

            fnative_perms.push(p);
        }

        Ok(PermissionCheck {
            kittycat_perms: fperms,
            native_perms: fnative_perms,
            inner_and,
            outer_and,
        })
    }

    /// Run permission checks given a members permissions and kittycat permissions
    #[rhai_fn(name = "run", global, return_raw)]
    pub fn run_permission_check(
        check: &mut PermissionCheck,
        member_kittycat_perms: rhai::Array,
        member_native_perms: rhai::Dynamic,
    ) -> Result<PermissionResult, Box<EvalAltResult>> {
        let mut fperms = Vec::new();

        for perm in member_kittycat_perms {
            let perm = KittycatPermission::from(
                perm.into_string()
                    .map_err(|v| format!("Failed to convert permission to string: {}", v))?,
            );

            fperms.push(perm);
        }

        let fnative_perms: serenity::all::Permissions =
            rhai::serde::from_dynamic(&member_native_perms)
                .map_err(|e| format!("Failed to deserialize native permissions: {}", e))?;

        Ok(check_perms_single(check, fnative_perms, &fperms))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rhai::module_resolvers::StaticModuleResolver;

    #[test]
    fn test_message() {
        let mut engine = Engine::new();
        let module = exported_module!(plugin);

        let mut resolver = StaticModuleResolver::new();
        resolver.insert("permissions", module);

        engine.set_module_resolver(resolver);

        // Add the event object for testing to scope
        let mut scope = rhai::Scope::new();

        let dyn_val: rhai::Dynamic = serde_json::from_value(serde_json::json!({
            "a": 123,
            "b": "c"
        }))
        .unwrap();

        scope.set_value("a", dyn_val);

        // a is now defined in the template as the object map #{"a": 123, "b": "c"}

        let script = r#"import "permissions" as permissions;
            let pc = permissions::create_permission_check(["foo.bar"], [8], true, false);
            let result = pc.run(["foo.bar"], '8');

            result.is_ok()
        "#;

        let result: bool = engine.eval_with_scope(&mut scope, script).unwrap();

        assert!(result);
    }
}
