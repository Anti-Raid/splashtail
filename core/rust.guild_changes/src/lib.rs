/// NOTE: This is *completely* experimental and will most likely be dropped
use moka::future::Cache;
use serde::{Deserialize, Serialize};
use serenity::all::Mentionable;
use std::collections::BTreeSet;
use strum_macros::{EnumString, VariantNames};
use tokio::sync::RwLock;

/// A single change that has been made to a server
///
/// Note that a single change may end up being updated as new information becomes available
#[derive(EnumString, PartialEq, VariantNames, Clone, Debug, Serialize, Hash, Eq, Deserialize)]
#[strum(serialize_all = "snake_case")]
pub enum GuildChange {
    /// A new member has joined the guild
    MemberAdd {
        /// The user id of the member that has joined
        user_id: serenity::all::UserId,
    },
    /// A member has updated their roles
    MemberRolesUpdated {
        /// The initiator of the role update
        ///
        /// Is optional if we don't know the initiator yet
        initiator: Option<serenity::all::UserId>,
        /// The target user id whose roles have been updated
        target: serenity::all::UserId,
        /// The old roles of the user
        old_roles: BTreeSet<serenity::all::UserId>,
        /// The new roles of the user
        new_roles: BTreeSet<serenity::all::UserId>,
    },
    /// A role has been created on the server
    RoleCreate {
        /// The initiator of the role update
        ///
        /// Is optional if we don't know the initiator yet
        initiator: Option<serenity::all::UserId>,
        /// The role id of the role that has been created
        role_id: serenity::all::RoleId,
    },
    RoleUpdate {
        /// The initiator of the role update
        ///
        /// Is optional if we don't know the initiator yet
        initiator: Option<serenity::all::UserId>,
        /// The role id of the role that has been created
        role_id: serenity::all::RoleId,
    },
    RoleRemove {
        /// The initiator of the role update
        ///
        /// Is optional if we don't know the initiator yet
        initiator: Option<serenity::all::UserId>,
        /// The role id of the role that has been created
        role_id: serenity::all::RoleId,
    },
    ChannelCreate {
        /// The initiator of the role update
        ///
        /// Is optional if we don't know the initiator yet
        initiator: Option<serenity::all::UserId>,
        /// The channel id of the channel that has been created
        channel_id: serenity::all::ChannelId,
    },
    ChannelUpdate {
        /// The initiator of the role update
        ///
        /// Is optional if we don't know the initiator yet
        initiator: Option<serenity::all::UserId>,
        /// The channel id of the channel that has been updated
        channel_id: serenity::all::ChannelId,
    },
    ChannelRemove {
        /// The initiator of the role update
        ///
        /// Is optional if we don't know the initiator yet
        initiator: Option<serenity::all::UserId>,
        /// The channel id of the channel that has been removed
        channel_id: serenity::all::ChannelId,
    },
    Kick {
        /// The initiator of the role update
        ///
        /// Is optional if we don't know the initiator yet
        initiator: Option<serenity::all::UserId>,
        /// The user id of the user that has been kicked
        user_id: serenity::all::UserId,
    },
    Ban {
        /// The initiator of the role update
        ///
        /// Is optional if we don't know the initiator yet
        initiator: Option<serenity::all::UserId>,
        /// The user id of the user that has been banned
        user_id: serenity::all::UserId,
    },
    Unban {
        /// The initiator of the role update
        ///
        /// Is optional if we don't know the initiator yet
        initiator: Option<serenity::all::UserId>,
        /// The user id of the user that has been unbanned
        user_id: serenity::all::UserId,
    },
    PruneMembers {
        /// The initiator of the role update
        ///
        /// Is optional if we don't know the initiator yet
        initiator: Option<serenity::all::UserId>,
        /// The number of members that have been pruned
        pruned_members: u64,
    },
}

impl GuildChange {
    /// Get a description of the change
    pub fn change_description(&self) -> String {
        match self {
            Self::MemberAdd { user_id } => format!("New member: {}", user_id),
            Self::MemberRolesUpdated {
                initiator,
                target,
                old_roles,
                new_roles,
            } => {
                // Get changes. Using BTreeSet guarantees an order
                let added_roles: Vec<_> = new_roles.difference(old_roles).collect();
                let removed_roles: Vec<_> = old_roles.difference(new_roles).collect();

                // Create description
                let mut description = format!(
                    "Added roles: {}",
                    added_roles
                        .into_iter()
                        .map(|r| r.mention().to_string())
                        .collect::<Vec<_>>()
                        .join(", ")
                );
                description.push_str(&format!(
                    "\nRemoved roles: {}",
                    removed_roles
                        .into_iter()
                        .map(|r| r.mention().to_string())
                        .collect::<Vec<_>>()
                        .join(", ")
                ));

                if let Some(initiator) = initiator {
                    description.push_str(&format!("\nInitiator: {}", initiator.mention()));
                }

                description.push_str(&format!("\nTarget: {}", target.mention()));

                description
            }
            Self::RoleCreate { initiator, role_id } => {
                if let Some(initiator) = initiator {
                    format!("Role created by {}: {}", initiator, role_id.mention())
                } else {
                    format!("Role created: {}", role_id.mention())
                }
            }
            Self::RoleUpdate { initiator, role_id } => {
                if let Some(initiator) = initiator {
                    format!("Role updated by {}: {}", initiator, role_id.mention())
                } else {
                    format!("Role updated: {}", role_id.mention())
                }
            }
            Self::RoleRemove { initiator, role_id } => {
                if let Some(initiator) = initiator {
                    format!("Role removed by {}: {}", initiator, role_id)
                } else {
                    format!("Role removed: {}", role_id)
                }
            }
            Self::ChannelCreate {
                initiator,
                channel_id,
            } => {
                if let Some(initiator) = initiator {
                    format!("Channel created by {}: {}", initiator, channel_id.mention())
                } else {
                    format!("Channel created: {}", channel_id.mention())
                }
            }
            Self::ChannelUpdate {
                initiator,
                channel_id,
            } => {
                if let Some(initiator) = initiator {
                    format!("Channel updated by {}: {}", initiator, channel_id.mention())
                } else {
                    format!("Channel updated: {}", channel_id.mention())
                }
            }
            Self::ChannelRemove {
                initiator,
                channel_id,
            } => {
                if let Some(initiator) = initiator {
                    format!("Channel removed by {}: {}", initiator, channel_id)
                } else {
                    format!("Channel removed: {}", channel_id)
                }
            }
            Self::Kick { initiator, user_id } => {
                if let Some(initiator) = initiator {
                    format!("User kicked by {}: {}", initiator, user_id)
                } else {
                    format!("User kicked: {}", user_id)
                }
            }
            Self::Ban { initiator, user_id } => {
                if let Some(initiator) = initiator {
                    format!("User banned by {}: {}", initiator, user_id)
                } else {
                    format!("User banned: {}", user_id)
                }
            }
            Self::Unban { initiator, user_id } => {
                if let Some(initiator) = initiator {
                    format!("User unbanned by {}: {}", initiator, user_id)
                } else {
                    format!("User unbanned: {}", user_id)
                }
            }
            Self::PruneMembers {
                initiator,
                pruned_members,
            } => {
                if let Some(initiator) = initiator {
                    format!("{} members pruned by {}", pruned_members, initiator)
                } else {
                    format!("{} members pruned", pruned_members)
                }
            }
        }
    }
}

impl std::fmt::Display for GuildChange {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.change_description())
    }
}

#[derive(Debug)]
pub struct GuildChangeList {
    /// The changes that have been made to the server
    pub changes: RwLock<Cache<u64, GuildChange>>,
}

impl GuildChangeList {
    pub fn new() -> Self {
        Self {
            changes: RwLock::new(
                Cache::builder()
                    .support_invalidation_closures()
                    .time_to_idle(std::time::Duration::from_secs(300))
                    .build(),
            ),
        }
    }

    pub async fn add(&self, created_change: GuildChange) {
        let change_guard = self.changes.write().await;

        let mut curr_key = 0;
        let mut have_coalesced = false; // Flag to store whether we have combined this change with another one

        match created_change {
            GuildChange::Ban {
                initiator: found_initiator,
                user_id: found_user_id,
            } => {
                while change_guard.contains_key(&curr_key) {
                    let change = change_guard.get(&curr_key).await.unwrap();

                    match change {
                        GuildChange::Ban { user_id, initiator } => {
                            if user_id != found_user_id {
                                curr_key += 1;
                                continue; // Skip to the next change
                            }

                            if initiator.is_none() {
                                change_guard
                                    .insert(
                                        curr_key,
                                        GuildChange::Ban {
                                            user_id: user_id,
                                            initiator: found_initiator,
                                        },
                                    )
                                    .await;
                                have_coalesced = true;
                            }
                        }
                        _ => {}
                    }

                    curr_key += 1;
                }
            }
            GuildChange::Kick {
                initiator: found_initiator,
                user_id: found_user_id,
            } => {
                while change_guard.contains_key(&curr_key) {
                    let change = change_guard.get(&curr_key).await.unwrap();

                    match change {
                        GuildChange::Kick { user_id, initiator } => {
                            if user_id != found_user_id {
                                curr_key += 1;
                                continue; // Skip to the next change
                            }

                            if initiator.is_none() {
                                change_guard
                                    .insert(
                                        curr_key,
                                        GuildChange::Kick {
                                            user_id: user_id,
                                            initiator: found_initiator,
                                        },
                                    )
                                    .await;
                                have_coalesced = true;
                            }
                        }
                        _ => {}
                    }
                }

                curr_key += 1;
            }
            GuildChange::Unban {
                initiator: found_initiator,
                user_id: found_user_id,
            } => {
                while change_guard.contains_key(&curr_key) {
                    let change = change_guard.get(&curr_key).await.unwrap();

                    match change {
                        GuildChange::Unban { user_id, initiator } => {
                            if user_id != found_user_id {
                                curr_key += 1;
                                continue; // Skip to the next change
                            }

                            if initiator.is_none() {
                                change_guard
                                    .insert(
                                        curr_key,
                                        GuildChange::Unban {
                                            user_id: user_id,
                                            initiator: found_initiator,
                                        },
                                    )
                                    .await;
                                have_coalesced = true;
                            }
                        }
                        _ => {}
                    }
                }

                curr_key += 1;
            }
            GuildChange::MemberRolesUpdated {
                initiator: found_initiator,
                target: found_target,
                old_roles: ref found_old_roles,
                new_roles: ref found_new_roles,
            } => {
                while change_guard.contains_key(&curr_key) {
                    let change = change_guard.get(&curr_key).await.unwrap();

                    match change {
                        GuildChange::MemberRolesUpdated {
                            initiator,
                            target,
                            old_roles,
                            new_roles,
                        } => {
                            if target != found_target {
                                curr_key += 1;
                                continue; // Skip to the next change
                            }

                            if initiator.is_none() {
                                change_guard
                                    .insert(
                                        curr_key,
                                        GuildChange::MemberRolesUpdated {
                                            initiator: found_initiator,
                                            target: target,
                                            old_roles: old_roles.clone(),
                                            new_roles: new_roles.clone(),
                                        },
                                    )
                                    .await;
                                have_coalesced = true;
                            } else {
                                // Check if we can merge the changes
                                if let Some(found_initiator) = found_initiator {
                                    if let Some(initiator) = initiator {
                                        if initiator == found_initiator {
                                            let mut new_roles = new_roles.clone();
                                            new_roles.extend(found_new_roles.clone());

                                            let mut old_roles = old_roles.clone();
                                            old_roles.extend(found_old_roles.clone());

                                            change_guard
                                                .insert(
                                                    curr_key,
                                                    GuildChange::MemberRolesUpdated {
                                                        initiator: Some(initiator),
                                                        target: target,
                                                        old_roles: old_roles,
                                                        new_roles: new_roles,
                                                    },
                                                )
                                                .await;
                                            have_coalesced = true;
                                        }
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                }

                curr_key += 1;
            }
            GuildChange::RoleCreate {
                initiator: found_initiator,
                role_id: found_role_id,
            } => {
                while change_guard.contains_key(&curr_key) {
                    let change = change_guard.get(&curr_key).await.unwrap();

                    match change {
                        GuildChange::RoleCreate { initiator, role_id } => {
                            if role_id != found_role_id {
                                curr_key += 1;
                                continue; // Skip to the next change
                            }

                            if initiator.is_none() {
                                change_guard
                                    .insert(
                                        curr_key,
                                        GuildChange::RoleCreate {
                                            initiator: found_initiator,
                                            role_id: role_id,
                                        },
                                    )
                                    .await;
                                have_coalesced = true;
                            }
                        }
                        _ => {}
                    }
                }

                curr_key += 1;
            }
            GuildChange::RoleRemove {
                initiator: found_initiator,
                role_id: found_role_id,
            } => {
                while change_guard.contains_key(&curr_key) {
                    let change = change_guard.get(&curr_key).await.unwrap();

                    match change {
                        GuildChange::RoleRemove { initiator, role_id } => {
                            if role_id != found_role_id {
                                curr_key += 1;
                                continue; // Skip to the next change
                            }

                            if initiator.is_none() {
                                change_guard
                                    .insert(
                                        curr_key,
                                        GuildChange::RoleRemove {
                                            initiator: found_initiator,
                                            role_id: role_id,
                                        },
                                    )
                                    .await;
                                have_coalesced = true;
                            }
                        }
                        _ => {}
                    }
                }

                curr_key += 1;
            }
            GuildChange::RoleUpdate {
                initiator: found_initiator,
                role_id: found_role_id,
            } => {
                while change_guard.contains_key(&curr_key) {
                    let change = change_guard.get(&curr_key).await.unwrap();

                    match change {
                        GuildChange::RoleUpdate { initiator, role_id } => {
                            if role_id != found_role_id {
                                curr_key += 1;
                                continue; // Skip to the next change
                            }

                            if initiator.is_none() {
                                change_guard
                                    .insert(
                                        curr_key,
                                        GuildChange::RoleUpdate {
                                            initiator: found_initiator,
                                            role_id: role_id,
                                        },
                                    )
                                    .await;
                                have_coalesced = true;
                            }
                        }
                        _ => {}
                    }
                }

                curr_key += 1;
            }
            GuildChange::ChannelCreate {
                initiator: found_initiator,
                channel_id: found_channel_id,
            } => {
                while change_guard.contains_key(&curr_key) {
                    let change = change_guard.get(&curr_key).await.unwrap();

                    match change {
                        GuildChange::ChannelCreate {
                            initiator,
                            channel_id,
                        } => {
                            if channel_id != found_channel_id {
                                curr_key += 1;
                                continue; // Skip to the next change
                            }

                            if initiator.is_none() {
                                change_guard
                                    .insert(
                                        curr_key,
                                        GuildChange::ChannelCreate {
                                            initiator: found_initiator,
                                            channel_id: channel_id,
                                        },
                                    )
                                    .await;
                                have_coalesced = true;
                            }
                        }
                        _ => {}
                    }
                }

                curr_key += 1;
            }
            GuildChange::ChannelRemove {
                initiator: found_initiator,
                channel_id: found_channel_id,
            } => {
                while change_guard.contains_key(&curr_key) {
                    let change = change_guard.get(&curr_key).await.unwrap();

                    match change {
                        GuildChange::ChannelRemove {
                            initiator,
                            channel_id,
                        } => {
                            if channel_id != found_channel_id {
                                curr_key += 1;
                                continue; // Skip to the next change
                            }

                            if initiator.is_none() {
                                change_guard
                                    .insert(
                                        curr_key,
                                        GuildChange::ChannelRemove {
                                            initiator: found_initiator,
                                            channel_id: channel_id,
                                        },
                                    )
                                    .await;
                                have_coalesced = true;
                            }
                        }
                        _ => {}
                    }
                }

                curr_key += 1;
            }
            GuildChange::ChannelUpdate {
                initiator: found_initiator,
                channel_id: found_channel_id,
            } => {
                while change_guard.contains_key(&curr_key) {
                    let change = change_guard.get(&curr_key).await.unwrap();

                    match change {
                        GuildChange::ChannelUpdate {
                            initiator,
                            channel_id,
                        } => {
                            if channel_id != found_channel_id {
                                curr_key += 1;
                                continue; // Skip to the next change
                            }

                            if initiator.is_none() {
                                change_guard
                                    .insert(
                                        curr_key,
                                        GuildChange::ChannelUpdate {
                                            initiator: found_initiator,
                                            channel_id: channel_id,
                                        },
                                    )
                                    .await;
                                have_coalesced = true;
                            }
                        }
                        _ => {}
                    }
                }

                curr_key += 1;
            }
            _ => {
                while change_guard.contains_key(&curr_key) {
                    curr_key += 1;
                }
            }
        };

        if !have_coalesced {
            change_guard.insert(curr_key, created_change).await;
        }
    }

    /// Returns the busy-ness of the guild
    ///
    /// Kick => +5
    /// Ban => +10
    /// PruneMember => +100
    /// _ => +1
    pub async fn busyness(&self) -> u64 {
        let change_guard = self.changes.read().await;

        let mut busyness = 0;
        for (_, change) in change_guard.iter() {
            match change {
                GuildChange::Kick { .. } => busyness += 5,
                GuildChange::Ban { .. } => busyness += 10,
                GuildChange::PruneMembers { .. } => busyness += 100,
                _ => busyness += 1,
            }
        }

        busyness
    }

    pub async fn count(&self, opt: GuildChangeCountOption) -> u64 {
        let change_guard = self.changes.read().await;

        match opt {
            GuildChangeCountOption::MemberRolesUpdated {
                initiator: found_initiator,
            } => {
                let mut count = 0;

                for (_, change) in change_guard.iter() {
                    match change {
                        GuildChange::MemberRolesUpdated { initiator, .. } => {
                            let Some(initiator) = initiator else {
                                continue;
                            };

                            if initiator == found_initiator {
                                count += 1;
                            }
                        }
                        _ => {}
                    }
                }

                count
            }
            GuildChangeCountOption::RoleCreate {
                initiator: found_initiator,
            } => {
                let mut count = 0;

                for (_, change) in change_guard.iter() {
                    match change {
                        GuildChange::RoleCreate { initiator, .. } => {
                            let Some(initiator) = initiator else {
                                continue;
                            };

                            if initiator == found_initiator {
                                count += 1;
                            }
                        }
                        _ => {}
                    }
                }

                count
            }
            GuildChangeCountOption::RoleUpdate {
                initiator: found_initiator,
            } => {
                let mut count = 0;

                for (_, change) in change_guard.iter() {
                    match change {
                        GuildChange::RoleUpdate { initiator, .. } => {
                            let Some(initiator) = initiator else {
                                continue;
                            };

                            if initiator == found_initiator {
                                count += 1;
                            }
                        }
                        _ => {}
                    }
                }

                count
            }
            GuildChangeCountOption::RoleRemove {
                initiator: found_initiator,
            } => {
                let mut count = 0;

                for (_, change) in change_guard.iter() {
                    match change {
                        GuildChange::RoleRemove { initiator, .. } => {
                            let Some(initiator) = initiator else {
                                continue;
                            };

                            if initiator == found_initiator {
                                count += 1;
                            }
                        }
                        _ => {}
                    }
                }

                count
            }
            GuildChangeCountOption::ChannelCreate {
                initiator: found_initiator,
            } => {
                let mut count = 0;

                for (_, change) in change_guard.iter() {
                    match change {
                        GuildChange::ChannelCreate { initiator, .. } => {
                            let Some(initiator) = initiator else {
                                continue;
                            };

                            if initiator == found_initiator {
                                count += 1;
                            }
                        }
                        _ => {}
                    }
                }

                count
            }
            GuildChangeCountOption::ChannelUpdate {
                initiator: found_initiator,
            } => {
                let mut count = 0;

                for (_, change) in change_guard.iter() {
                    match change {
                        GuildChange::ChannelUpdate { initiator, .. } => {
                            let Some(initiator) = initiator else {
                                continue;
                            };

                            if initiator == found_initiator {
                                count += 1;
                            }
                        }
                        _ => {}
                    }
                }

                count
            }
            GuildChangeCountOption::ChannelRemove {
                initiator: found_initiator,
            } => {
                let mut count = 0;

                for (_, change) in change_guard.iter() {
                    match change {
                        GuildChange::ChannelRemove { initiator, .. } => {
                            let Some(initiator) = initiator else {
                                continue;
                            };

                            if initiator == found_initiator {
                                count += 1;
                            }
                        }
                        _ => {}
                    }
                }

                count
            }
            GuildChangeCountOption::Kick {
                initiator: found_initiator,
            } => {
                let mut count = 0;

                for (_, change) in change_guard.iter() {
                    match change {
                        GuildChange::Kick { initiator, .. } => {
                            let Some(initiator) = initiator else {
                                continue;
                            };

                            if initiator == found_initiator {
                                count += 1;
                            }
                        }
                        _ => {}
                    }
                }

                count
            }
            GuildChangeCountOption::Ban {
                initiator: found_initiator,
            } => {
                let mut count = 0;

                for (_, change) in change_guard.iter() {
                    match change {
                        GuildChange::Ban { initiator, .. } => {
                            let Some(initiator) = initiator else {
                                continue;
                            };

                            if initiator == found_initiator {
                                count += 1;
                            }
                        }
                        _ => {}
                    }
                }

                count
            }
            GuildChangeCountOption::Unban {
                initiator: found_initiator,
            } => {
                let mut count = 0;

                for (_, change) in change_guard.iter() {
                    match change {
                        GuildChange::Unban { initiator, .. } => {
                            let Some(initiator) = initiator else {
                                continue;
                            };

                            if initiator == found_initiator {
                                count += 1;
                            }
                        }
                        _ => {}
                    }
                }

                count
            }
            GuildChangeCountOption::PruneMembers {
                initiator: found_initiator,
            } => {
                let mut count = 0;

                for (_, change) in change_guard.iter() {
                    match change {
                        GuildChange::PruneMembers { initiator, .. } => {
                            let Some(initiator) = initiator else {
                                continue;
                            };

                            if initiator == found_initiator {
                                count += 1;
                            }
                        }
                        _ => {}
                    }
                }

                count
            }
        }
    }
}

pub enum GuildChangeCountOption {
    MemberRolesUpdated { initiator: serenity::all::UserId },
    RoleCreate { initiator: serenity::all::UserId },
    RoleUpdate { initiator: serenity::all::UserId },
    RoleRemove { initiator: serenity::all::UserId },
    ChannelCreate { initiator: serenity::all::UserId },
    ChannelUpdate { initiator: serenity::all::UserId },
    ChannelRemove { initiator: serenity::all::UserId },
    Kick { initiator: serenity::all::UserId },
    Ban { initiator: serenity::all::UserId },
    Unban { initiator: serenity::all::UserId },
    PruneMembers { initiator: serenity::all::UserId },
}
