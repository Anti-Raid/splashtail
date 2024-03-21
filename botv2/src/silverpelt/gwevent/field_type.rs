use serenity::all::{
    ApplicationId, AuditLogEntryId, ChannelId, CommandId, EmojiId, EntitlementId,
    GenericId, GuildId, IntegrationId, MessageId, RoleId,
    RuleId as AutomodRuleId, ScheduledEventId, UserId,
};
use serenity::model::guild::automod::Action;
use serenity::model::timestamp::Timestamp;
use serenity::nonmax::{NonMaxU16, NonMaxU8};
use serenity::small_fixed_array::FixedString;

pub enum FieldType {
    /// A string
    Strings(Vec<String>),

    /// A boolean
    Bool(bool),

    /// A number
    Number(u64),

    /// Permission
    Permissions(serenity::all::Permissions),

    /// A user id
    UserIds(Vec<UserId>),

    /// A channel id
    Channels(Vec<ChannelId>),

    /// NSFW level
    NsfwLevels(Vec<serenity::model::guild::NsfwLevel>),

    /// A role id
    Roles(Vec<RoleId>),

    /// A message id
    Messages(Vec<MessageId>),

    /// A guild id
    Guild(GuildId),

    /// Command Id
    Command(CommandId),

    /// Entitlement ID
    Entitlement(EntitlementId),

    /// Application Id
    Application(ApplicationId),

    /// Audit Log Id
    AuditLogId(AuditLogEntryId),

    /// Scheduled Event Id
    ScheduledEventId(ScheduledEventId),

    /// Integration Id
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

    /// Trigger
    AutomodTrigger(serenity::model::guild::automod::Trigger),

    /// Timestamp
    Timestamp(serenity::model::timestamp::Timestamp),

    /// Changes
    AuditLogActionsChanges(Vec<serenity::model::guild::audit_log::Change>),

    /// Options
    AuditLogOptions(Vec<serenity::model::guild::audit_log::Options>),

    /// Emoji Map
    EmojiMap(Vec<serenity::model::guild::Emoji>),

    /// Sticker Map
    StickerMap(Vec<serenity::model::sticker::Sticker>),

    /// Users
    Users(Vec<serenity::model::user::User>),

    /// Embeds
    Embeds(Vec<serenity::model::channel::Embed>),

    /// Attachments
    Attachments(Vec<serenity::model::channel::Attachment>),

    /// Components
    Components(Vec<serenity::model::application::ActionRow>),

    /// ThreadMembers
    ThreadMembers(Vec<serenity::model::guild::ThreadMember>),

    /// None
    None,
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
    String => Strings,
    UserId => UserIds,
    ChannelId => Channels,
    serenity::model::guild::NsfwLevel => NsfwLevels,
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

from_field_type! {
    GuildId => Guild,
    serenity::all::Permissions => Permissions,
    IntegrationId => IntegrationId,
    AuditLogEntryId => AuditLogId,
    CommandId => Command,
    ScheduledEventId => ScheduledEventId,
    ApplicationId => Application,
    EntitlementId => Entitlement,
    Timestamp => Timestamp,
    bool => Bool,
    serenity::model::guild::automod::Trigger => AutomodTrigger,
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
