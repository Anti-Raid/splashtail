use log::{error, info};
use poise::serenity_prelude::{FullEvent, RoleAction};
use serenity::model::guild::audit_log::{Action, ChannelAction};
use std::collections::HashSet;

use crate::{Data, Error};

pub async fn event_listener(_ctx: &serenity::client::Context, event: &FullEvent, user_data: &Data) -> Result<(), Error> {
    match event {
        FullEvent::GuildAuditLogEntryCreate {
            entry,
            guild_id,
        } => {
            info!("Audit log created: {:?}. Guild: {}", entry, guild_id);

            super::handler::precheck_guild(&user_data.pool, *guild_id).await
                .map_err(|e| {
                    info!("Pre-check for guild {} failed: {}", guild_id, e);
                    e
                })?;

            let res = match entry.action {
                Action::Channel(ch) => {
                    let ch_id = entry.target_id.ok_or("No channel ID found")?;

                    match ch {
                        ChannelAction::Create => {
                            info!("Channel created: {}", ch_id);

                            super::handler::handle_mod_action(
                                *guild_id,
                                entry.user_id,
                                &user_data.pool,
                                &user_data.cache_http,
                                super::core::UserLimitTypes::ChannelAdd,
                                &serde_json::json!({
                                    "channel_id": ch_id.to_string(),
                                }),
                            )
                            .await
                        }
                        ChannelAction::Delete => {
                            info!("Channel deleted: {}", ch_id);

                            super::handler::handle_mod_action(
                                *guild_id,
                                entry.user_id,
                                &user_data.pool,
                                &user_data.cache_http,
                                super::core::UserLimitTypes::ChannelRemove,
                                &serde_json::json!(
                                    {
                                        "channel_id": ch_id.to_string(),
                                    }
                                )
                            )
                            .await
                        }
                        ChannelAction::Update => {
                            info!("Channel updated: {}", ch_id);

                            super::handler::handle_mod_action(
                                *guild_id,
                                entry.user_id,
                                &user_data.pool,
                                &user_data.cache_http,
                                super::core::UserLimitTypes::ChannelUpdate,
                                &serde_json::json!(
                                    {
                                        "channel_id": ch_id.to_string(),
                                    }
                                )
                            )
                            .await
                        }
                        _ => Ok(()),
                    }
                }
                Action::Role(ra) => {
                    let r_id = entry.target_id.ok_or("No role ID found")?;

                    match ra {
                        RoleAction::Create => {
                            info!("Role created: {}", r_id);

                            super::handler::handle_mod_action(
                                *guild_id,
                                entry.user_id,
                                &user_data.pool,
                                &user_data.cache_http,
                                super::core::UserLimitTypes::RoleAdd,
                                &serde_json::json!(
                                    {
                                        "role_id": r_id.to_string(),
                                    }
                                )
                            )
                            .await
                        }
                        RoleAction::Update => {
                            info!("Role updated: {}", r_id);

                            super::handler::handle_mod_action(
                                *guild_id,
                                entry.user_id,
                                &user_data.pool,
                                &user_data.cache_http,
                                super::core::UserLimitTypes::RoleUpdate,
                                &serde_json::json!(
                                    {
                                        "role_id": r_id.to_string(),
                                    }
                                ),
                            )
                            .await
                        }
                        RoleAction::Delete => {
                            info!("Role deleted: {}", r_id);

                            super::handler::handle_mod_action(
                                *guild_id,
                                entry.user_id,
                                &user_data.pool,
                                &user_data.cache_http,
                                super::core::UserLimitTypes::RoleRemove,
                                &serde_json::json!(
                                    {
                                        "role_id": r_id.to_string(),
                                    }
                                ),
                            )
                            .await
                        }
                        _ => Ok(()),
                    }
                },
                _ => Ok(()),
            };

            if let Err(res) = res {
                error!("Error while handling audit log: {}", res);
                return Err(res);
            }

            Ok(())
        },
        FullEvent::GuildMemberUpdate { old_if_available, new, event } => {
            super::handler::precheck_guild(&user_data.pool, event.guild_id).await?;

            let Some(old) = old_if_available else {
                error!("No old member found");
                return Err("No old member found".into());
            };

            let old_roles = old.roles.clone();

            let new_roles = if let Some(new) = new {
                new.roles.clone()
            } else {
                event.roles.clone()
            };

            // Get old and new roles
            let mut old_roles_hs = HashSet::new();
            let mut new_roles_hs = HashSet::new();

            for role in old_roles.iter() {
                old_roles_hs.insert(role);
            }
            
            for role in new_roles.iter() {
                new_roles_hs.insert(role);
            }

            let diff = old_roles_hs.symmetric_difference(&new_roles_hs).collect::<Vec<_>>();

            if !diff.is_empty() {
                info!("Roles changed for user {} in guild {}", event.user.id, event.guild_id);

                super::handler::handle_mod_action(
                    event.guild_id,
                    event.user.id,
                    &user_data.pool,
                    &user_data.cache_http,
                    super::core::UserLimitTypes::MemberRolesUpdated,
                    &serde_json::json!(
                        {
                            "old": old_roles_hs,
                            "new": new_roles_hs
                        }
                    ),
                )
                .await?;

                let mut added = Vec::new();
                let mut removed = Vec::new();

                for role in new_roles.iter() {
                    if !old_roles.contains(role) {
                        added.push(role);
                    }
                }

                for role in old_roles.iter() {
                    if !new_roles.contains(role) {
                        removed.push(role);
                    }
                }

                if !added.is_empty() {
                    info!("Added roles: {:?}", added);

                    super::handler::handle_mod_action(
                        event.guild_id,
                        event.user.id,
                        &user_data.pool,
                        &user_data.cache_http,
                        super::core::UserLimitTypes::RoleGivenToMember,
                        &serde_json::json!(
                            {
                                "old": old_roles_hs,
                                "new": new_roles_hs,
                                "added": added,
                                "removed": removed
                            }
                        ),
                    )
                    .await?;
                }

                if !removed.is_empty() {
                    info!("Removed roles: {:?}", removed);

                    super::handler::handle_mod_action(
                        event.guild_id,
                        event.user.id,
                        &user_data.pool,
                        &user_data.cache_http,
                        super::core::UserLimitTypes::RoleRemovedFromMember,
                        &serde_json::json!(
                            {
                                "old": old_roles_hs,
                                "new": new_roles_hs,
                                "added": added,
                                "removed": removed
                            }
                        ),
                    )
                    .await?;
                }
            }

            Ok(())
        },
        _ => Ok(())
    }
}