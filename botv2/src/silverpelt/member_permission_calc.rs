use serenity::all::{GuildId, RoleId, UserId};

/// Rederive permissions rederives the permissions given a member id and a list of roles
///
/// Calling rederive_perms has some side-effects
///
/// 0. The member will automatically be added to the guild_members table if they are not already in it
/// 1. Resolved_perms_cache will be updated in the guild_members table
pub async fn rederive_perms(
    pool: &sqlx::PgPool,
    guild_id: GuildId,
    user_id: UserId,
    roles: &[RoleId],
) -> Result<Vec<String>, crate::Error> {
    let roles_str = {
        let mut r = Vec::new();

        for role in roles {
            r.push(role.to_string());
        }

        r.push(guild_id.everyone_role().to_string());

        r
    };

    let mut tx = pool.begin().await?;

    let rec = sqlx::query!(
        "SELECT perm_overrides FROM guild_members WHERE guild_id = $1 AND user_id = $2",
        guild_id.to_string(),
        user_id.to_string()
    )
    .fetch_optional(&mut *tx)
    .await?;

    // Rederive permissions for the new perms
    let role_perms = sqlx::query!(
        "SELECT role_id, perms, index FROM guild_roles WHERE guild_id = $1 AND role_id = ANY($2)",
        guild_id.to_string(),
        &roles_str
    )
    .fetch_all(&mut *tx)
    .await?;

    let mut user_positions = Vec::new();

    for role in role_perms {
        user_positions.push(kittycat::perms::PartialStaffPosition {
            id: role.role_id,
            perms: role.perms,
            index: role.index,
        })
    }

    let (in_db, perm_overrides) = if let Some(rec) = rec {
        (true, rec.perm_overrides)
    } else {
        (false, Vec::new())
    };

    let resolved_perms = kittycat::perms::StaffPermissions {
        user_positions,
        perm_overrides,
    }
    .resolve();

    if in_db {
        sqlx::query!(
            "UPDATE guild_members SET roles = $1, resolved_perms_cache = $2, needs_perm_rederive = false WHERE guild_id = $3 AND user_id = $4",
            &roles_str,
            &resolved_perms,
            guild_id.to_string(),
            user_id.to_string()
        )
        .execute(&mut *tx)
        .await?;
    } else {
        // Check if guild is in the guilds table
        let guild_exists = sqlx::query!(
            "SELECT COUNT(*) FROM guilds WHERE id = $1",
            guild_id.to_string()
        )
        .fetch_one(&mut *tx)
        .await?;

        if guild_exists.count.unwrap_or_default() == 0 {
            sqlx::query!("INSERT INTO guilds (id) VALUES ($1)", guild_id.to_string())
                .execute(&mut *tx)
                .await?;
        }

        sqlx::query!(
            "INSERT INTO guild_members (guild_id, user_id, roles, resolved_perms_cache) VALUES ($1, $2, $3, $4)",
            guild_id.to_string(),
            user_id.to_string(),
            &roles_str,
            &resolved_perms
        )
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;

    Ok(resolved_perms)
}

/// Returns the kittycat permissions of a user. This function also takes into account permission overrides etc.
pub async fn get_kittycat_perms(
    pool: &sqlx::PgPool,
    guild_id: GuildId,
    guild_owner_id: UserId,
    user_id: UserId,
    roles: &[RoleId],
) -> Result<Vec<String>, crate::Error> {
    // For now, owners have full permission, this may change in the future (maybe??)
    if guild_owner_id == user_id {
        return Ok(vec!["global.*".to_string()]);
    }

    let everyone_role = guild_id.everyone_role();

    let rec = sqlx::query!("SELECT roles, needs_perm_rederive, resolved_perms_cache, perm_overrides FROM guild_members WHERE guild_id = $1 AND user_id = $2", guild_id.to_string(), user_id.to_string())
    .fetch_optional(pool)
    .await?;

    if let Some(rec) = rec {
        if rec.needs_perm_rederive {
            return rederive_perms(pool, guild_id, user_id, roles).await;
        }

        // Check user roles against db roles
        let db_roles = rec.roles;

        let mut roles_changed = false;

        for role in roles {
            if !db_roles.contains(&role.to_string()) {
                roles_changed = true;
                break;
            }
        }

        // Check everyone role too
        if !db_roles.contains(&everyone_role.to_string()) {
            roles_changed = true;
        }

        if !roles_changed {
            Ok(rec.resolved_perms_cache) // Then use the resolved perms cache
        } else {
            Ok(rederive_perms(pool, guild_id, user_id, roles).await?)
        }
    } else {
        // They have no column in db, we cannot have a fast-path as the everyone role may have permissions
        Ok(rederive_perms(pool, guild_id, user_id, roles).await?)
    }
}
