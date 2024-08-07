use log::{error, info};
use poise::serenity_prelude::{Change, FullEvent, MemberAction, RoleAction};
use serenity::model::guild::audit_log::{Action, ChannelAction};

use super::handler::handle_mod_action;
use silverpelt::EventHandlerContext;

pub async fn event_listener(ectx: &EventHandlerContext) -> Result<(), base_data::Error> {
    let ctx = &ectx.serenity_context;
    let event = &ectx.full_event;

    match event {
        FullEvent::GuildMemberAddition { new_member } => {
            handle_mod_action(
                ctx,
                &super::handler::HandleModAction {
                    guild_id: ectx.guild_id,
                    limit: super::core::LimitTypes::MemberAdd,
                    user_id: new_member.user.id,
                    target: None,
                    action_data: serde_json::json!({}),
                },
            )
            .await
        }
        FullEvent::Message { new_message } => {
            handle_mod_action(
                ctx,
                &super::handler::HandleModAction {
                    guild_id: ectx.guild_id,
                    limit: super::core::LimitTypes::MessageCreate,
                    user_id: new_message.author.id,
                    target: Some(new_message.id.to_string()),
                    action_data: serde_json::json!({}),
                },
            )
            .await
        }
        FullEvent::GuildAuditLogEntryCreate { entry, guild_id } => {
            info!("Audit log created: {:?}. Guild: {}", entry, guild_id);

            let Some(user_id) = entry.user_id else {
                return Ok(());
            };

            let res = match entry.action {
                Action::Channel(ch) => {
                    let ch_id = entry.target_id.ok_or("No channel ID found")?;

                    match ch {
                        ChannelAction::Create => {
                            info!("Channel created: {}", ch_id);

                            handle_mod_action(
                                ctx,
                                &super::handler::HandleModAction {
                                    guild_id: *guild_id,
                                    limit: super::core::LimitTypes::ChannelAdd,
                                    user_id,
                                    target: Some(ch_id.to_string()),
                                    action_data: serde_json::json!({
                                        "changes": entry.changes.clone(),
                                    }),
                                },
                            )
                            .await
                        }
                        ChannelAction::Delete => {
                            info!("Channel deleted: {}", ch_id);

                            handle_mod_action(
                                ctx,
                                &super::handler::HandleModAction {
                                    guild_id: *guild_id,
                                    limit: super::core::LimitTypes::ChannelRemove,
                                    user_id,
                                    target: Some(ch_id.to_string()),
                                    action_data: serde_json::json!({
                                        "changes": entry.changes.clone(),
                                    }),
                                },
                            )
                            .await
                        }
                        ChannelAction::Update => {
                            info!("Channel updated: {}", ch_id);

                            handle_mod_action(
                                ctx,
                                &super::handler::HandleModAction {
                                    guild_id: *guild_id,
                                    limit: super::core::LimitTypes::ChannelUpdate,
                                    user_id,
                                    target: Some(ch_id.to_string()),
                                    action_data: serde_json::json!({
                                        "changes": entry.changes.clone(),
                                    }),
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
                                ctx,
                                &super::handler::HandleModAction {
                                    guild_id: *guild_id,
                                    limit: super::core::LimitTypes::RoleAdd,
                                    user_id,
                                    target: Some(r_id.to_string()),
                                    action_data: serde_json::json!({
                                        "changes": entry.changes.clone(),
                                    }),
                                },
                            )
                            .await
                        }
                        RoleAction::Update => {
                            info!("Role updated: {}", r_id);

                            handle_mod_action(
                                ctx,
                                &super::handler::HandleModAction {
                                    guild_id: *guild_id,
                                    limit: super::core::LimitTypes::RoleUpdate,
                                    user_id,
                                    target: Some(r_id.to_string()),
                                    action_data: serde_json::json!({
                                        "changes": entry.changes.clone(),
                                    }),
                                },
                            )
                            .await
                        }
                        RoleAction::Delete => {
                            info!("Role deleted: {}", r_id);

                            handle_mod_action(
                                ctx,
                                &super::handler::HandleModAction {
                                    guild_id: *guild_id,
                                    limit: super::core::LimitTypes::RoleRemove,
                                    user_id,
                                    target: Some(r_id.to_string()),
                                    action_data: serde_json::json!({
                                        "changes": entry.changes.clone(),
                                    }),
                                },
                            )
                            .await
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
                                &super::handler::HandleModAction {
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
                                info!("Added roles: {:?}", added);

                                handle_mod_action(
                                    ctx,
                                    &super::handler::HandleModAction {
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
                                info!("Removed roles: {:?}", removed);

                                handle_mod_action(
                                    ctx,
                                    &super::handler::HandleModAction {
                                        guild_id: *guild_id,
                                        limit: super::core::LimitTypes::RoleRemovedFromMember,
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
