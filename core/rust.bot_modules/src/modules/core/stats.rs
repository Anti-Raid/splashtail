use poise::{serenity_prelude::CreateEmbed, CreateReply};
use sqlx::types::chrono;

type Error = crate::Error;
type Context<'a> = crate::Context<'a>;

// Various statistics
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const GIT_SHA: &str = env!("GIT_COMMIT_HASH");
pub const GIT_REPO: &str = env!("GIT_REPO");
pub const GIT_COMMIT_MSG: &str = env!("GIT_COMMIT_MESSAGE");
pub const BUILD_CPU: &str = env!("CPU_MODEL");
pub const CARGO_PROFILE: &str = env!("CARGO_PROFILE");
pub const RUSTC_VERSION: &str = env!("RUSTC_VERSION");

#[poise::command(category = "Stats", prefix_command, slash_command, user_cooldown = 1)]
pub async fn stats(ctx: Context<'_>) -> Result<(), Error> {
    let stats = ctx.data().props.statistics();
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
            .field(
                "Cluster",
                format!(
                    "{} ({} of {})",
                    stats.cluster_name,
                    stats.cluster_id,
                    stats.cluster_count - 1,
                ),
                true,
            )
            .field(
                "Clusters Available",
                format!("{}/{}", stats.available_clusters.len(), stats.cluster_count),
                true,
            )
            .field("Servers", stats.total_guilds.to_string(), true)
            .field("Users", stats.total_users.to_string(), true)
            .field("Commit Message", GIT_COMMIT_MSG, true)
            .field("Built On", BUILD_CPU, true)
            .field("Cargo Profile", CARGO_PROFILE, true),
    );

    ctx.send(msg).await?;
    Ok(())
}
