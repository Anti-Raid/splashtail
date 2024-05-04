/// From name_split, construct a list of all permutations of the command name from the root till the end
///
/// E.g: If subcommand is `limits hit`, then `limits` and `limits hit` will be constructed
///     as the list of commands to check
/// E.g 2: If subcommand is `limits hit add`, then `limits`, `limits hit` and `limits hit add`
///     will be constructed as the list of commands to check
pub fn permute_command_names(name: &str) -> Vec<String> {
    // Check if subcommand by splitting the command name
    let name_split = name.split(' ').collect::<Vec<&str>>();

    let mut commands_to_check = Vec::new();

    for i in 0..name_split.len() {
        let mut command = String::new();

        for (j, cmd) in name_split.iter().enumerate().take(i + 1) {
            command += cmd;

            if j != i {
                command += " ";
            }
        }

        commands_to_check.push(command);
    }

    commands_to_check
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_permute_command_names() {
        assert_eq!(permute_command_names(""), vec![""]);
        assert_eq!(permute_command_names("limits"), vec!["limits"]);
        assert_eq!(
            permute_command_names("limits hit"),
            vec!["limits", "limits hit"]
        );
        assert_eq!(
            permute_command_names("limits hit add"),
            vec!["limits", "limits hit", "limits hit add"]
        );
    }
}

pub mod serenity_utils {
    use serenity::all::{PartialGuild, UserId, Member, Role, RoleId};

    /// Gets the highest role a [`Member`] of this Guild has.
    ///
    /// Returns None if the member has no roles or the member from this guild.
    /// 
    /// Taken from https://serenity-rs.github.io/serenity/next/src/serenity/model/guild/mod.rs.html#1233-1278
    /// with patches to accept a partialguild
    pub fn member_highest_role<'a>(pg: &'a PartialGuild, member: &Member) -> Option<&'a Role> {
        let mut highest: Option<&Role> = None;

        for role_id in &member.roles {
            if let Some(role) = pg.roles.get(role_id) {
                // Skip this role if this role in iteration has:
                // - a position less than the recorded highest
                // - a position equal to the recorded, but a higher ID
                if let Some(highest) = highest {
                    if role.position < highest.position
                        || (role.position == highest.position && role.id > highest.id)
                    {
                        continue;
                    }
                }

                highest = Some(role);
            }
        }

        highest
    }

    /// Returns which of two [`User`]s has a higher [`Member`] hierarchy.
    ///
    /// Hierarchy is essentially who has the [`Role`] with the highest [`position`].
    ///
    /// Returns [`None`] if at least one of the given users' member instances is not present.
    /// Returns [`None`] if the users have the same hierarchy, as neither are greater than the
    /// other.
    ///
    /// If both user IDs are the same, [`None`] is returned. If one of the users is the guild
    /// owner, their ID is returned.
    ///
    /// [`position`]: Role::position
    ///
    /// Taken from https://serenity-rs.github.io/serenity/next/src/serenity/model/guild/mod.rs.html#1233-1278 
    /// with changes to use a `Member` object for lhs/rhs
    pub fn greater_member_hierarchy(pg: &PartialGuild, lhs: &Member, rhs: &Member) -> Option<UserId> {
        // Check that the IDs are the same. If they are, neither is greater.
        if lhs.user.id == rhs.user.id {
            return None;
        }

        // Check if either user is the guild owner.
        if lhs.user.id == pg.owner_id {
            return Some(lhs.user.id);
        } else if rhs.user.id == pg.owner_id {
            return Some(rhs.user.id);
        }

        let lhs_role = member_highest_role(pg, lhs)
            .map_or((RoleId::new(1), 0), |r| (r.id, r.position));

        let rhs_role = member_highest_role(pg, rhs)
            .map_or((RoleId::new(1), 0), |r| (r.id, r.position));

        // If LHS and RHS both have no top position or have the same role ID, then no one wins.
        if (lhs_role.1 == 0 && rhs_role.1 == 0) || (lhs_role.0 == rhs_role.0) {
            return None;
        }

        // If LHS's top position is higher than RHS, then LHS wins.
        if lhs_role.1 > rhs_role.1 {
            return Some(lhs.user.id);
        }

        // If RHS's top position is higher than LHS, then RHS wins.
        if rhs_role.1 > lhs_role.1 {
            return Some(rhs.user.id);
        }

        // If LHS and RHS both have the same position, but LHS has the lower role ID, then LHS
        // wins.
        //
        // If RHS has the higher role ID, then RHS wins.
        if lhs_role.1 == rhs_role.1 && lhs_role.0 < rhs_role.0 {
            Some(lhs.user.id)
        } else {
            Some(rhs.user.id)
        }
    }
}