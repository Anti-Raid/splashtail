use crate::Error;
use indexmap::IndexMap;
use log::warn;
use serenity::all::{
    ActionExecution, ChannelId, EmojiId, FullEvent, GenericId, GuildChannel, GuildId, MessageId,
    RoleId, RuleId as AutomodRuleId, UserId, CommandId, ApplicationId
};
use serenity::nonmax::NonMaxU16;
use serenity::model::guild::automod::Action;
use serenity::model::timestamp::Timestamp;
use small_fixed_array::FixedString;
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

    // Application Id
    Application(ApplicationId),

    /// An emoji id
    Emojis(Vec<EmojiId>),

    /// A generic id
    GenericIds(Vec<GenericId>),

    /// An automod action
    AutomodActions(Vec<serenity::model::guild::automod::Action>),

    /// An automod rule id
    AutomodRuleIds(Vec<AutomodRuleId>),

    // Trigger
    AutomodTrigger(serenity::model::guild::automod::Trigger),

    // TimeStamp
    TimeStamp(serenity::model::timestamp::Timestamp),
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
}

impl From<GuildId> for FieldType {
    fn from(s: GuildId) -> Self {
        Self::Guild(s)
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

impl From<NonMaxU16> for FieldType {
    fn from(s: NonMaxU16) -> Self {
        Self::Number(s.get().into())
    }
}

#[allow(dead_code)]
pub struct Field {
    /// The value of the field
    value: FieldType,
}

impl Field {
    /// Create a new field
    pub fn new(value: FieldType) -> Self {
        Self { value }
    }
}

/// Given an event, expand it to a hashmap of fields
#[allow(dead_code)]
pub fn expand_event(event: &FullEvent) -> Option<IndexMap<String, Field>> {
    let mut fields = IndexMap::new();

    fn insert_field<T: Into<FieldType>>(fields: &mut IndexMap<String, Field>, key: &str, value: T) {
        fields.insert(key.to_string(), Field::new(value.into()));
    }

    fn insert_optional_field<T: Into<FieldType>>(
        fields: &mut IndexMap<String, Field>,
        key: &str,
        option: Option<T>,
    ) {
        fields.insert(
            key.to_string(),
            Field::new(match option {
                Some(value) => value.into(),
                None => "None".to_string().into(),
            }),
        );
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
        insert_field(fields, "id", rule.id);
        insert_field(fields, "guild_id", rule.guild_id);
        insert_field(fields, "name", rule.name.clone());
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

        insert_field(fields, "enabled", rule.enabled);
        insert_field(fields, "exempt_roles", rule.exempt_roles.clone().into_vec());
        insert_field(
            fields,
            "exempt_channels",
            rule.exempt_channels.clone().into_vec(),
        );
    }

    fn expand_channel(fields: &mut IndexMap<String, Field>, channel: &GuildChannel) {
        insert_field(fields, "id", channel.id);
        insert_field(fields, "guild_id", channel.guild_id);
        insert_field(fields, "name", channel.name.clone());
        insert_field(fields, "nsfw", channel.nsfw);
        insert_field(
            fields,
            "kind",
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
        insert_optional_field(fields, "topic", channel.topic.clone());
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
        FullEvent::ChannelUpdate { new, .. } => {
            expand_channel(&mut fields, new);
        }
        FullEvent::CommandPermissionsUpdate { permission, .. } => {
            expand_command_permissions(&mut fields, permission);
        }

        _ => {}
    }

    Some(fields)
}
