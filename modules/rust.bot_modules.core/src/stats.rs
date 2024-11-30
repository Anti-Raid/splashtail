use poise::{serenity_prelude::CreateEmbed, CreateReply};
use rust_buildstats::{
    BUILD_CPU, CARGO_PROFILE, GIT_COMMIT_MSG, GIT_REPO, GIT_SHA, RUSTC_VERSION, VERSION,
};
use sqlx::types::chrono;

type Error = silverpelt::Error;
type Context<'a> = silverpelt::Context<'a>;

#[poise::command(category = "Stats", slash_command, user_cooldown = 1)]
pub async fn stats(ctx: Context<'_>) -> Result<(), Error> {
    let total_guilds = ctx.data().props.total_guilds().await?;
    let total_users = ctx.data().props.total_users().await?;
    let msg = CreateReply::default().embed(
        CreateEmbed::default()
            .title("Bot Stats")
            .field(
                "Bot name",
                ctx.serenity_context().cache.current_user().name.to_string(),
                true,
            )
            .field("Bot version", VERSION, true)
            .field("rustc", RUSTC_VERSION, true)
            .field(
                "Git Commit",
                format!("[{}]({}/commit/{})", GIT_SHA, GIT_REPO, GIT_SHA),
                true,
            )
            .field("Description", ctx.data().props.extra_description(), true)
            .field(
                "Uptime",
                {
                    let duration: std::time::Duration = std::time::Duration::from_secs(
                        (chrono::Utc::now().timestamp() - config::CONFIG.start_time) as u64,
                    );

                    let seconds = duration.as_secs() % 60;
                    let minutes = (duration.as_secs() / 60) % 60;
                    let hours = (duration.as_secs() / 60) / 60;

                    format!("{}h{}m{}s", hours, minutes, seconds)
                },
                true,
            )
            .field("Servers", total_guilds.to_string(), true)
            .field("Users", total_users.to_string(), true)
            .field("Commit Message", GIT_COMMIT_MSG, true)
            .field("Built On", BUILD_CPU, true)
            .field("Cargo Profile", CARGO_PROFILE, true),
    );

    ctx.send(msg).await?;
    Ok(())
}
