use poise::{serenity_prelude::CreateEmbed, CreateReply};
use sqlx::types::chrono;

type Error = crate::Error;
type Context<'a> = crate::Context<'a>;

// Various statistics
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const GIT_SHA: &str = env!("VERGEN_GIT_SHA");
pub const GIT_SEMVER: &str = env!("VERGEN_GIT_SEMVER");
pub const GIT_COMMIT_MSG: &str = env!("VERGEN_GIT_COMMIT_MESSAGE");
pub const BUILD_CPU: &str = env!("VERGEN_SYSINFO_CPU_BRAND");
pub const CARGO_PROFILE: &str = env!("VERGEN_CARGO_PROFILE");
pub const RUSTC_VERSION: &str = env!("VERGEN_RUSTC_SEMVER");

#[poise::command(category = "Stats", prefix_command, slash_command, user_cooldown = 1)]
pub async fn stats(ctx: Context<'_>) -> Result<(), Error> {
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
                GIT_SHA.to_string() + "(semver=" + GIT_SEMVER + ")",
                true,
            )
            .field(
                "Uptime",
                {
                    let duration: std::time::Duration = std::time::Duration::from_secs(
                        (chrono::Utc::now().timestamp() - crate::config::CONFIG.bot_start_time)
                            as u64,
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
                    crate::ipc::argparse::MEWLD_ARGS.cluster_name,
                    crate::ipc::argparse::MEWLD_ARGS.cluster_id,
                    crate::ipc::argparse::MEWLD_ARGS.cluster_count - 1,
                ),
                true,
            )
            .field(
                "Clusters Available",
                format!("{}/{}", ctx.data().mewld_ipc.cache.cluster_healths.len(), crate::ipc::argparse::MEWLD_ARGS.cluster_count),
                true,
            )
            .field(
                "Servers",
                ctx.data().mewld_ipc.cache.total_guilds().to_string(),
                true,
            )
            .field(
                "Users",
                ctx.data().mewld_ipc.cache.total_users().to_string(),
                true,
            )
            .field("Commit Message", GIT_COMMIT_MSG, true)
            .field("Built On", BUILD_CPU, true)
            .field("Cargo Profile", CARGO_PROFILE, true),
    );

    ctx.send(msg).await?;
    Ok(())
}
