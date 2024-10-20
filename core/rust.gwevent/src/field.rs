use crate::Error;
use serenity::all::Mentionable;
use serenity::model::timestamp::Timestamp;
use serenity::nonmax::{NonMaxU16, NonMaxU8};
use serenity::small_fixed_array::{FixedArray, FixedString};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", content = "field")]
pub enum Field {
    // Primitive Types
    Bool(bool),
    Number(u64),
    Strings(Vec<String>),

    // Discord Primitives
    CommandPermissions(serenity::all::CommandPermissions),
    GuildMemberFlags(serenity::all::GuildMemberFlags),
    NsfwLevels(Vec<serenity::all::NsfwLevel>),
    Permissions(serenity::all::Permissions),
    PermissionOverwrites(Vec<serenity::all::PermissionOverwrite>),

    // Discord ID Types
    AnswerId(serenity::all::AnswerId), // Used in polls
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
    WebhookIds(Vec<serenity::all::WebhookId>),

    // Discord Structures
    ActionRows(Vec<serenity::model::application::ActionRow>),
    Attachments(Vec<serenity::all::Attachment>),
    AuditLogAction(serenity::all::audit_log::Action),
    AuditLogActionsChanges(Vec<serenity::all::audit_log::Change>),
    AuditLogEntry(serenity::all::audit_log::AuditLogEntry),
    AuditLogOptions(Vec<serenity::all::audit_log::Options>),
    AutomodActions(Vec<serenity::all::automod::Action>),
    AutomodActionExecutions(Vec<serenity::all::automod::ActionExecution>),
    AutomodRules(Vec<serenity::all::automod::Rule>),
    AutomodTrigger(serenity::all::automod::Trigger),
    Channels(Vec<serenity::all::Channel>),
    Embeds(Vec<serenity::all::Embed>),
    Emojis(Vec<serenity::all::Emoji>),
    Entitlements(Vec<serenity::all::Entitlement>),
    Guild(serenity::all::Guild),
    Integrations(Vec<serenity::all::Integration>),
    Member(serenity::all::Member),
    PartialMember(serenity::all::PartialMember),
    Messages(Vec<serenity::all::Message>),
    PartialGuildChannels(Vec<serenity::all::PartialGuildChannel>),
    PartialGuild(serenity::all::PartialGuild),
    Roles(Vec<serenity::all::Role>),
    RoleSubscriptionData(serenity::all::RoleSubscriptionData),
    ScheduledEvents(Vec<serenity::all::ScheduledEvent>),
    StageInstances(Vec<serenity::all::StageInstance>),
    Stickers(Vec<serenity::model::sticker::Sticker>),
    StickerItems(Vec<serenity::model::sticker::StickerItem>),
    ThreadMembers(Vec<serenity::all::ThreadMember>),
    Timestamp(Timestamp),
    Users(Vec<serenity::model::user::User>),

    // Discord Message Structures
    ChannelMentions(Vec<serenity::all::ChannelMention>),
    MessageReactions(Vec<serenity::all::MessageReaction>),
    MessageType(serenity::all::MessageType),
    MessageActivity(serenity::all::MessageActivity),
    MessageApplication(serenity::all::MessageApplication),
    MessageReference(serenity::all::MessageReference),
    MessageFlags(serenity::all::MessageFlags),
    MessageInteractionMetadata(serenity::all::MessageInteractionMetadata),

    // Special Types
    JsonValue(serde_json::Value),
    None,
}

impl Field {
    /// Format the field into a string for use in templates
    pub fn template_format(&self) -> Result<String, Error> {
        match self {
            Field::Strings(s) => {
                let mut md = String::new();

                for (i, str) in s.iter().enumerate() {
                    md.push_str(&format!("``{}``", str));

                    if i != s.len() - 1 {
                        md.push_str(", ");
                    }
                }

                Ok(md)
            }
            Field::Bool(b) => Ok(if *b { "Yes" } else { "No" }.to_string()),
            Field::Number(n) => Ok(n.to_string()),
            Field::Permissions(p) => {
                let mut perms = Vec::new();

                for ip in p.iter() {
                    perms.push(format!("``{} ({})``", ip, ip.bits()));
                }

                Ok(perms.join(", "))
            }
            Field::PermissionOverwrites(p) => {
                let mut perms = Vec::new();

                for ip in p.iter() {
                    perms.push(format!("``Allow={}, Deny={}``", ip.allow, ip.deny));
                }

                Ok(perms.join(", "))
            }
            Field::GuildMemberFlags(p) => {
                let p_vec = p
                    .iter()
                    .map(|x| format!("``{:?}``", x))
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
            Field::Users(u) => {
                let mut users = Vec::new();

                for iu in u.iter() {
                    users.push(format!(
                        "{} [``{}``], avatar={}",
                        iu.mention(),
                        iu.name,
                        iu.face()
                    ));
                }

                Ok(users.join(", "))
            }
            Field::Member(m) => {
                let roles = m
                    .roles
                    .iter()
                    .map(|r| r.mention().to_string())
                    .collect::<Vec<String>>();
                Ok(format!(
                    "{} [``{}``], roles={} pending={}, timeout={}, nick=``{}``, avatar={}",
                    m.user.mention(),
                    m.user.name,
                    roles.join(", "),
                    m.pending(),
                    m.communication_disabled_until
                        .map(|t| t.to_string())
                        .unwrap_or_else(|| "None".to_string()),
                    m.nick.as_deref().unwrap_or("None"),
                    m.face(),
                ))
            }
            Field::ChannelIds(c) => {
                let mut channels = Vec::new();

                for ic in c.iter() {
                    channels.push(ic.mention().to_string());
                }

                Ok(channels.join(", "))
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
                    nsfw_levels.push(format!("{:?}", inl));
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
            Field::RoleIds(r) => {
                let mut roles = Vec::new();

                for ir in r.iter() {
                    roles.push(format!("{} [``{}``]", ir.mention(), ir));
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
            Field::Attachments(a) => {
                let mut attachments = Vec::new();

                for ia in a.iter() {
                    attachments.push(ia.url.clone());
                }

                Ok(attachments.join(", "))
            }
            Field::MessageFlags(f) => {
                let mut flags = Vec::new();

                for ip in f.iter() {
                    flags.push(format!("{:?} ({})", ip, ip.bits()));
                }

                Ok(flags.join(", "))
            }
            Field::JsonValue(v) => match serde_json::to_string(v) {
                Ok(s) => Ok(format!("``{}``", s)),
                Err(e) => Err(e.into()),
            },
            Field::StickerItems(s) => {
                let mut sticker_items = Vec::new();

                for isi in s.iter() {
                    sticker_items.push(
                        isi.image_url()
                            .unwrap_or(format!("{} {} (unknown image)", isi.id, isi.name)),
                    );
                }

                Ok(sticker_items.join(", "))
            }
            Field::Stickers(s) => {
                let mut stickers = Vec::new();

                for isi in s.iter() {
                    stickers.push(
                        isi.image_url()
                            .unwrap_or(format!("{} {} (unknown image)", isi.id, isi.name)),
                    );
                }

                Ok(stickers.join(", "))
            }
            Field::None => Ok("None".to_string()),
            Field::ChannelMentions(c) => {
                let mut channels = Vec::new();

                for ic in c.iter() {
                    channels.push(format!("{} [{}, {:?}]", ic.id.mention(), ic.name, ic.kind));
                }

                Ok(channels.join(", "))
            }
            Field::StageInstances(s) => {
                let mut stage_instances = Vec::new();

                for isi in s.iter() {
                    stage_instances.push(format!(
                        "Stage instance {}, privacy={:?}, topic={}",
                        isi.channel_id.mention(),
                        isi.privacy_level,
                        isi.topic
                    ));
                }

                Ok(stage_instances.join(", "))
            }
            Field::Messages(m) => {
                let mut messages = Vec::new();

                for im in m.iter() {
                    messages.push(format!(
                        "{}, channel={}, author={} ({}), content={}",
                        im.link(),
                        im.channel_id.mention(),
                        im.author.mention(),
                        im.author.name,
                        {
                            if im.content.len() > 50 {
                                format!("{}...", &im.content[..50])
                            } else {
                                im.content.to_string()
                            }
                        }
                    ));
                }

                Ok(messages.join(", "))
            }
            Field::Emojis(e) => {
                let mut emojis = Vec::new();

                for ie in e.iter() {
                    emojis.push(ie.to_string());
                }

                Ok(emojis.join(", "))
            }
            Field::AutomodActions(a) => {
                let mut actions = Vec::new();

                for ia in a.iter() {
                    actions.push(format!("{:?}", ia));
                }

                Ok(actions.join(", "))
            }
            Field::ActionRows(a) => {
                let mut action_rows = Vec::new();

                for ia in a.iter() {
                    action_rows.push(format!("{:?}", ia));
                }

                Ok(action_rows.join(", "))
            }
            _ => serde_json::to_string(self).map_err(Into::into),
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

            impl From<Box<$t>> for Field {
                fn from(s: Box<$t>) -> Self {
                    Self::$variant(*s)
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
            impl From<Box<$t>> for Field {
                fn from(s: Box<$t>) -> Self {
                    Self::$variant(vec![*s])
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
    serenity::all::NsfwLevel => NsfwLevels,
    serenity::all::PermissionOverwrite => PermissionOverwrites,

    // Discord ID Types
    serenity::all::ChannelId => ChannelIds,
    serenity::all::GenericId => GenericIds,
    serenity::all::MessageId => MessageIds,
    serenity::all::RoleId => RoleIds,
    serenity::all::UserId => UserIds,
    serenity::all::WebhookId => WebhookIds,

    // Discord Structures
    serenity::all::Attachment => Attachments,
    serenity::model::application::ActionRow => ActionRows,
    serenity::all::audit_log::Change => AuditLogActionsChanges,
    serenity::all::audit_log::Options => AuditLogOptions,
    serenity::all::automod::Action => AutomodActions,
    serenity::all::automod::ActionExecution => AutomodActionExecutions,
    serenity::all::automod::Rule => AutomodRules,
    serenity::all::Channel => Channels,
    serenity::all::Embed => Embeds,
    serenity::all::Emoji => Emojis,
    serenity::all::Entitlement => Entitlements,
    serenity::all::Integration => Integrations,
    serenity::all::Message => Messages,
    serenity::all::PartialGuildChannel => PartialGuildChannels,
    serenity::all::Role => Roles,
    serenity::all::ScheduledEvent => ScheduledEvents,
    serenity::all::StageInstance => StageInstances,
    serenity::model::sticker::Sticker => Stickers,
    serenity::model::sticker::StickerItem => StickerItems,
    serenity::all::ThreadMember => ThreadMembers,
    serenity::model::user::User => Users,

    // Discord Message Structures
    serenity::all::ChannelMention => ChannelMentions,
    serenity::all::MessageReaction => MessageReactions,
}

from_field! {
    // Primitive Types
    bool => Bool,

    // Discord Primitives
    serenity::all::CommandPermissions => CommandPermissions,
    serenity::all::GuildMemberFlags => GuildMemberFlags,
    serenity::all::Permissions => Permissions,

    // Discord ID Types
    serenity::all::AnswerId => AnswerId,
    serenity::all::ApplicationId => ApplicationId,
    serenity::all::AuditLogEntryId => AuditLogId,
    serenity::all::GuildId => GuildId,
    serenity::all::IntegrationId => IntegrationId,
    serenity::all::ScheduledEventId => ScheduledEventId,

    // Discord Structures
    serenity::all::audit_log::Action => AuditLogAction,
    serenity::all::audit_log::AuditLogEntry => AuditLogEntry,
    serenity::all::automod::Trigger => AutomodTrigger,
    serenity::all::Guild => Guild,
    serenity::all::Member => Member,
    serenity::all::PartialMember => PartialMember,
    serenity::all::PartialGuild => PartialGuild,
    serenity::all::RoleSubscriptionData => RoleSubscriptionData,
    serenity::all::Timestamp => Timestamp,

    // Discord Message Structures
    serenity::all::MessageType => MessageType,
    serenity::all::MessageActivity => MessageActivity,
    serenity::all::MessageApplication => MessageApplication,
    serenity::all::MessageReference => MessageReference,
    serenity::all::MessageFlags => MessageFlags,
    serenity::all::MessageInteractionMetadata => MessageInteractionMetadata,

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
impl From<serenity::all::GuildChannel> for Field {
    fn from(s: serenity::all::GuildChannel) -> Self {
        Self::Channels(vec![serenity::all::Channel::Guild(s)])
    }
}

impl From<Vec<serenity::all::GuildChannel>> for Field {
    fn from(s: Vec<serenity::all::GuildChannel>) -> Self {
        Self::Channels(s.into_iter().map(serenity::all::Channel::Guild).collect())
    }
}

impl From<FixedArray<serenity::all::GuildChannel>> for Field {
    fn from(s: FixedArray<serenity::all::GuildChannel>) -> Self {
        Self::Channels(s.into_iter().map(serenity::all::Channel::Guild).collect())
    }
}
