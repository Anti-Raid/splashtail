use crate::Error;
use log::warn;
use serenity::all::{FullEvent, GuildId, UserId};
use strum::VariantNames;

/// Returns all events
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
        FullEvent::MessagePollVoteAdd { event } => {
            if let Some(guild_id) = &event.guild_id {
                guild_id.to_owned()
            } else {
                return Err(None);
            }
        }
        FullEvent::MessagePollVoteRemove { event } => {
            if let Some(guild_id) = &event.guild_id {
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
        FullEvent::PresenceUpdate { .. } => return Err(None), // We dont handle precenses
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
        FullEvent::MessagePollVoteAdd { event } => event.user_id,
        FullEvent::MessagePollVoteRemove { event } => event.user_id,
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
        FullEvent::PresenceUpdate { .. } => return Err(None), // We dont handle precenses
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
