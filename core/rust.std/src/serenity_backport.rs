//use extract_map::ExtractMap;
use extract_map::ExtractMap;
use log::{error, warn};
use serenity::all::{GuildId, Member, PartialGuild, Permissions, Role, RoleId, UserId};

pub fn member_permissions(guild: &PartialGuild, member: &Member) -> Permissions {
    user_permissions(
        member.user.id,
        &member.roles,
        guild.id,
        &guild.roles,
        guild.owner_id,
    )
}

/// Helper function that can also be used from [`PartialGuild`].
/// Backported from https://github.com/serenity-rs/serenity/blob/efb5820cd5dd3325dff767bb2f96d380715dce30/src/model/guild/mod.rs
pub fn user_permissions(
    member_user_id: UserId,
    member_roles: &[RoleId],
    guild_id: GuildId,
    guild_roles: &ExtractMap<RoleId, Role>,
    guild_owner_id: UserId,
) -> Permissions {
    calculate_permissions(CalculatePermissions {
        is_guild_owner: member_user_id == guild_owner_id,
        everyone_permissions: if let Some(role) = guild_roles.get(&RoleId::new(guild_id.get())) {
            role.permissions
        } else {
            error!("@everyone role missing in {}", guild_id);
            Permissions::empty()
        },
        user_roles_permissions: member_roles
            .iter()
            .map(|role_id| {
                if let Some(role) = guild_roles.get(role_id) {
                    role.permissions
                } else {
                    warn!(
                        "{} on {} has non-existent role {:?}",
                        member_user_id, guild_id, role_id
                    );
                    Permissions::empty()
                }
            })
            .collect(),
    })
}

struct CalculatePermissions {
    /// Whether the guild member is the guild owner
    pub is_guild_owner: bool,
    /// Base permissions given to @everyone (guild level)
    pub everyone_permissions: Permissions,
    /// Permissions allowed to a user by their roles (guild level)
    pub user_roles_permissions: Vec<Permissions>,
}

/// Translated from the pseudo code at https://discord.com/developers/docs/topics/permissions#permission-overwrites
///
/// The comments within this file refer to the above link
fn calculate_permissions(data: CalculatePermissions) -> Permissions {
    if data.is_guild_owner {
        return Permissions::all();
    }

    // 1. Base permissions given to @everyone are applied at a guild level
    let mut permissions = data.everyone_permissions;
    // 2. Permissions allowed to a user by their roles are applied at a guild level
    for role_permission in data.user_roles_permissions {
        permissions |= role_permission;
    }

    if permissions.contains(Permissions::ADMINISTRATOR) {
        return Permissions::all();
    }

    permissions
}
