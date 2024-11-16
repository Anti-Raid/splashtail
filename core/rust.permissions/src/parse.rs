use crate::types::PermissionCheck;

const MAX_KITTYCAT_PERMS: usize = 10;
const MAX_INDIVIDUAL_KITTYCAT_PERM_SIZE: usize = 128;
const MAX_NATIVE_PERMS: usize = 10;

// Parses a user-inputted PermissionCheck object into a parsed PermissionCheck object.
pub async fn parse_permission_check(
    check: &PermissionCheck,
) -> Result<PermissionCheck, crate::Error> {
    if check.kittycat_perms.is_empty() && check.native_perms.is_empty() {
        return Ok(check.clone());
    }

    let mut parsed_check = PermissionCheck {
        kittycat_perms: check.kittycat_perms.clone(),
        native_perms: check.native_perms.clone(),
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
        return Err(format!("too many native perms: {}", parsed_check.native_perms.len()).into());
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

    Ok(parsed_check)
}
