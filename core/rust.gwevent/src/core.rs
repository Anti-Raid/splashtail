use crate::field::{CategorizedField, Field};
use base_data::Error;
use indexmap::IndexMap;
use log::warn;
use serenity::all::{FullEvent, GuildId, UserId};
use strum::VariantNames;

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
        FullEvent::GuildAuditLogEntryCreate { entry, .. } => {
            if let Some(user_id) = entry.user_id {
                user_id.to_owned()
            } else {
                return Err(None);
            }
        }
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
        }
        FullEvent::GuildScheduledEventDelete { event, .. } => {
            if let Some(ref creator) = event.creator {
                creator.id.to_owned()
            } else {
                return Err(None);
            }
        }
        FullEvent::GuildScheduledEventUpdate { event, .. } => {
            if let Some(ref creator) = event.creator {
                creator.id.to_owned()
            } else {
                return Err(None);
            }
        }
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
        }
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
        }
        FullEvent::TypingStart { .. } => return Err(None), // We dont handle typing start
        FullEvent::UserUpdate { .. } => return Err(None),  // We dont handle user updates
        FullEvent::VoiceChannelStatusUpdate { .. } => return Err(None), // We dont handle voice right now
        FullEvent::VoiceServerUpdate { .. } => return Err(None), // We dont handle voice right now
        FullEvent::VoiceStateUpdate { .. } => return Err(None),  // We dont handle voice right now
        FullEvent::WebhookUpdate { .. } => return Err(None), // Doesn't have a known user just from event
    };

    Ok(user_id)
}

/// Given an event, expand it to a hashmap of fields
#[allow(dead_code)]
// @ci.expand_event_check.start
pub fn expand_event(event: FullEvent) -> Option<IndexMap<String, CategorizedField>> {
    let mut fields = IndexMap::new();

    /// Inserts a field to the fields hashmap
    ///
    /// Note that existing fields will be replaced, to avoid this, use the old-new pattern
    /// which is also handled by audit logs
    fn insert_field<T: Into<Field>>(
        fields: &mut IndexMap<String, CategorizedField>,
        category: &str,
        key: &str,
        value: T,
    ) {
        let value = value.into();
        fields.insert(
            key.to_string(),
            CategorizedField {
                category: category.to_string(),
                field: value,
            },
        );
    }

    fn insert_optional_field<T: Into<Field>>(
        fields: &mut IndexMap<String, CategorizedField>,
        category: &str,
        key: &str,
        option: Option<T>,
    ) {
        match option {
            Some(value) => {
                let value = value.into();
                fields.insert(
                    key.to_string(),
                    CategorizedField {
                        category: category.to_string(),
                        field: value,
                    },
                );
            }
            None => {
                fields.insert(
                    key.to_string(),
                    CategorizedField {
                        category: category.to_string(),
                        field: Field::None,
                    },
                );
            }
        }
    }

    match event {
        // @ci.expand_event_check AutoModActionExecution none
        FullEvent::AutoModActionExecution { execution } => {
            insert_field(&mut fields, "execution", "execution", execution);
        }
        // @ci.expand_event_check AutoModRuleCreate none
        FullEvent::AutoModRuleCreate { rule } => {
            insert_field(&mut fields, "rule", "rule", rule);
        }
        // @ci.expand_event_check AutoModRuleDelete none
        FullEvent::AutoModRuleDelete { rule } => {
            insert_field(&mut fields, "rule", "rule", rule);
        }
        // @ci.expand_event_check AutoModRuleUpdate none
        FullEvent::AutoModRuleUpdate { rule } => {
            insert_field(&mut fields, "rule", "rule", rule);
        }
        // @ci.expand_event_check CacheReady none
        FullEvent::CacheReady { .. } => return None, // We don't want this to be propogated anyways and it's not a guild event
        // @ci.expand_event_check CategoryCreate none
        FullEvent::CategoryCreate { category } => {
            insert_field(&mut fields, "category", "category", category);
        }
        // @ci.expand_event_check CategoryDelete none
        FullEvent::CategoryDelete { category } => {
            insert_field(&mut fields, "category", "category", category);
        }
        // @ci.expand_event_check ChannelCreate none
        FullEvent::ChannelCreate { channel } => {
            insert_field(&mut fields, "channel", "channel", channel);
        }
        // @ci.expand_event_check ChannelDelete none
        FullEvent::ChannelDelete { channel, messages } => {
            insert_field(&mut fields, "channel", "channel", channel);

            if let Some(messages) = messages {
                insert_field(
                    &mut fields,
                    "channel_delete_ext",
                    "number_of_messages",
                    messages.len(),
                );
            }
        }
        // @ci.expand_event_check ChannelPinsUpdate event:pin,ChannelPinsUpdateEvent
        FullEvent::ChannelPinsUpdate { pin } => {
            insert_optional_field(&mut fields, "guild", "guild_id", pin.guild_id);
            insert_field(&mut fields, "channel", "channel_id", pin.channel_id);
            insert_optional_field(
                &mut fields,
                "timestamp",
                "last_pin_timestamp",
                pin.last_pin_timestamp,
            );
        }
        // @ci.expand_event_check ChannelUpdate none
        FullEvent::ChannelUpdate { old, new } => {
            if let Some(old) = old {
                insert_field(&mut fields, "channel", "old", old);
            }
            insert_field(&mut fields, "channel", "new", new);
        }
        // @ci.expand_event_check CommandPermissionsUpdate none
        FullEvent::CommandPermissionsUpdate { permission } => {
            insert_field(&mut fields, "permission", "permission", permission);
        }
        // @ci.expand_event_check EntitlementCreate none
        FullEvent::EntitlementCreate { entitlement } => {
            insert_field(&mut fields, "entitlement", "entitlement", entitlement);
        }
        // @ci.expand_event_check EntitlementDelete none
        FullEvent::EntitlementDelete { entitlement } => {
            insert_field(&mut fields, "entitlement", "entitlement", entitlement);
        }
        // @ci.expand_event_check EntitlementUpdate none
        FullEvent::EntitlementUpdate { entitlement } => {
            insert_field(&mut fields, "entitlement", "entitlement", entitlement);
        }
        // @ci.expand_event_check GuildAuditLogEntryCreate none
        FullEvent::GuildAuditLogEntryCreate {
            guild_id, entry, ..
        } => {
            insert_field(&mut fields, "guild", "guild_id", guild_id);
            insert_field(&mut fields, "entry", "entry", entry);
        }
        // @ci.expand_event_check GuildBanAddition none
        FullEvent::GuildBanAddition {
            guild_id,
            banned_user,
        } => {
            insert_field(&mut fields, "guild", "guild_id", guild_id);
            insert_field(&mut fields, "user", "user", banned_user.clone());
        }
        // @ci.expand_event_check GuildBanRemoval none
        FullEvent::GuildBanRemoval {
            guild_id,
            unbanned_user,
        } => {
            insert_field(&mut fields, "guild", "guild_id", guild_id);
            insert_field(&mut fields, "user", "user", unbanned_user.clone());
        }
        // @ci.expand_event_check GuildCreate none
        FullEvent::GuildCreate { guild, is_new } => {
            insert_field(&mut fields, "guild", "guild", guild);
            insert_optional_field(&mut fields, "guild_ext", "is_new", is_new);
        }
        // @ci.expand_event_check GuildDelete none
        FullEvent::GuildDelete { incomplete, full } => {
            insert_field(
                &mut fields,
                "is_full_available",
                "is_full_available",
                full.is_some(),
            );

            if let Some(full) = full {
                insert_field(&mut fields, "guild", "guild", full);
            } else {
                insert_field(&mut fields, "guild", "guild_id", incomplete.id);
            }
        }
        // @ci.expand_event_check GuildEmojisUpdate none
        FullEvent::GuildEmojisUpdate {
            guild_id,
            current_state,
        } => {
            insert_field(&mut fields, "guild", "guild_id", guild_id);

            insert_field(&mut fields, "map", "emojis", {
                let mut emojis = Vec::new();
                for emoji in current_state.iter() {
                    emojis.push(emoji.clone());
                }
                emojis
            });
        }
        // @ci.expand_event_check GuildIntegrationsUpdate none
        FullEvent::GuildIntegrationsUpdate { guild_id } => {
            insert_field(&mut fields, "guild_ext", "guild_id", guild_id);
        }
        // @ci.expand_event_check GuildMemberAddition none
        FullEvent::GuildMemberAddition { new_member } => {
            insert_field(&mut fields, "member", "new_member", new_member.clone());
        }
        // @ci.expand_event_check GuildMemberRemoval none
        FullEvent::GuildMemberRemoval {
            guild_id,
            user,
            member_data_if_available,
        } => {
            if let Some(member_data_if_available) = member_data_if_available {
                insert_field(
                    &mut fields,
                    "member",
                    "member",
                    member_data_if_available.clone(),
                );
            } else {
                insert_field(&mut fields, "guild", "guild_id", guild_id);
                insert_field(&mut fields, "user", "user", user);
            }
        }
        // @ci.expand_event_check GuildMemberUpdate none
        FullEvent::GuildMemberUpdate {
            old_if_available,
            new,
            ..
        } => {
            if let Some(old) = old_if_available {
                insert_field(&mut fields, "member", "old", old);
            }
            if let Some(new) = new {
                insert_field(&mut fields, "member", "new", new);
            };
        }
        // @ci.expand_event_check GuildMembersChunk none
        FullEvent::GuildMembersChunk { .. } => return None,
        // @ci.expand_event_check GuildRoleCreate none
        FullEvent::GuildRoleCreate { new } => {
            insert_field(&mut fields, "role", "role", new);
        }
        // @ci.expand_event_check GuildRoleDelete none
        FullEvent::GuildRoleDelete {
            guild_id,
            removed_role_id,
            removed_role_data_if_available,
        } => {
            insert_field(&mut fields, "guild", "guild_id", guild_id);

            insert_field(&mut fields, "guild_role_delete", "role_id", removed_role_id);

            if let Some(removed_role_data) = removed_role_data_if_available {
                insert_field(&mut fields, "role", "role", removed_role_data.clone());
            }
        }
        // @ci.expand_event_check GuildRoleUpdate none
        FullEvent::GuildRoleUpdate {
            old_data_if_available,
            new,
        } => {
            if let Some(old) = old_data_if_available {
                insert_field(&mut fields, "role", "old", old.clone());
            }

            insert_field(&mut fields, "role", "new", new.clone());
        }
        // @ci.expand_event_check GuildScheduledEventCreate none
        FullEvent::GuildScheduledEventCreate { event } => {
            insert_field(&mut fields, "event", "event", event.clone());
        }
        // @ci.expand_event_check GuildScheduledEventDelete none
        FullEvent::GuildScheduledEventDelete { event } => {
            insert_field(&mut fields, "event", "event", event.clone());
        }
        // @ci.expand_event_check GuildScheduledEventUpdate none
        FullEvent::GuildScheduledEventUpdate { event } => {
            insert_field(&mut fields, "event", "event", event.clone());
        }
        // @ci.expand_event_check GuildScheduledEventUserAdd event:subscribed,GuildScheduledEventUserAddEvent
        FullEvent::GuildScheduledEventUserAdd { subscribed } => {
            insert_field(
                &mut fields,
                "guild_scheduled_event_user_add",
                "guild_id",
                subscribed.guild_id,
            );
            insert_field(
                &mut fields,
                "guild_scheduled_event_user_add",
                "event_id",
                subscribed.scheduled_event_id,
            );
            insert_field(
                &mut fields,
                "guild_scheduled_event_user_add",
                "user_id",
                subscribed.user_id,
            );
        }
        // @ci.expand_event_check GuildScheduledEventUserRemove event:unsubscribed,GuildScheduledEventUserRemoveEvent
        FullEvent::GuildScheduledEventUserRemove { unsubscribed } => {
            insert_field(
                &mut fields,
                "guild_scheduled_event_user_remove",
                "guild_id",
                unsubscribed.guild_id,
            );
            insert_field(
                &mut fields,
                "guild_scheduled_event_user_remove",
                "scheduled_event_id",
                unsubscribed.scheduled_event_id,
            );
            insert_field(
                &mut fields,
                "guild_scheduled_event_user_remove",
                "user_id",
                unsubscribed.user_id,
            );
        }
        // @ci.expand_event_check GuildStickersUpdate none
        FullEvent::GuildStickersUpdate {
            guild_id,
            current_state,
        } => {
            insert_field(&mut fields, "guild", "guild_id", guild_id);

            insert_field(&mut fields, "map", "stickers", {
                let mut stickers = Vec::new();
                for sticker in current_state.iter() {
                    stickers.push(sticker.clone());
                }
                stickers
            });
        }
        // @ci.expand_event_check GuildUpdate none
        FullEvent::GuildUpdate {
            old_data_if_available,
            new_data,
        } => {
            if let Some(old) = old_data_if_available {
                insert_field(&mut fields, "guild", "old", old);
            }

            insert_field(&mut fields, "guild", "new", new_data);
        }
        // @ci.expand_event_check IntegrationCreate none
        FullEvent::IntegrationCreate { integration } => {
            insert_field(&mut fields, "integration", "integration", integration);
        }
        // @ci.expand_event_check IntegrationDelete none
        FullEvent::IntegrationDelete {
            guild_id,
            integration_id,
            application_id,
        } => {
            insert_field(&mut fields, "integration", "guild_id", guild_id);
            insert_field(&mut fields, "integration", "integration_id", integration_id);
            insert_optional_field(&mut fields, "integration", "application_id", application_id);
        }
        // @ci.expand_event_check IntegrationUpdate none
        FullEvent::IntegrationUpdate { integration } => {
            insert_field(&mut fields, "integration", "integration", integration);
        }
        // @ci.expand_event_check InteractionCreate none
        FullEvent::InteractionCreate { interaction: _ } => return None, // We dont handle interactions create events in expand_events
        // @ci.expand_event_check InviteCreate event:data,InviteCreateEvent
        FullEvent::InviteCreate { data } => {
            insert_field(&mut fields, "invite", "channel_id", data.channel_id);
            insert_field(&mut fields, "invite", "code", data.code.to_string());
            insert_field(&mut fields, "invite", "created_at", data.created_at);
            insert_optional_field(&mut fields, "invite", "guild_id", data.guild_id);
            insert_optional_field(&mut fields, "invite", "inviter", data.inviter);
            insert_field(&mut fields, "expiry", "max_age", data.max_age);
            insert_field(&mut fields, "expiry", "max_uses", data.max_uses);
            insert_optional_field(
                &mut fields,
                "invite",
                "target_type",
                data.target_type.map(|x| match x {
                    serenity::all::InviteTargetType::Stream => "Stream".to_string(),
                    serenity::all::InviteTargetType::EmbeddedApplication => {
                        "EmbeddedApplication".to_string()
                    }
                    _ => "Unknown".to_string(),
                }),
            );
            insert_optional_field(&mut fields, "invite", "target_user", data.target_user);
            insert_optional_field(
                &mut fields,
                "invite",
                "target_application",
                data.target_application,
            );
            insert_field(&mut fields, "invite", "temporary", data.temporary);
            insert_field(&mut fields, "invite", "uses", data.uses);
        }
        // @ci.expand_event_check InviteDelete event:data,InviteDeleteEvent
        FullEvent::InviteDelete { data } => {
            insert_field(&mut fields, "invite_delete", "channel_id", data.channel_id);
            insert_optional_field(&mut fields, "invite_delete", "guild_id", data.guild_id);
            insert_field(&mut fields, "invite_delete", "code", data.code.to_string());
        }
        // @ci.expand_event_check Message none
        FullEvent::Message { new_message } => {
            insert_field(&mut fields, "message", "message", new_message.clone());
        }
        // @ci.expand_event_check MessageDelete none
        FullEvent::MessageDelete {
            guild_id,
            deleted_message_id,
            channel_id,
        } => {
            insert_optional_field(&mut fields, "message", "guild_id", guild_id);
            insert_field(&mut fields, "message", "message_id", deleted_message_id);
            insert_field(&mut fields, "message", "channel_id", channel_id);
        }
        // @ci.expand_event_check MessageDeleteBulk none
        FullEvent::MessageDeleteBulk {
            guild_id,
            channel_id,
            multiple_deleted_messages_ids,
        } => {
            insert_optional_field(&mut fields, "message", "guild_id", guild_id);
            insert_field(&mut fields, "message", "channel_id", channel_id);
            insert_field(
                &mut fields,
                "message",
                "message_ids",
                multiple_deleted_messages_ids,
            );
        }
        // @ci.expand_event_check MessageUpdate event:event,MessageUpdateEvent
        FullEvent::MessageUpdate {
            old_if_available,
            new,
            event,
        } => {
            if let Some(old) = old_if_available {
                insert_field(&mut fields, "message", "old", old.clone());
            }
            if let Some(new) = new {
                insert_field(&mut fields, "message", "new", new.clone());
            } else {
                insert_field(
                    &mut fields,
                    "warning",
                    "warning",
                    "This message has not been cached by Anti-Raid!".to_string(),
                );
            }

            insert_field(&mut fields, "event", "id", event.id);
            insert_field(&mut fields, "event", "channel_id", event.channel_id);
            insert_optional_field(&mut fields, "event", "author", event.author);
            insert_optional_field(&mut fields, "event", "content", event.content);
            insert_optional_field(&mut fields, "event", "timestamp", event.timestamp);
            insert_optional_field(
                &mut fields,
                "event",
                "edited_timestamp",
                event.edited_timestamp,
            );
            insert_optional_field(&mut fields, "event", "tts", event.tts);
            insert_optional_field(
                &mut fields,
                "event",
                "mention_everyone",
                event.mention_everyone,
            );
            insert_optional_field(&mut fields, "event", "mentions", event.mentions);
            insert_optional_field(&mut fields, "event", "mention_roles", event.mention_roles);
            insert_optional_field(
                &mut fields,
                "event",
                "mention_channels",
                event.mention_channels,
            );
            insert_optional_field(&mut fields, "event", "attachments", event.attachments);
            insert_optional_field(&mut fields, "event", "embeds", event.embeds);
            insert_optional_field(&mut fields, "event", "reactions", event.reactions);
            insert_optional_field(&mut fields, "event", "pinned", event.pinned);
            insert_optional_field(
                &mut fields,
                "event",
                "webhook_id",
                event.webhook_id.and_then(|x| x),
            );
            insert_optional_field(&mut fields, "event", "kind", event.kind);
            insert_optional_field(
                &mut fields,
                "event",
                "activity",
                event.activity.and_then(|x| x),
            );
            insert_optional_field(
                &mut fields,
                "event",
                "application",
                event.application.and_then(|x| x),
            );
            insert_optional_field(
                &mut fields,
                "event",
                "application_id",
                event.application_id.and_then(|x| x),
            );
            insert_optional_field(
                &mut fields,
                "event",
                "message_reference",
                event.message_reference.and_then(|x| x),
            );
            insert_optional_field(&mut fields, "event", "flags", event.flags.and_then(|x| x));
            insert_optional_field(
                &mut fields,
                "event",
                "referenced_message",
                event.referenced_message.and_then(|x| x.map(|x| *x)),
            );
            insert_optional_field(
                &mut fields,
                "event",
                "interaction",
                event.interaction.and_then(|x| x.map(|x| *x)),
            );
            insert_optional_field(
                &mut fields,
                "event",
                "thread",
                event.thread.and_then(|x| x.map(|x| *x)),
            );
            insert_optional_field(&mut fields, "event", "components", event.components);
            insert_optional_field(&mut fields, "event", "sticker_items", event.sticker_items);
            insert_optional_field(
                &mut fields,
                "event",
                "position",
                event.position.and_then(|x| x.map(|x| x.get())),
            );
            insert_optional_field(
                &mut fields,
                "event",
                "role_subscription_data",
                event.role_subscription_data.and_then(|x| x),
            );
            insert_optional_field(&mut fields, "event", "guild_id", event.guild_id);
            insert_optional_field(
                &mut fields,
                "event",
                "member",
                event.member.and_then(|x| x.map(|x| (*x))),
            );
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
        // @ci.expand_event_check StageInstanceCreate none
        FullEvent::StageInstanceCreate { stage_instance } => {
            insert_field(
                &mut fields,
                "stage_instance",
                "stage_instance",
                stage_instance,
            );
        }
        // @ci.expand_event_check StageInstanceDelete none
        FullEvent::StageInstanceDelete { stage_instance } => {
            insert_field(
                &mut fields,
                "stage_instance",
                "stage_instance",
                stage_instance,
            );
        }
        // @ci.expand_event_check StageInstanceUpdate none
        FullEvent::StageInstanceUpdate { stage_instance } => {
            insert_field(
                &mut fields,
                "stage_instance",
                "stage_instance",
                stage_instance,
            );
        }
        // @ci.expand_event_check ThreadCreate none
        FullEvent::ThreadCreate { thread } => {
            insert_field(&mut fields, "thread", "thread", thread);
        }
        // @ci.expand_event_check ThreadDelete none
        FullEvent::ThreadDelete {
            thread,
            full_thread_data,
        } => {
            if let Some(ftd) = full_thread_data {
                insert_field(&mut fields, "thread", "thread", ftd);
            } else {
                insert_field(&mut fields, "thread", "thread", thread);
            }
        }
        // @ci.expand_event_check ThreadListSync event:thread_list_sync,ThreadListSyncEvent
        FullEvent::ThreadListSync { thread_list_sync } => {
            insert_optional_field(
                &mut fields,
                "channel",
                "channel_id",
                thread_list_sync.channel_ids,
            );
            insert_field(
                &mut fields,
                "thread_list_sync",
                "guild_id",
                thread_list_sync.guild_id,
            );
            insert_field(
                &mut fields,
                "thread_list_sync",
                "threads",
                thread_list_sync.threads,
            );

            insert_field(
                &mut fields,
                "thread_list_sync",
                "members",
                thread_list_sync.members,
            );
        }
        // @ci.expand_event_check ThreadMemberUpdate none
        FullEvent::ThreadMemberUpdate { thread_member } => {
            if let Some(ref member) = thread_member.member {
                insert_field(&mut fields, "user", "member", member.clone());
            } else {
                insert_field(&mut fields, "user", "user_id", thread_member.user_id);
            }

            insert_optional_field(&mut fields, "thread", "guild_id", thread_member.guild_id);
            insert_field(&mut fields, "thread", "channel_id", thread_member.id);
        }
        // @ci.expand_event_check ThreadMembersUpdate event:thread_members_update,ThreadMembersUpdateEvent
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
                thread_members_update.added_members.into_vec(),
            );
            insert_field(
                &mut fields,
                "thread_members",
                "removed_member_ids",
                thread_members_update.removed_member_ids.into_vec(),
            );
        }
        // @ci.expand_event_check ThreadUpdate none
        FullEvent::ThreadUpdate { new, old } => {
            if let Some(old) = old {
                insert_field(&mut fields, "thread", "old", old);
            }

            insert_field(&mut fields, "thread", "new", new);
        }
        FullEvent::TypingStart { .. } => return None,
        FullEvent::UserUpdate { .. } => return None,
        FullEvent::VoiceChannelStatusUpdate { .. } => return None,
        FullEvent::VoiceServerUpdate { .. } => return None,
        FullEvent::VoiceStateUpdate { .. } => return None,
        // @ci.expand_event_check WebhookUpdate none
        FullEvent::WebhookUpdate {
            guild_id,
            belongs_to_channel_id,
        } => {
            insert_field(&mut fields, "webhook", "guild_id", guild_id);
            insert_field(
                &mut fields,
                "webhook",
                "belongs_to_channel_id",
                belongs_to_channel_id,
            );
        }
    }

    Some(fields)
}
// @ci.expand_event_check.end
