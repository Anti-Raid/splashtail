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

            let recs = sqlx::query!(
                "SELECT id, reason, expires_at FROM afk__afks WHERE guild_id = $1 AND user_id = $2",
                guild_id.to_string(),
                new_message.author.id.to_string(),
            )
            .fetch_all(&ectx.data.pool)
            .await?;

            if recs.is_empty() {
                return Ok(());
            }

            let mut reason = None;
            let mut expires_at = None;
            for rec in recs {
                if rec.expires_at < chrono::Utc::now() {
                    sqlx::query!("DELETE FROM afk__afks WHERE id = $1", rec.id,)
                        .execute(&ectx.data.pool)
                        .await?;
                }

                reason = Some(rec.reason);
                expires_at = Some(rec.expires_at);
            }

            if reason.is_none() || expires_at.is_none() {
                return Ok(());
            }

            let cm = serenity::all::CreateMessage::new()
                .reference_message(new_message)
                .embed(
                    serenity::all::CreateEmbed::default()
                        .title("AFK")
                        .field("User", new_message.author.tag(), true)
                        .description(reason.unwrap_or("No reason provided".to_string())),
                );

            // Reply to the new_message itself
            new_message.channel_id.send_message(&ctx.http, cm).await?;

            Ok(())
        }
        _ => Ok(()),
    }
}
