use log::{error, info};
use poise::serenity_prelude::{FullEvent, RoleAction, MemberAction, Change};
use serenity::model::guild::audit_log::{Action, ChannelAction};

use crate::{Data, Error};

pub async fn event_listener(_ctx: &serenity::client::Context, event: &FullEvent, user_data: &Data) -> Result<(), Error> {
    match event {
        FullEvent::GuildAuditLogEntryCreate {
            entry,
            guild_id,
        } => {
            info!("Audit log created: {:?}. Guild: {}", entry, guild_id);

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
                Action::Member(ma) => {
                    let Some(target) = entry.target_id else {
                        error!("MEMBER update: No target ID found");
                        return Err("No target ID found".into());
                    };

                    #[allow(clippy::single_match)] // Plans to add further events are coming
                    match ma {
                        MemberAction::RoleUpdate => {
                            let Some(ref changes) = entry.changes else {
                                error!("MEMBER update: No changes found");
                                return Err("No changes found".into());
                            };

                            let mut added = Vec::new();
                            let mut removed = Vec::new();
                            let mut old_roles = Vec::new();

                            for change in changes {
                                match change {
                                    Change::RolesAdded { old, new } => {
                                        let Some(old) = old else {
                                            error!("MEMBER update: No old roles found");
                                            continue;
                                        };

                                        if old_roles.is_empty() {
                                            for role in old.iter() {
                                               old_roles.push(role.id);
                                            }
                                        }

                                        let Some(new) = new else {
                                            error!("MEMBER update: No new roles found");
                                            continue;
                                        };

                                        for role in new.iter() {
                                            if !old.contains(role) {
                                                added.push(role.id);
                                            }
                                        }
                                    },
                                    Change::RolesRemove {
                                        old,
                                        new,
                                    } => {
                                        let Some(old) = old else {
                                            error!("MEMBER update: No old roles found");
                                            continue;
                                        };

                                        if old_roles.is_empty() {
                                            for role in old.iter() {
                                                old_roles.push(role.id);
                                            }
                                        }

                                        let Some(new) = new else {
                                            error!("MEMBER update: No new roles found");
                                            continue;
                                        };

                                        for role in old.iter() {
                                            if !new.contains(role) {
                                                removed.push(role.id);
                                            }
                                        }
                                    },
                                    _ => {}
                                }
                            }
                            
                            super::handler::handle_mod_action(
                                *guild_id,
                                entry.user_id,
                                &user_data.pool,
                                &user_data.cache_http,
                                super::core::UserLimitTypes::MemberRolesUpdated,
                                &serde_json::json!(
                                    {
                                        "old": old_roles,
                                        "added": added,
                                        "removed": removed,
                                        "target": target.to_string(),
                                    }
                                ),
                            )
                            .await?;

                            if !added.is_empty() {
                                info!("Added roles: {:?}", added);

                                for role in added.iter() {
                                    super::handler::handle_mod_action(
                                        *guild_id,
                                        entry.user_id,
                                        &user_data.pool,
                                        &user_data.cache_http,
                                        super::core::UserLimitTypes::RoleGivenToMember,
                                        &serde_json::json!(
                                            {
                                                "old": old_roles,
                                                "added": added,
                                                "removed": removed,
                                                "for": role.to_string(),
                                                "target": target.to_string(),
                                            }
                                        ),
                                    )
                                    .await?;
                                }
                            }

                            if !removed.is_empty() {
                                info!("Removed roles: {:?}", removed);

                                for role in removed.iter() {
                                    super::handler::handle_mod_action(
                                        *guild_id,
                                        entry.user_id,
                                        &user_data.pool,
                                        &user_data.cache_http,
                                        super::core::UserLimitTypes::RoleRemovedFromMember,
                                        &serde_json::json!(
                                            {
                                                "old": old_roles,
                                                "added": added,
                                                "removed": removed,
                                                "for": role.to_string(),
                                                "target": target.to_string(),
                                            }
                                        ),
                                    )
                                    .await?;
                                }
                            }
                        }
                        _ => {}
                    }

                    Ok(())
                },
                _ => Ok(()),
            };

            if let Err(res) = res {
                error!("Error while handling audit log: {}", res);
                return Err(res);
            }

            Ok(())
        },
        _ => Ok(()),
    }
}