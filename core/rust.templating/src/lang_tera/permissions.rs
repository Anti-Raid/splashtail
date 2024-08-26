use permissions::check_perms_single;
use permissions_types::{PermissionCheck, PermissionResult};
use std::sync::{Arc, RwLock};
use tera::Tera;

pub struct InternalTemplateExecuteState {
    /// The current native permissions of the member
    member_native_perms: serenity::all::Permissions,
    /// The current kittycat permissions of the member
    member_kittycat_perms: Vec<kittycat::perms::Permission>,
    /// The current permission result
    result: RwLock<Option<PermissionResult>>,
}

// Has kittycat perm function
pub struct HasKittycatPermFunction {
    state: Arc<InternalTemplateExecuteState>,
}

// has_kittycat_permission(perm = string)
impl tera::Function for HasKittycatPermFunction {
    fn call(
        &self,
        args: &std::collections::HashMap<String, tera::Value>,
    ) -> tera::Result<tera::Value> {
        let perm = args
            .get("perm")
            .ok_or("missing perm")?
            .as_str()
            .ok_or("perm is not an array")?;

        let res = kittycat::perms::has_perm(
            &self.state.member_kittycat_perms,
            &kittycat::perms::Permission::from(perm),
        );

        Ok(serde_json::Value::Bool(res))
    }
}

// Run permission check function
pub struct RunPermissionCheckFunction {
    state: Arc<InternalTemplateExecuteState>,
}

// run_permission_check(kittycat_perms = string[], native_permissions = Permissions, inner_and = BOOLEAN)
impl tera::Function for RunPermissionCheckFunction {
    fn call(
        &self,
        args: &std::collections::HashMap<String, tera::Value>,
    ) -> tera::Result<tera::Value> {
        let kittycat_perms = args
            .get("kittycat_perms")
            .ok_or("missing kittycat_perms")?
            .as_array()
            .ok_or("kittycat_perms is not an array")?
            .iter()
            .map(|v| {
                v.as_str()
                    .ok_or("kittycat_perm is not a string")
                    .map(|s| s.to_string())
            })
            .collect::<Result<Vec<String>, _>>()?;

        let native_perms = match args.get("native_perms").ok_or("missing native_perms")? {
            tera::Value::String(s) => {
                vec![serenity::all::Permissions::from_bits_truncate(
                    s.parse::<u64>()
                        .map_err(|_| "native_perms is not a number")?,
                )]
            }
            tera::Value::Number(n) => {
                vec![serenity::all::Permissions::from_bits_truncate(
                    n.as_u64().ok_or("native_perms is not a number")?,
                )]
            }
            tera::Value::Array(a) => a
                .iter()
                .map(|v| match v {
                    tera::Value::String(s) => Ok(serenity::all::Permissions::from_bits_truncate(
                        s.parse::<u64>()
                            .map_err(|_| "native_perms is not a number")?,
                    )),
                    tera::Value::Number(n) => Ok(serenity::all::Permissions::from_bits_truncate(
                        n.as_u64().ok_or("native_perms is not a number")?,
                    )),
                    _ => Err("native_perms is not a number"),
                })
                .collect::<Result<Vec<serenity::all::Permissions>, _>>()?,
            _ => return Err("native_perms is not a number".into()),
        };

        let check_all = args
            .get("check_all")
            .ok_or("missing check_all")?
            .as_bool()
            .ok_or("check_all is not a boolean")?;

        let res = check_perms_single(
            &PermissionCheck {
                kittycat_perms,
                native_perms,
                inner_and: check_all,
                outer_and: false,
            },
            self.state.member_native_perms,
            &self.state.member_kittycat_perms,
        );

        let value = serde_json::to_value(&res)
            .map_err(|e| format!("failed to serialize PermissionResult: {}", e))?;

        Ok(serde_json::json!({
            "ok": res.is_ok(),
            "result": value,
        }))
    }
}

// Run permission check function
struct PermissionResultFilter {
    state: Arc<InternalTemplateExecuteState>,
}

// permission_result (result=PermissionResult)
impl tera::Filter for PermissionResultFilter {
    fn filter(
        &self,
        value: &tera::Value,
        _args: &std::collections::HashMap<String, tera::Value>,
    ) -> tera::Result<tera::Value> {
        let value = match value {
            tera::Value::String(s) => serde_json::from_str::<PermissionResult>(s),
            tera::Value::Object(m) => {
                serde_json::from_value::<PermissionResult>(tera::Value::Object(m.clone()))
            }
            _ => return Err("value is not a string".into()),
        }
        .map_err(|e| format!("failed to parse PermissionResult: {}", e))?;

        let mut writer = self
            .state
            .result
            .write()
            .map_err(|_| "failed to lock result")?;

        *writer = Some(value);

        Ok(tera::Value::Null)
    }
}

pub async fn execute_permissions_template(
    tera: &mut Tera,
    tctx: crate::core::PermissionTemplateContext,
) -> PermissionResult {
    let mut context = tera::Context::new();

    if let Err(e) = context.insert("ctx", &tctx) {
        return PermissionResult::GenericError {
            error: format!("failed to insert permission context into context: {}", e),
        };
    }

    let state = Arc::new(InternalTemplateExecuteState {
        member_native_perms: tctx.member_native_permissions,
        member_kittycat_perms: tctx.member_kittycat_permissions,
        result: RwLock::new(None),
    });

    tera.register_function(
        "run_permission_check",
        RunPermissionCheckFunction {
            state: state.clone(),
        },
    );

    tera.register_function(
        "has_kittycat_permission",
        HasKittycatPermFunction {
            state: state.clone(),
        },
    );

    tera.register_filter(
        "permission_result",
        PermissionResultFilter {
            state: state.clone(),
        },
    );

    // Execute the template
    match super::execute_template(tera, &context).await {
        Ok(r) => r,
        Err(e) => {
            return PermissionResult::GenericError {
                error: format!("failed to execute template: {}", e),
            }
        }
    };

    let mut writer = match state
        .result
        .write()
        .map_err(|e| format!("failed to lock result: {:?}", e))
    {
        Ok(r) => r,
        Err(e) => return PermissionResult::GenericError { error: e },
    };

    (*writer).take().unwrap_or(PermissionResult::Ok {})
}
