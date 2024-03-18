use crate::Error;
use indexmap::IndexMap;
use log::warn;
use serenity::all::{
    ActionExecution, ApplicationId, AuditLogEntryId, ChannelId, CommandId, EmojiId, EntitlementId,
    FullEvent, GenericId, GuildChannel, GuildId, MessageId, RoleId, RuleId as AutomodRuleId,
    ScheduledEventId, StickerId, UserId, IntegrationId
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
    UserIds(Vec<UserId>),

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

    // Scheduled Event Id
    ScheduledEventId(ScheduledEventId),

    // Integration Id
    IntegrationId(IntegrationId),

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

    // Sticker Map
    StickerMap(Vec<serenity::model::sticker::Sticker>),
    
    // Users
    Users(Vec<serenity::model::user::User>),

    //Embeds
    Embeds(Vec<serenity::model::channel::Embed>),

    // Attachments
    Attachments(Vec<serenity::model::channel::Attachment>),

    // Components
    Components(Vec<serenity::model::application::ActionRow>),

    // ThreadMembers
    ThreadMembers(Vec<serenity::model::guild::ThreadMember>),
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
    UserId => UserIds,
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
    serenity::model::guild::Emoji => EmojiMap,
    serenity::model::sticker::Sticker => StickerMap,
    serenity::model::user::User => Users,
    serenity::model::channel::Embed => Embeds,
    serenity::model::channel::Attachment => Attachments,
    serenity::model::application::ActionRow => Components,
    serenity::model::guild::ThreadMember => ThreadMembers,
}

impl From<GuildId> for FieldType {
    fn from(s: GuildId) -> Self {
        Self::Guild(s)
    }
}
impl From<IntegrationId> for FieldType {
    fn from(s: IntegrationId) -> Self {
        Self::IntegrationId(s)
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
impl From<ScheduledEventId> for FieldType {
    fn from(s: ScheduledEventId) -> Self {
        Self::ScheduledEventId(s)
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
impl From<u32> for FieldType {
    fn from(s: u32) -> Self {
        Self::Number(s.into())
    }
}
impl From<u8> for FieldType {
    fn from(s: u8) -> Self {
        Self::Number(s.into())
    }
}
impl From<i16> for FieldType {
    fn from(s: i16) -> Self {
        Self::Number(s as u64)
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

        // Handle Thread IDs
        if let Some(parent_id) = channel.parent_id {
            insert_field(fields, "channel_id", parent_id);
            insert_field(fields, "thread_id", channel.id);
        } else {
            insert_field(fields, "channel_id", channel.id);
        }
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

    fn expand_scheduled_event(
        fields: &mut IndexMap<String, Field>,
        event: serenity::model::guild::ScheduledEvent,
    ) {
        insert_field(fields, "event_id", event.id);
        insert_field(fields, "guild_id", event.guild_id);
        insert_field(fields, "event_name", event.name.clone());
        insert_field(fields, "event_start_time", event.start_time);
        insert_field(
            fields,
            "event_privacy_level",
            format!("{:?}", event.privacy_level).to_lowercase(),
        );
        insert_field(
            fields,
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
        insert_optional_field(fields, "event_channel_id", event.channel_id);
        insert_optional_field(fields, "creator_id", event.creator_id);
        insert_optional_field(fields, "event_description", event.description.clone());
        insert_optional_field(fields, "event_end_time", event.end_time);
    }
    fn expand_sticker_map(
        fields: &mut IndexMap<String, Field>,
        sticker_map: &HashMap<StickerId, serenity::model::sticker::Sticker>,
        guild_id: &GuildId,
    ) {
        insert_field(fields, "guild_id", *guild_id);

        insert_field(
            fields,
            "stickers",
            sticker_map.values().cloned().collect::<Vec<_>>(),
        );
    }

    fn expand_guild(fields: &mut IndexMap<String, Field>, guild: serenity::model::guild::Guild) {
        insert_field(fields, "guild_id", guild.id);
        insert_field(fields, "guild_name", guild.name.clone());
        insert_field(fields, "guild_owner_id", guild.owner_id);
        // insert_field(fields, "guild_nsfw_level", guild.nsfw_level);
        //optional fields
        insert_optional_field(fields, "guild_description", guild.description.clone());

    }

    fn expand_partial_guild(
        fields: &mut IndexMap<String, Field>,
        guild: serenity::model::guild::PartialGuild,
    ) {
        insert_field(fields, "guild_id", guild.id);
        insert_field(fields, "guild_name", guild.name.clone());
        insert_field(fields, "guild_owner_id", guild.owner_id);
        // insert_field(fields, "guild_nsfw_level", guild.nsfw_level.to_string());
        //optional fields
        insert_optional_field(fields, "guild_description", guild.description.clone());
    }

    fn expand_integration(fields: &mut IndexMap<String, Field>, integration: serenity::model::guild::Integration) {
        insert_field(fields, "integration_id", integration.id);
        insert_field(fields, "integration_name", integration.name.clone());
        insert_field(fields, "integration_type", integration.kind.clone());
        insert_field(fields, "integration_enabled", integration.enabled());
        insert_field(fields, "integration_account_id", integration.account.id.clone());
        insert_field(fields, "integration_account_name", integration.account.name.clone());

        //optional fields
        insert_optional_field(fields, "integration_syncing_status", integration.syncing());
        insert_optional_field(fields, "integration_role_id", integration.role_id);
        insert_optional_field(fields, "integration_guild_id", integration.guild_id);
        if let Some(user) = integration.user {
            insert_field(fields, "integration_user_id", user.id);
        }
    }

    fn expand_invite_create(fields: &mut IndexMap<String, Field>, data: serenity::model::event::InviteCreateEvent) {
        insert_field(fields, "invite_code", data.code);
        insert_field(fields, "invite_channel_id", data.channel_id);
        insert_field(fields, "invite_created_at", data.created_at);
        insert_field(fields, "invite_max_age", data.max_age);
        insert_field(fields, "invite_max_uses", data.max_uses);

        //optional fields
        insert_optional_field(fields, "invite_guild_id", data.guild_id);

    }

    fn expand_message(fields: &mut IndexMap<String, Field>, message: serenity::model::channel::Message) {
        insert_field(fields, "message_id", message.id);
        insert_field(fields, "message_channel_id", message.channel_id);
        insert_field(fields, "message_author", message.author.clone());
        insert_field(fields, "message_content", message.content.clone());
        insert_field(fields, "message_created_at", message.timestamp);
        insert_field(fields, "message_embeds", message.embeds.clone().into_vec());
        insert_field(fields, "message_attachments", message.attachments.clone().into_vec());
        insert_field(fields, "message_components", message.components.clone().into_vec());
        insert_field(fields, "message_kind", match message.kind {
            serenity::model::channel::MessageType::Regular => "Regular".to_string(),
            serenity::model::channel::MessageType::GroupRecipientAddition => "GroupRecipientAddition".to_string(),
            serenity::model::channel::MessageType::GroupRecipientRemoval => "GroupRecipientRemoval".to_string(),
            serenity::model::channel::MessageType::GroupCallCreation => "GroupCallCreation".to_string(),
            serenity::model::channel::MessageType::GroupNameUpdate => "GroupNameUpdate".to_string(),
            serenity::model::channel::MessageType::GroupIconUpdate => "GroupIconUpdate".to_string(),
            serenity::model::channel::MessageType::PinsAdd => "PinsAdd".to_string(),
            serenity::model::channel::MessageType::MemberJoin => "MemberJoin".to_string(),
            serenity::model::channel::MessageType::NitroBoost => "NitroBoost".to_string(),
            serenity::model::channel::MessageType::NitroTier1 => "NitroTier1".to_string(),
            serenity::model::channel::MessageType::NitroTier2 => "NitroTier2".to_string(),
            serenity::model::channel::MessageType::NitroTier3 => "NitroTier3".to_string(),
            serenity::model::channel::MessageType::ChannelFollowAdd => "ChannelFollowAdd".to_string(),
            serenity::model::channel::MessageType::GuildDiscoveryDisqualified => "GuildDiscoveryDisqualified".to_string(),
            serenity::model::channel::MessageType::GuildDiscoveryRequalified => "GuildDiscoveryRequalified".to_string(),
            serenity::model::channel::MessageType::GuildDiscoveryGracePeriodInitialWarning => "GuildDiscoveryGracePeriodInitialWarning".to_string(),
            serenity::model::channel::MessageType::GuildDiscoveryGracePeriodFinalWarning => "GuildDiscoveryGracePeriodFinalWarning".to_string(),
            serenity::model::channel::MessageType::ThreadCreated => "ThreadCreated".to_string(),
            serenity::model::channel::MessageType::InlineReply => "InlineReply".to_string(),
            serenity::model::channel::MessageType::ChatInputCommand => "ChatInputCommand".to_string(),
            serenity::model::channel::MessageType::ThreadStarterMessage => "ThreadStarterMessage".to_string(),
            serenity::model::channel::MessageType::GuildInviteReminder => "GuildInviteReminder".to_string(),
            serenity::model::channel::MessageType::ContextMenuCommand => "ContextMenuCommand".to_string(),
            serenity::model::channel::MessageType::AutoModAction => "AutoModAction".to_string(),
            serenity::model::channel::MessageType::RoleSubscriptionPurchase => "RoleSubscriptionPurchase".to_string(),
            serenity::model::channel::MessageType::InteractionPremiumUpsell => "InteractionPremiumUpsell".to_string(),
            serenity::model::channel::MessageType::StageStart => "StageStart".to_string(),
            serenity::model::channel::MessageType::StageEnd => "StageEnd".to_string(),
            serenity::model::channel::MessageType::StageSpeaker => "StageSpeaker".to_string(),
            serenity::model::channel::MessageType::StageTopic => "StageTopic".to_string(),
            serenity::model::channel::MessageType::GuildApplicationPremiumSubscription => "GuildApplicationPremiumSubscription".to_string(),
            _ => "Unknown".to_string(),
        });


        //optional fields
        insert_optional_field(fields, "message_updated_at", message.edited_timestamp);
        insert_optional_field(fields, "message_guild_id", message.guild_id);
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
            insert_field(&mut fields, "guild_id", subscribed.guild_id);
            insert_field(&mut fields, "event_id", subscribed.scheduled_event_id);
            insert_field(&mut fields, "user_id", subscribed.user_id);
        }
        FullEvent::GuildScheduledEventUserRemove { unsubscribed } => {
            insert_field(&mut fields, "guild_id", unsubscribed.guild_id);
            insert_field(&mut fields, "event_id", unsubscribed.scheduled_event_id);
            insert_field(&mut fields, "user_id", unsubscribed.user_id);
        }
        FullEvent::GuildStickersUpdate {
            guild_id,
            current_state,
        } => {
            expand_sticker_map(&mut fields, current_state, guild_id);
        }
        FullEvent::GuildUpdate { old_data_if_available, new_data, .. } => {
            expand_partial_guild(&mut fields, new_data.clone());
            if let Some(old) = old_data_if_available {
                expand_guild(&mut fields, old.clone());
            }
        }
        FullEvent::IntegrationCreate { integration, .. } => {
            expand_integration(&mut fields, integration.clone());
        }
        FullEvent::IntegrationDelete { guild_id, integration_id, application_id } => {
            insert_field(&mut fields, "guild_id", *guild_id);
            insert_field(&mut fields, "integration_id", *integration_id);
            insert_optional_field(&mut fields, "integration_application_id", *application_id);
        }
        FullEvent::IntegrationUpdate { integration, .. } => {
            expand_integration(&mut fields, integration.clone());
        }
        FullEvent::InteractionCreate { .. } => return None,
        FullEvent::InviteCreate { data, .. } => {
            expand_invite_create(&mut fields, data.clone());
        }
        FullEvent::InviteDelete { data, .. } => {
            insert_field(&mut fields, "invite_code", data.code.clone());
            insert_field(&mut fields, "invite_channel_id", data.channel_id);
            insert_optional_field(&mut fields, "invite_guild_id", data.guild_id);
        }
        FullEvent::Message { new_message, .. } => {
            expand_message(&mut fields, new_message.clone());
        }
        FullEvent::MessageDelete { guild_id, deleted_message_id, channel_id, .. } => {
            insert_optional_field(&mut fields, "guild_id", *guild_id);
            insert_field(&mut fields, "message_id", *deleted_message_id);
            insert_field(&mut fields, "channel_id", *channel_id);
        }
        FullEvent::MessageDeleteBulk { guild_id, channel_id, multiple_deleted_messages_ids, .. } => {
            insert_optional_field(&mut fields, "guild_id", *guild_id);
            insert_field(&mut fields, "channel_id", *channel_id);
            insert_field(&mut fields, "message_ids", multiple_deleted_messages_ids.clone());
        }
        FullEvent::MessageUpdate { old_if_available, new, .. } => {
            if let Some(old) = old_if_available {
                expand_message(&mut fields, old.clone());
            }
            if let Some(new) = new {
                expand_message(&mut fields, new.clone());
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
        FullEvent::ThreadCreate { thread, .. } => {
            expand_channel(&mut fields, thread);
        }
        FullEvent::ThreadDelete { thread, .. } => {
            insert_field(&mut fields, "guild_id", thread.guild_id);
            insert_field(&mut fields, "thread_id", thread.id);
            insert_field(&mut fields, "channel_id", thread.parent_id);
            insert_field(
                &mut fields,
                "channel_type",
                match thread.kind {
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
        FullEvent::ThreadListSync { .. } => {
            // expand_channel(&mut fields, thread_list_sync);
            // NO NEED TO HANDLE THIS...
            return None
        }
        FullEvent::ThreadMemberUpdate { thread_member, .. } => {

            if let Some(member) = thread_member.member.clone() {
                expand_member(&mut fields, member);
            }
            insert_optional_field(&mut fields, "guild_id", thread_member.guild_id);
            insert_field(&mut fields, "channel_id", thread_member.id);
            insert_field(&mut fields, "user_id", thread_member.user_id);
        }
        FullEvent::ThreadMembersUpdate { thread_members_update, .. } => {
            insert_field(&mut fields, "guild_id", thread_members_update.guild_id);
            insert_field(&mut fields, "channel_id", thread_members_update.id);
            insert_field(&mut fields, "thread_member_count", thread_members_update.member_count);
            insert_field(&mut fields, "removed_member_ids", thread_members_update.removed_member_ids.clone().into_vec());
        }
        FullEvent::ThreadUpdate { new, .. } => {
            expand_channel(&mut fields, new);
        }
        FullEvent::TypingStart { .. } => return None,
        FullEvent::UserUpdate { .. } => return None,
        FullEvent::VoiceChannelStatusUpdate { .. } => return None,
        FullEvent::VoiceServerUpdate { .. } => return None,
        FullEvent::VoiceStateUpdate { .. } => return None,
        FullEvent::WebhookUpdate { .. } => return None,
    }

    Some(fields)
}
