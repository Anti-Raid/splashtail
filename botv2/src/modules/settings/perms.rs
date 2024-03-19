use crate::silverpelt::member_permission_calc::get_kittycat_perms;
use poise::CreateReply;
use serenity::all::{Role, RoleId};

#[poise::command(
    prefix_command,
    slash_command,
    user_cooldown = 1,
    guild_cooldown = 1,
    subcommands("perms_modrole", "perms_list", "perms_deleterole")
)]
pub async fn perms(_ctx: crate::Context<'_>) -> Result<(), crate::Error> {
    Ok(())
}

/// Lists all roles with the setup permission and index
#[poise::command(
    prefix_command,
    slash_command,
    user_cooldown = 1,
    guild_cooldown = 1,
    rename = "list"
)]
pub async fn perms_list(ctx: crate::Context<'_>) -> Result<(), crate::Error> {
    let data = ctx.data();

    let Some(guild_id) = ctx.guild_id() else {
        return Err("You must be in a server to run this command".into());
    };

    let mut tx = data.pool.begin().await?;

    let roles = sqlx::query!(
        "SELECT role_id, perms, index FROM guild_roles WHERE guild_id = $1 ORDER BY index",
        guild_id.to_string()
    )
    .fetch_all(&mut *tx)
    .await?;

    let mut embed = serenity::all::CreateEmbed::default()
        .title("Configured Roles")
        .description("The roles with setup permissions and their indexes");

    for role in roles {
        let Ok(role_id) = role.role_id.parse::<RoleId>() else {
            continue;
        };

        embed = embed.field(
            format!("<@&{}>", role_id),
            format!(
                "ID: {}, Permissions: {}\nIndex: {}",
                role_id,
                role.perms.join(", "),
                role.index
            ),
            false,
        );
    }

    ctx.send(CreateReply::default().embed(embed)).await?;

    Ok(())
}

/// Edits the permissions for a specific role
#[poise::command(
    prefix_command,
    slash_command,
    user_cooldown = 1,
    guild_cooldown = 1,
    rename = "modrole"
)]
pub async fn perms_modrole(
    ctx: crate::Context<'_>,
    #[description = "The role to edit"] role: Role,
    #[description = "The permissions to set, separated by commas"] perms: String,
    #[description = "The index of the role"] index: Option<i32>,
) -> Result<(), crate::Error> {
    let mut perms_vec = Vec::new();

    for perm in perms.split(',') {
        if !perm.contains('.') {
            return Err("Invalid permission format. Permission must be in format `<namespace>.<permission>`".into());
        }

        perms_vec.push(perm.to_string());
    }

    let data = ctx.data();

    let Some(member) = ctx.author_member().await else {
        return Err("You must be in a server to run this command".into());
    };

    {
        let Some(guild) = ctx.guild() else {
            return Err("You must be in a server to run this command".into());
        };

        // Get highest role of user if not owner
        if guild.owner_id != member.user.id {
            let Some(first_role) = member.roles.first() else {
                return Err("You must have at least one role to run this command!".into());
            };

            let Some(first_role) = guild.roles.get(first_role) else {
                return Err("You must have at least one role to run this command!".into());
            };

            let mut highest_role = first_role;

            for r in &member.roles {
                let Some(r) = guild.roles.get(r) else {
                    continue;
                };
                if r.position > highest_role.position {
                    highest_role = r;
                }
            }

            if highest_role.position < role.position {
                return Err("You do not have permission to edit this role's permissions as they are higher than you".into());
            }
        }
    }

    let Some(guild_id) = ctx.guild_id() else {
        return Err("You must be in a server to run this command".into());
    };

    let author_kittycat_perms =
        get_kittycat_perms(&data.pool, guild_id, member.user.id, &member.roles).await?;

    let mut tx = data.pool.begin().await?;

    let current = sqlx::query!(
        "SELECT perms FROM guild_roles WHERE guild_id = $1 AND role_id = $2",
        guild_id.to_string(),
        role.id.to_string()
    )
    .fetch_optional(&mut *tx)
    .await?;

    if let Some(current) = current {
        kittycat::perms::check_patch_changes(&author_kittycat_perms, &current.perms, &perms_vec)
            .map_err(|e| {
                format!(
                    "You do not have permission to edit this role's permissions: {}",
                    e
                )
            })?;

        sqlx::query!(
            "UPDATE guild_roles SET perms = $1 WHERE guild_id = $2 AND role_id = $3",
            &perms_vec,
            guild_id.to_string(),
            role.id.to_string()
        )
        .execute(&mut *tx)
        .await?;

        if let Some(index) = index {
            // Check for index collisions
            let existing = sqlx::query!(
                "SELECT role_id FROM guild_roles WHERE guild_id = $1 AND index = $2",
                guild_id.to_string(),
                index
            )
            .fetch_optional(&mut *tx)
            .await?;

            if existing.is_some() {
                // To avoid index collisions, take all indexes higher than the given index and add one to them
                sqlx::query!(
                    "UPDATE guild_roles SET index = index + 1 WHERE guild_id = $1 AND index >= $2 AND role_id != $3",
                    guild_id.to_string(),
                    index,
                    role.id.to_string()
                )
                .execute(&mut *tx)
                .await?;
            }

            sqlx::query!(
                "UPDATE guild_roles SET index = $1 WHERE guild_id = $2 AND role_id = $3",
                index,
                guild_id.to_string(),
                role.id.to_string()
            )
            .execute(&mut *tx)
            .await?;
        }
    } else {
        let true_index = {
            if index.is_none() {
                // First fetch highest index and add one to go to top
                let highest_index = sqlx::query!(
                    "SELECT MAX(index) FROM guild_roles WHERE guild_id = $1",
                    guild_id.to_string()
                )
                .fetch_one(&mut *tx)
                .await?;

                highest_index.max.unwrap_or(0) + 1
            } else {
                // Check for index collisions
                let index = index.unwrap();

                let existing = sqlx::query!(
                    "SELECT role_id FROM guild_roles WHERE guild_id = $1 AND index = $2",
                    guild_id.to_string(),
                    index
                )
                .fetch_optional(&mut *tx)
                .await?;

                if existing.is_some() {
                    // To avoid index collisions, take all indexes higher than the given index and add one to them
                    sqlx::query!(
                        "UPDATE guild_roles SET index = index + 1 WHERE guild_id = $1 AND index >= $2 AND role_id != $3",
                        guild_id.to_string(),
                        index,
                        role.id.to_string()
                    )
                    .execute(&mut *tx)
                    .await?;
                }

                index
            }
        };

        sqlx::query!(
            "INSERT INTO guild_roles (guild_id, role_id, perms, index) VALUES ($1, $2, $3, $4)",
            guild_id.to_string(),
            role.id.to_string(),
            &perms_vec,
            true_index
        )
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;

    ctx.say("Permissions updated successfully for role").await?;

    Ok(())
}

/// Deletes role configuration
#[poise::command(
    prefix_command,
    slash_command,
    user_cooldown = 1,
    guild_cooldown = 1,
    rename = "deleterole"
)]
pub async fn perms_deleterole(
    ctx: crate::Context<'_>,
    #[description = "The role to delete"] role: Role,
) -> Result<(), crate::Error> {
    let data = ctx.data();

    let Some(member) = ctx.author_member().await else {
        return Err("You must be in a server to run this command".into());
    };

    {
        let Some(guild) = ctx.guild() else {
            return Err("You must be in a server to run this command".into());
        };

        // Get highest role of user if not owner
        if guild.owner_id != member.user.id {
            let Some(first_role) = member.roles.first() else {
                return Err("You must have at least one role to run this command!".into());
            };

            let Some(first_role) = guild.roles.get(first_role) else {
                return Err("You must have at least one role to run this command!".into());
            };

            let mut highest_role = first_role;

            for r in &member.roles {
                let Some(r) = guild.roles.get(r) else {
                    continue;
                };
                if r.position > highest_role.position {
                    highest_role = r;
                }
            }

            if highest_role.position < role.position {
                return Err("You do not have permission to edit this role's permissions as they are higher than you".into());
            }
        }
    }

    let Some(guild_id) = ctx.guild_id() else {
        return Err("You must be in a server to run this command".into());
    };

    let author_kittycat_perms =
        get_kittycat_perms(&data.pool, guild_id, member.user.id, &member.roles).await?;

    let mut tx = data.pool.begin().await?;

    let current = sqlx::query!(
        "SELECT perms FROM guild_roles WHERE guild_id = $1 AND role_id = $2",
        guild_id.to_string(),
        role.id.to_string()
    )
    .fetch_optional(&mut *tx)
    .await?;

    if current.is_none() {
        return Err("Role has not been configured yet!".into());
    }

    let current = current.unwrap();

    // Check if the user has permission to delete the role (that is, permissions to remove all permissions)
    if !current.perms.is_empty() {
        kittycat::perms::check_patch_changes(&author_kittycat_perms, &current.perms, &[]).map_err(
            |e| {
                format!(
                    "You do not have permission to delete this role's permissions: {}",
                    e
                )
            },
        )?;
    }

    sqlx::query!(
        "DELETE FROM guild_roles WHERE guild_id = $1 AND role_id = $2",
        guild_id.to_string(),
        role.id.to_string()
    )
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    ctx.say("Role configuration deleted successfully").await?;

    Ok(())
}
