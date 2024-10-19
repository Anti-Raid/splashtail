use permissions::types::{PermissionCheck, PermissionChecks};

const MAX_PERM_CHECK: usize = 10;
const MAX_KITTYCAT_PERMS: usize = 10;
const MAX_INDIVIDUAL_KITTYCAT_PERM_SIZE: usize = 128;
const MAX_NATIVE_PERMS: usize = 10;

// Parses a user-inputted PermissionChecks object into a parsed PermissionChecks object.
pub async fn parse_permission_checks(
    guild_id: serenity::all::GuildId,
    pool: sqlx::PgPool,
    cache_http: botox::cache::CacheHttpImpl,
    pc: &PermissionChecks,
) -> Result<PermissionChecks, crate::Error> {
    match pc {
        PermissionChecks::Simple { checks } => {
            if checks.len() > MAX_PERM_CHECK {
                return Err(format!("too many checks: {}", checks.len()).into());
            }

            let mut parsed_checks = Vec::with_capacity(checks.len());
            for check in checks {
                if check.kittycat_perms.is_empty() && check.native_perms.is_empty() {
                    continue;
                }

                let mut parsed_check = PermissionCheck {
                    kittycat_perms: check.kittycat_perms.clone(),
                    native_perms: check.native_perms.clone(),
                    outer_and: check.outer_and,
                    inner_and: check.inner_and,
                };

                if parsed_check.kittycat_perms.len() > MAX_KITTYCAT_PERMS {
                    return Err(format!(
                        "too many kittycat perms: {}",
                        parsed_check.kittycat_perms.len()
                    )
                    .into());
                }

                if parsed_check.native_perms.len() > MAX_NATIVE_PERMS {
                    return Err(format!(
                        "too many native perms: {}",
                        parsed_check.native_perms.len()
                    )
                    .into());
                }

                for native_perm in &mut parsed_check.native_perms {
                    let native_perm_without_unknown_bits = native_perm.iter_names().fold(
                        serenity::model::permissions::Permissions::empty(),
                        |acc, (_p_name, perm)| acc | perm,
                    );

                    *native_perm = native_perm_without_unknown_bits;
                }

                for perm in &parsed_check.kittycat_perms {
                    if perm.len() > MAX_INDIVIDUAL_KITTYCAT_PERM_SIZE {
                        return Err(format!(
                            "kittycat perm too long: max={}",
                            MAX_INDIVIDUAL_KITTYCAT_PERM_SIZE
                        )
                        .into());
                    }
                }

                parsed_checks.push(parsed_check);
            }

            Ok(PermissionChecks::Simple {
                checks: parsed_checks,
            })
        }
        PermissionChecks::Template { template } => {
            templating::parse(guild_id, template, pool, cache_http).await?;
            Ok(PermissionChecks::Template {
                template: template.clone(),
            })
        }
    }
}
