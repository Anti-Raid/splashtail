use std::{collections::HashMap, fmt::Display};
use std::time::Duration;
use futures_util::StreamExt;
use crate::{Context, Error};
use std::sync::Arc;
use serenity::all::{EditMessage, CreateEmbed};
use crate::ipc::animus_magic::{client::{AnimusTarget, AnimusMessage, AnimusResponse}, jobserver::{JobserverAnimusMessage, JobserverAnimusResponse}};
use serenity::small_fixed_array::TruncatingInto;

/*
// Options that can be set when creatng a backup
type BackupCreateOpts struct {
	PerChannel                int            `description:"The number of messages per channel"`
	MaxMessages               int            `description:"The maximum number of messages to backup"`
	BackupMessages            bool           `description:"Whether to backup messages or not"`
	BackupAttachments         bool           `description:"Whether to backup attachments or not"`
	BackupGuildAssets         []string       `description:"What assets to back up"`
	IgnoreMessageBackupErrors bool           `description:"Whether to ignore errors while backing up messages or not and skip these channels"`
	RolloverLeftovers         bool           `description:"Whether to attempt rollover of leftover message quota to another channels or not"`
	SpecialAllocations        map[string]int `description:"Specific channel allocation overrides"`
	Encrypt                   string         `description:"The key to encrypt backups with, if any"`
}

// Options that can be set when restoring a backup
type BackupRestoreOpts struct {
	IgnoreRestoreErrors bool               `description:"Whether to ignore errors while restoring or not and skip these channels/roles"`
	ProtectedChannels   []string           `description:"Channels to protect from being deleted"`
	ProtectedRoles      []string           `description:"Roles to protect from being deleted"`
	BackupSource        string             `description:"The source of the backup"`
	Decrypt             string             `description:"The key to decrypt backups with, if any"`
	ChannelRestoreMode  ChannelRestoreMode `description:"Channel backup restore method. Use 'full' if unsure"`
	RoleRestoreMode     RoleRestoreMode    `description:"Role backup restore method. Use 'full' if unsure"`
}
*/

/// Create, load and get info on backups of your server!
#[poise::command(
    prefix_command,
    slash_command,
    guild_only,
    user_cooldown = "5",
    aliases("backup"),
    subcommands("backups_create", "backups_list", "backups_restore")
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

    // Create backup
    let backup_args = serde_json::json!({
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
    });

    let data = ctx.data();

    let backup_task_id = match data.animus_magic_ipc.request(
        AnimusTarget::Jobserver, 
        AnimusMessage::Jobserver(
            JobserverAnimusMessage::SpawnTask { 
                name: "guild_create_backup".to_string(),
                data: backup_args,
                create: true,
                execute: true,
                task_id: None,
             }
        )
    )
    .await
    .map_err(|e| {
        format!("Failed to create backup task: {}", e)
    })? {
        AnimusResponse::Jobserver(JobserverAnimusResponse::SpawnTask { task_id }) => task_id,
        _ => return Err("Invalid response from jobserver".into()),
    };

    base_message
    .edit(
        &ctx,
        serenity::all::EditMessage::default()
        .embed(
            CreateEmbed::default()
            .title("Creating Backup...")
            .description(format!(":yellow_circle: Created task with Task ID of {}", backup_task_id))
        )
    )
    .await?;

    let ch = crate::impls::cache::CacheHttpImpl {
        cache: ctx.serenity_context().cache.clone(),
        http: ctx.serenity_context().http.clone(),
    };

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
        &ch,
        &ctx.data().pool,
        &backup_task_id,
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

    let mut backup_tasks = crate::jobserver::Task::from_guild_and_task_name(guild_id, "guild_create_backup", &ctx.data().pool)
    .await
    .map_err(|e| {
       format!("Failed to get backup tasks: {}", e)
    })?;

    if backup_tasks.is_empty() {
        ctx.say("You don't have any backups yet!\n\n**TIP:** Use `/backups create` to create your first server backup :heart:").await?;
        return Ok(());
    }

    fn create_embed_for_task<'a>(task: &crate::jobserver::Task) -> serenity::all::CreateEmbed<'a> {    
        let mut initial_desc = format!("Task ID: {}\nTask Name: {}\nTask State: {}\n\n**Created At**: <t:{}:f> (<t:{}:R>)", task.task_id, task.task_name, task.state, task.created_at.timestamp(), task.created_at.timestamp());
        
        let embed = poise::serenity_prelude::CreateEmbed::default()
        .title(
            format!(
                "{} | Server Backup",
                crate::jobserver::get_icon_of_state(task.state.as_str())
            )
        );

        if let Some(ref output) = task.output {
            let furl = format!("{}/tasks/{}/ioauth/download-link", crate::config::CONFIG.sites.api.get(), task.task_id);
            
            initial_desc += &format!("\n\n:link: [Download {}]({})", output.filename, &furl);
        }

        embed
        .description(initial_desc)
        .color(poise::serenity_prelude::Colour::DARK_GREEN)
    }

    fn create_reply<'a>(
        index: usize,
        backup_tasks: &[crate::jobserver::Task]
    ) -> Result<poise::CreateReply<'a>, Error> {
        if backup_tasks.is_empty() || index >= backup_tasks.len() {
            return Err("No backups found".into());
        }

        let cr = poise::CreateReply::default()
        .embed(create_embed_for_task(&backup_tasks[index]))
        .components(
            vec![
                serenity::all::CreateActionRow::Buttons(
                    vec![
                        serenity::all::CreateButton::new("backups_previous")
                        .label("Previous")
                        .emoji(serenity::all::ReactionType::Unicode("◀️".to_string().trunc_into()))
                        .style(serenity::all::ButtonStyle::Primary)
                        .disabled(index == 0),
                        serenity::all::CreateButton::new("backups_next")
                        .label("Next")
                        .emoji(serenity::all::ReactionType::Unicode("▶️".to_string().trunc_into()))
                        .style(serenity::all::ButtonStyle::Primary)
                        .disabled(index >= backup_tasks.len()),
                        serenity::all::CreateButton::new("backups_last")
                        .label("Last")
                        .emoji(serenity::all::ReactionType::Unicode("⏩".to_string().trunc_into()))
                        .style(serenity::all::ButtonStyle::Primary)
                        .disabled(index >= backup_tasks.len()),
                        serenity::all::CreateButton::new("backups_first")
                        .label("First")
                        .emoji(serenity::all::ReactionType::Unicode("⏪".to_string().trunc_into()))
                        .style(serenity::all::ButtonStyle::Primary)
                        .disabled(index == 0),
                        serenity::all::CreateButton::new("backups_delete")
                        .label("Delete")
                        .style(serenity::all::ButtonStyle::Danger)
                    ]
                )
            ]
        );
    
        Ok(cr)
    }

    let mut index = 0;

    let cr = create_reply(index, &backup_tasks)?;

    let msg = ctx.send(cr)
    .await?
    .into_message()
    .await?;

    let collector = msg.await_component_interactions(ctx.serenity_context())
    .author_id(ctx.author().id)
    .timeout(Duration::from_secs(180));

    let mut collect_stream = collector.stream();

    while let Some(item) = collect_stream.next().await {
        let item_id = item.data.custom_id.as_str();

        match item_id {
            "backups_previous" => {
                if index == 0 {
                    continue;
                }

                index -= 1;
            },
            "backups_next" => {
                if index >= backup_tasks.len() {
                    continue;
                }

                index += 1;
            },
            "backups_last" => {
                index = backup_tasks.len() - 1;
            },
            "backups_first" => {
                index = 0;
            },
            "backups_delete" => {
                item.defer(&ctx.serenity_context()).await?;

                let mut confirm = ctx.send(
                    poise::reply::CreateReply::default()
                    .content("Are you sure you want to delete this backup?\n\n**This action is irreversible!**")
                    .components(
                        vec![
                            serenity::all::CreateActionRow::Buttons(
                                vec![
                                    serenity::all::CreateButton::new("backups_delete_confirm")
                                    .label("Yes")
                                    .style(serenity::all::ButtonStyle::Success),
                                    serenity::all::CreateButton::new("backups_delete_cancel")
                                    .label("No")
                                    .style(serenity::all::ButtonStyle::Danger),
                                ]
                            )
                        ]
                    )
                )
                .await?
                .into_message()
                .await?;

                let confirm_collector = confirm
                .await_component_interaction(ctx.serenity_context())
                .author_id(ctx.author().id)
                .timeout(Duration::from_secs(30))
                .await;

                if confirm_collector.is_none() {
                    // Edit the message to say that the user took too long to respond
                    confirm.edit(
                        &ctx.serenity_context(), 
                        EditMessage::default()
                        .content("You took too long to respond")
                    ).await?;
                }

                let confirm_item = confirm_collector.unwrap();

                match confirm_item.data.custom_id.as_str() {
                    "backups_delete_confirm" => {
                        // Take out the current backup task
                        let task = backup_tasks.remove(index);

                        // Respond to the interaction
                        confirm_item.create_response(
                            &ctx,
                            serenity::all::CreateInteractionResponse::Message(
                                serenity::all::CreateInteractionResponseMessage::default()
                                .embed(
                                    CreateEmbed::default()
                                    .title("Deleting Backup...")
                                    .description(":yellow_circle: Please wait while we delete this backup")
                                )    
                            )
                        )
                        .await?;

                        let mut status = Vec::new();

                        match task.delete_from_storage(&ctx.data().object_store).await {
                            Ok(_) => {
                                status.push(":white_check_mark: Successfully deleted the backup from storage".to_string());
                            },
                            Err(e) => {
                                status.push(format!(":x: Failed to delete the backup from storage: {}", e));
                            }
                        };

                        if let Err(e) = confirm_item
                        .edit_response(
                            &ctx,
                            serenity::all::EditInteractionResponse::default()
                            .embed(
                                CreateEmbed::default()
                                .title("Deleting Backup")
                                .description(status.join("\n"))
                            )
                        )
                        .await {
                            log::error!("Failed to edit message: {}", e);
                        }

                        // Lastly deleting the task from the database
                        match task.delete_from_db(&ctx.data().pool).await {
                            Ok(_) => {
                                status.push(":white_check_mark: Successfully deleted the backup task from database".to_string());
                            },
                            Err(e) => {
                                status.push(format!(":x: Failed to delete the backup task from database: {}", e));
                            }
                        };

                        if let Err(e) = confirm_item
                        .edit_response(
                            &ctx,
                            serenity::all::EditInteractionResponse::default()
                            .embed(
                                CreateEmbed::default()
                                .title("Deleting Backup")
                                .description(status.join("\n"))
                            )
                        )
                        .await {
                            log::error!("Failed to edit message: {}", e);
                        }
                    },
                    _ => {
                        continue;
                    }
                }
            },
            _ => {
                continue;
            }
        }

        if index >= backup_tasks.len() {
            index = backup_tasks.len() - 1;
        }

        let cr = create_reply(index, &backup_tasks)?;

        item.edit_response(
            ctx.serenity_context(), 
            cr.to_slash_initial_response_edit(serenity::all::EditInteractionResponse::default())
        )
        .await?;
    }    

    Ok(())
}

#[derive(poise::ChoiceParameter)]
enum ChannelRestoreMode {
    #[name = "full"]
    Full,
    #[name = "partial"]
    Partial,
    #[name = "none"]
    None,
}

impl Display for ChannelRestoreMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ChannelRestoreMode::Full => write!(f, "full"),
            ChannelRestoreMode::Partial => write!(f, "partial"),
            ChannelRestoreMode::None => write!(f, "none"),
        }
    }
}

#[derive(poise::ChoiceParameter)]
enum RoleRestoreMode {
    #[name = "full"]
    Full,
}

impl Display for RoleRestoreMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RoleRestoreMode::Full => write!(f, "full"),
        }
    }
}

/// Restores a created backup
#[poise::command(
    prefix_command,
    slash_command,
    guild_only,
    user_cooldown = "20",
    guild_cooldown = "30",
    rename = "restore",
)]
#[allow(clippy::too_many_arguments)] // This function needs these arguments due to poise
pub async fn backups_restore(
    ctx: Context<'_>,

    #[description = "The backup attachment to restore"]
    backup_file: serenity::all::Attachment,

    #[description = "Password to decrypt backup with. Should not be reused"]
    password: Option<String>,

    #[description = "Channel restore mode. Defaults to full. Use 'full' if unsure"]
    channel_restore_mode: Option<ChannelRestoreMode>,

    #[description = "Role restore mode. Defaults to full. Use 'full' if unsure"]
    role_restore_mode: Option<RoleRestoreMode>,

    #[description = "Channels to protect from being deleted, comma seperated"]
    protected_channels: Option<String>,

    #[description = "Roles to protect from being deleted, comma seperated"]
    protected_roles: Option<String>,

    #[description = "Whether to ignore errors while restoring or not"]
    ignore_restore_errors: Option<bool>,
) -> Result<(), Error> {
    let protected_channels = {
        let mut p = Vec::new();
        let protected_channels = protected_channels.unwrap_or_default();
        let protected_channels_split = protected_channels.split(',');

        for protected_channel in protected_channels_split {
            let trimmed = protected_channel.trim().to_string();
            if trimmed.is_empty() {
                continue;
            }

            if trimmed == ctx.channel_id().to_string() {
                continue;
            }

            p.push(trimmed);
        }

        p.push(ctx.channel_id().to_string());

        p
    };

    let protected_roles = {
        let mut p = Vec::new();
        let protected_roles = protected_roles.unwrap_or_default();
        let protected_roles_split = protected_roles.split(',');

        for protected_role in protected_roles_split {
            let trimmed = protected_role.trim().to_string();
            if trimmed.is_empty() {
                continue;
            }

            p.push(trimmed);
        }

        p
    };

    let mut base_message = ctx.send(
        poise::CreateReply::default()
        .embed(
            CreateEmbed::default()
            .title("Restoring Backup...")
            .description(":yellow_circle: Please wait, starting backup task...")
        )
    )
    .await?
    .into_message()
    .await?;

    let json = serde_json::json!({
        "ServerID": ctx.guild_id().unwrap().to_string(),
        "Options": {
            "IgnoreRestoreErrors": ignore_restore_errors.unwrap_or(false),
            "ProtectedChannels": protected_channels,
            "ProtectedRoles": protected_roles,
            "BackupSource": backup_file.url,
            "Decrypt": password.unwrap_or_default(),
            "ChannelRestoreMode": channel_restore_mode.unwrap_or(ChannelRestoreMode::Full).to_string(),
            "RoleRestoreMode": role_restore_mode.unwrap_or(RoleRestoreMode::Full).to_string(),
        },
    });

    // Restore backup
    let restore_task_id = match ctx.data().animus_magic_ipc.request(
        AnimusTarget::Jobserver, 
        AnimusMessage::Jobserver(
            JobserverAnimusMessage::SpawnTask { 
                name: "guild_restore_backup".to_string(),
                data: json,
                create: true,
                execute: true,
                task_id: None,
             }
        )
    )
    .await
    .map_err(|e| {
        format!("Failed to create restore backup task: {}", e)
    })? {
        AnimusResponse::Jobserver(JobserverAnimusResponse::SpawnTask { task_id }) => task_id,
        _ => return Err("Invalid response from jobserver".into()),
    };

    base_message
    .edit(
        &ctx,
        serenity::all::EditMessage::default()
        .embed(
            CreateEmbed::default()
            .title("Restoring Backup...")
            .description(format!(":yellow_circle: Created task with Task ID of {}", restore_task_id))
        )
    )
    .await?;

    let ch = crate::impls::cache::CacheHttpImpl {
        cache: ctx.serenity_context().cache.clone(),
        http: ctx.serenity_context().http.clone(),
    };
    
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
        &ch,
        &ctx.data().pool,
        restore_task_id.as_str(),
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