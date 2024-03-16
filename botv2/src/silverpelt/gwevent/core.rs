use crate::Error;
use indexmap::IndexMap;
use log::warn;
use serenity::all::{
    ActionExecution, ApplicationId, AuditLogEntryId, ChannelId, CommandId, EmojiId, EntitlementId,
    FullEvent, GenericId, GuildChannel, GuildId, MessageId, RoleId, RuleId as AutomodRuleId,
    UserId,
};
use serenity::model::guild::automod::Action;
use serenity::model::timestamp::Timestamp;
use serenity::nonmax::{NonMaxU16, NonMaxU8};
use small_fixed_array::FixedString;
use std::collections::HashMap;
use strum::VariantNames;
/// Returns all events
#[allow(dead_code)]
pub const fn event_list() -> &'static [&'static str] {
    FullEvent::VARIANTS
}

/// Given an event and a module, return whether or not to filter said event
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

pub enum FieldType {
    /// A string
    Strings(Vec<String>),

    Bool(bool),

    Number(u64),

    /// A user id
    Users(Vec<UserId>),

    /// A channel id
    Channels(Vec<ChannelId>),

    /// A role id
    Roles(Vec<RoleId>),

    /// A message id
    Messages(Vec<MessageId>),

    /// A guild id
    Guild(GuildId),

    // Command Id
    Command(CommandId),

    // Entitlement ID
    Entitlement(EntitlementId),

    // Application Id
    Application(ApplicationId),

    // Audit Log Id
    AuditLogId(AuditLogEntryId),

    /// An emoji id
    Emojis(Vec<EmojiId>),

    /// A generic id
    GenericIds(Vec<GenericId>),

    /// An automod action
    AutomodActions(Vec<serenity::model::guild::automod::Action>),

    // Audit log Actions
    AuditLogActions(Vec<serenity::model::guild::audit_log::Action>),

    /// An automod rule id
    AutomodRuleIds(Vec<AutomodRuleId>),

    // Trigger
    AutomodTrigger(serenity::model::guild::automod::Trigger),

    // TimeStamp
    TimeStamp(serenity::model::timestamp::Timestamp),

    // Changes
    AuditLogActionsChanges(Vec<serenity::model::guild::audit_log::Change>),

    // Options
    AuditLogOptions(Vec<serenity::model::guild::audit_log::Options>),

    // Emoji Map
    EmojiMap(Vec<serenity::model::guild::Emoji>),
}

macro_rules! from_field_type {
    ($($t:ty => $variant:ident),* $(,)?) => {
        $(
            impl From<$t> for FieldType {
                fn from(s: $t) -> Self {
                    Self::$variant(vec![s])
                }
            }
            impl From<Vec<$t>> for FieldType {
                fn from(s: Vec<$t>) -> Self {
                    Self::$variant(s)
                }
            }
        )*
    };
}

from_field_type! {
    String => Strings,
    UserId => Users,
    ChannelId => Channels,
    RoleId => Roles,
    MessageId => Messages,
    EmojiId => Emojis,
    GenericId => GenericIds,
    Action => AutomodActions,
    AutomodRuleId => AutomodRuleIds,
    serenity::model::guild::audit_log::Action => AuditLogActions,
    serenity::model::guild::audit_log::Change => AuditLogActionsChanges,
    serenity::model::guild::audit_log::Options => AuditLogOptions,
    serenity::model::guild::Emoji => EmojiMap
}

impl From<GuildId> for FieldType {
    fn from(s: GuildId) -> Self {
        Self::Guild(s)
    }
}

impl From<AuditLogEntryId> for FieldType {
    fn from(s: AuditLogEntryId) -> Self {
        Self::AuditLogId(s)
    }
}

impl From<CommandId> for FieldType {
    fn from(s: CommandId) -> Self {
        Self::Command(s)
    }
}
impl From<ApplicationId> for FieldType {
    fn from(s: ApplicationId) -> Self {
        Self::Application(s)
    }
}
impl From<EntitlementId> for FieldType {
    fn from(s: EntitlementId) -> Self {
        Self::Entitlement(s)
    }
}

impl From<Timestamp> for FieldType {
    fn from(s: Timestamp) -> Self {
        Self::TimeStamp(s)
    }
}

impl From<bool> for FieldType {
    fn from(s: bool) -> Self {
        Self::Bool(s)
    }
}

impl From<serenity::model::guild::automod::Trigger> for FieldType {
    fn from(s: serenity::model::guild::automod::Trigger) -> Self {
        Self::AutomodTrigger(s)
    }
}

impl From<FixedString<u32>> for FieldType {
    fn from(s: FixedString<u32>) -> Self {
        Self::Strings(vec![s.to_string()])
    }
}

impl From<FixedString<u16>> for FieldType {
    fn from(s: FixedString<u16>) -> Self {
        Self::Strings(vec![s.to_string()])
    }
}
impl From<FixedString<u8>> for FieldType {
    fn from(s: FixedString<u8>) -> Self {
        Self::Strings(vec![s.to_string()])
    }
}

impl From<NonMaxU16> for FieldType {
    fn from(s: NonMaxU16) -> Self {
        Self::Number(s.get().into())
    }
}
impl From<NonMaxU8> for FieldType {
    fn from(s: NonMaxU8) -> Self {
        Self::Number(s.get().into())
    }
}

#[allow(dead_code)]
pub struct Field {
    /// The value of the field
    value: Vec<FieldType>,
}

impl Field {
    /// Create a new field
    pub fn new(value: FieldType) -> Self {
        Self { value: vec![value] }
    }
}

/// Given an event, expand it to a hashmap of fields
#[allow(dead_code)]
pub fn expand_event(event: &FullEvent) -> Option<IndexMap<String, Field>> {
    let mut fields = IndexMap::new();

    fn insert_field<T: Into<FieldType>>(fields: &mut IndexMap<String, Field>, key: &str, value: T) {
        let value = value.into();
        match fields.entry(key.to_string()) {
            indexmap::map::Entry::Occupied(mut entry) => {
                entry.get_mut().value.push(value);
            }
            indexmap::map::Entry::Vacant(entry) => {
                entry.insert(Field::new(value));
            }
        }
    }

    fn insert_optional_field<T: Into<FieldType>>(
        fields: &mut IndexMap<String, Field>,
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
                        entry.insert(Field::new(value));
                    }
                }
            }
            None => {
                fields.insert(key.to_string(), Field::new("None".to_string().into()));
            }
        }
    }

    fn expand_action_execution(fields: &mut IndexMap<String, Field>, execution: &ActionExecution) {
        insert_field(fields, "guild_id", execution.guild_id);
        insert_field(fields, "action", execution.action.clone());
        insert_field(fields, "rule_id", execution.rule_id);
        insert_field(
            fields,
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
        insert_field(fields, "content", execution.content.clone().into_string());
        insert_field(fields, "user_id", execution.user_id);

        insert_optional_field(fields, "channel_id", execution.channel_id);
        insert_optional_field(fields, "message_id", execution.message_id);
        insert_optional_field(
            fields,
            "alert_system_message_id",
            execution.alert_system_message_id,
        );
        insert_optional_field(fields, "matched_keyword", execution.matched_keyword.clone());
        insert_optional_field(fields, "matched_content", execution.matched_content.clone());
    }

    fn expand_rule(
        fields: &mut IndexMap<String, Field>,
        rule: &serenity::model::guild::automod::Rule,
    ) {
        insert_field(fields, "rule_id", rule.id);
        insert_field(fields, "guild_id", rule.guild_id);
        insert_field(fields, "rule_name", rule.name.clone());
        insert_field(fields, "creator_id", rule.creator_id);
        insert_field(
            fields,
            "event_type",
            match rule.event_type {
                serenity::model::guild::automod::EventType::MessageSend => {
                    "MessageSend".to_string()
                }
                _ => "Unknown".to_string(),
            },
        );
        insert_field(fields, "trigger", rule.trigger.clone());
        insert_field(fields, "actions", rule.actions.clone().into_vec());

        insert_field(fields, "rule_enabled", rule.enabled);
        insert_field(fields, "exempt_roles", rule.exempt_roles.clone().into_vec());
        insert_field(
            fields,
            "exempt_channels",
            rule.exempt_channels.clone().into_vec(),
        );
    }

    fn expand_channel(fields: &mut IndexMap<String, Field>, channel: &GuildChannel) {
        insert_field(fields, "channel_id", channel.id);
        insert_field(fields, "guild_id", channel.guild_id);
        insert_field(fields, "channel_name", channel.name.clone());
        insert_field(fields, "nsfw", channel.nsfw);
        insert_field(
            fields,
            "channel_type",
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

        // Optional fields
        insert_optional_field(fields, "channel_topic", channel.topic.clone());
        insert_optional_field(fields, "rate_limit_per_user", channel.rate_limit_per_user);
        insert_optional_field(fields, "parent_id", channel.parent_id);
        insert_optional_field(fields, "user_limit", channel.user_limit);
    }

    fn expand_command_permissions(
        fields: &mut IndexMap<String, Field>,
        permission: &serenity::model::application::CommandPermissions,
    ) {
        insert_field(fields, "command_id", permission.id);
        insert_field(fields, "application_id", permission.application_id);
        // Continue from here
    }

    fn expand_entitlement(
        fields: &mut IndexMap<String, Field>,
        entitlement: &serenity::model::monetization::Entitlement,
    ) {
        insert_field(fields, "entitlement_id", entitlement.id);
        insert_field(fields, "application_id", entitlement.application_id);
        insert_field(
            fields,
            "entitlement_type",
            format!("{:?}", entitlement.kind).to_lowercase(),
        );
        insert_field(fields, "entitlement_deleted", entitlement.deleted);

        // Optional Fields
        insert_optional_field(fields, "guild_id", entitlement.guild_id);
        insert_optional_field(fields, "user_id", entitlement.user_id);
        insert_optional_field(fields, "entitlement_starts_at", entitlement.starts_at);
        insert_optional_field(fields, "entitlement_ends_at", entitlement.ends_at);
    }

    fn expand_audit_log_entry(
        fields: &mut IndexMap<String, Field>,
        entry: &serenity::model::guild::audit_log::AuditLogEntry,
        guild_id: &GuildId,
    ) {
        insert_field(fields, "guild_id", *guild_id);
        insert_field(fields, "action", entry.action.clone());
        insert_field(fields, "user_id", entry.user_id);
        insert_field(fields, "audit_log_id", entry.id);
        insert_optional_field(fields, "reason", entry.reason.clone());
        insert_optional_field(fields, "audit_log_target_id", entry.target_id);
        insert_optional_field(fields, "audit_log_chages", entry.changes.clone());
        insert_optional_field(fields, "audit_log_options", entry.options.clone());
    }

    fn expand_user(
        fields: &mut IndexMap<String, Field>,
        user: serenity::model::user::User,
        guild_id: &GuildId,
    ) {
        insert_field(fields, "guild_id", *guild_id);
        insert_field(fields, "user_id", user.id);
        insert_field(fields, "username", user.name.clone());
        insert_field(fields, "is_bot", user.bot());

        //optional fields
        insert_optional_field(fields, "global_username", user.global_name);
    }

    fn expand_emoji_map(
        fields: &mut IndexMap<String, Field>,
        emoji_map: &HashMap<EmojiId, serenity::model::guild::Emoji>,
        guild_id: &GuildId,
    ) {
        insert_field(fields, "guild_id", *guild_id);

        insert_field(
            fields,
            "emojis",
            emoji_map.values().cloned().collect::<Vec<_>>(),
        );
    }

    fn expand_member(fields: &mut IndexMap<String, Field>, member: serenity::model::guild::Member) {
        expand_user(fields, member.user, &member.guild_id);
        insert_field(fields, "roles", member.roles.clone().into_vec());
        //optional fields
        insert_optional_field(fields, "nick", member.nick);
        insert_optional_field(fields, "joined_timestamp", member.joined_at);
        insert_optional_field(fields, "premium_since", member.premium_since);
    }

    fn expand_role(fields: &mut IndexMap<String, Field>, role: serenity::model::guild::Role) {
        insert_field(fields, "role_id", role.id);
        insert_field(fields, "guild_id", role.guild_id);
        insert_field(fields, "is_hoisted", role.hoist());
        insert_field(fields, "is_managed", role.managed());
        insert_field(fields, "is_mentionable", role.mentionable());
        insert_field(fields, "role_name", role.name.clone());
        // insert_field(fields, "role_position", role.position.into());
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
        FullEvent::ChannelDelete { channel, .. } => {
            expand_channel(&mut fields, channel);
        }
        FullEvent::ChannelPinsUpdate { pin } => {
            insert_field(&mut fields, "channel_id", pin.channel_id);
            insert_optional_field(&mut fields, "last_pin_timestamp", pin.last_pin_timestamp);
        }
        FullEvent::ChannelUpdate { old, new, .. } => {
            if let Some(old) = old {
                expand_channel(&mut fields, old);
            }
            expand_channel(&mut fields, new);
        }
        FullEvent::CommandPermissionsUpdate { permission, .. } => {
            expand_command_permissions(&mut fields, permission);
        }
        FullEvent::EntitlementCreate { entitlement, .. } => {
            expand_entitlement(&mut fields, entitlement);
        }
        FullEvent::EntitlementDelete { entitlement, .. } => {
            expand_entitlement(&mut fields, entitlement);
        }
        FullEvent::EntitlementUpdate { entitlement, .. } => {
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
            expand_user(&mut fields, banned_user.clone(), guild_id);
        }
        FullEvent::GuildBanRemoval {
            guild_id,
            unbanned_user,
        } => {
            expand_user(&mut fields, unbanned_user.clone(), guild_id);
        }
        FullEvent::GuildCreate { guild, is_new } => {
            insert_field(&mut fields, "guild_id", guild.id);
            insert_optional_field(&mut fields, "is_new", *is_new);
        }
        FullEvent::GuildDelete { incomplete, .. } => {
            insert_field(&mut fields, "guild_id", incomplete.id);
        }
        FullEvent::GuildEmojisUpdate {
            guild_id,
            current_state,
        } => {
            expand_emoji_map(&mut fields, current_state, guild_id);
        }
        FullEvent::GuildIntegrationsUpdate { guild_id, .. } => {
            insert_field(&mut fields, "guild_id", *guild_id);
        }
        FullEvent::GuildMemberAddition { new_member } => {
            expand_member(&mut fields, new_member.clone());
        }
        FullEvent::GuildMemberRemoval { guild_id, user, .. } => {
            expand_user(&mut fields, user.clone(), guild_id);
        }
        FullEvent::GuildMemberUpdate {
            old_if_available,
            new,
            ..
        } => {
            if let Some(old) = old_if_available {
                expand_member(&mut fields, old.clone());
            }
            if let Some(new) = new {
                expand_member(&mut fields, new.clone());
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
            insert_field(&mut fields, "guild_id", *guild_id);
            insert_field(&mut fields, "role_id", *removed_role_id);
            if let Some(removed_role_data) = removed_role_data_if_available {
                expand_role(&mut fields, removed_role_data.clone());
            }
        }
        FullEvent::GuildRoleUpdate { old_data_if_available, new } => {
            if let Some(old) = old_data_if_available {
                expand_role(&mut fields, old.clone());
            }
            expand_role(&mut fields, new.clone());
        }
        _ => {}
    }

    Some(fields)
}
