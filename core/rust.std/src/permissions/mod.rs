use crate::types::silverpelt::{PermissionCheck, PermissionResult};

/// This function runs a single permission check on a command without taking any branching decisions
///
/// This may be useful when mocking or visualizing a permission check
pub fn check_perms_single(
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
