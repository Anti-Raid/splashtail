use base_data::Error;
use serenity::all::Mentionable;
use serenity::model::timestamp::Timestamp;
use serenity::nonmax::{NonMaxU16, NonMaxU8};
use serenity::small_fixed_array::{FixedArray, FixedString};

/// A CategorizedField is a field that contains metadata such as category (and potentially more in the future)
#[derive(serde::Serialize, serde::Deserialize)]
pub struct CategorizedField {
    /// The category of the field
    pub category: String,
    /// The field itself
    pub field: Field,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub enum Field {
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

impl Field {
    pub fn name(&self) -> &'static str {
        match self {
            Field::Bool(_) => "Bool",
            Field::Number(_) => "Number",
            Field::Strings(_) => "Strings",
            Field::CommandPermissions(_) => "CommandPermissions",
            Field::GuildMemberFlags(_) => "GuildMemberFlags",
            Field::NsfwLevels(_) => "NsfwLevels",
            Field::Permissions(_) => "Permissions",
            Field::PermissionOverwrites(_) => "PermissionOverwrites",
            Field::ApplicationId(_) => "ApplicationId",
            Field::AuditLogId(_) => "AuditLogId",
            Field::ChannelIds(_) => "ChannelIds",
            Field::GenericIds(_) => "GenericIds",
            Field::GuildId(_) => "GuildId",
            Field::IntegrationId(_) => "IntegrationId",
            Field::MessageIds(_) => "MessageIds",
            Field::RoleIds(_) => "RoleIds",
            Field::ScheduledEventId(_) => "ScheduledEventId",
            Field::UserIds(_) => "UserIds",
            Field::ActionRows(_) => "ActionRows",
            Field::Attachment(_) => "Attachment",
            Field::AuditLogAction(_) => "AuditLogAction",
            Field::AuditLogActionsChanges(_) => "AuditLogActionsChanges",
            Field::AuditLogEntry(_) => "AuditLogEntry",
            Field::AuditLogOptions(_) => "AuditLogOptions",
            Field::AutomodActions(_) => "AutomodActions",
            Field::AutomodActionExecutions(_) => "AutomodActionExecutions",
            Field::AutomodRules(_) => "AutomodRules",
            Field::AutomodTrigger(_) => "AutomodTrigger",
            Field::Channels(_) => "Channels",
            Field::Embeds(_) => "Embeds",
            Field::Emojis(_) => "Emojis",
            Field::Entitlements(_) => "Entitlements",
            Field::Guild(_) => "Guild",
            Field::Integrations(_) => "Integrations",
            Field::Member(_) => "Member",
            Field::Messages(_) => "Messages",
            Field::MessageUpdateEvent(_) => "MessageUpdateEvent",
            Field::PartialGuildChannels(_) => "PartialGuildChannels",
            Field::PartialGuild(_) => "PartialGuild",
            Field::Roles(_) => "Roles",
            Field::ScheduledEvents(_) => "ScheduledEvents",
            Field::StageInstances(_) => "StageInstances",
            Field::Stickers(_) => "Stickers",
            Field::ThreadMembers(_) => "ThreadMembers",
            Field::Timestamp(_) => "Timestamp",
            Field::Users(_) => "Users",
            Field::JsonValue(_) => "JsonValue",
            Field::None => "None",
        }
    }

    /// Format the field into a string for use in templates
    pub fn template_format(&self) -> Result<String, Error> {
        // Given a serde_json::Value, loop over all keys and resolve them (recursively if needed)
        fn serde_resolver(v: &serde_json::Value) -> Result<String, Error> {
            match v {
                serde_json::Value::Null => Ok("None".to_string()),
                serde_json::Value::Bool(b) => Ok(if *b { "Yes" } else { "No" }.to_string()),
                serde_json::Value::Number(n) => Ok(n.to_string()),
                serde_json::Value::String(s) => Ok(s.to_string()),
                serde_json::Value::Object(o) => {
                    let mut resolved = Vec::new();

                    for (k, v) in o.iter() {
                        resolved.push(format!(
                            "{} => {}",
                            k.split('_')
                                .map(|s| {
                                    let mut c = s.chars();
                                    match c.next() {
                                        None => String::new(),
                                        Some(f) => f.to_uppercase().chain(c).collect(),
                                    }
                                })
                                .collect::<Vec<String>>()
                                .join(" "),
                            serde_resolver(v)?
                        ));
                    }

                    Ok(resolved.join("\n"))
                }
                serde_json::Value::Array(v) => {
                    let mut resolved = Vec::new();

                    for i in v.iter() {
                        resolved.push(serde_resolver(i)?);
                    }

                    Ok(resolved.join("\n\n"))
                }
            }
        }

        match self {
            Field::Strings(s) => {
                let joined = s.join(", ");
                Ok(joined)
            }
            Field::Bool(b) => Ok(if *b { "Yes" } else { "No" }.to_string()),
            Field::Number(n) => Ok(n.to_string()),
            Field::Permissions(p) => {
                let mut perms = Vec::new();

                for ip in p.iter() {
                    perms.push(format!("{} ({})", ip, ip.bits()));
                }

                Ok(perms.join(", "))
            }
            Field::PermissionOverwrites(p) => {
                let mut perms = Vec::new();

                for ip in p.iter() {
                    perms.push(format!("Allow={}, Deny={}", ip.allow, ip.deny));
                }

                Ok(perms.join(", "))
            }
            Field::GuildMemberFlags(p) => {
                let p_vec = p
                    .iter()
                    .map(|x| format!("{:#?}", x))
                    .collect::<Vec<String>>();

                if p_vec.is_empty() {
                    return Ok("None".to_string());
                }

                Ok(p_vec.join(", "))
            }
            Field::UserIds(u) => {
                let mut users = Vec::new();

                for iu in u.iter() {
                    users.push(iu.mention().to_string());
                }

                Ok(users.join(", "))
            }
            Field::Channels(c) => {
                let mut channels = Vec::new();

                for ic in c.iter() {
                    channels.push(ic.mention().to_string());
                }

                Ok(channels.join(", "))
            }
            Field::NsfwLevels(n) => {
                let mut nsfw_levels = Vec::new();

                for inl in n.iter() {
                    nsfw_levels.push(format!("{:#?}", inl));
                }

                Ok(nsfw_levels.join(", "))
            }
            Field::Roles(r) => {
                let mut roles = Vec::new();

                for ir in r.iter() {
                    roles.push(ir.mention().to_string());
                }

                Ok(roles.join(", "))
            }
            Field::GenericIds(g) => {
                let mut generic_ids = Vec::new();

                for ig in g.iter() {
                    generic_ids.push(ig.to_string());
                }

                Ok(generic_ids.join(", "))
            }
            Field::Timestamp(t) => Ok(t.to_string()),
            Field::Attachment(a) => Ok(a.url.to_string()),
            Field::JsonValue(v) => match serde_json::to_string(v) {
                Ok(s) => Ok(format!("``{}``", s)),
                Err(e) => Err(e.into()),
            },
            Field::None => Ok("None".to_string()),
            _ => {
                let s = serde_resolver(&serde_json::to_value(self)?)?;
                Ok(s)
            }
        }
    }
}

macro_rules! from_field {
    ($($t:ty => $variant:ident),* $(,)?) => {
        $(
            impl From<$t> for Field {
                fn from(s: $t) -> Self {
                    Self::$variant(s)
                }
            }
        )*
    };
}

macro_rules! from_field_multiple {
    ($($t:ty => $variant:ident),* $(,)?) => {
        $(
            impl From<$t> for Field {
                fn from(s: $t) -> Self {
                    Self::$variant(vec![s])
                }
            }
            impl From<Vec<$t>> for Field {
                fn from(s: Vec<$t>) -> Self {
                    Self::$variant(s)
                }
            }
            impl From<FixedArray<$t>> for Field {
                fn from(s: FixedArray<$t>) -> Self {
                    Self::$variant(s.into_iter().collect())
                }
            }
        )*
    };
}

macro_rules! from_field_tostring {
    ($($t:ty => $variant:ident),* $(,)?) => {
        $(
            impl From<$t> for Field {
                fn from(s: $t) -> Self {
                    Self::$variant(vec![s.to_string()])
                }
            }

            impl From<Vec<$t>> for Field {
                fn from(s: Vec<$t>) -> Self {
                    Self::$variant(s.into_iter().map(|s| s.to_string()).collect())
                }
            }

            impl From<FixedArray<$t>> for Field {
                fn from(s: FixedArray<$t>) -> Self {
                    Self::$variant(s.into_iter().map(|s| s.to_string()).collect())
                }
            }
        )*
    };
}

macro_rules! from_field_nonmax {
    ($($t:ty => $variant:ident),* $(,)?) => {
        $(
            impl From<$t> for Field {
                fn from(s: $t) -> Self {
                    Self::$variant(s.get().into())
                }
            }
        )*
    };
}

macro_rules! from_field_number {
    ($($t:ty => $variant:ident),* $(,)?) => {
        $(
            impl From<$t> for Field {
                fn from(s: $t) -> Self {
                    Self::$variant(s as u64)
                }
            }
        )*
    };
}

from_field_multiple! {
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

from_field! {
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

from_field_tostring! {
    FixedString<u32> => Strings,
    FixedString<u16> => Strings,
    FixedString<u8> => Strings,
}

from_field_nonmax! {
    NonMaxU16 => Number,
    NonMaxU8 => Number,
}

from_field_number! {
    u64 => Number,
    u32 => Number,
    i32 => Number,
    u16 => Number,
    i16 => Number,
    u8 => Number,
    usize => Number,
}

// Special case: Channel and guild channel
impl From<serenity::model::channel::GuildChannel> for Field {
    fn from(s: serenity::model::channel::GuildChannel) -> Self {
        Self::Channels(vec![serenity::model::channel::Channel::Guild(s)])
    }
}

impl From<Vec<serenity::model::channel::GuildChannel>> for Field {
    fn from(s: Vec<serenity::model::channel::GuildChannel>) -> Self {
        Self::Channels(
            s.into_iter()
                .map(serenity::model::channel::Channel::Guild)
                .collect(),
        )
    }
}

impl From<FixedArray<serenity::model::channel::GuildChannel>> for Field {
    fn from(s: FixedArray<serenity::model::channel::GuildChannel>) -> Self {
        Self::Channels(
            s.into_iter()
                .map(serenity::model::channel::Channel::Guild)
                .collect(),
        )
    }
}
