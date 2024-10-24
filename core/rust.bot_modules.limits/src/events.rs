use log::{debug, error};
use poise::serenity_prelude::{Change, FullEvent, MemberAction, RoleAction};
use serenity::model::guild::audit_log::{Action, ChannelAction};

use super::core::HandleModAction;
use super::handler::handle_mod_action;
use silverpelt::ar_event::{AntiraidEvent, EventHandlerContext};

pub(crate) async fn event_listener(ectx: &EventHandlerContext) -> Result<(), silverpelt::Error> {
    let ctx = &ectx.serenity_context;

    match ectx.event {
        AntiraidEvent::Discord(ref event) => {
            // Handle events
            match event {
                FullEvent::GuildMemberAddition { new_member } => {
                    handle_mod_action(
                        ctx,
                        &HandleModAction {
                            guild_id: ectx.guild_id,
                            limit: super::core::LimitTypes::MemberAdd,
                            user_id: new_member.user.id,
                            target: None,
                            action_data: serde_json::json!({}),
                        },
                    )
                    .await?;

                    Ok(())
                }
                FullEvent::Message { new_message } => {
                    handle_mod_action(
                        ctx,
                        &HandleModAction {
                            guild_id: ectx.guild_id,
                            limit: super::core::LimitTypes::MessageCreate,
                            user_id: new_message.author.id,
                            target: Some(new_message.id.to_string()),
                            action_data: serde_json::json!({}),
                        },
                    )
                    .await?;

                    Ok(())
                }
                FullEvent::GuildAuditLogEntryCreate { entry, guild_id } => {
                    debug!("Audit log created: {:?}. Guild: {}", entry, guild_id);

                    let Some(user_id) = entry.user_id else {
                        return Ok(());
                    };

                    let res = match entry.action {
                        Action::Channel(ch) => {
                            let ch_id = entry.target_id.ok_or("No channel ID found")?;

                            match ch {
                                ChannelAction::Create => {
                                    debug!("Channel created: {}", ch_id);

                                    handle_mod_action(
                                        ctx,
                                        &HandleModAction {
                                            guild_id: *guild_id,
                                            limit: super::core::LimitTypes::ChannelAdd,
                                            user_id,
                                            target: Some(ch_id.to_string()),
                                            action_data: serde_json::json!({
                                                "changes": entry.changes.clone(),
                                            }),
                                        },
                                    )
                                    .await?;

                                    Ok(())
                                }
                                ChannelAction::Delete => {
                                    debug!("Channel deleted: {}", ch_id);

                                    handle_mod_action(
                                        ctx,
                                        &HandleModAction {
                                            guild_id: *guild_id,
                                            limit: super::core::LimitTypes::ChannelRemove,
                                            user_id,
                                            target: Some(ch_id.to_string()),
                                            action_data: serde_json::json!({
                                                "changes": entry.changes.clone(),
                                            }),
                                        },
                                    )
                                    .await?;

                                    Ok(())
                                }
                                ChannelAction::Update => {
                                    debug!("Channel updated: {}", ch_id);

                                    handle_mod_action(
                                        ctx,
                                        &HandleModAction {
                                            guild_id: *guild_id,
                                            limit: super::core::LimitTypes::ChannelUpdate,
                                            user_id,
                                            target: Some(ch_id.to_string()),
                                            action_data: serde_json::json!({
                                                "changes": entry.changes.clone(),
                                            }),
                                        },
                                    )
                                    .await?;

                                    Ok(())
                                }
                                _ => Ok(()),
                            }
                        }
                        Action::Role(ra) => {
                            let r_id = entry.target_id.ok_or("No role ID found")?;

                            match ra {
                                RoleAction::Create => {
                                    debug!("Role created: {}", r_id);

                                    handle_mod_action(
                                        ctx,
                                        &HandleModAction {
                                            guild_id: *guild_id,
                                            limit: super::core::LimitTypes::RoleAdd,
                                            user_id,
                                            target: Some(r_id.to_string()),
                                            action_data: serde_json::json!({
                                                "changes": entry.changes.clone(),
                                            }),
                                        },
                                    )
                                    .await?;

                                    Ok(())
                                }
                                RoleAction::Update => {
                                    debug!("Role updated: {}", r_id);

                                    handle_mod_action(
                                        ctx,
                                        &HandleModAction {
                                            guild_id: *guild_id,
                                            limit: super::core::LimitTypes::RoleUpdate,
                                            user_id,
                                            target: Some(r_id.to_string()),
                                            action_data: serde_json::json!({
                                                "changes": entry.changes.clone(),
                                            }),
                                        },
                                    )
                                    .await?;

                                    Ok(())
                                }
                                RoleAction::Delete => {
                                    debug!("Role deleted: {}", r_id);

                                    handle_mod_action(
                                        ctx,
                                        &HandleModAction {
                                            guild_id: *guild_id,
                                            limit: super::core::LimitTypes::RoleRemove,
                                            user_id,
                                            target: Some(r_id.to_string()),
                                            action_data: serde_json::json!({
                                                "changes": entry.changes.clone(),
                                            }),
                                        },
                                    )
                                    .await?;

                                    Ok(())
                                }
                                _ => Ok(()),
                            }
                        }
                        // TODO: DEAL WITH THIS HELL LATER.
                        Action::Member(ma) => {
                            let Some(target) = entry.target_id else {
                                error!("MEMBER update: No target ID found");
                                return Err("No target ID found".into());
                            };

                            #[allow(clippy::single_match)] // Plans to add further events are coming
                            match ma {
                                MemberAction::RoleUpdate => {
                                    let mut added = Vec::new();
                                    let mut removed = Vec::new();
                                    let mut old_roles = Vec::new();

                                    for change in entry.changes.iter() {
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
                                        ctx,
                                        &HandleModAction {
                                            guild_id: *guild_id,
                                            limit: super::core::LimitTypes::MemberRolesUpdated,
                                            user_id,
                                            target: Some(target.to_string()),
                                            action_data: serde_json::json!({
                                                "old": old_roles,
                                                "added": added,
                                                "removed": removed,
                                            }),
                                        },
                                    )
                                    .await?;

                                    if !added.is_empty() {
                                        debug!("Added roles: {:?}", added);

                                        handle_mod_action(
                                            ctx,
                                            &HandleModAction {
                                                guild_id: *guild_id,
                                                limit: super::core::LimitTypes::RoleGivenToMember,
                                                user_id,
                                                target: Some(target.to_string()),
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
                                        debug!("Removed roles: {:?}", removed);

                                        handle_mod_action(
                                            ctx,
                                            &HandleModAction {
                                                guild_id: *guild_id,
                                                limit:
                                                    super::core::LimitTypes::RoleRemovedFromMember,
                                                user_id,
                                                target: Some(target.to_string()),
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
                                MemberAction::BanAdd => {
                                    handle_mod_action(
                                        ctx,
                                        &HandleModAction {
                                            guild_id: *guild_id,
                                            limit: super::core::LimitTypes::Ban,
                                            user_id,
                                            target: Some(target.to_string()),
                                            action_data: serde_json::json!({
                                                "changes": entry.changes.clone(),
                                                "ar": false,
                                            }),
                                        },
                                    )
                                    .await?;
                                }
                                MemberAction::BanRemove => {
                                    handle_mod_action(
                                        ctx,
                                        &HandleModAction {
                                            guild_id: *guild_id,
                                            limit: super::core::LimitTypes::Unban,
                                            user_id,
                                            target: Some(target.to_string()),
                                            action_data: serde_json::json!({
                                                "changes": entry.changes.clone(),
                                                "ar": false,
                                            }),
                                        },
                                    )
                                    .await?;
                                }
                                MemberAction::Kick => {
                                    handle_mod_action(
                                        ctx,
                                        &HandleModAction {
                                            guild_id: *guild_id,
                                            limit: super::core::LimitTypes::Kick,
                                            user_id,
                                            target: Some(target.to_string()),
                                            action_data: serde_json::json!({
                                                "changes": entry.changes.clone(),
                                            }),
                                        },
                                    )
                                    .await?;
                                }
                                MemberAction::Prune => {
                                    handle_mod_action(
                                        ctx,
                                        &HandleModAction {
                                            guild_id: *guild_id,
                                            limit: super::core::LimitTypes::PruneMembers,
                                            user_id,
                                            target: Some(target.to_string()),
                                            action_data: serde_json::json!({
                                                "changes": entry.changes.clone(),
                                            }),
                                        },
                                    )
                                    .await?;
                                }
                                _ => {}
                            }

                            Ok(())
                        }
                        _ => Ok(()),
                    };

                    if let Err(res) = res {
                        error!("Error while handling events: {}", res);
                        return Err(res);
                    }

                    Ok(())
                }
                _ => Ok(()),
            }
        }
        AntiraidEvent::Custom(ref event) => {
            if event.target() == std_events::limit::LIMIT_TARGET_ID
                && event.event_name() == "Limits:HandleLimitActionEvent"
            {
                let Some(event) = event
                    .as_any()
                    .downcast_ref::<std_events::limit::HandleLimitActionEvent>()
                else {
                    return Ok(()); // Ignore unknown events
                };

                let is_limited = handle_mod_action(
                    ctx,
                    &HandleModAction {
                        guild_id: ectx.guild_id,
                        limit: super::core::LimitTypes::from_std_events(event.limit),
                        user_id: event.user_id,
                        target: event.target.clone(),
                        action_data: event.action_data.clone(),
                    },
                )
                .await?;

                // Push data back to client
                if let Some(send_chan) = &event.send_chan {
                    send_chan
                        .send(std_events::limit::HandleLimitActionEventResponse { is_limited })
                        .await?;
                }
            }

            Ok(())
        }
        _ => Ok(()), // Ignore non-discord events
    }
}
