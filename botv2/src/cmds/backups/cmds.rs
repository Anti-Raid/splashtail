use std::collections::HashMap;
use std::time::Duration;
use futures_util::StreamExt;

use crate::{Context, Error};
use std::sync::Arc;
use serenity::all::CreateEmbed;

/*
type BackupCreateOpts struct {
	I PerChannel                int            `json:"per_channel" description:"The number of messages per channel"`
	I MaxMessages               int            `json:"max_messages" description:"The maximum number of messages to backup"`
	I BackupMessages            bool           `json:"backup_messages" description:"Whether to backup messages or not"`
	I BackupAttachments         bool           `json:"backup_attachments" description:"Whether to backup attachments or not"`
	I BackupGuildAssets         []string       `json:"backup_guild_assets" description:"What assets to back up"`
    I IgnoreMessageBackupErrors bool           `json:"ignore_message_backup_errors" description:"Whether to ignore errors while backing up messages or not and skip these channels"`
	I RolloverLeftovers         bool           `json:"rollover_leftovers" description:"Whether to attempt rollover of leftover message quota to another channels or not"`
	SpecialAllocations          map[string]int `json:"special_allocations" description:"Specific channel allocation overrides"`
	I Encrypt                   string           `json:"encrypt" description:"Whether to encrypt the backup or not"`
}

type BackupRestoreOpts struct {
    IgnoreRestoreErrors bool     `json:"ignore_restore_errors" description:"Whether to ignore errors while restoring or not"`
	I ProtectedChannels []string `json:"protected_channels" description:"Channels to protect from being deleted"`
	I BackupSource      string   `json:"backup_source" description:"The source of the backup"`
	I Decrypt           string   `json:"decrypt" description:"The key to decrypt backups with, if any"`
	I ChannelRestoreMode ChannelRestoreMode `json:"channel_restore_mode" description:"Channel backup restore method. Use 'full' if unsure"`
    RoleRestoreMode    RoleRestoreMode    `json:"role_restore_mode" description:"Role backup restore method. Use 'full' if unsure"`
}
*/

/// Create, load and get info on backups of your server!
#[poise::command(
    prefix_command,
    slash_command,
    guild_only,
    user_cooldown = "5",
    aliases("backup"),
    subcommands("backups_create", "backups_list")
)]
pub async fn backups(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}

/// Create a backup of the current server
#[poise::command(
    prefix_command, 
    slash_command,
    guild_only,
    rename = "create",
)]
#[allow(clippy::too_many_arguments)] // This function needs these arguments due to poise
pub async fn backups_create(
    ctx: Context<'_>,

    #[description = "Whether to include messages in the backup (up to 500)"]
    messages: Option<bool>,

    #[description = "Whether to include attachments in the backup. Requires 'messages' to be enabled"]
    attachments: Option<bool>,

    #[description = "What assets to back up in comma-seperated form (icon,splash,banner)"]
    backup_guild_assets: Option<String>,

    #[description = "Roll over leftover message quotas to other channels. May make backups slower. Defaults to true"]
    rollover_leftovers: Option<bool>,

    #[description = "Whether to ignore errors while backing up messages or not and skip these channels"]
    ignore_message_backup_errors: Option<bool>,

    #[description = "The maximum number of messages to backup. Defaults to 500"]
    max_messages: Option<i32>,

    #[description = "The number of messages per channel to backup. Defaults to 100"]
    per_channel: Option<i32>,

    #[description = "Specific channel allocation overrides. Format: channel_id=number,channel_id=number"]
    special_allocations: Option<String>,

    #[description = "Password to encrypt the backup with. If not provided, the backup will not be encrypted"]
    password: Option<String>,
) -> Result<(), Error> {
    /*
    let messages = ctx.interaction.options.getBoolean("messages")
                let attachments = ctx.interaction.options.getBoolean("attachments")
                let backupGuildAssets = ctx.interaction.options.getString("backup_guild_assets")?.split(",") || defaultAssets
                let maxMessages = ctx.interaction.options.getInteger("max_messages")
                let perChannel = ctx.interaction.options.getInteger("per_channel")
                let rolloverLeftovers = ctx.interaction.options.getBoolean("rollover_leftovers")
                let ignoreMessageBackupErrors = ctx.interaction.options.getBoolean("ignore_message_backup_errors")
                let password = ctx.interaction.options.getString("password") || ""

                if(backupGuildAssets.length > 0) {
                    backupGuildAssets = backupGuildAssets?.map((v) => v.trim())?.filter((v) => v.length > 0)
                } */
    let messages = messages.unwrap_or(false);
    let attachments = attachments.unwrap_or(false);
    let backup_guild_assets = backup_guild_assets.unwrap_or_default();
    let rollover_leftovers = rollover_leftovers.unwrap_or(true);
    let ignore_message_backup_errors = ignore_message_backup_errors.unwrap_or(false);
    let max_messages = max_messages.unwrap_or(500);
    let per_channel = per_channel.unwrap_or(100);
    let special_allocations = special_allocations.unwrap_or_default();
    let password = password.unwrap_or_default();

    if !messages && attachments {
        return Err("You must backup messages to backup attachments".into());
    }

    let backup_guild_assets = {
        let split = backup_guild_assets.split(',').collect::<Vec<&str>>();

        if !split.is_empty() {
            split.iter().map(|v| v.trim()).filter(|v| !v.is_empty()).collect::<Vec<&str>>()
        } else {
            vec!["icon", "splash", "banner"]
        }
    };

    let special_allocations = {
        let split = special_allocations.split(',').collect::<Vec<&str>>();

        if !split.is_empty() {
            let mut map = HashMap::new();

            for v in split {
                if v.is_empty() {
                    continue;
                }

                let split = v.split('=').collect::<Vec<&str>>();

                if split.len() != 2 {
                    return Err("Invalid special allocation format".into());
                }

                let channel_id = split[0].to_string();
                let number = split[1].parse::<u32>()?;

                map.insert(channel_id, number);
            }

            map
        } else {
            HashMap::new()
        }
    };

    let mut base_message = ctx.send(
        poise::CreateReply::default()
        .embed(
            CreateEmbed::default()
            .title("Creating Backup...")
            .description(":yellow_circle: Please wait, starting backup task...")
        )
    )
    .await?
    .into_message()
    .await?;

    // Create reqwest client
    let client = reqwest::Client::builder()
    .timeout(std::time::Duration::from_secs(10))
    .user_agent(
        format!("Splashtail/botv2 {} (cluster {})", env!("CARGO_PKG_VERSION"), crate::ipc::argparse::MEWLD_ARGS.cluster_id)
    )
    .build()?;

    // Create backup
    let backup = client.post(format!("{}/ipc/create_task", crate::config::CONFIG.meta.jobserver_url.get()))
    .json(&serde_json::json!({
        "args": {
            "name": "guild_create_backup",
            "execute": true,
            "data": {
                "ServerID": ctx.guild_id().unwrap().to_string(),
                "Options": {
                    "PerChannel": per_channel,
                    "MaxMessages": max_messages,
                    "BackupMessages": messages,
                    "BackupAttachments": attachments,
                    "BackupGuildAssets": backup_guild_assets,
                    "IgnoreMessageBackupErrors": ignore_message_backup_errors,
                    "RolloverLeftovers": rollover_leftovers,
                    "SpecialAllocations": special_allocations,
                    "Encrypt": password
                }
            }
        }
    }))
    .header(
        "Authorization",
        format!(
            "{} {}",
            "bot",
            crate::config::CONFIG.meta.jobserver_secrets.get().get("bot").expect("No jobserver secret set")
        )
    )
    .send()
    .await?
    .json::<crate::jobserver::WrappedTaskCreateResponse>()
    .await?
    .tcr;

    base_message
    .edit(
        &ctx,
        serenity::all::EditMessage::default()
        .embed(
            CreateEmbed::default()
            .title("Creating Backup...")
            .description(format!(":yellow_circle: Created task with Task ID of {}", backup.task_id))
        )
    )
    .await?;

    async fn update_base_message(
        cache_http: crate::impls::cache::CacheHttpImpl,
        mut base_message: serenity::model::channel::Message,
        task: Arc<crate::jobserver::Task>,
    ) -> Result<(), Error> {
        let new_task_msg = crate::jobserver::taskpoll::embed(&task)?;   
    
        base_message
        .edit(
            &cache_http,
            new_task_msg
            .to_prefix_edit(serenity::all::EditMessage::default())
        )
        .await?;

        Ok(())
    }

    // Use jobserver::reactive to keep updating the message
    crate::jobserver::taskpoll::reactive(
        &ctx.data().cache_http,
        &ctx.data().pool,
        backup.task_id.as_str(),
        |cache_http, task| {
            Box::pin(
                update_base_message(cache_http.clone(), base_message.clone(), task.clone())
            )
        },
        crate::jobserver::taskpoll::PollTaskOptions {
            interval: Some(1),
        }
    )
    .await?;

    Ok(())
}

/// List all currently made backups and allow for downloading them
#[poise::command(
    prefix_command,
    slash_command,
    guild_only,
    user_cooldown = "5",
    rename = "list",
)]
pub async fn backups_list(ctx: Context<'_>) -> Result<(), Error> {
    let Some(guild_id) = ctx.guild_id() else {
        return Err("This command can only be used in a guild".into());
    };

    let backup_tasks = crate::jobserver::Task::from_guild_and_task_name(guild_id, "guild_create_backup", &ctx.data().pool)
    .await
    .map_err(|e| {
       format!("Failed to get backup tasks: {}", e)
    })?;

    const MAX_TASKS_PER_PAGE: usize = 9;

    fn create_embeds(current_embed_page: usize, backup_tasks: &[crate::jobserver::Task]) -> Vec<serenity::all::CreateEmbed> {
        let mut embeds = vec![
            poise::serenity_prelude::CreateEmbed::default()
            .title("Backups")
            .description("Here are all the backups for this server")
            .color(poise::serenity_prelude::Colour::DARK_GREEN)
        ];
    
        let backup_tasks_iter = backup_tasks.iter().skip(current_embed_page * MAX_TASKS_PER_PAGE).take(MAX_TASKS_PER_PAGE);
    
        for task in backup_tasks_iter {
            let mut embed = poise::serenity_prelude::CreateEmbed::default()
            .title(format!("Backup Task {}", task.task_id))
            .description(format!("Task Name: {}\nTask State: {}\nTask Created At: {}", task.task_name, task.state, task.created_at));
    
            if let Some(ref output) = task.output {
                let furl = format!("{}/tasks/{}/ioauth/download-link", crate::config::CONFIG.sites.api.get(), task.task_id);
                embed = embed
                .description(format!("Task Name: {}\nTask State: {}\nTask Created At: {}\n\n:link: [Download {}]({})", task.task_name, task.state, task.created_at, output.filename, &furl));
            }
    
            embed = embed
            .color(poise::serenity_prelude::Colour::DARK_GREEN);
    
            embeds.push(embed);
        }

        embeds
    }

    fn create_reply(
        current_embed_page: usize, 
        backup_tasks: &[crate::jobserver::Task],
    ) -> poise::CreateReply {
        let mut cr = poise::CreateReply::default()
        .components(
            vec![
                serenity::all::CreateActionRow::Buttons(
                    vec![
                        serenity::all::CreateButton::new("backups_list_previous")
                        .label("Previous")
                        .style(serenity::all::ButtonStyle::Primary)
                        .disabled(current_embed_page == 0),
                        serenity::all::CreateButton::new("backups_list_next")
                        .label("Next")
                        .style(serenity::all::ButtonStyle::Primary)
                        .disabled(current_embed_page >= backup_tasks.len() / MAX_TASKS_PER_PAGE),
                    ]
                )
            ]
        );
    
        for embed in create_embeds(current_embed_page, backup_tasks) {
            cr = cr.embed(embed);
        }
    
        cr
    }

    let mut current_embed_page = 0;

    let cr = create_reply(current_embed_page, &backup_tasks);

    let msg = ctx.send(cr)
    .await?
    .into_message()
    .await?;

    let collector = msg.await_component_interactions(ctx.serenity_context())
    .author_id(ctx.author().id)
    .timeout(Duration::from_secs(120));

    let mut collect_stream = collector.stream();

    while let Some(item) = collect_stream.next().await {
        let item_id = item.data.custom_id.as_str();

        match item_id {
            "backups_list_previous" => {
                if current_embed_page == 0 {
                    continue;
                }

                current_embed_page -= 1;
            },
            "backups_list_next" => {
                if current_embed_page >= backup_tasks.len() / MAX_TASKS_PER_PAGE {
                    continue;
                }

                current_embed_page += 1;
            },
            _ => {
                continue;
            }
        }

        let cr = create_reply(current_embed_page, &backup_tasks);

        item.create_response(
            ctx.serenity_context(), 
            serenity::all::CreateInteractionResponse::Message(
                cr.to_slash_initial_response(serenity::all::CreateInteractionResponseMessage::default())
            )
        )
        .await?;
    }    

    Ok(())
}