pub mod parse;
pub mod types;

use types::{PermissionCheck, PermissionResult};

pub type Error = Box<dyn std::error::Error + Send + Sync>; // This is constant and should be copy pasted

/// This function runs a permission check on a command
pub fn check_perms(
    check: &PermissionCheck,
    member_native_perms: serenity::all::Permissions,
    member_kittycat_perms: &[kittycat::perms::Permission],
) -> PermissionResult {
    if check.kittycat_perms.is_empty() && check.native_perms.is_empty() {
        return PermissionResult::Ok {}; // Short-circuit if we don't have any permissions to check
    }

    // Check if we have ADMINISTRATOR
    let is_discord_admin = member_native_perms.contains(serenity::all::Permissions::ADMINISTRATOR);

    // Kittycat
    if check.inner_and {
        // inner AND, short-circuit if we don't have the permission
        for perm in &check.kittycat_perms {
            if !kittycat::perms::has_perm(
                member_kittycat_perms,
                &kittycat::perms::Permission::from_string(perm),
            ) {
                return PermissionResult::MissingKittycatPerms {
                    check: check.clone(),
                };
            }
        }

        if !is_discord_admin {
            for perm in &check.native_perms {
                if !member_native_perms.contains(*perm) {
                    return PermissionResult::MissingNativePerms {
                        check: check.clone(),
                    };
                }
            }
        }
    } else {
        // inner OR, short-circuit if we have the permission
        let has_any_np = check
            .native_perms
            .iter()
            .any(|perm| is_discord_admin || member_native_perms.contains(*perm));

        if !has_any_np {
            let has_any_kc = {
                let mut has_kc = false;
                for perm in check.kittycat_perms.iter() {
                    let kc = kittycat::perms::Permission::from_string(perm);

                    if kittycat::perms::has_perm(member_kittycat_perms, &kc) {
                        has_kc = true;
                        break;
                    }
                }

                has_kc
            };

            if !has_any_kc {
                return PermissionResult::MissingAnyPerms {
                    check: check.clone(),
                };
            }
        }
    }

    PermissionResult::Ok {}
}

#[cfg(test)]
mod tests {
    use super::*;

    fn err_with_code(e: PermissionResult, code: &str) -> bool {
        let code_got = e.code();
        println!("test_check_perms: {} == {}", code_got, code);
        code == code_got
    }

    #[test]
    fn test_check_perms() {
        // Basic tests
        assert!(err_with_code(
            check_perms(
                &PermissionCheck {
                    kittycat_perms: vec![],
                    native_perms: vec![serenity::all::Permissions::ADMINISTRATOR],
                    inner_and: false,
                },
                serenity::all::Permissions::empty(),
                &["abc.test".into()],
            ),
            "missing_any_perms"
        ));

        assert!(check_perms(
            &PermissionCheck {
                kittycat_perms: vec![],
                native_perms: vec![],
                inner_and: false,
            },
            serenity::all::Permissions::empty(),
            &["abc.test".into()],
        )
        .is_ok());

        // With inner and
        assert!(err_with_code(
            check_perms(
                &PermissionCheck {
                    kittycat_perms: vec![],
                    native_perms: vec![
                        serenity::all::Permissions::ADMINISTRATOR,
                        serenity::all::Permissions::BAN_MEMBERS
                    ],
                    inner_and: true,
                },
                serenity::all::Permissions::BAN_MEMBERS,
                &["abc.test".into()],
            ),
            "missing_native_perms"
        ));

        // Admin overrides other native perms
        assert!(check_perms(
            &PermissionCheck {
                kittycat_perms: vec![],
                native_perms: vec![serenity::all::Permissions::BAN_MEMBERS],
                inner_and: false,
            },
            serenity::all::Permissions::ADMINISTRATOR,
            &["abc.test".into()],
        )
        .is_ok());

        // Kittycat
        assert!(err_with_code(
            check_perms(
                &PermissionCheck {
                    kittycat_perms: vec!["backups.create".to_string()],
                    native_perms: vec![],
                    inner_and: false,
                },
                serenity::all::Permissions::ADMINISTRATOR,
                &[],
            ),
            "missing_any_perms"
        ));
    }
}
