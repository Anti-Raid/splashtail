use crate::{silverpelt::EventHandlerContext, Error};
use poise::serenity_prelude::FullEvent;

pub async fn event_listener(ectx: &EventHandlerContext) -> Result<(), Error> {
    let ctx = &ectx.serenity_context;
    let event = &ectx.full_event;

    match event {
        FullEvent::Message { new_message } => {
            let Some(guild_id) = new_message.guild_id else {
                return Err("No guild ID found".into());
            };

            // Get all mentioned users in the message
            let mentioned_users = new_message
                .mentions
                .iter()
                .filter(|u| !u.bot() && u.id != new_message.author.id) // Filter out bots and the author itself
                .take(20) // Collect first 20 mentioned users
                .collect::<Vec<_>>();

            if mentioned_users.is_empty() {
                return Ok(());
            }

            // We auto-expire them every week automatically to both preserve performance to avoid table bloat
            let recs = sqlx::query!(
                "SELECT user_id, reason, expires_at FROM afk__afks WHERE guild_id = $1 AND user_id = ANY ($2) AND expires_at > NOW()",
                guild_id.to_string(),
                &mentioned_users.iter().map(|u| u.id.to_string()).collect::<Vec<_>>(),
            )
            .fetch_all(&ectx.data.pool)
            .await?;

            if recs.is_empty() {
                return Ok(());
            }

            let mut embeds = Vec::new();

            for rec in recs {
                if embeds.len() > base_data::limits::embed_limits::EMBED_MAX_COUNT {
                    break;
                }

                let Some(user) = mentioned_users
                    .iter()
                    .find(|u| u.id.to_string() == rec.user_id)
                else {
                    continue;
                };

                let reason = rec.reason;
                let expires_at = rec.expires_at;

                let embed = serenity::all::CreateEmbed::default()
                    .title("AFK")
                    .field("User", user.tag(), true)
                    .description(format!(
                        "**Reason:** {}\n**Expires at:** <t:{}>:R",
                        reason,
                        expires_at.timestamp()
                    ));

                embeds.push(embed);
            }

            let cm: serenity::all::CreateMessage = serenity::all::CreateMessage::new()
                .reference_message(new_message)
                .embeds(embeds);

            // Reply to the new_message itself
            new_message.channel_id.send_message(&ctx.http, cm).await?;

            Ok(())
        }
        _ => Ok(()),
    }
}
