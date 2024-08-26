use std::num::NonZeroU16;

use poise::{serenity_prelude::CreateEmbed, CreateReply};
use serenity::builder::EditMessage;

type Error = silverpelt::Error;
type Context<'a> = silverpelt::Context<'a>;

#[poise::command(category = "Stats", prefix_command, slash_command, user_cooldown = 1)]
pub async fn ping(ctx: Context<'_>) -> Result<(), Error> {
    let msg = CreateReply::default().embed(
        CreateEmbed::default()
            .title("Pong")
            .field(
                "Local WS Ping",
                format!("{}μs", ctx.ping().await.as_micros()),
                true,
            )
            .field("Edit Latency", "Calculating...", true)
            .field("Real WS Latency", "Finding...", true),
    );

    let real_ws_latency = {
        if let Some(psd) = ctx.data().props.get_proxysupport_data().await.as_ref() {
            // Due to Sandwich Virtual Sharding etc, we need to reshard the guild id
            let sid = {
                if let Some(guild_id) = ctx.guild_id() {
                    serenity::utils::shard_id(
                        guild_id,
                        NonZeroU16::new(psd.shard_conns.len().try_into()?)
                            .unwrap_or(NonZeroU16::new(1).unwrap()),
                    )
                } else {
                    0 // DMs etc. go to shard 0
                }
            };

            // Convert u16 to i64
            let sid = sid as i64;

            psd.shard_conns.get(&sid).map(|sc| sc.real_latency)
        } else {
            None
        }
    };

    let st = std::time::Instant::now();

    let mut msg = ctx.send(msg).await?.into_message().await?;

    let new_st = std::time::Instant::now();

    msg.edit(
        ctx,
        EditMessage::new().embed(
            CreateEmbed::default()
                .title("Pong")
                .field(
                    "Local WS Ping",
                    format!("{}μs", ctx.ping().await.as_micros()),
                    true,
                )
                .field(
                    "Local Edit Ping",
                    format!("{}ms", new_st.duration_since(st).as_millis()),
                    true,
                )
                .field(
                    "Real WS Latency",
                    real_ws_latency
                        .map(|latency| format!("{}ms", latency))
                        .unwrap_or_else(|| "Unknown".to_string()),
                    true,
                ),
        ),
    )
    .await?;

    Ok(())
}
