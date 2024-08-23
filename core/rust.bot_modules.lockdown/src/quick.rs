use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// The base permissions for quick lockdown
///
/// If any of these permissions are provided, quick lockdown cannot proceed
static BASE_PERMS: [serenity::model::permissions::Permissions; 2] = [
    serenity::all::Permissions::VIEW_CHANNEL,
    serenity::all::Permissions::SEND_MESSAGES,
    //serenity::all::Permissions::SEND_MESSAGES_IN_THREADS,
    //serenity::all::Permissions::CONNECT,
];

static LOCKDOWN_PERMS: std::sync::LazyLock<serenity::model::permissions::Permissions> =
    std::sync::LazyLock::new(|| serenity::all::Permissions::VIEW_CHANNEL);

#[derive(Debug, Serialize, Deserialize, Clone, Copy, Hash, PartialEq)]
pub enum ChangeOp {
    Add,
    Remove,
}

impl std::fmt::Display for ChangeOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ChangeOp::Add => write!(f, "Add"),
            ChangeOp::Remove => write!(f, "Remove"),
        }
    }
}

/// The result of a `test_quick_lockdown` call
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct QuickLockdownTestResult {
    /// Which roles need to be changed/fixed combined with the target perms
    pub changes_needed:
        std::collections::HashMap<serenity::all::RoleId, (ChangeOp, serenity::all::Permissions)>,
    /// The critical roles (either member roles or the `@everyone` role)
    pub critical_roles: HashSet<serenity::all::RoleId>,
}

impl QuickLockdownTestResult {
    /// Returns whether the guild is in a state where quick lockdown can be applied perfectly
    pub fn can_apply_perfectly(&self) -> bool {
        self.changes_needed.is_empty()
    }
}

/// Returns the critical roles given a [PartialGuild](`serenity::all::PartialGuild`) and a set of member roles
pub fn get_critical_roles(
    pg: &serenity::all::PartialGuild,
    member_roles: &HashSet<serenity::all::RoleId>,
) -> Result<HashSet<serenity::all::RoleId>, silverpelt::Error> {
    if member_roles.is_empty() {
        // Find the everyone role
        let everyone_role = pg
            .roles
            .iter()
            .find(|r| r.id.get() == pg.id.get())
            .ok_or_else(|| silverpelt::Error::from("No @everyone role found"))?;

        Ok(std::iter::once(everyone_role.id).collect())
    } else {
        Ok(member_roles.clone())
    }
}

/// Given a [PartialGuild](`serenity::all::PartialGuild`) and a set of member roles, `test_quick_lockdown` will check if the guild meets the requirements for quick lockdown.
///
/// The requirements for quick lockdown are listed in README.md and the basic idea is listed below:
///
/// - One can define a set of critical roles which are either the member roles or the ``@everyone`` role, all other roles must not have View Channel, Send Messages and/or Send Messages In Threads permissions
pub async fn test_quick_lockdown(
    pg: &serenity::all::PartialGuild,
    member_roles: &HashSet<serenity::all::RoleId>,
) -> Result<QuickLockdownTestResult, silverpelt::Error> {
    let critical_roles = get_critical_roles(pg, member_roles)?;

    let mut changes_needed = std::collections::HashMap::new();

    // From here on out, we only need to care about critical and non critical roles
    for role in pg.roles.iter() {
        if critical_roles.contains(&role.id) {
            let mut needed_perms = serenity::all::Permissions::empty();

            let mut missing = false;
            for perm in BASE_PERMS {
                if !role.permissions.contains(perm) {
                    needed_perms |= perm;
                    missing = true;
                }
            }

            if missing {
                changes_needed.insert(role.id, (ChangeOp::Add, needed_perms));
            }
        } else {
            let mut perms_to_remove = serenity::all::Permissions::empty();

            let mut needs_perms_removed = false;
            for perm in BASE_PERMS {
                if role.permissions.contains(perm) {
                    perms_to_remove |= perm;
                    needs_perms_removed = true;
                }
            }

            if needs_perms_removed {
                changes_needed.insert(role.id, (ChangeOp::Remove, perms_to_remove));
            }
        }
    }

    Ok(QuickLockdownTestResult {
        changes_needed,
        critical_roles,
    })
}

/// Creates a new quick lockdown given a [PartialGuild](`serenity::all::PartialGuild`) and a set of critical roles
///
/// This is achieved by **removing** the `BASE_PERMS` from the critical roles
///
/// NOTE: Callers must *save* the old role permissions before calling this function
pub async fn create_quick_lockdown(
    ctx: &botox::cache::CacheHttpImpl,
    pg: &mut serenity::all::PartialGuild,
    critical_roles: HashSet<serenity::all::RoleId>,
) -> Result<(), silverpelt::Error> {
    let mut new_roles = Vec::new();
    for role in pg.roles.iter() {
        if critical_roles.contains(&role.id) {
            new_roles.push(
                pg.id
                    .edit_role(
                        &ctx.http,
                        role.id,
                        serenity::all::EditRole::new().permissions(*LOCKDOWN_PERMS),
                    )
                    .await?,
            );
        }
    }

    for role in new_roles {
        pg.roles.insert(role);
    }

    Ok(())
}

/// Reverts a quick lockdown given a [PartialGuild](`serenity::all::PartialGuild`) and a set of critical roles
///
/// This is achieved by RESTORING the old role permissions
pub async fn revert_quick_lockdown(
    ctx: &botox::cache::CacheHttpImpl,
    pg: &mut serenity::all::PartialGuild,
    critical_roles: HashSet<serenity::all::RoleId>,
    old_permissions: std::collections::HashMap<serenity::all::RoleId, serenity::all::Permissions>,
) -> Result<(), silverpelt::Error> {
    let mut new_roles = Vec::new();
    for role in pg.roles.iter() {
        if critical_roles.contains(&role.id) {
            let perms = old_permissions.get(&role.id).copied().unwrap_or(
                BASE_PERMS
                    .iter()
                    .copied()
                    .fold(serenity::all::Permissions::empty(), |acc, perm| acc | perm),
            );

            new_roles.push(
                pg.id
                    .edit_role(
                        &ctx.http,
                        role.id,
                        serenity::all::EditRole::new().permissions(perms),
                    )
                    .await?,
            );
        }
    }

    for role in new_roles {
        pg.roles.insert(role);
    }

    Ok(())
}
