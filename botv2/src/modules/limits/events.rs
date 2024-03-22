use log::{error, info};
use poise::serenity_prelude::{Change, FullEvent, MemberAction, RoleAction};
use serenity::model::guild::audit_log::{Action, ChannelAction};

use super::handler::handle_mod_action;
use crate::{silverpelt::EventHandlerContext, Data, Error};

pub async fn event_listener(
    ctx: &serenity::client::Context,
    event: &FullEvent,
    _: EventHandlerContext,
) -> Result<(), Error> {
    let user_data = ctx.data::<Data>();

    let cache_http = bothelpers::cache::CacheHttpImpl {
        cache: ctx.cache.clone(),
        http: ctx.http.clone(),
    };

    match event {
        FullEvent::GuildAuditLogEntryCreate { entry, guild_id } => {
            info!("Audit log created: {:?}. Guild: {}", entry, guild_id);

            let res = match entry.action {
                Action::Channel(ch) => {
                    let ch_id = entry.target_id.ok_or("No channel ID found")?;

                    match ch {
                        ChannelAction::Create => {
                            info!("Channel created: {}", ch_id);

                            handle_mod_action(
                                &user_data.pool,
                                &user_data.surreal_cache,
                                &cache_http,
                                &super::handler::HandleModAction {
                                    guild_id: *guild_id,
                                    limit: super::core::UserLimitTypes::ChannelAdd,
                                    user_id: entry.user_id,
                                    target: ch_id.to_string(),
                                    action_data: serde_json::json!({}),
                                },
                            )
                            .await
                        }
                        ChannelAction::Delete => {
                            info!("Channel deleted: {}", ch_id);

                            handle_mod_action(
                                &user_data.pool,
                                &user_data.surreal_cache,
                                &cache_http,
                                &super::handler::HandleModAction {
                                    guild_id: *guild_id,
                                    limit: super::core::UserLimitTypes::ChannelRemove,
                                    user_id: entry.user_id,
                                    target: ch_id.to_string(),
                                    action_data: serde_json::json!({}),
                                },
                            )
                            .await
                        }
                        ChannelAction::Update => {
                            info!("Channel updated: {}", ch_id);
                            handle_mod_action(
                                &user_data.pool,
                                &user_data.surreal_cache,
                                &cache_http,
                                &super::handler::HandleModAction {
                                    guild_id: *guild_id,
                                    limit: super::core::UserLimitTypes::ChannelUpdate,
                                    user_id: entry.user_id,
                                    target: ch_id.to_string(),
                                    action_data: serde_json::json!({}),
                                },
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

                            handle_mod_action(
                                &user_data.pool,
                                &user_data.surreal_cache,
                                &cache_http,
                                &super::handler::HandleModAction {
                                    guild_id: *guild_id,
                                    limit: super::core::UserLimitTypes::RoleAdd,
                                    user_id: entry.user_id,
                                    target: r_id.to_string(),
                                    action_data: serde_json::json!({}),
                                },
                            )
                            .await
                        }
                        RoleAction::Update => {
                            info!("Role updated: {}", r_id);

                            handle_mod_action(
                                &user_data.pool,
                                &user_data.surreal_cache,
                                &cache_http,
                                &super::handler::HandleModAction {
                                    guild_id: *guild_id,
                                    limit: super::core::UserLimitTypes::RoleUpdate,
                                    user_id: entry.user_id,
                                    target: r_id.to_string(),
                                    action_data: serde_json::json!({}),
                                },
                            )
                            .await
                        }
                        RoleAction::Delete => {
                            info!("Role deleted: {}", r_id);

                            handle_mod_action(
                                &user_data.pool,
                                &user_data.surreal_cache,
                                &cache_http,
                                &super::handler::HandleModAction {
                                    guild_id: *guild_id,
                                    limit: super::core::UserLimitTypes::RoleRemove,
                                    user_id: entry.user_id,
                                    target: r_id.to_string(),
                                    action_data: serde_json::json!({}),
                                },
                            )
                            .await
                        }
                        _ => Ok(()),
                    }
                }
                // DEAL WITH THIS HELL LATER.
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
                                    }
                                    Change::RolesRemove { old, new } => {
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
                                    }
                                    _ => {}
                                }
                            }

                            handle_mod_action(
                                &user_data.pool,
                                &user_data.surreal_cache,
                                &cache_http,
                                &super::handler::HandleModAction {
                                    guild_id: *guild_id,
                                    limit: super::core::UserLimitTypes::MemberRolesUpdated,
                                    user_id: entry.user_id,
                                    target: target.to_string(),
                                    action_data: serde_json::json!({
                                        "old": old_roles,
                                        "added": added,
                                        "removed": removed,
                                    }),
                                },
                            )
                            .await?;

                            if !added.is_empty() {
                                info!("Added roles: {:?}", added);

                                handle_mod_action(
                                    &user_data.pool,
                                    &user_data.surreal_cache,
                                    &cache_http,
                                    &super::handler::HandleModAction {
                                        guild_id: *guild_id,
                                        limit: super::core::UserLimitTypes::RoleGivenToMember,
                                        user_id: entry.user_id,
                                        target: target.to_string(),
                                        action_data: serde_json::json!({
                                            "old": old_roles,
                                            "added": added,
                                            "removed": removed,
                                        }),
                                    },
                                )
                                .await?;
                            }

                            if !removed.is_empty() {
                                info!("Removed roles: {:?}", removed);

                                handle_mod_action(
                                    &user_data.pool,
                                    &user_data.surreal_cache,
                                    &cache_http,
                                    &super::handler::HandleModAction {
                                        guild_id: *guild_id,
                                        limit: super::core::UserLimitTypes::RoleRemovedFromMember,
                                        user_id: entry.user_id,
                                        target: target.to_string(),
                                        action_data: serde_json::json!({
                                            "old": old_roles,
                                            "added": added,
                                            "removed": removed,
                                        }),
                                    },
                                )
                                .await?;
                            }
                        }
                        _ => {}
                    }

                    Ok(())
                }
                _ => Ok(()),
            };

            if let Err(res) = res {
                error!("Error while handling audit log: {}", res);
                return Err(res);
            }

            Ok(())
        }
        _ => Ok(()),
    }
}
