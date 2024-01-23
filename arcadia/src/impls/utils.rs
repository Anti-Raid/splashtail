use kittycat::perms::{StaffPermissions, PartialStaffPosition};

use sqlx::PgPool;

/// Get the permissions of a user
/// 
/// Note that while this should be in kittycat, this is not being done right now to avoid coupling
/// kittycat with the database and to contain the database logic in one place.
pub async fn get_user_perms(pool: &PgPool, user_id: &str) -> Result<StaffPermissions, crate::Error> {
    let rec = sqlx::query!("SELECT positions, perm_overrides FROM staff_members WHERE user_id = $1", user_id)
        .fetch_one(pool)
        .await
        .map_err(|e| format!("Error while getting staff perms of user {}: {}", user_id, e))?;

    let pos = sqlx::query!("SELECT id, index, perms FROM staff_positions WHERE id = ANY($1)", &rec.positions)
        .fetch_all(pool)
        .await
        .map_err(|e| format!("Error while getting staff perms of user {}: {}", user_id, e))?;

    Ok(StaffPermissions {
        user_positions: pos.iter().map(|p| PartialStaffPosition {
            id: p.id.hyphenated().to_string(),
            index: p.index,
            perms: p.perms.clone(),
        }).collect(),
        perm_overrides: rec.perm_overrides,
    })
}

