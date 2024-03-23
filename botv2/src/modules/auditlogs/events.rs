use poise::serenity_prelude::FullEvent;
use crate::{silverpelt::EventHandlerContext, Data, Error};
use gwevent::field_type::FieldType;
use serenity::all::Mentionable;
use bothelpers::cache::CacheHttpImpl;

pub fn can_audit_log_event(event: &FullEvent) -> bool {
    match event {
        FullEvent::InteractionCreate { .. } => false, // Spams too much
        FullEvent::Message { .. } => false, // Spams too much
        _ => true,
    }
}

/*

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
*/

pub fn resolve_gwevent_field(field: &FieldType) -> String {
    match field {
        FieldType::Strings(s) => s.join(", "),
        FieldType::Bool(b) => if *b { "Yes" } else { "No" }.into(),
        FieldType::Number(n) => n.to_string(),
        FieldType::Permissions(p) => {
            let mut perms = Vec::new();

            for ip in p.iter() {
                perms.push(format!("{} ({})", ip, ip.bits()));
            }

            perms.join(", ")
        },
        FieldType::UserIds(u) => {
            let mut users = Vec::new();

            for iu in u.iter() {
                users.push(iu.mention().to_string());
            }

            users.join(", ")
        },
        FieldType::Channels(c) => {
            let mut channels = Vec::new();

            for ic in c.iter() {
                channels.push(ic.mention().to_string());
            }

            channels.join(", ")
        },
        FieldType::NsfwLevels(n) => {
            let mut nsfw_levels = Vec::new();

            for inl in n.iter() {
                nsfw_levels.push(format!("{:#?}", inl));
            }

            nsfw_levels.join(", ")
        },
        FieldType::Roles(r) => {
            let mut roles = Vec::new();

            for ir in r.iter() {
                roles.push(ir.mention().to_string());
            }

            roles.join(", ")
        },
        FieldType::Messages(m) => {
            let mut messages = Vec::new();

            for im in m.iter() {
                messages.push(im.to_string()); // TODO: improve this if possible
            }

            messages.join(", ")
        },
        FieldType::Guild(g) => g.to_string(),
        FieldType::Command(c) => c.to_string(),
        FieldType::Entitlement(e) => e.to_string(),
        FieldType::Application(a) => a.to_string(),
        FieldType::AuditLogId(a) => a.to_string(),
        FieldType::ScheduledEventId(s) => s.to_string(),
        FieldType::IntegrationId(i) => i.to_string(),
        FieldType::Emojis(e) => {
            let mut emojis = Vec::new();

            for ie in e.iter() {
                emojis.push(ie.to_string());
            }

            emojis.join(", ")
        },
        FieldType::GenericIds(g) => {
            let mut generic_ids = Vec::new();

            for ig in g.iter() {
                generic_ids.push(ig.to_string());
            }

            generic_ids.join(", ")
        },
        FieldType::AutomodActions(a) => {
            let mut automod_actions = Vec::new();

            for ia in a.iter() {
                automod_actions.push(format!("{:#?}", ia));
            }

            automod_actions.join(", ")
        },
        FieldType::AuditLogActions(a) => {
            let mut audit_log_actions = Vec::new();

            for ia in a.iter() {
                audit_log_actions.push(format!("{:#?}", ia));
            }

            audit_log_actions.join(", ")
        },
        FieldType::AutomodRuleIds(a) => {
            let mut automod_rule_ids = Vec::new();

            for ia in a.iter() {
                automod_rule_ids.push(ia.to_string());
            }

            automod_rule_ids.join(", ")
        },
        FieldType::AutomodTrigger(a) => format!("{:#?}", a),
        FieldType::Timestamp(t) => t.to_string(),
        FieldType::AuditLogActionsChanges(a) => {
            let mut audit_log_actions_changes = Vec::new();

            for ia in a.iter() {
                audit_log_actions_changes.push(format!("{:#?}", ia));
            }

            audit_log_actions_changes.join(", ")
        },
        FieldType::AuditLogOptions(a) => {
            let mut audit_log_options = Vec::new();

            for ia in a.iter() {
                audit_log_options.push(format!("{:#?}", ia));
            }

            audit_log_options.join(", ")
        },
        FieldType::EmojiMap(e) => {
            let mut emoji_map = Vec::new();

            for ie in e.iter() {
                emoji_map.push(format!("{:#?}", ie)); // TODO: better formatting for emojis
            }

            emoji_map.join(", ")
        },
        FieldType::StickerMap(s) => {
            let mut sticker_map = Vec::new();

            for is in s.iter() {
                sticker_map.push(format!("{:#?}", is)); // TODO: better formatting for stickers
            }

            sticker_map.join(", ")
        },
        FieldType::Users(u) => {
            let mut users = Vec::new();

            for iu in u.iter() {
                users.push(iu.mention().to_string());
            }

            users.join(", ")
        },
        FieldType::Embeds(e) => {
            let mut embeds = Vec::new();

            for ie in e.iter() {
                embeds.push(format!("<embed, title={:#?}, description={:#?}>", ie.title, ie.description.as_ref().map(|x| {
                    if x.len() > 100 {
                        format!("{}...", &x[..100])
                    } else {
                        x.to_string()
                    }
                }))); // TODO: better formatting for embeds
            }

            embeds.join(", ")
        },
        FieldType::Attachments(a) => {
            let mut attachments = Vec::new();

            for ia in a.iter() {
                attachments.push(ia.url.clone()); // TODO: better formatting for attachments
            }

            attachments.join(", ")
        },
        FieldType::Components(c) => {
            let mut components = Vec::new();

            for ic in c.iter() {
                components.push(format!("{:#?}", ic)); // TODO: better formatting for components
            }

            components.join(", ")
        },
        FieldType::ThreadMembers(t) => {
            let mut thread_members = Vec::new();

            for it in t.iter() {
                thread_members.push(it.user_id.mention().to_string()); // TODO: better formatting for thread members
            }

            thread_members.join(", ")
        },
        FieldType::None => "None".into(),
    }
}

pub async fn event_listener(
    ctx: &serenity::client::Context,
    event: &FullEvent,
    _: EventHandlerContext,
) -> Result<(), Error> {
    if !can_audit_log_event(event) {
        return Ok(());
    }

    let Some(expanded_event) = gwevent::core::expand_event(event) else {
        // Event cannot be expanded, ignore
        return Ok(());
    };

    // Convert to titlecase by capitalizing the first letter of each word
    let event_titlename = event.snake_case_name().split('_').map(|s| {
        let mut c = s.chars();
        match c.next() {
            None => String::new(),
            Some(f) => f.to_uppercase().chain(c).collect(),
        }
    }).collect::<Vec<String>>().join(" ");

    let mut event_embed_len = event_titlename.len();
    let mut event_embed = serenity::all::CreateEmbed::new()
    .title(&event_titlename);

    // Keep adding fields until length becomes > 6000
    for (k, v) in expanded_event {
        let kc = k.split('_').map(|s| {
            let mut c = s.chars();
            match c.next() {
                None => String::new(),
                Some(f) => f.to_uppercase().chain(c).collect(),
            }
        }).collect::<Vec<String>>().join(" ");

        let vc = {
            let mut vcs = Vec::new();

            for ft in v.value {
                vcs.push(resolve_gwevent_field(&ft));
            }

            vcs.join(" -> ")
        };

        let field_len = kc.len() + vc.len();
        if event_embed_len + field_len > 6000 {
            break;
        }

        event_embed_len += field_len;
        event_embed = event_embed.field(kc, vc, false);
    }

    let user_data = ctx.data::<Data>();

    let cache_http = bothelpers::cache::CacheHttpImpl {
        cache: ctx.cache.clone(),
        http: ctx.http.clone(),
    };

    Ok(())
}