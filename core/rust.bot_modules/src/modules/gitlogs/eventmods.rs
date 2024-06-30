// Event modifier related commands

use poise::serenity_prelude::ChannelId;
use rand::distributions::{Alphanumeric, DistString};

use crate::{Context, Error};

/// Event modifier base command
#[poise::command(
    category = "Event Modifiers",
    prefix_command,
    slash_command,
    guild_cooldown = 10,
    subcommands("create", "delete")
)]
pub async fn eventmod(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}

/// Creates a event modifier on a webhook
#[poise::command(
    slash_command,
    prefix_command,
    guild_only,
    guild_cooldown = 60,
    required_permissions = "MANAGE_GUILD"
)]
#[allow(clippy::too_many_arguments)]
pub async fn create(
    ctx: Context<'_>,
    #[description = "The webhook ID"] webhook_id: String,
    #[description = "The events to match against, comma/space seperated"] events: String,
    #[description = "Blacklist the events"] blacklisted: bool,
    #[description = "Whitelist the events. Other events will not be allowed"] whitelisted: bool,
    #[description = "Priority. Use 0 for normal priority"] priority: Option<i32>,
    // Lazy = "prefer to parse the current argument as the other params first"
    #[description = "Repository ID, will match all if unset"]
    #[lazy]
    repo_id: Option<String>,
    #[description = "Redirect channel ID"] redirect_channel: Option<ChannelId>,
) -> Result<(), Error> {
    let data = ctx.data();

    // Check if the guild exists on our DB
    let guild = sqlx::query!(
        "SELECT COUNT(1) FROM gitlogs__guilds WHERE guild_id = $1",
        ctx.guild_id().unwrap().to_string()
    )
    .fetch_one(&data.pool)
    .await?;

    if guild.count.unwrap_or_default() == 0 {
        // If it doesn't, return a error
        return Err("You don't have any webhooks in this guild! Use ``/newhook`` (or ``git!newhook``) to create one".into());
    }

    // Check webhook count
    let webhook_count = sqlx::query!(
        "SELECT COUNT(1) FROM gitlogs__webhooks WHERE guild_id = $1",
        ctx.guild_id().unwrap().to_string()
    )
    .fetch_one(&data.pool)
    .await?;

    let count = webhook_count.count.unwrap_or_default();

    if count == 0 {
        Err("You don't have any webhooks in this guild! Use ``/newhook`` (or ``git!newhook``) to create one".into())
    } else {
        // Check if the webhook exists
        let webhook = sqlx::query!(
            "SELECT COUNT(1) FROM gitlogs__webhooks WHERE id = $1 AND guild_id = $2",
            webhook_id,
            ctx.guild_id().unwrap().to_string()
        )
        .fetch_one(&data.pool)
        .await?;

        if webhook.count.unwrap_or_default() == 0 {
            return Err(
                "That webhook doesn't exist! Use ``/newhook`` (or ``git!newhook``) to create one"
                    .into(),
            );
        }

        let mut parsed_repo_id = repo_id.clone(); // Since prefix commands suck without this

        if let Some(ref inner_repo_id) = repo_id {
            if inner_repo_id.is_empty() || inner_repo_id == "None" || inner_repo_id == "none" {
                parsed_repo_id = None;
            } else {
                // Check if the repo exists
                let repo = sqlx::query!(
                    "SELECT COUNT(1) FROM gitlogs__repos WHERE id = $1 AND webhook_id = $2",
                    inner_repo_id,
                    webhook_id
                )
                .fetch_one(&data.pool)
                .await?;

                if repo.count.unwrap_or_default() == 0 {
                    return Err("That repo doesn't exist! Use ``/newrepo`` (or ``git!newrepo``) to create one".into());
                }
            }
        }

        let events = events
            .replace('`', "")
            .replace(',', " ")
            .replace("  ", " ")
            .split(' ')
            .map(|s| s.to_string())
            .collect::<Vec<String>>();

        // Check the number of modifiers we already have
        let modifier_count = sqlx::query!(
            "SELECT COUNT(1) FROM gitlogs__event_modifiers WHERE webhook_id = $1",
            webhook_id
        )
        .fetch_one(&data.pool)
        .await?;

        let count = modifier_count.count.unwrap_or_default();

        if count >= 10 {
            return Err("You can only have 10 event modifiers per webhook!".into());
        }

        // Create the event modifier
        let modifier_id = Alphanumeric.sample_string(&mut rand::thread_rng(), 256);
        sqlx::query!(
            "INSERT INTO gitlogs__event_modifiers (id, webhook_id, events, repo_id, blacklisted, whitelisted, redirect_channel, guild_id, priority) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
            modifier_id,
            webhook_id,
            &events,
            parsed_repo_id,
            blacklisted,
            whitelisted,
            redirect_channel.map(|c| c.to_string()),
            ctx.guild_id().unwrap().to_string(),
            priority.unwrap_or_default()
        )
        .execute(&data.pool)
        .await?;

        ctx.say(format!("Modifier created with ID ``{}``.", modifier_id))
            .await?;

        Ok(())
    }
}

/// Deletes a event modifier by id
#[poise::command(
    slash_command,
    prefix_command,
    guild_only,
    guild_cooldown = 60,
    required_permissions = "MANAGE_GUILD"
)]
pub async fn delete(
    ctx: Context<'_>,
    #[description = "The modifier ID"] modifier_id: String,
) -> Result<(), Error> {
    let data = ctx.data();

    // Check if the guild exists on our DB
    let guild = sqlx::query!(
        "SELECT COUNT(1) FROM gitlogs__guilds WHERE guild_id = $1",
        ctx.guild_id().unwrap().to_string()
    )
    .fetch_one(&data.pool)
    .await?;

    if guild.count.unwrap_or_default() == 0 {
        // If it doesn't, return a error
        return Err("You don't have any webhooks in this guild! Use ``/gitlogs newhook`` (or ``%gitlogs newhook``) to create one".into());
    }

    // Check for event modifiers
    let modifier_count = sqlx::query!(
        "SELECT COUNT(1) FROM gitlogs__event_modifiers WHERE guild_id = $1 AND id = $2",
        ctx.guild_id().unwrap().to_string(),
        modifier_id
    )
    .fetch_one(&data.pool)
    .await?;

    let count = modifier_count.count.unwrap_or_default();

    if count == 0 {
        return Err("That modifier doesn't exist!".into());
    }

    // Delete the event modifier
    sqlx::query!(
        "DELETE FROM gitlogs__event_modifiers WHERE id = $1",
        modifier_id
    )
    .execute(&data.pool)
    .await?;

    ctx.say("Modifier deleted!").await?;

    Ok(())
}
