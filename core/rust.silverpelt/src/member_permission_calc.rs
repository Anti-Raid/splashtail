use kittycat::perms::Permission;
use serenity::all::{GuildId, RoleId, UserId};

/// ``create_roles_list_for_guild`` creates a list of roles for a guild including the everyone role as a string
///
/// This is needed by other functions to rederive permissions such as ``rederive_perms_impl`` and ``get_user_positions_from_db``
pub fn create_roles_list_for_guild(roles: &[RoleId], guild_id: GuildId) -> Vec<String> {
    let mut roles_str = Vec::new();

    for role in roles {
        roles_str.push(role.to_string());
    }

    roles_str.push(guild_id.everyone_role().to_string());

    roles_str
}

/// Returns the user positions of the member. This can be useful for caching or to reduce DB calls
///
/// ``roles_str`` is the list of roles as strings. This can be obtained by calling ``create_roles_list_for_guild``
pub async fn get_user_positions_from_db(
    conn: &mut sqlx::PgConnection,
    guild_id: GuildId,
    roles_str: &[String],
) -> Result<Vec<kittycat::perms::PartialStaffPosition>, crate::Error> {
    // Rederive permissions for the new perms
    let role_perms = sqlx::query!(
        "SELECT role_id, perms, index FROM guild_roles WHERE guild_id = $1 AND role_id = ANY($2)",
        guild_id.to_string(),
        &roles_str
    )
    .fetch_all(&mut *conn)
    .await?;

    let mut user_positions = Vec::new();

    for role in role_perms {
        user_positions.push(kittycat::perms::PartialStaffPosition {
            id: role.role_id,
            perms: role
                .perms
                .iter()
                .map(|x| Permission::from_string(x))
                .collect(),
            index: role.index,
        })
    }

    Ok(user_positions)
}

/// Rederive permissions rederives the permissions given a member id and a list of roles
///
/// Calling rederive_perms has some side-effects
///
/// 0. The member will automatically be added to the guild_members table if they are not already in it
/// 1. Resolved_perms_cache will be updated in the guild_members table
pub fn rederive_perms_impl(
    guild_id: GuildId,
    user_id: UserId,
    user_positions: Vec<kittycat::perms::PartialStaffPosition>,
    perm_overrides: Vec<Permission>,
) -> Vec<Permission> {
    // We hardcode root users for the main server to ensure root users have control over the bot even under extreme circumstances
    if guild_id == config::CONFIG.servers.main.get()
        && config::CONFIG.discord_auth.root_users.contains(&user_id)
    {
        return vec!["global.*".into()];
    }

    let resolved_perms = kittycat::perms::StaffPermissions {
        user_positions,
        perm_overrides,
    }
    .resolve();

    resolved_perms
}

/// Rederive permissions rederives the permissions given a member id and a list of roles
///
/// Calling rederive_perms_and_update_db has some side-effects. Use rederive_perms_impl if you do not want to update the database
///
/// 0. The member will automatically be added to the guild_members table if they are not already in it
/// 1. Resolved_perms_cache will be updated in the guild_members table
pub async fn rederive_perms_and_update_db(
    conn: &mut sqlx::PgConnection,
    guild_id: GuildId,
    user_id: UserId,
    roles: &[RoleId],
) -> Result<Vec<Permission>, crate::Error> {
    let rec = sqlx::query!(
        "SELECT perm_overrides FROM guild_members WHERE guild_id = $1 AND user_id = $2",
        guild_id.to_string(),
        user_id.to_string()
    )
    .fetch_optional(&mut *conn)
    .await?;

    let (in_db, perm_overrides) = if let Some(rec) = rec {
        (
            true,
            rec.perm_overrides
                .iter()
                .map(|x| Permission::from_string(x))
                .collect(),
        )
    } else {
        (false, Vec::new())
    };

    let roles_str = create_roles_list_for_guild(roles, guild_id);
    let user_positions = get_user_positions_from_db(&mut *conn, guild_id, &roles_str).await?;

    let resolved_perms = rederive_perms_impl(guild_id, user_id, user_positions, perm_overrides);

    if in_db {
        sqlx::query!(
            "UPDATE guild_members SET roles = $1, resolved_perms_cache = $2, needs_perm_rederive = false WHERE guild_id = $3 AND user_id = $4",
            &roles_str,
            &resolved_perms.iter().map(|x| x.to_string()).collect::<Vec<String>>(),
            guild_id.to_string(),
            user_id.to_string()
        )
        .execute(&mut *conn)
        .await?;
    } else {
        // Check if guild is in the guilds table
        let guild_exists = sqlx::query!(
            "SELECT COUNT(*) FROM guilds WHERE id = $1",
            guild_id.to_string()
        )
        .fetch_one(&mut *conn)
        .await?;

        if guild_exists.count.unwrap_or_default() == 0 {
            sqlx::query!("INSERT INTO guilds (id) VALUES ($1)", guild_id.to_string())
                .execute(&mut *conn)
                .await?;
        }

        sqlx::query!(
            "INSERT INTO guild_members (guild_id, user_id, roles, resolved_perms_cache) VALUES ($1, $2, $3, $4)",
            guild_id.to_string(),
            user_id.to_string(),
            &roles_str,
            &resolved_perms.iter().map(|x| x.to_string()).collect::<Vec<String>>()
        )
        .execute(&mut *conn)
        .await?;
    }

    Ok(resolved_perms)
}

/// Returns the kittycat permissions of a user. This function also takes into account permission overrides etc.
pub async fn get_kittycat_perms(
    conn: &mut sqlx::PgConnection,
    guild_id: GuildId,
    guild_owner_id: UserId,
    user_id: UserId,
    roles: &[RoleId],
) -> Result<Vec<Permission>, crate::Error> {
    // For now, owners have full permission, this may change in the future (maybe??)
    if guild_owner_id == user_id {
        return Ok(vec!["global.*".into()]);
    }

    let everyone_role = guild_id.everyone_role();

    let rec = sqlx::query!("SELECT roles, needs_perm_rederive, resolved_perms_cache, perm_overrides FROM guild_members WHERE guild_id = $1 AND user_id = $2", guild_id.to_string(), user_id.to_string())
    .fetch_optional(&mut *conn)
    .await?;

    if let Some(rec) = rec {
        if rec.needs_perm_rederive {
            return rederive_perms_and_update_db(&mut *conn, guild_id, user_id, roles).await;
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
            Ok(rec
                .resolved_perms_cache
                .iter()
                .map(|x| Permission::from_string(x))
                .collect::<Vec<Permission>>()) // Then use the resolved perms cache
        } else {
            Ok(rederive_perms_and_update_db(&mut *conn, guild_id, user_id, roles).await?)
        }
    } else {
        // They have no column in db, we cannot have a fast-path as the everyone role may have permissions
        Ok(rederive_perms_and_update_db(&mut *conn, guild_id, user_id, roles).await?)
    }
}
