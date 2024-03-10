use serenity::all::{
    ChannelId, EmojiId, FullEvent, GenericId, GuildId, MessageId, RoleId, RuleId as AutomodRuleId, UserId, ActionExecution
};
use small_fixed_array::FixedString;
use indexmap::IndexMap;
use crate::Error;
use log::warn;

/// Given an event and a module, return whether or not to filter said event
pub fn get_event_guild_id(
    event: &FullEvent,
) -> Result<GuildId, Option<Error>> {
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
        },
        FullEvent::ChannelUpdate { new, .. } => new.guild_id,
        FullEvent::CommandPermissionsUpdate { permission, .. } => permission.guild_id,
        FullEvent::EntitlementCreate { entitlement, .. } => {
            if let Some(guild_id) = entitlement.guild_id {
                guild_id.to_owned()
            } else {
                return Err(None);
            }
        },
        FullEvent::EntitlementDelete { entitlement, .. } => {
            if let Some(guild_id) = entitlement.guild_id {
                guild_id.to_owned()
            } else {
                return Err(None);
            }
        },
        FullEvent::EntitlementUpdate { entitlement, .. } => {
            if let Some(guild_id) = entitlement.guild_id {
                guild_id.to_owned()
            } else {
                return Err(None);
            }
        },
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
        },
        FullEvent::IntegrationDelete { guild_id, .. } => *guild_id,
        FullEvent::IntegrationUpdate { integration, .. } => {
            if let Some(guild_id) = integration.guild_id {
                guild_id.to_owned()
            } else {
                return Err(None);
            }
        },
        FullEvent::InteractionCreate { .. } => return Err(None), // We dont handle interactions create events in event handlers
        FullEvent::InviteCreate { data, .. } => {
            if let Some(guild_id) = data.guild_id {
                guild_id.to_owned()
            } else {
                return Err(None);
            }
        },
        FullEvent::InviteDelete { data, .. } => {
            if let Some(guild_id) = data.guild_id {
                guild_id.to_owned()
            } else {
                return Err(None);
            }
        },
        FullEvent::Message { new_message, .. } => {
            if let Some(guild_id) = &new_message.guild_id {
                guild_id.to_owned()
            } else {
                return Err(None);
            }
        },
        FullEvent::MessageDelete { guild_id, .. } => {
            if let Some(guild_id) = guild_id {
                guild_id.to_owned()
            } else {
                return Err(None);
            }
        },
        FullEvent::MessageDeleteBulk { guild_id, .. } => {
            if let Some(guild_id) = guild_id {
                guild_id.to_owned()
            } else {
                return Err(None);
            }
        },
        FullEvent::MessageUpdate { event, .. } => {
            if let Some(guild_id) = &event.guild_id {
                guild_id.to_owned()
            } else {
                return Err(None);
            }
        },
        FullEvent::PresenceReplace { .. } => return Err(None), // We dont handle precenses
        FullEvent::PresenceUpdate { .. } => return Err(None), // We dont handle precenses
        FullEvent::Ratelimit { data, .. } => {
            // Warn i guess
            warn!("Ratelimit event recieved: {:?}", data);
            return Err(None);
        },
        FullEvent::ReactionAdd { .. } => return Err(None), // We dont handle reactions right now
        FullEvent::ReactionRemove { .. } => return Err(None), // We dont handle reactions right now
        FullEvent::ReactionRemoveAll { .. } => return Err(None), // We dont handle reactions right now
        FullEvent::ReactionRemoveEmoji { .. } => return Err(None), // We dont handle reactions right now
        FullEvent::Ready { .. } => return Err(None), // We dont handle ready events
        FullEvent::Resume { .. } => return Err(None), // We dont handle resume events
        FullEvent::ShardStageUpdate { .. } => return Err(None), // We dont handle shard stage updates
        FullEvent::ShardsReady { .. } => return Err(None), // We dont handle shards ready
        FullEvent::StageInstanceCreate { .. } => return Err(None), // We dont handle stage instances right now
        FullEvent::StageInstanceDelete { .. } => return Err(None), // We dont handle stage instances right now
        FullEvent::StageInstanceUpdate { .. } => return Err(None), // We dont handle stage instances right now
        FullEvent::ThreadCreate { thread, .. } => thread.guild_id, 
        FullEvent::ThreadDelete { thread, .. } => thread.guild_id, 
        FullEvent::ThreadListSync { thread_list_sync, .. } => thread_list_sync.guild_id,
        FullEvent::ThreadMemberUpdate { thread_member, .. } => {
            if let Some(guild_id) = thread_member.guild_id {
                guild_id.to_owned()
            } else {
                return Err(None);
            }
        },
        FullEvent::ThreadMembersUpdate { thread_members_update, .. } => thread_members_update.guild_id,
        FullEvent::ThreadUpdate { new, .. } => new.guild_id,
        FullEvent::TypingStart { .. } => return Err(None), // We dont handle typing start
        FullEvent::UserUpdate { .. } => return Err(None), // We dont handle user updates
        FullEvent::VoiceChannelStatusUpdate { guild_id, .. } => *guild_id,
        FullEvent::VoiceServerUpdate { .. } => return Err(None), // We dont handle voice right now
        FullEvent::VoiceStateUpdate { .. } => return Err(None), // We dont handle voice right now
        FullEvent::WebhookUpdate { guild_id, .. } => *guild_id,
        _ => {
            return Err(
                Some(format!("Unhandled event: {:?}", event).into()),
            );
        }
    };

    Ok(guild_id)
}

pub enum FieldType {
    /// A string
    String(String),

    /// A user id
    User(UserId),

    /// A channel id
    Channel(ChannelId),

    /// A role id
    Role(RoleId),

    /// A message id
    Message(MessageId),

    /// A guild id
    Guild(GuildId),

    /// An emoji id
    Emoji(EmojiId),

    /// A generic id
    GenericId(GenericId),

    /// An automod action
    AutomodAction(serenity::model::guild::automod::Action),

    /// An automod rule id
    AutomodRuleId(AutomodRuleId),
}

impl From<String> for FieldType {
    fn from(s: String) -> Self {
        Self::String(s)
    }
}

impl From<FixedString<u32>> for FieldType {
    fn from(s: FixedString<u32>) -> Self {
        Self::String(s.to_string())
    }
}

impl From<UserId> for FieldType {
    fn from(s: UserId) -> Self {
        Self::User(s)
    }
}

impl From<ChannelId> for FieldType {
    fn from(s: ChannelId) -> Self {
        Self::Channel(s)
    }
}

impl From<RoleId> for FieldType {
    fn from(s: RoleId) -> Self {
        Self::Role(s)
    }
}

impl From<MessageId> for FieldType {
    fn from(s: MessageId) -> Self {
        Self::Message(s)
    }
}

impl From<GuildId> for FieldType {
    fn from(s: GuildId) -> Self {
        Self::Guild(s)
    }
}

impl From<EmojiId> for FieldType {
    fn from(s: EmojiId) -> Self {
        Self::Emoji(s)
    }
}

impl From<GenericId> for FieldType {
    fn from(s: GenericId) -> Self {
        Self::GenericId(s)
    }
}

impl From<serenity::model::guild::automod::Action> for FieldType {
    fn from(s: serenity::model::guild::automod::Action) -> Self {
        Self::AutomodAction(s)
    }
} 

impl From<AutomodRuleId> for FieldType {
    fn from(s: AutomodRuleId) -> Self {
        Self::AutomodRuleId(s)
    }
} 

pub struct Field {
    /// The value of the field
    value: Option<FieldType>,
}

impl Field {
    /// Create a new field
    pub fn new(value: Option<FieldType>) -> Self {
        Self { value }
    }
}

/// Given an event, expand it to a hashmap of fields
pub fn expand_event(
    event: &FullEvent,
) -> Option<IndexMap<String, Field>> {
    let mut fields = IndexMap::new();

            /*
pub struct ActionExecution {
    /// ID of the guild in which the action was executed.
    pub guild_id: GuildId,
    /// Action which was executed.
    pub action: Action,
    /// ID of the rule which action belongs to.
    pub rule_id: RuleId,
    /// Trigger type of rule which was triggered.
    #[serde(rename = "rule_trigger_type")]
    pub trigger_type: TriggerType,
    /// ID of the user which generated the content which triggered the rule.
    pub user_id: UserId,
    /// ID of the channel in which user content was posted.
    pub channel_id: Option<ChannelId>,
    /// ID of any user message which content belongs to.
    ///
    /// Will be `None` if message was blocked by automod or content was not part of any message.
    pub message_id: Option<MessageId>,
    /// ID of any system auto moderation messages posted as a result of this action.
    ///
    /// Will be `None` if this event does not correspond to an action with type [`Action::Alert`].
    pub alert_system_message_id: Option<MessageId>,
    /// User generated text content.
    ///
    /// Requires [`GatewayIntents::MESSAGE_CONTENT`] to receive non-empty values.
    ///
    /// [`GatewayIntents::MESSAGE_CONTENT`]: crate::model::gateway::GatewayIntents::MESSAGE_CONTENT
    pub content: FixedString,
    /// Word or phrase configured in the rule that triggered the rule.
    pub matched_keyword: Option<FixedString>,
    /// Substring in content that triggered the rule.
    ///
    /// Requires [`GatewayIntents::MESSAGE_CONTENT`] to receive non-empty values.
    ///
    /// [`GatewayIntents::MESSAGE_CONTENT`]: crate::model::gateway::GatewayIntents::MESSAGE_CONTENT
    pub matched_content: Option<FixedString>,
}
         */
    fn expand_action_execution(fields: &mut IndexMap<String, Field>, execution: &ActionExecution) {
        fields.insert("guild_id".to_string(), Field::new(Some(execution.guild_id.into())));
        fields.insert("action".to_string(), Field::new(Some(execution.action.clone().into())));
        fields.insert("rule_id".to_string(), Field::new(Some(execution.rule_id.into())));
        fields.insert("trigger_type".to_string(), Field::new(Some(
            match execution.trigger_type {
                serenity::model::guild::automod::TriggerType::Keyword => "Keyword".to_string(),
                serenity::model::guild::automod::TriggerType::Spam => "Spam".to_string(),
                serenity::model::guild::automod::TriggerType::KeywordPreset => "KeywordPreset".to_string(),
                serenity::model::guild::automod::TriggerType::MentionSpam => "MentionSpam".to_string(),
                serenity::model::guild::automod::TriggerType::Unknown(b) => format!("Unknown({})", b),
                _ => "Unknown".to_string(),
            }.into()
        )));
        fields.insert("user_id".to_string(), Field::new(Some(execution.user_id.into())));
        fields.insert("channel_id".to_string(), Field::new(execution.channel_id.map(|x| x.into())));
        fields.insert("message_id".to_string(), Field::new(execution.message_id.map(|x| x.into())));
        fields.insert("alert_system_message_id".to_string(), Field::new(execution.alert_system_message_id.map(|x| x.into())));
        fields.insert("content".to_string(), Field::new(Some(execution.content.clone().into())));
        fields.insert("matched_keyword".to_string(), Field::new(execution.matched_keyword.clone().map(|x| x.into())));
        fields.insert("matched_content".to_string(), Field::new(execution.matched_content.clone().map(|x| x.into())));
    }

    match event {
        FullEvent::AutoModActionExecution { execution } => {
            expand_action_execution(&mut fields, execution);
        },
        _ => {
            return None;
        }
    }

    Some(fields)
}