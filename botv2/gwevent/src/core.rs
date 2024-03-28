use crate::Error;
use indexmap::IndexMap;
use log::warn;
use serenity::all::{
    ActionExecution, EmojiId,
    FullEvent, GuildChannel, GuildId,
    StickerId, UserId,
};
use std::collections::HashMap;
use strum::VariantNames;
use super::field_type::FieldType;

/// Returns all events
#[allow(dead_code)]
pub const fn event_list() -> &'static [&'static str] {
    FullEvent::VARIANTS
}

/// Given an event and a module, return its guild id (for filtering etc.)
pub fn get_event_guild_id(event: &FullEvent) -> Result<GuildId, Option<Error>> {
    let guild_id = match event {
        FullEvent::AutoModActionExecution { execution } => execution.guild_id,
        FullEvent::AutoModRuleCreate { rule, .. } => rule.guild_id,
        FullEvent::AutoModRuleDelete { rule, .. } => rule.guild_id,
        FullEvent::AutoModRuleUpdate { rule, .. } => rule.guild_id,
        FullEvent::CacheReady { .. } => return Err(None), // We don't want this to be propogated anyways and it's not a guild event
        FullEvent::CategoryCreate { category, .. } => category.guild_id,
        FullEvent::CategoryDelete { category, .. } => category.guild_id,
        FullEvent::ChannelCreate { channel, .. } => channel.guild_id,
        FullEvent::ChannelDelete { channel, .. } => channel.guild_id,
        FullEvent::ChannelPinsUpdate { pin } => {
            if let Some(guild_id) = pin.guild_id {
                guild_id.to_owned()
            } else {
                return Err(None);
            }
        }
        FullEvent::ChannelUpdate { new, .. } => new.guild_id,
        FullEvent::CommandPermissionsUpdate { permission, .. } => permission.guild_id,
        FullEvent::EntitlementCreate { entitlement, .. } => {
            if let Some(guild_id) = entitlement.guild_id {
                guild_id.to_owned()
            } else {
                return Err(None);
            }
        }
        FullEvent::EntitlementDelete { entitlement, .. } => {
            if let Some(guild_id) = entitlement.guild_id {
                guild_id.to_owned()
            } else {
                return Err(None);
            }
        }
        FullEvent::EntitlementUpdate { entitlement, .. } => {
            if let Some(guild_id) = entitlement.guild_id {
                guild_id.to_owned()
            } else {
                return Err(None);
            }
        }
        FullEvent::GuildAuditLogEntryCreate { guild_id, .. } => *guild_id,
        FullEvent::GuildBanAddition { guild_id, .. } => *guild_id,
        FullEvent::GuildBanRemoval { guild_id, .. } => *guild_id,
        FullEvent::GuildCreate { guild, .. } => guild.id,
        FullEvent::GuildDelete { incomplete, .. } => incomplete.id,
        FullEvent::GuildEmojisUpdate { guild_id, .. } => *guild_id,
        FullEvent::GuildIntegrationsUpdate { guild_id, .. } => *guild_id,
        FullEvent::GuildMemberAddition { new_member, .. } => new_member.guild_id,
        FullEvent::GuildMemberRemoval { guild_id, .. } => *guild_id,
        FullEvent::GuildMemberUpdate { event, .. } => event.guild_id,
        FullEvent::GuildMembersChunk { chunk, .. } => chunk.guild_id,
        FullEvent::GuildRoleCreate { new, .. } => new.guild_id,
        FullEvent::GuildRoleDelete { guild_id, .. } => *guild_id,
        FullEvent::GuildRoleUpdate { new, .. } => new.guild_id,
        FullEvent::GuildScheduledEventCreate { event, .. } => event.guild_id,
        FullEvent::GuildScheduledEventDelete { event, .. } => event.guild_id,
        FullEvent::GuildScheduledEventUpdate { event, .. } => event.guild_id,
        FullEvent::GuildScheduledEventUserAdd { subscribed, .. } => subscribed.guild_id,
        FullEvent::GuildScheduledEventUserRemove { unsubscribed, .. } => unsubscribed.guild_id,
        FullEvent::GuildStickersUpdate { guild_id, .. } => *guild_id,
        FullEvent::GuildUpdate { new_data, .. } => new_data.id,
        FullEvent::IntegrationCreate { integration, .. } => {
            if let Some(guild_id) = integration.guild_id {
                guild_id.to_owned()
            } else {
                return Err(None);
            }
        }
        FullEvent::IntegrationDelete { guild_id, .. } => *guild_id,
        FullEvent::IntegrationUpdate { integration, .. } => {
            if let Some(guild_id) = integration.guild_id {
                guild_id.to_owned()
            } else {
                return Err(None);
            }
        }
        FullEvent::InteractionCreate { .. } => return Err(None), // We dont handle interactions create events in event handlers
        FullEvent::InviteCreate { data, .. } => {
            if let Some(guild_id) = data.guild_id {
                guild_id.to_owned()
            } else {
                return Err(None);
            }
        }
        FullEvent::InviteDelete { data, .. } => {
            if let Some(guild_id) = data.guild_id {
                guild_id.to_owned()
            } else {
                return Err(None);
            }
        }
        FullEvent::Message { new_message, .. } => {
            if let Some(guild_id) = &new_message.guild_id {
                guild_id.to_owned()
            } else {
                return Err(None);
            }
        }
        FullEvent::MessageDelete { guild_id, .. } => {
            if let Some(guild_id) = guild_id {
                guild_id.to_owned()
            } else {
                return Err(None);
            }
        }
        FullEvent::MessageDeleteBulk { guild_id, .. } => {
            if let Some(guild_id) = guild_id {
                guild_id.to_owned()
            } else {
                return Err(None);
            }
        }
        FullEvent::MessageUpdate { event, .. } => {
            if let Some(guild_id) = &event.guild_id {
                guild_id.to_owned()
            } else {
                return Err(None);
            }
        }
        FullEvent::PresenceReplace { .. } => return Err(None), // We dont handle precenses
        FullEvent::PresenceUpdate { .. } => return Err(None),  // We dont handle precenses
        FullEvent::Ratelimit { data, .. } => {
            // Warn i guess
            warn!("Ratelimit event recieved: {:?}", data);
            return Err(None);
        }
        FullEvent::ReactionAdd { .. } => return Err(None), // We dont handle reactions right now
        FullEvent::ReactionRemove { .. } => return Err(None), // We dont handle reactions right now
        FullEvent::ReactionRemoveAll { .. } => return Err(None), // We dont handle reactions right now
        FullEvent::ReactionRemoveEmoji { .. } => return Err(None), // We dont handle reactions right now
        FullEvent::Ready { .. } => return Err(None),               // We dont handle ready events
        FullEvent::Resume { .. } => return Err(None),              // We dont handle resume events
        FullEvent::ShardStageUpdate { .. } => return Err(None), // We dont handle shard stage updates
        FullEvent::ShardsReady { .. } => return Err(None),      // We dont handle shards ready
        FullEvent::StageInstanceCreate { .. } => return Err(None), // We dont handle stage instances right now
        FullEvent::StageInstanceDelete { .. } => return Err(None), // We dont handle stage instances right now
        FullEvent::StageInstanceUpdate { .. } => return Err(None), // We dont handle stage instances right now
        FullEvent::ThreadCreate { thread, .. } => thread.guild_id,
        FullEvent::ThreadDelete { thread, .. } => thread.guild_id,
        FullEvent::ThreadListSync {
            thread_list_sync, ..
        } => thread_list_sync.guild_id,
        FullEvent::ThreadMemberUpdate { thread_member, .. } => {
            if let Some(guild_id) = thread_member.guild_id {
                guild_id.to_owned()
            } else {
                return Err(None);
            }
        }
        FullEvent::ThreadMembersUpdate {
            thread_members_update,
            ..
        } => thread_members_update.guild_id,
        FullEvent::ThreadUpdate { new, .. } => new.guild_id,
        FullEvent::TypingStart { .. } => return Err(None), // We dont handle typing start
        FullEvent::UserUpdate { .. } => return Err(None),  // We dont handle user updates
        FullEvent::VoiceChannelStatusUpdate { guild_id, .. } => *guild_id,
        FullEvent::VoiceServerUpdate { .. } => return Err(None), // We dont handle voice right now
        FullEvent::VoiceStateUpdate { .. } => return Err(None),  // We dont handle voice right now
        FullEvent::WebhookUpdate { guild_id, .. } => *guild_id,
    };

    Ok(guild_id)
}

/// Given an event and a module, return its user id
pub fn get_event_user_id(event: &FullEvent) -> Result<UserId, Option<Error>> {
    let user_id = match event {
        FullEvent::AutoModActionExecution { execution } => execution.user_id,
        FullEvent::AutoModRuleCreate { rule, .. } => rule.creator_id,
        FullEvent::AutoModRuleDelete { rule, .. } => rule.creator_id,
        FullEvent::AutoModRuleUpdate { rule, .. } => rule.creator_id,
        FullEvent::CacheReady { .. } => return Err(None), // We don't want this to be propogated anyways and it's not a guild event
        FullEvent::CategoryCreate { .. } => return Err(None), // Doesn't have a known user just from event
        FullEvent::CategoryDelete { .. } => return Err(None), // Doesn't have a known user just from event
        FullEvent::ChannelCreate { .. } => return Err(None), // Doesn't have a known user just from event
        FullEvent::ChannelDelete { .. } => return Err(None), // Doesn't have a known user just from event
        FullEvent::ChannelPinsUpdate { .. } => return Err(None), // Doesn't have a known user just from event
        FullEvent::ChannelUpdate { .. } => return Err(None), // Doesn't have a known user just from event
        FullEvent::CommandPermissionsUpdate { .. } => return Err(None), // Doesn't have a known user just from event,
        FullEvent::EntitlementCreate { entitlement, .. } => {
            if let Some(user_id) = entitlement.user_id {
                user_id.to_owned()
            } else {
                return Err(None);
            }
        }
        FullEvent::EntitlementDelete { entitlement, .. } => {
            if let Some(user_id) = entitlement.user_id {
                user_id.to_owned()
            } else {
                return Err(None);
            }
        }
        FullEvent::EntitlementUpdate { entitlement, .. } => {
            if let Some(user_id) = entitlement.user_id {
                user_id.to_owned()
            } else {
                return Err(None);
            }
        }
        FullEvent::GuildAuditLogEntryCreate { entry, .. } => entry.user_id,
        FullEvent::GuildBanAddition { banned_user, .. } => banned_user.id,
        FullEvent::GuildBanRemoval { unbanned_user, .. } => unbanned_user.id,
        FullEvent::GuildCreate { guild, .. } => guild.owner_id,
        FullEvent::GuildDelete { .. } => return Err(None), // Doesn't have a known user just from event
        FullEvent::GuildEmojisUpdate { .. } => return Err(None), // Doesn't have a known user just from event
        FullEvent::GuildIntegrationsUpdate { .. } => return Err(None), // Doesn't have a known user just from event,
        FullEvent::GuildMemberAddition { new_member, .. } => new_member.user.id,
        FullEvent::GuildMemberRemoval { user, .. } => user.id,
        FullEvent::GuildMemberUpdate { event, .. } => event.user.id,
        FullEvent::GuildMembersChunk { .. } => return Err(None), // Doesn't have a known user just from event,
        FullEvent::GuildRoleCreate { .. } => return Err(None), // Doesn't have a known user just from event
        FullEvent::GuildRoleDelete { .. } => return Err(None), // Doesn't have a known user just from event,
        FullEvent::GuildRoleUpdate { .. } => return Err(None), // Doesn't have a known user just from event
        FullEvent::GuildScheduledEventCreate { event, .. } => {
            if let Some(ref creator) = event.creator {
                creator.id.to_owned()
            } else {
                return Err(None);
            }
        },
        FullEvent::GuildScheduledEventDelete { event, .. } => {
            if let Some(ref creator) = event.creator {
                creator.id.to_owned()
            } else {
                return Err(None);
            }
        },
        FullEvent::GuildScheduledEventUpdate { event, .. } => {
            if let Some(ref creator) = event.creator {
                creator.id.to_owned()
            } else {
                return Err(None);
            }
        },
        FullEvent::GuildScheduledEventUserAdd { subscribed, .. } => subscribed.user_id,
        FullEvent::GuildScheduledEventUserRemove { unsubscribed, .. } => unsubscribed.user_id,
        FullEvent::GuildStickersUpdate { .. } => return Err(None), // Doesn't have a known user just from event,
        FullEvent::GuildUpdate { .. } => return Err(None), // Doesn't have a known user just from event
        FullEvent::IntegrationCreate { integration, .. } => {
            if let Some(ref user) = integration.user {
                user.id.to_owned()
            } else {
                return Err(None);
            }
        }
        FullEvent::IntegrationDelete { .. } => return Err(None), // Doesn't have a known user just from event,
        FullEvent::IntegrationUpdate { integration, .. } => {
            if let Some(ref user) = integration.user {
                user.id.to_owned()
            } else {
                return Err(None);
            }
        }
        FullEvent::InteractionCreate { .. } => return Err(None), // We dont handle interactions create events in event handlers
        FullEvent::InviteCreate { data, .. } => {
            if let Some(ref inviter) = data.inviter {
                inviter.id.to_owned()
            } else {
                return Err(None);
            }
        }
        FullEvent::InviteDelete { .. } => return Err(None), // Doesn't have a known user just from event
        FullEvent::Message { new_message, .. } => new_message.author.id,
        FullEvent::MessageDelete { .. } => return Err(None), // Doesn't have a known user just from event
        FullEvent::MessageDeleteBulk { .. } => return Err(None), // Doesn't have a known user just from event
        FullEvent::MessageUpdate { event, new, .. } => {
            if let Some(new) = new {
                new.author.id.to_owned()
            } else if let Some(author) = &event.author {
                author.id.to_owned()
            } else {
                warn!("No author found in message update event: {:?}", event);
                return Err(None);
            }
        }
        FullEvent::PresenceReplace { .. } => return Err(None), // We dont handle precenses
        FullEvent::PresenceUpdate { .. } => return Err(None),  // We dont handle precenses
        FullEvent::Ratelimit { data, .. } => {
            // Warn i guess
            warn!("Ratelimit event recieved: {:?}", data);
            return Err(None);
        }
        FullEvent::ReactionAdd { .. } => return Err(None), // We dont handle reactions right now
        FullEvent::ReactionRemove { .. } => return Err(None), // We dont handle reactions right now
        FullEvent::ReactionRemoveAll { .. } => return Err(None), // We dont handle reactions right now
        FullEvent::ReactionRemoveEmoji { .. } => return Err(None), // We dont handle reactions right now
        FullEvent::Ready { .. } => return Err(None),               // We dont handle ready events
        FullEvent::Resume { .. } => return Err(None),              // We dont handle resume events
        FullEvent::ShardStageUpdate { .. } => return Err(None), // We dont handle shard stage updates
        FullEvent::ShardsReady { .. } => return Err(None),      // We dont handle shards ready
        FullEvent::StageInstanceCreate { .. } => return Err(None), // We dont handle stage instances right now
        FullEvent::StageInstanceDelete { .. } => return Err(None), // We dont handle stage instances right now
        FullEvent::StageInstanceUpdate { .. } => return Err(None), // We dont handle stage instances right now
        FullEvent::ThreadCreate { thread, .. } => {
            if let Some(opener) = thread.owner_id {
                opener.to_owned()
            } else {
                return Err(None);
            }
        },
        FullEvent::ThreadDelete { .. } => return Err(None), // Doesn't have a known user just from event,
        FullEvent::ThreadListSync { .. } => return Err(None), // Doesn't have a known user just from event
        FullEvent::ThreadMemberUpdate { thread_member, .. } => thread_member.user_id,
        FullEvent::ThreadMembersUpdate { .. } => return Err(None), // Doesn't have a known user just from event
        FullEvent::ThreadUpdate { new, .. } => {
            if let Some(opener) = new.owner_id {
                opener.to_owned()
            } else {
                return Err(None);
            }
        
        },
        FullEvent::TypingStart { .. } => return Err(None), // We dont handle typing start
        FullEvent::UserUpdate { .. } => return Err(None),  // We dont handle user updates
        FullEvent::VoiceChannelStatusUpdate { .. } => return Err(None), // We dont handle voice right now
        FullEvent::VoiceServerUpdate { .. } => return Err(None), // We dont handle voice right now
        FullEvent::VoiceStateUpdate { .. } => return Err(None),  // We dont handle voice right now
        FullEvent::WebhookUpdate { .. } => return Err(None), // Doesn't have a known user just from event
    };

    Ok(user_id)
}

#[allow(dead_code)]
pub struct Field {
    /// The value of the field
    pub value: Vec<FieldType>,

    /// The category of the field
    pub category: String,
}

/// Given an event, expand it to a hashmap of fields
#[allow(dead_code)]
pub fn expand_event(event: &FullEvent) -> Option<IndexMap<String, Field>> {
    let mut fields = IndexMap::new();

    fn insert_field<T: Into<FieldType>>(
        fields: &mut IndexMap<String, Field>,
        category: &str,
        key: &str,
        value: T,
    ) {
        let value = value.into();
        match fields.entry(key.to_string()) {
            indexmap::map::Entry::Occupied(mut entry) => {
                entry.get_mut().value.push(value);
            }
            indexmap::map::Entry::Vacant(entry) => {
                entry.insert(Field {
                    value: vec![value],
                    category: category.to_string(),
                });
            }
        }
    }

    fn insert_optional_field<T: Into<FieldType>>(
        fields: &mut IndexMap<String, Field>,
        category: &str,
        key: &str,
        option: Option<T>,
    ) {
        match option {
            Some(value) => {
                let value = value.into();
                match fields.entry(key.to_string()) {
                    indexmap::map::Entry::Occupied(mut entry) => {
                        entry.get_mut().value.push(value);
                    }
                    indexmap::map::Entry::Vacant(entry) => {
                        entry.insert(Field {
                            value: vec![value],
                            category: category.to_string(),
                        });
                    }
                }
            }
            None => {
                fields.insert(
                    key.to_string(),
                    Field {
                        value: vec![FieldType::None],
                        category: category.to_string(),
                    },
                );
            }
        }
    }

    fn expand_action_execution(fields: &mut IndexMap<String, Field>, execution: &ActionExecution) {
        insert_field(fields, "action_execution", "guild_id", execution.guild_id);
        insert_field(
            fields,
            "action_execution",
            "action",
            execution.action.clone(),
        );
        insert_field(fields, "action_execution", "rule_id", execution.rule_id);
        insert_field(
            fields,
            "action_execution",
            "trigger_type",
            match execution.trigger_type {
                serenity::model::guild::automod::TriggerType::Keyword => "Keyword".to_string(),
                serenity::model::guild::automod::TriggerType::Spam => "Spam".to_string(),
                serenity::model::guild::automod::TriggerType::KeywordPreset => {
                    "KeywordPreset".to_string()
                }
                serenity::model::guild::automod::TriggerType::MentionSpam => {
                    "MentionSpam".to_string()
                }
                serenity::model::guild::automod::TriggerType::Unknown(b) => {
                    format!("Unknown({})", b)
                }
                _ => "Unknown".to_string(),
            },
        );
        insert_field(
            fields,
            "action_execution",
            "content",
            execution.content.clone().into_string(),
        );
        insert_field(fields, "action_execution", "user_id", execution.user_id);

        insert_optional_field(
            fields,
            "action_execution",
            "channel_id",
            execution.channel_id,
        );
        insert_optional_field(
            fields,
            "action_execution",
            "message_id",
            execution.message_id,
        );
        insert_optional_field(
            fields,
            "action_execution",
            "alert_system_message_id",
            execution.alert_system_message_id,
        );
        insert_optional_field(
            fields,
            "action_execution",
            "matched_keyword",
            execution.matched_keyword.clone(),
        );
        insert_optional_field(
            fields,
            "action_execution",
            "matched_content",
            execution.matched_content.clone(),
        );
    }

    fn expand_rule(
        fields: &mut IndexMap<String, Field>,
        rule: &serenity::model::guild::automod::Rule,
    ) {
        insert_field(fields, "automod_rule", "rule_id", rule.id);
        insert_field(fields, "automod_rule", "guild_id", rule.guild_id);
        insert_field(fields, "automod_rule", "rule_name", rule.name.clone());
        insert_field(fields, "automod_rule", "creator_id", rule.creator_id);
        insert_field(
            fields,
            "automod_rule",
            "event_type",
            match rule.event_type {
                serenity::model::guild::automod::EventType::MessageSend => {
                    "MessageSend".to_string()
                }
                _ => "Unknown".to_string(),
            },
        );
        insert_field(fields, "automod_rule", "trigger", rule.trigger.clone());
        insert_field(
            fields,
            "automod_rule",
            "actions",
            rule.actions.clone().into_vec(),
        );

        insert_field(fields, "automod_rule", "rule_enabled", rule.enabled);
        insert_field(
            fields,
            "automod_rule",
            "exempt_roles",
            rule.exempt_roles.clone().into_vec(),
        );
        insert_field(
            fields,
            "automod_rule",
            "exempt_channels",
            rule.exempt_channels.clone().into_vec(),
        );
    }

    fn expand_channel(fields: &mut IndexMap<String, Field>, channel: &GuildChannel) {
        insert_field(fields, "channel", "guild_id", channel.guild_id);
        insert_field(fields, "channel", "channel_name", channel.name.clone());
        insert_field(fields, "channel", "nsfw", channel.nsfw);
        insert_field(
            fields,
            "channel",
            "type",
            format!("{:?}", channel.kind),
        );

        // Optional fields
        insert_optional_field(fields, "channel", "channel_topic", channel.topic.clone());
        insert_optional_field(
            fields,
            "channel",
            "rate_limit_per_user",
            channel.rate_limit_per_user,
        );
        insert_optional_field(fields, "channel", "parent_id", channel.parent_id);
        insert_optional_field(fields, "channel", "user_limit", channel.user_limit);

        // Handle Thread IDs
        if let Some(parent_id) = channel.parent_id {
            insert_field(fields, "channel", "parent_id", parent_id);
        }

        insert_field(fields, "channel", "channel_id", channel.id);
    }

    fn expand_partial_guild_channel(
        fields: &mut IndexMap<String, Field>,
        channel: &serenity::all::PartialGuildChannel,
    ) {
        insert_field(fields, "channel", "id", channel.id);
        insert_field(fields, "channel", "parent_id", channel.parent_id);
        insert_field(fields, "channel", "guild_id", channel.guild_id);
        insert_field(
            fields,
            "channel",
            "type",
            match channel.kind {
                serenity::model::channel::ChannelType::Text => "Text".to_string(),
                serenity::model::channel::ChannelType::Voice => "Voice".to_string(),
                serenity::model::channel::ChannelType::Private => "PrivateChannel".to_string(),
                serenity::model::channel::ChannelType::GroupDm => "GroupDm".to_string(),
                serenity::model::channel::ChannelType::Category => "Category".to_string(),
                serenity::model::channel::ChannelType::News => "News".to_string(),
                serenity::model::channel::ChannelType::NewsThread => "NewsThread".to_string(),
                serenity::model::channel::ChannelType::PublicThread => "PublicThread".to_string(),
                serenity::model::channel::ChannelType::PrivateThread => "PrivateThread".to_string(),
                serenity::model::channel::ChannelType::Stage => "Stage".to_string(),
                serenity::model::channel::ChannelType::Directory => "Directory".to_string(),
                _ => "Unknown".to_string(),
            },
        );
    }

    fn expand_command_permissions(
        fields: &mut IndexMap<String, Field>,
        permission: &serenity::model::application::CommandPermissions,
    ) {
        insert_field(fields, "command_permissions", "command_id", permission.id);
        insert_field(
            fields,
            "command_permissions",
            "application_id",
            permission.application_id,
        );
        // Continue from here
    }

    fn expand_entitlement(
        fields: &mut IndexMap<String, Field>,
        entitlement: &serenity::model::monetization::Entitlement,
    ) {
        insert_field(fields, "entitlement", "entitlement_id", entitlement.id);
        insert_field(
            fields,
            "entitlement",
            "application_id",
            entitlement.application_id,
        );
        insert_field(
            fields,
            "entitlement",
            "entitlement_type",
            format!("{:?}", entitlement.kind).to_lowercase(),
        );
        insert_field(
            fields,
            "entitlement",
            "entitlement_deleted",
            entitlement.deleted,
        );

        // Optional Fields
        insert_optional_field(fields, "entitlement", "guild_id", entitlement.guild_id);
        insert_optional_field(fields, "entitlement", "user_id", entitlement.user_id);
        insert_optional_field(
            fields,
            "entitlement",
            "entitlement_starts_at",
            entitlement.starts_at,
        );
        insert_optional_field(
            fields,
            "entitlement",
            "entitlement_ends_at",
            entitlement.ends_at,
        );
    }

    fn expand_audit_log_entry(
        fields: &mut IndexMap<String, Field>,
        entry: &serenity::model::guild::audit_log::AuditLogEntry,
        guild_id: &GuildId,
    ) {
        insert_field(fields, "audit_log_entry", "guild_id", *guild_id);
        insert_field(fields, "audit_log_entry", "action", entry.action);
        insert_field(fields, "audit_log_entry", "user_id", entry.user_id);
        insert_field(fields, "audit_log_entry", "audit_log_id", entry.id);
        insert_optional_field(
            fields,
            "audit_log_entry",
            "audit_log_entry",
            entry.reason.clone(),
        );
        insert_optional_field(
            fields,
            "audit_log_entry",
            "audit_log_target_id",
            entry.target_id,
        );
        insert_optional_field(
            fields,
            "audit_log_entry",
            "audit_log_changes",
            entry.changes.clone(),
        );
        insert_optional_field(
            fields,
            "audit_log_entry",
            "audit_log_options",
            entry.options.clone(),
        );
    }

    fn expand_user(fields: &mut IndexMap<String, Field>, user: &serenity::model::user::User) {
        insert_field(fields, "user", "user_id", user.id);
        insert_field(fields, "user", "username", user.name.clone());
        insert_field(fields, "user", "is_bot", user.bot());

        //optional fields
        insert_optional_field(fields, "user", "global_username", user.global_name.clone());
    }

    fn expand_emoji_map(
        fields: &mut IndexMap<String, Field>,
        emoji_map: &HashMap<EmojiId, serenity::model::guild::Emoji>,
        guild_id: &GuildId,
    ) {
        insert_field(fields, "emoji_map", "guild_id", *guild_id);

        insert_field(
            fields,
            "emoji_map",
            "emojis",
            emoji_map.values().cloned().collect::<Vec<_>>(),
        );
    }

    fn expand_member(
        fields: &mut IndexMap<String, Field>,
        member: &serenity::model::guild::Member,
    ) {
        insert_field(fields, "guild", "guild_id", member.guild_id);
        expand_user(fields, &member.user);
        insert_field(fields, "member", "roles", member.roles.clone().into_vec());
        //optional fields
        insert_optional_field(fields, "member", "nick", member.nick.clone());
        insert_optional_field(fields, "member", "joined_timestamp", member.joined_at);
        insert_optional_field(fields, "member", "premium_since", member.premium_since);
        insert_optional_field(fields, "member", "permissions", member.permissions);
        insert_optional_field(
            fields,
            "member",
            "communication_disabled_until",
            member.communication_disabled_until,
        );
        insert_optional_field(
            fields,
            "member",
            "unusual_dm_activity_until",
            member.unusual_dm_activity_until,
        );
    }

    fn expand_role(fields: &mut IndexMap<String, Field>, role: serenity::model::guild::Role) {
        insert_field(fields, "role", "role_id", role.id);
        insert_field(fields, "role", "guild_id", role.guild_id);
        insert_field(fields, "role", "is_hoisted", role.hoist());
        insert_field(fields, "role", "is_managed", role.managed());
        insert_field(fields, "role", "is_mentionable", role.mentionable());
        insert_field(fields, "role", "role_name", role.name.clone());
        insert_field(fields, "role", "position", role.position);
    }

    fn expand_scheduled_event(
        fields: &mut IndexMap<String, Field>,
        event: serenity::model::guild::ScheduledEvent,
    ) {
        insert_field(fields, "scheduled_event", "event_id", event.id);
        insert_field(fields, "scheduled_event", "guild_id", event.guild_id);
        insert_field(fields, "scheduled_event", "event_name", event.name.clone());
        insert_field(
            fields,
            "scheduled_event",
            "event_start_time",
            event.start_time,
        );
        insert_field(
            fields,
            "scheduled_event",
            "event_privacy_level",
            format!("{:?}", event.privacy_level).to_lowercase(),
        );
        insert_field(
            fields,
            "scheduled_event",
            "event_type",
            match event.kind {
                serenity::model::guild::ScheduledEventType::StageInstance => {
                    "StageInstance".to_string()
                }
                serenity::model::guild::ScheduledEventType::Voice => "VoiceChannel".to_string(),
                serenity::model::guild::ScheduledEventType::External => "External".to_string(),
                _ => "Unknown".to_string(),
            },
        );

        //optional
        insert_optional_field(
            fields,
            "scheduled_event",
            "event_channel_id",
            event.channel_id,
        );
        insert_optional_field(fields, "scheduled_event", "creator_id", event.creator_id);
        insert_optional_field(
            fields,
            "scheduled_event",
            "event_description",
            event.description.clone(),
        );
        insert_optional_field(fields, "scheduled_event", "event_end_time", event.end_time);
    }

    fn expand_sticker_map(
        fields: &mut IndexMap<String, Field>,
        sticker_map: &HashMap<StickerId, serenity::model::sticker::Sticker>,
        guild_id: &GuildId,
    ) {
        insert_field(fields, "sticker_map", "guild_id", *guild_id);

        insert_field(
            fields,
            "sticker_map",
            "stickers",
            sticker_map.values().cloned().collect::<Vec<_>>(),
        );
    }

    // to finish expanding
    fn expand_guild(fields: &mut IndexMap<String, Field>, guild: &serenity::model::guild::Guild) {
        insert_field(fields, "guild", "guild_id", guild.id);
        insert_field(fields, "guild", "name", guild.name.clone());
        insert_field(fields, "guild", "owner_id", guild.owner_id);
        insert_field(fields, "guild", "nsfw_level", guild.nsfw_level);
        insert_optional_field(fields, "guild", "description", guild.description.clone());
    }

    fn expand_partial_guild(
        fields: &mut IndexMap<String, Field>,
        guild: &serenity::model::guild::PartialGuild,
    ) {
        insert_field(fields, "guild", "guild_id", guild.id);
        insert_field(fields, "guild", "name", guild.name.clone());
        insert_field(fields, "guild", "owner_id", guild.owner_id);
        insert_field(fields, "guild", "nsfw_level", guild.nsfw_level);
        insert_optional_field(fields, "guild", "description", guild.description.clone());
    }

    fn expand_integration(
        fields: &mut IndexMap<String, Field>,
        integration: serenity::model::guild::Integration,
    ) {
        insert_field(fields, "integration", "id", integration.id);
        insert_field(fields, "integration", "name", integration.name.clone());
        insert_field(fields, "integration", "type", integration.kind.clone());
        insert_field(fields, "integration", "enabled", integration.enabled());
        insert_field(
            fields,
            "integration",
            "account_id",
            integration.account.id.clone(),
        );
        insert_field(
            fields,
            "integration",
            "account_name",
            integration.account.name.clone(),
        );

        //optional fields
        insert_optional_field(
            fields,
            "integration",
            "syncing_status",
            integration.syncing(),
        );
        insert_optional_field(fields, "integration", "role_id", integration.role_id);
        insert_optional_field(fields, "integration", "guild_id", integration.guild_id);
        if let Some(user) = integration.user {
            insert_field(fields, "integration", "user_id", user.id);
        }
    }

    fn expand_invite_create(
        fields: &mut IndexMap<String, Field>,
        data: &serenity::model::event::InviteCreateEvent,
    ) {
        insert_field(fields, "invite_create", "code", data.code.to_string());
        insert_field(fields, "invite_create", "channel_id", data.channel_id);
        insert_field(fields, "invite_create", "created_at", data.created_at);
        insert_field(fields, "invite_create", "max_age", data.max_age);
        insert_field(fields, "invite_create", "max_uses", data.max_uses);

        // optional fields
        insert_optional_field(fields, "invite_create", "guild_id", data.guild_id);
    }

    fn expand_invite_delete(
        fields: &mut IndexMap<String, Field>,
        data: &serenity::model::event::InviteDeleteEvent,
    ) {
        insert_field(fields, "invite_delete", "code", data.code.to_string());
        insert_field(fields, "invite_delete", "channel_id", data.channel_id);

        // optional fields
        insert_optional_field(fields, "invite_delete", "guild_id", data.guild_id);
    }

    fn expand_message(
        fields: &mut IndexMap<String, Field>,
        message: serenity::model::channel::Message,
    ) {
        insert_field(fields, "message", "id", message.id);
        insert_field(fields, "message", "channel_id", message.channel_id);
        insert_field(fields, "message", "author", message.author.clone());
        insert_field(fields, "message", "content", message.content.clone());
        insert_field(fields, "message", "created_at", message.timestamp);
        insert_field(
            fields,
            "message",
            "embeds",
            message.embeds.clone().into_vec(),
        );
        insert_field(
            fields,
            "message",
            "attachments",
            message.attachments.clone().into_vec(),
        );
        insert_field(
            fields,
            "message",
            "components",
            message.components.clone().into_vec(),
        );
        insert_field(
            fields,
            "message",
            "kind",
            match message.kind {
                serenity::model::channel::MessageType::Regular => "Regular".to_string(),
                serenity::model::channel::MessageType::GroupRecipientAddition => {
                    "GroupRecipientAddition".to_string()
                }
                serenity::model::channel::MessageType::GroupRecipientRemoval => {
                    "GroupRecipientRemoval".to_string()
                }
                serenity::model::channel::MessageType::GroupCallCreation => {
                    "GroupCallCreation".to_string()
                }
                serenity::model::channel::MessageType::GroupNameUpdate => {
                    "GroupNameUpdate".to_string()
                }
                serenity::model::channel::MessageType::GroupIconUpdate => {
                    "GroupIconUpdate".to_string()
                }
                serenity::model::channel::MessageType::PinsAdd => "PinsAdd".to_string(),
                serenity::model::channel::MessageType::MemberJoin => "MemberJoin".to_string(),
                serenity::model::channel::MessageType::NitroBoost => "NitroBoost".to_string(),
                serenity::model::channel::MessageType::NitroTier1 => "NitroTier1".to_string(),
                serenity::model::channel::MessageType::NitroTier2 => "NitroTier2".to_string(),
                serenity::model::channel::MessageType::NitroTier3 => "NitroTier3".to_string(),
                serenity::model::channel::MessageType::ChannelFollowAdd => {
                    "ChannelFollowAdd".to_string()
                }
                serenity::model::channel::MessageType::GuildDiscoveryDisqualified => {
                    "GuildDiscoveryDisqualified".to_string()
                }
                serenity::model::channel::MessageType::GuildDiscoveryRequalified => {
                    "GuildDiscoveryRequalified".to_string()
                }
                serenity::model::channel::MessageType::GuildDiscoveryGracePeriodInitialWarning => {
                    "GuildDiscoveryGracePeriodInitialWarning".to_string()
                }
                serenity::model::channel::MessageType::GuildDiscoveryGracePeriodFinalWarning => {
                    "GuildDiscoveryGracePeriodFinalWarning".to_string()
                }
                serenity::model::channel::MessageType::ThreadCreated => "ThreadCreated".to_string(),
                serenity::model::channel::MessageType::InlineReply => "InlineReply".to_string(),
                serenity::model::channel::MessageType::ChatInputCommand => {
                    "ChatInputCommand".to_string()
                }
                serenity::model::channel::MessageType::ThreadStarterMessage => {
                    "ThreadStarterMessage".to_string()
                }
                serenity::model::channel::MessageType::GuildInviteReminder => {
                    "GuildInviteReminder".to_string()
                }
                serenity::model::channel::MessageType::ContextMenuCommand => {
                    "ContextMenuCommand".to_string()
                }
                serenity::model::channel::MessageType::AutoModAction => "AutoModAction".to_string(),
                serenity::model::channel::MessageType::RoleSubscriptionPurchase => {
                    "RoleSubscriptionPurchase".to_string()
                }
                serenity::model::channel::MessageType::InteractionPremiumUpsell => {
                    "InteractionPremiumUpsell".to_string()
                }
                serenity::model::channel::MessageType::StageStart => "StageStart".to_string(),
                serenity::model::channel::MessageType::StageEnd => "StageEnd".to_string(),
                serenity::model::channel::MessageType::StageSpeaker => "StageSpeaker".to_string(),
                serenity::model::channel::MessageType::StageTopic => "StageTopic".to_string(),
                serenity::model::channel::MessageType::GuildApplicationPremiumSubscription => {
                    "GuildApplicationPremiumSubscription".to_string()
                }
                _ => "Unknown".to_string(),
            },
        );

        //optional fields
        insert_optional_field(
            fields,
            "message",
            "message_updated_at",
            message.edited_timestamp,
        );
        insert_optional_field(fields, "message", "message_guild_id", message.guild_id);
    }

    fn expand_channel_pins_update(
        fields: &mut IndexMap<String, Field>,
        pin: &serenity::model::event::ChannelPinsUpdateEvent,
    ) {
        insert_field(fields, "channel_pins_update", "channel_id", pin.channel_id);
        insert_optional_field(
            fields,
            "channel_pins_update",
            "last_pin_timestamp",
            pin.last_pin_timestamp,
        );
    }

    fn expand_guild_scheduled_event_user_add(
        fields: &mut IndexMap<String, Field>,
        subscribed: &serenity::all::GuildScheduledEventUserAddEvent,
    ) {
        insert_field(
            fields,
            "guild_scheduled_event_user_add",
            "guild_id",
            subscribed.guild_id,
        );
        insert_field(
            fields,
            "guild_scheduled_event_user_add",
            "event_id",
            subscribed.scheduled_event_id,
        );
        insert_field(
            fields,
            "guild_scheduled_event_user_add",
            "user_id",
            subscribed.user_id,
        );
    }

    fn expand_guild_scheduled_event_user_remove(
        fields: &mut IndexMap<String, Field>,
        subscribed: &serenity::all::GuildScheduledEventUserRemoveEvent,
    ) {
        insert_field(
            fields,
            "guild_scheduled_event_user_remove",
            "guild_id",
            subscribed.guild_id,
        );
        insert_field(
            fields,
            "guild_scheduled_event_user_remove",
            "event_id",
            subscribed.scheduled_event_id,
        );
        insert_field(
            fields,
            "guild_scheduled_event_user_remove",
            "user_id",
            subscribed.user_id,
        );
    }

    fn expand_message_update(
        fields: &mut IndexMap<String, Field>,
        update: &serenity::model::event::MessageUpdateEvent,
    ) {
        insert_field(fields, "message_update_event", "warning", "This message has not been cached by Anti-Raid!".to_string());

        insert_field(fields, "message_update_event", "id", update.id);
        insert_field(fields, "message_update_event", "channel_id", update.channel_id);

        if let Some(user) = &update.author {
            expand_user(fields, user);
        }

        insert_optional_field(fields, "message_update_event", "content", update.content.clone());
        insert_optional_field(fields, "message_update_event", "timestamp", update.timestamp);
        insert_optional_field(fields, "message_update_event", "edited_timestamp", update.edited_timestamp);
        insert_optional_field(fields, "message_update_event", "tts", update.tts);
        insert_optional_field(fields, "message_update_event", "mention_everyone", update.mention_everyone);
        insert_optional_field(fields, "message_update_event", "mentions", update.mentions.clone());
        insert_optional_field(fields, "message_update_event", "mention_roles", update.mention_roles.clone());
        //TODO: insert_optional_field(fields, "message_update_event", "mention_channels", update.mention_channels.clone());
        insert_optional_field(fields, "message_update_event", "attachments", update.attachments.clone());
        insert_optional_field(fields, "message_update_event", "embeds", update.embeds.clone());
        //TODO: insert_optional_field(fields, "message_update_event", "reactions", update.reactions.clone());
        insert_optional_field(fields, "message_update_event", "pinned", update.pinned);
    }

    match event {
        FullEvent::AutoModActionExecution { execution } => {
            expand_action_execution(&mut fields, execution);
        }
        FullEvent::AutoModRuleCreate { rule } => {
            expand_rule(&mut fields, rule);
        }
        FullEvent::AutoModRuleDelete { rule } => {
            expand_rule(&mut fields, rule);
        }
        FullEvent::AutoModRuleUpdate { rule } => {
            expand_rule(&mut fields, rule);
        }
        FullEvent::CacheReady { .. } => return None, // We don't want this to be propogated anyways and it's not a guild event
        FullEvent::CategoryCreate { category } => {
            expand_channel(&mut fields, category);
        }
        FullEvent::CategoryDelete { category } => {
            expand_channel(&mut fields, category);
        }
        FullEvent::ChannelCreate { channel } => {
            expand_channel(&mut fields, channel);
        }
        FullEvent::ChannelDelete { channel, messages } => {
            expand_channel(&mut fields, channel);

            if let Some(messages) = messages {
                insert_field(
                    &mut fields,
                    "channel_delete_ext",
                    "number_of_messages",
                    messages.len(),
                );
            }
        }
        FullEvent::ChannelPinsUpdate { pin } => {
            expand_channel_pins_update(&mut fields, pin);
        }
        FullEvent::ChannelUpdate { old, new } => {
            if let Some(old) = old {
                expand_channel(&mut fields, old);
            }
            expand_channel(&mut fields, new);
        }
        FullEvent::CommandPermissionsUpdate { permission } => {
            expand_command_permissions(&mut fields, permission);
        }
        FullEvent::EntitlementCreate { entitlement } => {
            expand_entitlement(&mut fields, entitlement);
        }
        FullEvent::EntitlementDelete { entitlement } => {
            expand_entitlement(&mut fields, entitlement);
        }
        FullEvent::EntitlementUpdate { entitlement } => {
            expand_entitlement(&mut fields, entitlement);
        }
        FullEvent::GuildAuditLogEntryCreate {
            guild_id, entry, ..
        } => {
            expand_audit_log_entry(&mut fields, entry, guild_id);
        }
        FullEvent::GuildBanAddition {
            guild_id,
            banned_user,
        } => {
            insert_field(&mut fields, "guild", "guild_id", *guild_id);
            expand_user(&mut fields, banned_user);
        }
        FullEvent::GuildBanRemoval {
            guild_id,
            unbanned_user,
        } => {
            insert_field(&mut fields, "guild", "guild_id", *guild_id);
            expand_user(&mut fields, unbanned_user);
        }
        FullEvent::GuildCreate { guild, is_new } => {
            expand_guild(&mut fields, guild);
            insert_optional_field(&mut fields, "guild_ext", "is_new", *is_new);
        }
        FullEvent::GuildDelete { incomplete, full } => {
            if let Some(full) = full {
                expand_guild(&mut fields, full);
            } else {
                insert_field(&mut fields, "guild_ext", "guild_id", incomplete.id);
            }
        }
        FullEvent::GuildEmojisUpdate {
            guild_id,
            current_state,
        } => {
            expand_emoji_map(&mut fields, current_state, guild_id);
        }
        FullEvent::GuildIntegrationsUpdate { guild_id } => {
            insert_field(&mut fields, "guild_ext", "guild_id", *guild_id);
        }
        FullEvent::GuildMemberAddition { new_member } => {
            expand_member(&mut fields, new_member);
        }
        FullEvent::GuildMemberRemoval {
            guild_id,
            user,
            member_data_if_available,
        } => {
            if let Some(member_data_if_available) = member_data_if_available {
                expand_member(&mut fields, member_data_if_available);
            } else {
                insert_field(&mut fields, "guild", "guild_id", *guild_id);
                expand_user(&mut fields, user);
            }
        }
        FullEvent::GuildMemberUpdate {
            old_if_available,
            new,
            ..
        } => {
            if let Some(old) = old_if_available {
                expand_member(&mut fields, old);
            }
            if let Some(new) = new {
                expand_member(&mut fields, new);
            };
        }
        FullEvent::GuildMembersChunk { .. } => return None,
        FullEvent::GuildRoleCreate { new } => {
            expand_role(&mut fields, new.clone());
        }
        FullEvent::GuildRoleDelete {
            guild_id,
            removed_role_id,
            removed_role_data_if_available,
        } => {
            insert_field(&mut fields, "guild", "guild_id", *guild_id);
            insert_field(
                &mut fields,
                "guild_role_delete",
                "role_id",
                *removed_role_id,
            );

            if let Some(removed_role_data) = removed_role_data_if_available {
                expand_role(&mut fields, removed_role_data.clone());
            }
        }
        FullEvent::GuildRoleUpdate {
            old_data_if_available,
            new,
        } => {
            if let Some(old) = old_data_if_available {
                expand_role(&mut fields, old.clone());
            }
            expand_role(&mut fields, new.clone());
        }
        FullEvent::GuildScheduledEventCreate { event } => {
            expand_scheduled_event(&mut fields, event.clone());
        }
        FullEvent::GuildScheduledEventDelete { event } => {
            expand_scheduled_event(&mut fields, event.clone());
        }
        FullEvent::GuildScheduledEventUpdate { event } => {
            expand_scheduled_event(&mut fields, event.clone());
        }
        FullEvent::GuildScheduledEventUserAdd { subscribed } => {
            expand_guild_scheduled_event_user_add(&mut fields, subscribed);
        }
        FullEvent::GuildScheduledEventUserRemove { unsubscribed } => {
            expand_guild_scheduled_event_user_remove(&mut fields, unsubscribed);
        }
        FullEvent::GuildStickersUpdate {
            guild_id,
            current_state,
        } => {
            expand_sticker_map(&mut fields, current_state, guild_id);
        }
        FullEvent::GuildUpdate {
            old_data_if_available,
            new_data,
        } => {
            expand_partial_guild(&mut fields, new_data);
            if let Some(old) = old_data_if_available {
                expand_guild(&mut fields, old);
            }
        }
        FullEvent::IntegrationCreate { integration } => {
            expand_integration(&mut fields, integration.clone());
        }
        FullEvent::IntegrationDelete {
            guild_id,
            integration_id,
            application_id,
        } => {
            insert_field(&mut fields, "integration", "guild_id", *guild_id);
            insert_field(&mut fields, "integration", "id", *integration_id);
            insert_optional_field(
                &mut fields,
                "integration",
                "application_id",
                *application_id,
            );
        }
        FullEvent::IntegrationUpdate { integration } => {
            expand_integration(&mut fields, integration.clone());
        }
        FullEvent::InteractionCreate { interaction: _ } => return None,
        FullEvent::InviteCreate { data } => {
            expand_invite_create(&mut fields, data);
        }
        FullEvent::InviteDelete { data } => {
            expand_invite_delete(&mut fields, data);
        }
        FullEvent::Message { new_message } => {
            expand_message(&mut fields, new_message.clone());
        }
        FullEvent::MessageDelete {
            guild_id,
            deleted_message_id,
            channel_id,
        } => {
            insert_optional_field(&mut fields, "message", "guild_id", *guild_id);
            insert_field(&mut fields, "message", "message_id", *deleted_message_id);
            insert_field(&mut fields, "message", "channel_id", *channel_id);
        }
        FullEvent::MessageDeleteBulk {
            guild_id,
            channel_id,
            multiple_deleted_messages_ids,
        } => {
            insert_optional_field(&mut fields, "message", "guild_id", *guild_id);
            insert_field(&mut fields, "message", "channel_id", *channel_id);
            insert_field(
                &mut fields,
                "message",
                "message_ids",
                multiple_deleted_messages_ids.clone(),
            );
        }
        FullEvent::MessageUpdate {
            old_if_available,
            new,
            event
        } => {
            if let Some(old) = old_if_available {
                expand_message(&mut fields, old.clone());
            }
            if let Some(new) = new {
                expand_message(&mut fields, new.clone());
            } else {
                expand_message_update(&mut fields, event);
            }
        }
        FullEvent::PresenceReplace { .. } => return None,
        FullEvent::PresenceUpdate { .. } => return None,
        FullEvent::Ratelimit { .. } => return None,
        FullEvent::ReactionAdd { .. } => return None,
        FullEvent::ReactionRemove { .. } => return None,
        FullEvent::ReactionRemoveAll { .. } => return None,
        FullEvent::ReactionRemoveEmoji { .. } => return None,
        FullEvent::Ready { .. } => return None,
        FullEvent::Resume { .. } => return None,
        FullEvent::ShardStageUpdate { .. } => return None,
        FullEvent::ShardsReady { .. } => return None,
        FullEvent::StageInstanceCreate { .. } => return None,
        FullEvent::StageInstanceDelete { .. } => return None,
        FullEvent::StageInstanceUpdate { .. } => return None,
        FullEvent::ThreadCreate { thread } => {
            expand_channel(&mut fields, thread);
        }
        FullEvent::ThreadDelete {
            thread,
            full_thread_data,
        } => {
            if let Some(ftd) = full_thread_data {
                expand_channel(&mut fields, ftd);
            } else {
                expand_partial_guild_channel(&mut fields, thread);
            }
        }
        FullEvent::ThreadListSync { .. } => {
            // We don't need to support this event tbrh
            return None;
        }
        FullEvent::ThreadMemberUpdate { thread_member } => {
            if let Some(ref member) = thread_member.member {
                expand_member(&mut fields, member);
            } else {
                insert_field(&mut fields, "user", "user_id", thread_member.user_id);
            }

            insert_optional_field(&mut fields, "thread", "guild_id", thread_member.guild_id);
            insert_field(&mut fields, "thread", "channel_id", thread_member.id);
        }
        FullEvent::ThreadMembersUpdate {
            thread_members_update,
        } => {
            insert_field(
                &mut fields,
                "thread_members",
                "id",
                thread_members_update.id,
            );
            insert_field(
                &mut fields,
                "thread_members",
                "guild_id",
                thread_members_update.guild_id,
            );
            insert_field(
                &mut fields,
                "thread_members",
                "member_count",
                thread_members_update.member_count,
            );
            insert_field(
                &mut fields,
                "thread_members",
                "added_members",
                thread_members_update.added_members.clone().into_vec(),
            );
            insert_field(
                &mut fields,
                "thread_members",
                "removed_member_ids",
                thread_members_update.removed_member_ids.clone().into_vec(),
            );
        }
        FullEvent::ThreadUpdate { new, old } => {
            expand_channel(&mut fields, new);

            if let Some(old) = old {
                expand_channel(&mut fields, old);
            }
        }
        FullEvent::TypingStart { .. } => return None,
        FullEvent::UserUpdate { .. } => return None,
        FullEvent::VoiceChannelStatusUpdate { .. } => return None,
        FullEvent::VoiceServerUpdate { .. } => return None,
        FullEvent::VoiceStateUpdate { .. } => return None,
        FullEvent::WebhookUpdate {
            guild_id,
            belongs_to_channel_id,
        } => {
            insert_field(&mut fields, "webhook", "guild_id", *guild_id);
            insert_field(&mut fields, "webhook", "channel_id", *belongs_to_channel_id);
        }
    }

    Some(fields)
}
