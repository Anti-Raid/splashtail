use serenity::model::timestamp::Timestamp;
use serenity::nonmax::{NonMaxU16, NonMaxU8};
use serenity::small_fixed_array::{FixedArray, FixedString};

#[derive(serde::Serialize, serde::Deserialize)]
pub enum FieldType {
    // Primitive Types
    Bool(bool),
    Number(u64),
    Strings(Vec<String>),

    // Discord Primitives
    CommandPermissions(serenity::all::CommandPermissions),
    GuildMemberFlags(serenity::all::GuildMemberFlags),
    NsfwLevels(Vec<serenity::model::guild::NsfwLevel>),
    Permissions(serenity::all::Permissions),
    PermissionOverwrites(Vec<serenity::all::PermissionOverwrite>),

    // Discord ID Types
    ApplicationId(serenity::all::ApplicationId),
    AuditLogId(serenity::all::AuditLogEntryId),
    ChannelIds(Vec<serenity::all::ChannelId>),
    GenericIds(Vec<serenity::all::GenericId>),
    GuildId(serenity::all::GuildId),
    IntegrationId(serenity::all::IntegrationId),
    MessageIds(Vec<serenity::all::MessageId>),
    RoleIds(Vec<serenity::all::RoleId>),
    ScheduledEventId(serenity::all::ScheduledEventId),
    UserIds(Vec<serenity::all::UserId>),

    // Discord Structures
    ActionRows(Vec<serenity::model::application::ActionRow>),
    Attachment(serenity::model::channel::Attachment),
    AuditLogAction(serenity::model::guild::audit_log::Action),
    AuditLogActionsChanges(Vec<serenity::model::guild::audit_log::Change>),
    AuditLogEntry(serenity::model::guild::audit_log::AuditLogEntry),
    AuditLogOptions(Vec<serenity::model::guild::audit_log::Options>),
    AutomodActions(Vec<serenity::model::guild::automod::Action>),
    AutomodActionExecutions(Vec<serenity::model::guild::automod::ActionExecution>),
    AutomodRules(Vec<serenity::model::guild::automod::Rule>),
    AutomodTrigger(serenity::model::guild::automod::Trigger),
    Channels(Vec<serenity::model::channel::Channel>),
    Embeds(Vec<serenity::model::channel::Embed>),
    Emojis(Vec<serenity::model::guild::Emoji>),
    Entitlements(Vec<serenity::all::Entitlement>),
    Guild(serenity::model::guild::Guild),
    Integrations(Vec<serenity::model::guild::Integration>),
    Member(serenity::model::guild::Member),
    Messages(Vec<serenity::model::channel::Message>),
    MessageUpdateEvent(serenity::model::event::MessageUpdateEvent),
    PartialGuildChannels(Vec<serenity::all::PartialGuildChannel>),
    PartialGuild(serenity::model::guild::PartialGuild),
    Roles(Vec<serenity::model::guild::Role>),
    ScheduledEvents(Vec<serenity::model::guild::ScheduledEvent>),
    StageInstances(Vec<serenity::all::StageInstance>),
    Stickers(Vec<serenity::model::sticker::Sticker>),
    ThreadMembers(Vec<serenity::model::guild::ThreadMember>),
    Timestamp(Timestamp),
    Users(Vec<serenity::model::user::User>),

    // Special Types
    JsonValue(serde_json::Value),
    None,
}

impl FieldType {
    pub fn name(&self) -> &'static str {
        match self {
            FieldType::Bool(_) => "Bool",
            FieldType::Number(_) => "Number",
            FieldType::Strings(_) => "Strings",
            FieldType::CommandPermissions(_) => "CommandPermissions",
            FieldType::GuildMemberFlags(_) => "GuildMemberFlags",
            FieldType::NsfwLevels(_) => "NsfwLevels",
            FieldType::Permissions(_) => "Permissions",
            FieldType::PermissionOverwrites(_) => "PermissionOverwrites",
            FieldType::ApplicationId(_) => "ApplicationId",
            FieldType::AuditLogId(_) => "AuditLogId",
            FieldType::ChannelIds(_) => "ChannelIds",
            FieldType::GenericIds(_) => "GenericIds",
            FieldType::GuildId(_) => "GuildId",
            FieldType::IntegrationId(_) => "IntegrationId",
            FieldType::MessageIds(_) => "MessageIds",
            FieldType::RoleIds(_) => "RoleIds",
            FieldType::ScheduledEventId(_) => "ScheduledEventId",
            FieldType::UserIds(_) => "UserIds",
            FieldType::ActionRows(_) => "ActionRows",
            FieldType::Attachment(_) => "Attachment",
            FieldType::AuditLogAction(_) => "AuditLogAction",
            FieldType::AuditLogActionsChanges(_) => "AuditLogActionsChanges",
            FieldType::AuditLogEntry(_) => "AuditLogEntry",
            FieldType::AuditLogOptions(_) => "AuditLogOptions",
            FieldType::AutomodActions(_) => "AutomodActions",
            FieldType::AutomodActionExecutions(_) => "AutomodActionExecutions",
            FieldType::AutomodRules(_) => "AutomodRules",
            FieldType::AutomodTrigger(_) => "AutomodTrigger",
            FieldType::Channels(_) => "Channels",
            FieldType::Embeds(_) => "Embeds",
            FieldType::Emojis(_) => "Emojis",
            FieldType::Entitlements(_) => "Entitlements",
            FieldType::Guild(_) => "Guild",
            FieldType::Integrations(_) => "Integrations",
            FieldType::Member(_) => "Member",
            FieldType::Messages(_) => "Messages",
            FieldType::MessageUpdateEvent(_) => "MessageUpdateEvent",
            FieldType::PartialGuildChannels(_) => "PartialGuildChannels",
            FieldType::PartialGuild(_) => "PartialGuild",
            FieldType::Roles(_) => "Roles",
            FieldType::ScheduledEvents(_) => "ScheduledEvents",
            FieldType::StageInstances(_) => "StageInstances",
            FieldType::Stickers(_) => "Stickers",
            FieldType::ThreadMembers(_) => "ThreadMembers",
            FieldType::Timestamp(_) => "Timestamp",
            FieldType::Users(_) => "Users",
            FieldType::JsonValue(_) => "JsonValue",
            FieldType::None => "None",
        }
    }
}

macro_rules! from_field_type {
    ($($t:ty => $variant:ident),* $(,)?) => {
        $(
            impl From<$t> for FieldType {
                fn from(s: $t) -> Self {
                    Self::$variant(s)
                }
            }
        )*
    };
}

macro_rules! from_field_type_multiple {
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
            impl From<FixedArray<$t>> for FieldType {
                fn from(s: FixedArray<$t>) -> Self {
                    Self::$variant(s.into_iter().collect())
                }
            }
        )*
    };
}

macro_rules! from_field_type_tostring {
    ($($t:ty => $variant:ident),* $(,)?) => {
        $(
            impl From<$t> for FieldType {
                fn from(s: $t) -> Self {
                    Self::$variant(vec![s.to_string()])
                }
            }

            impl From<Vec<$t>> for FieldType {
                fn from(s: Vec<$t>) -> Self {
                    Self::$variant(s.into_iter().map(|s| s.to_string()).collect())
                }
            }

            impl From<FixedArray<$t>> for FieldType {
                fn from(s: FixedArray<$t>) -> Self {
                    Self::$variant(s.into_iter().map(|s| s.to_string()).collect())
                }
            }
        )*
    };
}

macro_rules! from_field_type_nonmax {
    ($($t:ty => $variant:ident),* $(,)?) => {
        $(
            impl From<$t> for FieldType {
                fn from(s: $t) -> Self {
                    Self::$variant(s.get().into())
                }
            }
        )*
    };
}

macro_rules! from_field_type_number {
    ($($t:ty => $variant:ident),* $(,)?) => {
        $(
            impl From<$t> for FieldType {
                fn from(s: $t) -> Self {
                    Self::$variant(s as u64)
                }
            }
        )*
    };
}

from_field_type_multiple! {
    // Primitive Types
    String => Strings,

    // Discord Primitives
    serenity::model::guild::NsfwLevel => NsfwLevels,
    serenity::all::PermissionOverwrite => PermissionOverwrites,

    // Discord ID Types
    serenity::all::ChannelId => ChannelIds,
    serenity::all::GenericId => GenericIds,
    serenity::all::MessageId => MessageIds,
    serenity::all::RoleId => RoleIds,
    serenity::all::UserId => UserIds,

    // Discord Structures
    serenity::model::application::ActionRow => ActionRows,
    serenity::model::guild::audit_log::Change => AuditLogActionsChanges,
    serenity::model::guild::audit_log::Options => AuditLogOptions,
    serenity::model::guild::automod::Action => AutomodActions,
    serenity::model::guild::automod::ActionExecution => AutomodActionExecutions,
    serenity::model::guild::automod::Rule => AutomodRules,
    serenity::model::channel::Channel => Channels,
    serenity::model::channel::Embed => Embeds,
    serenity::model::guild::Emoji => Emojis,
    serenity::all::Entitlement => Entitlements,
    serenity::all::Integration => Integrations,
    serenity::model::channel::Message => Messages,
    serenity::all::PartialGuildChannel => PartialGuildChannels,
    serenity::model::guild::Role => Roles,
    serenity::model::guild::ScheduledEvent => ScheduledEvents,
    serenity::all::StageInstance => StageInstances,
    serenity::model::sticker::Sticker => Stickers,
    serenity::model::guild::ThreadMember => ThreadMembers,
    serenity::model::user::User => Users,
}

from_field_type! {
    // Primitive Types
    bool => Bool,

    // Discord Primitives
    serenity::all::CommandPermissions => CommandPermissions,
    serenity::all::GuildMemberFlags => GuildMemberFlags,
    serenity::all::Permissions => Permissions,

    // Discord ID Types
    serenity::all::ApplicationId => ApplicationId,
    serenity::all::AuditLogEntryId => AuditLogId,
    serenity::all::GuildId => GuildId,
    serenity::all::IntegrationId => IntegrationId,
    serenity::all::ScheduledEventId => ScheduledEventId,

    // Discord Structures
    serenity::all::Attachment => Attachment,
    serenity::model::guild::audit_log::Action => AuditLogAction,
    serenity::model::guild::audit_log::AuditLogEntry => AuditLogEntry,
    serenity::model::guild::automod::Trigger => AutomodTrigger,
    serenity::all::Guild => Guild,
    serenity::model::event::MessageUpdateEvent => MessageUpdateEvent,
    serenity::all::Member => Member,
    serenity::model::guild::PartialGuild => PartialGuild,
    serenity::all::Timestamp => Timestamp,

    // Special Types
    serde_json::Value => JsonValue
}

from_field_type_tostring! {
    FixedString<u32> => Strings,
    FixedString<u16> => Strings,
    FixedString<u8> => Strings,
}

from_field_type_nonmax! {
    NonMaxU16 => Number,
    NonMaxU8 => Number,
}

from_field_type_number! {
    u64 => Number,
    u32 => Number,
    i32 => Number,
    u16 => Number,
    i16 => Number,
    u8 => Number,
    usize => Number,
}

// Special case: Channel and guild channel
impl From<serenity::model::channel::GuildChannel> for FieldType {
    fn from(s: serenity::model::channel::GuildChannel) -> Self {
        Self::Channels(vec![serenity::model::channel::Channel::Guild(s)])
    }
}

impl From<Vec<serenity::model::channel::GuildChannel>> for FieldType {
    fn from(s: Vec<serenity::model::channel::GuildChannel>) -> Self {
        Self::Channels(
            s.into_iter()
                .map(serenity::model::channel::Channel::Guild)
                .collect(),
        )
    }
}

impl From<FixedArray<serenity::model::channel::GuildChannel>> for FieldType {
    fn from(s: FixedArray<serenity::model::channel::GuildChannel>) -> Self {
        Self::Channels(
            s.into_iter()
                .map(serenity::model::channel::Channel::Guild)
                .collect(),
        )
    }
}
