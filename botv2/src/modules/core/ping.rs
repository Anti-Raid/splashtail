use poise::{serenity_prelude::CreateEmbed, CreateReply};
use serenity::builder::EditMessage;

type Error = crate::Error;
type Context<'a> = crate::Context<'a>;

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
            .field("Edit Latency", "Calculating...", true),
    );

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
                ),
        ),
    )
    .await?;

    Ok(())
}
