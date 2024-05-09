use crate::{
    silverpelt::{module_config::is_module_enabled, EventHandlerContext},
    Error,
};
use serenity::all::FullEvent;

/// The maximum number of mentions before the anti-everyone trigger is activated
const MAX_MENTIONS: u32 = 10;

bitflags::bitflags! {
    #[derive(PartialEq)]
    pub struct TriggeredFlags: u32 {
        const NONE = 0;
        const ANTI_INVITE = 1 << 0;
        const ANTI_EVERYONE = 1 << 1;
        const MINIMUM_ACCOUNT_AGE = 1 << 2;
        const MAXIMUM_ACCOUNT_AGE = 1 << 3;
        const FAKE_BOT_DETECTION = 1 << 4;
    }
}

pub async fn event_listener(ectx: &EventHandlerContext) -> Result<(), Error> {
    let ctx = &ectx.serenity_context;
    let event = &ectx.full_event;

    match event {
        FullEvent::Message { new_message } => {
            let data = &ectx.data;
            let config = super::cache::get_config(&data.pool, ectx.guild_id).await?;

            let mut triggered_flags = TriggeredFlags::NONE;
            let mut triggered_stings = 0;

            if let Some(ai_stings) = config.anti_invite {
                let trimmed_msg = new_message
                    .content
                    .trim()
                    .replace("dot", ".")
                    .replace("slash", "/")
                    .replace(['(', ')'], "");

                if trimmed_msg.contains("discord.gg")
                    || trimmed_msg.contains("discordapp.com/invite")
                    || trimmed_msg.contains("discord.com/invite")
                {
                    triggered_flags |= TriggeredFlags::ANTI_INVITE;
                    triggered_stings += ai_stings;
                }
            }

            if let Some(ae_stings) = config.anti_everyone {
                if new_message.content.contains("@everyone")
                    || new_message.mention_everyone()
                    || new_message.mentions.len() > MAX_MENTIONS
                {
                    triggered_flags |= TriggeredFlags::ANTI_EVERYONE;
                    triggered_stings += ae_stings;
                }
            }

            if triggered_flags != TriggeredFlags::NONE {
                // For now, don't do anything, punishment support is coming soon
                new_message
                    .delete(
                        &ctx.http,
                        Some(&format!("Message triggered flags: {:?}", {
                            let mut tf = vec![];

                            for (name, _) in triggered_flags.iter_names() {
                                tf.push(name);
                            }

                            tf.join(", ")
                        })),
                    )
                    .await?;

                // Apply stings
                if triggered_stings > 0
                    && is_module_enabled(&data.pool, ectx.guild_id, "punishments").await?
                {
                    sqlx::query!(
                        "INSERT INTO basic_antispam__punishments (user_id, guild_id, stings, triggered_flags) VALUES ($1, $2, $3, $4)",
                        new_message.author.id.to_string(),
                        ectx.guild_id.to_string(),
                        triggered_stings as i32,
                        i64::from(triggered_flags.bits())
                    )
                    .execute(&data.pool)
                    .await?;
                }
            }

            Ok(())
        }
        FullEvent::GuildMemberAddition { new_member } => {
            // Get account creation
            let data = &ectx.data;
            let config = super::cache::get_config(&data.pool, ectx.guild_id).await?;

            let mut triggered_flags = TriggeredFlags::NONE;

            if let Some(minimum_account_age) = config.minimum_account_age {
                let account_age = new_member.user.created_at();
                let now = chrono::Utc::now();
                if let Some(duration) = chrono::Duration::try_seconds(minimum_account_age) {
                    let diff = now - *account_age;

                    if diff < duration {
                        triggered_flags |= TriggeredFlags::MINIMUM_ACCOUNT_AGE;
                    }
                }
            }

            if let Some(maximum_account_age) = config.maximum_account_age {
                let account_age = new_member.user.created_at();
                let now = chrono::Utc::now();

                if let Some(duration) = chrono::Duration::try_seconds(maximum_account_age) {
                    let diff = now - *account_age;

                    if diff > duration {
                        triggered_flags |= TriggeredFlags::MAXIMUM_ACCOUNT_AGE;
                    }
                }
            }

            if config.fake_bot_detection && new_member.user.bot() {
                // Normalize the bots name
                let normalized_name = plsfix::fix_text(&new_member.user.name.to_lowercase(), None);

                let mut found = false;
                for fb in super::cache::FAKE_BOTS_CACHE.iter() {
                    let val = fb.value();

                    if val.official_bot_ids.contains(&new_member.user.id) {
                        continue;
                    }

                    if normalized_name.contains(&val.name) {
                        found = true;
                        break;
                    }

                    let (diff, _) = text_diff::diff(&normalized_name, &val.name, "");

                    if diff < 2 {
                        found = true;
                        break;
                    }
                }

                if found {
                    triggered_flags |= TriggeredFlags::FAKE_BOT_DETECTION;
                }
            }

            if triggered_flags != TriggeredFlags::NONE {
                new_member
                    .kick(
                        &ctx.http,
                        Some("Below configured minimum/maximum account age"),
                    )
                    .await?;
            }

            Ok(())
        }
        _ => Ok(()),
    }
}
