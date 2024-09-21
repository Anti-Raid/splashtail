use futures_util::StreamExt;
use serenity::all::{ChannelId, CreateEmbed, EditMessage};
use serenity::small_fixed_array::TruncatingInto;
use serenity::utils::shard_id;
use silverpelt::jobserver::{embed as embed_task, get_icon_of_state};
use silverpelt::Context;
use silverpelt::Error;
use splashcore_rs::jobserver;
use splashcore_rs::utils::{
    create_special_allocation_from_str, parse_numeric_list, REPLACE_CHANNEL,
};
use sqlx::types::uuid::Uuid;
use std::fmt::Display;
use std::sync::Arc;
use std::time::Duration;

/*
// Options that can be set when creatng a backup
type BackupCreateOpts struct {
    Channels                  []string       `description:"If set, the channels to prune messages from"`
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

/// Checks backup encryption, when encrypted, Options->Encrypt is not empty ("SET" / a random string)
fn is_backup_encrypted(fields: &indexmap::IndexMap<String, serde_json::Value>) -> bool {
    fields
        .get("Options")
        .and_then(|options| options.get("Encrypt"))
        .map(|v| !v.as_str().unwrap_or_default().is_empty())
        .unwrap_or_default()
}

/// Create, load and get info on backups of your server!
#[poise::command(
    prefix_command,
    slash_command,
    guild_only,
    user_cooldown = "5",
    aliases("backup"),
    subcommands("backups_create", "backups_list", "backups_delete", "backups_restore")
)]
pub async fn backups(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}

/// Create a backup of the current server
#[poise::command(prefix_command, slash_command, guild_only, rename = "create")]
#[allow(clippy::too_many_arguments)] // This function needs these arguments due to poise
pub async fn backups_create(
    ctx: Context<'_>,

    #[description = "Whether to include messages in the backup (up to 500)"] messages: Option<bool>,

    #[description = "Channels to backup messages from, otherwise all channels will have messages backed up"]
    channels: Option<String>,

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
    let Some(guild_id) = ctx.guild_id() else {
        return Err("This command can only be used in a guild".into());
    };

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
            split
                .iter()
                .map(|v| v.trim())
                .filter(|v| !v.is_empty())
                .collect::<Vec<&str>>()
        } else {
            vec!["icon", "splash", "banner"]
        }
    };

    let channels: Vec<ChannelId> = if let Some(channels) = channels {
        parse_numeric_list(&channels, &REPLACE_CHANNEL)?
    } else {
        vec![]
    };

    let special_allocations = create_special_allocation_from_str(&special_allocations)?;

    let mut base_message = ctx
        .send(
            poise::CreateReply::default().embed(
                CreateEmbed::default()
                    .title("Creating Backup...")
                    .description(":yellow_circle: Please wait, starting backup task..."),
            ),
        )
        .await?
        .into_message()
        .await?;

    // Create backup
    let backup_args = serde_json::json!({
        "ServerID": guild_id.to_string(),
        "Options": {
            "Channels": channels,
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

    // Make request to jobserver
    let jobserver_cluster_id = shard_id(guild_id, data.props.shard_count().await?.try_into()?);
    let resp = data
        .reqwest
        .post(format!(
            "{}:{}/spawn-task",
            config::CONFIG.base_ports.jobserver_base_addr.get(),
            config::CONFIG.base_ports.jobserver.get() + jobserver_cluster_id
        ))
        .json(&splashcore_rs::jobserver::JobserverSpawnTaskRequest {
            name: "guild_create_backup".to_string(),
            data: backup_args,
            create: true,
            execute: true,
            id: None,
            user_id: ctx.author().id.to_string(),
        })
        .send()
        .await
        .map_err(|e| format!("Failed to create backup task: {}", e))?
        .error_for_status()
        .map_err(|e| format!("Failed to create backup task: {}", e))?;

    let backup_id = resp
        .json::<splashcore_rs::jobserver::JobserverSpawnTaskResponse>()
        .await?
        .id;

    base_message
        .edit(
            &ctx,
            serenity::all::EditMessage::default().embed(
                CreateEmbed::default()
                    .title("Creating Backup...")
                    .description(format!(
                        ":yellow_circle: Created task with Task ID of {}",
                        backup_id
                    )),
            ),
        )
        .await?;

    let ch = botox::cache::CacheHttpImpl {
        cache: ctx.serenity_context().cache.clone(),
        http: ctx.serenity_context().http.clone(),
    };

    async fn update_base_message(
        cache_http: botox::cache::CacheHttpImpl,
        mut base_message: serenity::model::channel::Message,
        task: Arc<jobserver::Task>,
    ) -> Result<(), Error> {
        let new_task_msg = embed_task(&config::CONFIG.sites.api.get(), &task, vec![], true)?;

        base_message
            .edit(
                &cache_http,
                new_task_msg.to_prefix_edit(serenity::all::EditMessage::default()),
            )
            .await?;

        Ok(())
    }

    // Use jobserver::reactive to keep updating the message
    jobserver::taskpoll::reactive(
        &ch,
        &ctx.data().pool,
        &backup_id,
        |cache_http, task| {
            Box::pin(update_base_message(
                cache_http.clone(),
                base_message.clone(),
                task.clone(),
            ))
        },
        jobserver::taskpoll::PollTaskOptions::default(),
    )
    .await?;

    Ok(())
}

/// Lists all currently made backups + download/delete them
#[poise::command(
    prefix_command,
    slash_command,
    guild_only,
    user_cooldown = "5",
    rename = "list"
)]
pub async fn backups_list(ctx: Context<'_>) -> Result<(), Error> {
    let Some(guild_id) = ctx.guild_id() else {
        return Err("This command can only be used in a guild".into());
    };

    let data = ctx.data();

    let mut backup_tasks =
        jobserver::Task::from_guild_and_name(guild_id, "guild_create_backup", &data.pool)
            .await
            .map_err(|e| format!("Failed to get backup tasks: {}", e))?;

    if backup_tasks.is_empty() {
        ctx.say("You don't have any backups yet!\n\n**TIP:** Use `/backups create` to create your first server backup :heart:").await?;
        return Ok(());
    }

    fn create_embed_for_task<'a>(task: &jobserver::Task) -> serenity::all::CreateEmbed<'a> {
        let mut initial_desc = format!(
            "Task ID: {}\nTask Name: {}\nTask State: {}\n**Encrypted:** {}\n\n**Created At**: <t:{}:f> (<t:{}:R>)",
            task.id,
            task.name,
            task.state,
            is_backup_encrypted(&task.fields),
            task.created_at.and_utc().timestamp(),
            task.created_at.and_utc().timestamp()
        );

        let embed = poise::serenity_prelude::CreateEmbed::default().title(format!(
            "{} | Server Backup",
            get_icon_of_state(task.state.as_str())
        ));

        if let Some(ref output) = task.output {
            let furl = format!(
                "{}/tasks/{}/ioauth/download-link",
                config::CONFIG.sites.api.get(),
                task.id
            );

            initial_desc += &format!("\n\n:link: [Download {}]({})", output.filename, &furl);
        }

        embed
            .description(initial_desc)
            .color(poise::serenity_prelude::Colour::DARK_GREEN)
    }

    fn create_reply<'a>(
        index: usize,
        backup_tasks: &[jobserver::Task],
    ) -> Result<poise::CreateReply<'a>, Error> {
        if backup_tasks.is_empty() || index >= backup_tasks.len() {
            return Err("No backups found".into());
        }

        let cr = poise::CreateReply::default()
            .embed(create_embed_for_task(&backup_tasks[index]))
            .ephemeral(true)
            .components(vec![
                serenity::all::CreateActionRow::Buttons(vec![
                    serenity::all::CreateButton::new("backups_previous")
                        .label("Previous")
                        .emoji(serenity::all::ReactionType::Unicode(
                            "◀️".to_string().trunc_into(),
                        ))
                        .style(serenity::all::ButtonStyle::Primary)
                        .disabled(index == 0),
                    serenity::all::CreateButton::new("backups_next")
                        .label("Next")
                        .emoji(serenity::all::ReactionType::Unicode(
                            "▶️".to_string().trunc_into(),
                        ))
                        .style(serenity::all::ButtonStyle::Primary)
                        .disabled(index >= backup_tasks.len()),
                    serenity::all::CreateButton::new("backups_last")
                        .label("Last")
                        .emoji(serenity::all::ReactionType::Unicode(
                            "⏩".to_string().trunc_into(),
                        ))
                        .style(serenity::all::ButtonStyle::Primary)
                        .disabled(index >= backup_tasks.len()),
                    serenity::all::CreateButton::new("backups_first")
                        .label("First")
                        .emoji(serenity::all::ReactionType::Unicode(
                            "⏪".to_string().trunc_into(),
                        ))
                        .style(serenity::all::ButtonStyle::Primary)
                        .disabled(index == 0),
                ]),
                serenity::all::CreateActionRow::Buttons(vec![
                    serenity::all::CreateButton::new("backups_restore")
                        .label("Restore")
                        .style(serenity::all::ButtonStyle::Danger),
                    serenity::all::CreateButton::new("backups_delete")
                        .label("Delete")
                        .style(serenity::all::ButtonStyle::Danger),
                ]),
            ]);

        Ok(cr)
    }

    let mut index = 0;

    let cr = create_reply(index, &backup_tasks)?;

    let msg = ctx.send(cr).await?.into_message().await?;

    let collector = msg
        .await_component_interactions(ctx.serenity_context().shard.clone())
        .author_id(ctx.author().id)
        .timeout(Duration::from_secs(180));

    let mut collect_stream = collector.stream();

    while let Some(item) = collect_stream.next().await {
        let item_id = item.data.custom_id.as_str();

        let mut followup_done = false;

        match item_id {
            "backups_previous" => {
                if index == 0 {
                    continue;
                }

                index -= 1;
            }
            "backups_next" => {
                if index >= backup_tasks.len() {
                    continue;
                }

                index += 1;
            }
            "backups_last" => {
                index = backup_tasks.len() - 1;
            }
            "backups_first" => {
                index = 0;
            }
            "backups_restore" => {
                // Check permission
                let perm_res = silverpelt::cmd::check_command(
                    &data.silverpelt_cache,
                    "backups restore",
                    guild_id,
                    ctx.author().id,
                    &ctx.data().pool,
                    &botox::cache::CacheHttpImpl::from_ctx(ctx.serenity_context()),
                    &data.reqwest,
                    &Some(ctx),
                    silverpelt::cmd::CheckCommandOptions::default(), // TODO: Maybe change this to allow backups restore to be disabled?
                )
                .await;

                if !perm_res.is_ok() {
                    item.create_response(
                        &ctx.serenity_context().http,
                        serenity::all::CreateInteractionResponse::Message(
                            serenity::all::CreateInteractionResponseMessage::default()
                                .ephemeral(true)
                                .content(perm_res.to_markdown()),
                        ),
                    )
                    .await?;

                    continue;
                }

                item.defer(&ctx.serenity_context().http).await?;

                followup_done = true;

                // Check for encryption, is so give a prompt
                let task = &backup_tasks[index];

                let mut password = None;
                if is_backup_encrypted(&task.fields) {
                    let mut password_preinput_warning = ctx.send(
                        poise::reply::CreateReply::default()
                        .content("This backup is encrypted. Please provide the password to decrypt it!")
                        .ephemeral(true)
                        .components(
                            vec![
                                serenity::all::CreateActionRow::Buttons(
                                    vec![
                                        serenity::all::CreateButton::new("backups_restore_enc_cont")
                                        .label("Continue")
                                        .style(serenity::all::ButtonStyle::Success),
                                        serenity::all::CreateButton::new("backups_restore_enc_cancel")
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

                    let password_preinp_collector = password_preinput_warning
                        .await_component_interaction(ctx.serenity_context().shard.clone())
                        .author_id(ctx.author().id)
                        .timeout(Duration::from_secs(30))
                        .await;

                    if password_preinp_collector.is_none() {
                        // Edit the message to say that the user took too long to respond
                        password_preinput_warning
                            .edit(
                                &ctx.serenity_context().http,
                                EditMessage::default().content("You took too long to respond"),
                            )
                            .await?;
                    }

                    let item = password_preinp_collector.unwrap();

                    if item.data.custom_id.as_str() == "backups_restore_enc_cancel" {
                        item.create_response(
                            &ctx.serenity_context().http,
                            serenity::all::CreateInteractionResponse::Message(
                                serenity::all::CreateInteractionResponseMessage::default()
                                    .ephemeral(true)
                                    .content("Cancelled restoration of backup"),
                            ),
                        )
                        .await?;

                        continue;
                    }

                    // Ask for password in modal
                    let password_modal = serenity::all::CreateQuickModal::new("Password")
                        .short_field("Password")
                        .timeout(std::time::Duration::from_secs(300));

                    let Some(password_modal) = item
                        .quick_modal(ctx.serenity_context(), password_modal)
                        .await?
                    else {
                        continue;
                    };

                    password = Some(password_modal.inputs[0].to_string());
                }

                // Ask for final confirmation
                let mut confirm = ctx.send(
                    poise::reply::CreateReply::default()
                    .content("Are you sure you want to restore this backup?\n\n**This action is irreversible!**")
                    .ephemeral(true)
                    .components(
                        vec![
                            serenity::all::CreateActionRow::Buttons(
                                vec![
                                    serenity::all::CreateButton::new("backups_restore_confirm")
                                    .label("Yes")
                                    .style(serenity::all::ButtonStyle::Success),
                                    serenity::all::CreateButton::new("backups_restore_cancel")
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
                    .await_component_interaction(ctx.serenity_context().shard.clone())
                    .author_id(ctx.author().id)
                    .timeout(Duration::from_secs(30))
                    .await;

                if confirm_collector.is_none() {
                    // Edit the message to say that the user took too long to respond
                    confirm
                        .edit(
                            &ctx.serenity_context().http,
                            EditMessage::default().content("You took too long to respond"),
                        )
                        .await?;
                }

                let confirm_item = confirm_collector.unwrap();

                if confirm_item.data.custom_id.as_str() == "backups_restore_cancel" {
                    confirm_item
                        .create_response(
                            &ctx.serenity_context().http,
                            serenity::all::CreateInteractionResponse::Message(
                                serenity::all::CreateInteractionResponseMessage::default()
                                    .ephemeral(true)
                                    .content("Cancelled restoration of backup"),
                            ),
                        )
                        .await?;

                    continue;
                }

                // Take out the current backup task
                let task = &backup_tasks[index];

                let url = {
                    if task.format_task_for_simplex() != format!("g/{}", guild_id) {
                        return Err("Backup task is not for this guild".into());
                    }

                    let Some(path) = task.get_file_path() else {
                        return Err("Failed to get backup path".into());
                    };

                    format!("task:///{}", path)
                };

                let mut base_message = ctx
                    .send(
                        poise::CreateReply::default().embed(
                            CreateEmbed::default()
                                .title("Restoring Backup...")
                                .description(
                                    ":yellow_circle: Please wait, starting backup task...",
                                ),
                        ),
                    )
                    .await?
                    .into_message()
                    .await?;

                let json = serde_json::json!({
                    "ServerID": guild_id.to_string(),
                    "Options": {
                        "IgnoreRestoreErrors": false,
                        "BackupSource": url,
                        "Decrypt": password.unwrap_or_default(),
                        "ChannelRestoreMode": ChannelRestoreMode::Full.to_string(),
                        "RoleRestoreMode": RoleRestoreMode::Full.to_string(),
                    },
                });

                // Restore backup
                let jobserver_cluster_id =
                    shard_id(guild_id, data.props.shard_count().await?.try_into()?);
                let resp = data
                    .reqwest
                    .post(format!(
                        "{}:{}/spawn-task",
                        config::CONFIG.base_ports.jobserver_base_addr.get(),
                        config::CONFIG.base_ports.jobserver.get() + jobserver_cluster_id
                    ))
                    .json(&splashcore_rs::jobserver::JobserverSpawnTaskRequest {
                        name: "guild_restore_backup".to_string(),
                        data: json,
                        create: true,
                        execute: true,
                        id: None,
                        user_id: ctx.author().id.to_string(),
                    })
                    .send()
                    .await
                    .map_err(|e| format!("Failed to create backup task: {}", e))?
                    .error_for_status()
                    .map_err(|e| format!("Failed to create backup task: {}", e))?;

                let restore_id = resp
                    .json::<splashcore_rs::jobserver::JobserverSpawnTaskResponse>()
                    .await?
                    .id;

                base_message
                    .edit(
                        &ctx,
                        serenity::all::EditMessage::default().embed(
                            CreateEmbed::default()
                                .title("Restoring Backup...")
                                .description(format!(
                                    ":yellow_circle: Created task with Task ID of {}",
                                    restore_id
                                )),
                        ),
                    )
                    .await?;

                let ch = botox::cache::CacheHttpImpl {
                    cache: ctx.serenity_context().cache.clone(),
                    http: ctx.serenity_context().http.clone(),
                };

                async fn update_base_message(
                    cache_http: botox::cache::CacheHttpImpl,
                    mut base_message: serenity::model::channel::Message,
                    task: Arc<jobserver::Task>,
                ) -> Result<(), Error> {
                    let new_task_msg =
                        embed_task(&config::CONFIG.sites.api.get(), &task, vec![], true)?;

                    base_message
                        .edit(
                            &cache_http,
                            new_task_msg.to_prefix_edit(serenity::all::EditMessage::default()),
                        )
                        .await?;

                    Ok(())
                }

                // Use jobserver::reactive to keep updating the message
                jobserver::taskpoll::reactive(
                    &ch,
                    &ctx.data().pool,
                    restore_id.as_str(),
                    |cache_http, task| {
                        Box::pin(update_base_message(
                            cache_http.clone(),
                            base_message.clone(),
                            task.clone(),
                        ))
                    },
                    jobserver::taskpoll::PollTaskOptions::default(),
                )
                .await?;
            }
            "backups_delete" => {
                // Check permission
                let perm_res = silverpelt::cmd::check_command(
                    &data.silverpelt_cache,
                    "backups delete",
                    guild_id,
                    ctx.author().id,
                    &ctx.data().pool,
                    &botox::cache::CacheHttpImpl::from_ctx(ctx.serenity_context()),
                    &data.reqwest,
                    &Some(ctx),
                    silverpelt::cmd::CheckCommandOptions::default(), // TODO: Maybe change this to allow backups delete to be disabled?
                )
                .await;

                if !perm_res.is_ok() {
                    item.create_response(
                        &ctx.serenity_context().http,
                        serenity::all::CreateInteractionResponse::Message(
                            serenity::all::CreateInteractionResponseMessage::default()
                                .ephemeral(true)
                                .content(perm_res.to_markdown()),
                        ),
                    )
                    .await?;

                    continue;
                }

                item.defer(&ctx.serenity_context().http).await?;

                followup_done = true;

                let mut confirm = ctx.send(
                    poise::reply::CreateReply::default()
                    .content("Are you sure you want to delete this backup?\n\n**This action is irreversible!**")
                    .ephemeral(true)
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
                    .await_component_interaction(ctx.serenity_context().shard.clone())
                    .author_id(ctx.author().id)
                    .timeout(Duration::from_secs(30))
                    .await;

                if confirm_collector.is_none() {
                    // Edit the message to say that the user took too long to respond
                    confirm
                        .edit(
                            &ctx.serenity_context().http,
                            EditMessage::default().content("You took too long to respond"),
                        )
                        .await?;
                }

                let confirm_item = confirm_collector.unwrap();

                match confirm_item.data.custom_id.as_str() {
                    "backups_delete_confirm" => {
                        // Take out the current backup task
                        let task = backup_tasks.remove(index);

                        // Respond to the interaction
                        confirm_item.create_response(
                            &ctx.serenity_context().http,
                            serenity::all::CreateInteractionResponse::Message(
                                serenity::all::CreateInteractionResponseMessage::default()
                                .ephemeral(true)
                                .embed(
                                    CreateEmbed::default()
                                    .title("Deleting Backup...")
                                    .description(":yellow_circle: Please wait while we delete this backup")
                                )
                            )
                        )
                        .await?;

                        let mut status = Vec::new();

                        match task
                            .delete_from_storage(&data.reqwest, &data.object_store)
                            .await
                        {
                            Ok(_) => {
                                status.push(":white_check_mark: Successfully deleted the backup from storage".to_string());
                            }
                            Err(e) => {
                                status.push(format!(
                                    ":x: Failed to delete the backup from storage: {}",
                                    e
                                ));
                            }
                        };

                        if let Err(e) = confirm_item
                            .edit_response(
                                &ctx.serenity_context().http,
                                serenity::all::EditInteractionResponse::default().embed(
                                    CreateEmbed::default()
                                        .title("Deleting Backup")
                                        .description(status.join("\n")),
                                ),
                            )
                            .await
                        {
                            log::error!("Failed to edit message: {}", e);
                        }

                        // Lastly deleting the task from the database
                        match task.delete_from_db(&data.pool).await {
                            Ok(_) => {
                                status.push(":white_check_mark: Successfully deleted the backup task from database".to_string());
                            }
                            Err(e) => {
                                status.push(format!(
                                    ":x: Failed to delete the backup task from database: {}",
                                    e
                                ));
                            }
                        };

                        if let Err(e) = confirm_item
                            .edit_response(
                                &ctx.serenity_context().http,
                                serenity::all::EditInteractionResponse::default().embed(
                                    CreateEmbed::default()
                                        .title("Deleting Backup")
                                        .description(status.join("\n")),
                                ),
                            )
                            .await
                        {
                            log::error!("Failed to edit message: {}", e);
                        }
                    }
                    "backups_delete_cancel" => {
                        // Respond to the interaction
                        confirm_item
                            .create_response(
                                &ctx.serenity_context().http,
                                serenity::all::CreateInteractionResponse::Message(
                                    serenity::all::CreateInteractionResponseMessage::default()
                                        .ephemeral(true)
                                        .content("Cancelled deletion of backup"),
                                ),
                            )
                            .await?;
                    }
                    _ => {
                        continue;
                    }
                }
            }
            _ => {
                continue;
            }
        }

        if index >= backup_tasks.len() {
            index = backup_tasks.len() - 1;
        }

        if !followup_done {
            item.defer(&ctx.serenity_context().http).await?;
        }

        let cr = create_reply(index, &backup_tasks)?;

        item.edit_response(
            &ctx.serenity_context().http,
            cr.to_slash_initial_response_edit(serenity::all::EditInteractionResponse::default()),
        )
        .await?;
    }

    Ok(())
}

/// Deletes a backup given its Task ID
#[poise::command(
    prefix_command,
    slash_command,
    guild_only,
    user_cooldown = "5",
    rename = "delete"
)]
pub async fn backups_delete(ctx: Context<'_>, id: String) -> Result<(), Error> {
    let task = jobserver::Task::from_id(id.parse::<Uuid>()?, &ctx.data().pool)
        .await
        .map_err(|e| format!("Failed to get backup task: {}", e))?;

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
        .await_component_interaction(ctx.serenity_context().shard.clone())
        .author_id(ctx.author().id)
        .timeout(Duration::from_secs(30))
        .await;

    if confirm_collector.is_none() {
        // Edit the message to say that the user took too long to respond
        confirm
            .edit(
                &ctx.serenity_context().http,
                EditMessage::default().content("You took too long to respond"),
            )
            .await?;
    }

    let confirm_item = confirm_collector.unwrap();

    match confirm_item.data.custom_id.as_str() {
        "backups_delete_confirm" => {
            // Respond to the interaction
            confirm_item
                .create_response(
                    &ctx.serenity_context().http,
                    serenity::all::CreateInteractionResponse::Message(
                        serenity::all::CreateInteractionResponseMessage::default().embed(
                            CreateEmbed::default()
                                .title("Deleting Backup...")
                                .description(
                                    ":yellow_circle: Please wait while we delete this backup",
                                ),
                        ),
                    ),
                )
                .await?;

            let mut status = Vec::new();

            let data = &ctx.data();
            match task
                .delete_from_storage(&data.reqwest, &data.object_store)
                .await
            {
                Ok(_) => {
                    status.push(
                        ":white_check_mark: Successfully deleted the backup from storage"
                            .to_string(),
                    );
                }
                Err(e) => {
                    status.push(format!(
                        ":x: Failed to delete the backup from storage: {}",
                        e
                    ));
                }
            };

            if let Err(e) = confirm_item
                .edit_response(
                    &ctx.serenity_context().http,
                    serenity::all::EditInteractionResponse::default().embed(
                        CreateEmbed::default()
                            .title("Deleting Backup")
                            .description(status.join("\n")),
                    ),
                )
                .await
            {
                log::error!("Failed to edit message: {}", e);
            }

            // Lastly deleting the task from the database
            match task.delete_from_db(&ctx.data().pool).await {
                Ok(_) => {
                    status.push(
                        ":white_check_mark: Successfully deleted the backup task from database"
                            .to_string(),
                    );
                }
                Err(e) => {
                    status.push(format!(
                        ":x: Failed to delete the backup task from database: {}",
                        e
                    ));
                }
            };

            if let Err(e) = confirm_item
                .edit_response(
                    &ctx.serenity_context().http,
                    serenity::all::EditInteractionResponse::default().embed(
                        CreateEmbed::default()
                            .title("Deleting Backup")
                            .description(status.join("\n")),
                    ),
                )
                .await
            {
                log::error!("Failed to edit message: {}", e);
            }
        }
        "backups_delete_cancel" => {
            ctx.say("Cancelled deletion of backup").await?;
        }
        _ => {
            return Err("Invalid interaction".into());
        }
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

/// Restores a created backup. Either backup_file or backup_id must be provided
#[poise::command(
    prefix_command,
    slash_command,
    guild_only,
    user_cooldown = "20",
    guild_cooldown = "30",
    rename = "restore"
)]
#[allow(clippy::too_many_arguments)] // This function needs these arguments due to poise
pub async fn backups_restore(
    ctx: Context<'_>,

    #[description = "The backup attachment to restore"] backup_file: Option<
        serenity::all::Attachment,
    >,

    #[description = "The task id of the backup to restore"] backup_id: Option<String>,

    #[description = "Password to decrypt backup with. Should not be reused"] password: Option<
        String,
    >,

    #[description = "Channel restore mode. Defaults to full. Use 'full' if unsure"]
    channel_restore_mode: Option<ChannelRestoreMode>,

    #[description = "Role restore mode. Defaults to full. Use 'full' if unsure"]
    role_restore_mode: Option<RoleRestoreMode>,

    #[description = "Channels to protect from being deleted, comma seperated"]
    protected_channels: Option<String>,

    #[description = "Roles to protect from being deleted, comma seperated"] protected_roles: Option<
        String,
    >,

    #[description = "Whether to ignore errors while restoring or not"]
    ignore_restore_errors: Option<bool>,
) -> Result<(), Error> {
    let data = ctx.data();

    let Some(guild_id) = ctx.guild_id() else {
        return Err("This command can only be used in a guild".into());
    };

    if backup_file.is_some() && backup_id.is_some() {
        return Err("You can only provide either a backup file or a backup id".into());
    }

    if backup_file.is_none() && backup_id.is_none() {
        return Err("You must provide either a backup file or a backup id".into());
    }

    let backup_url = {
        if let Some(backup_file) = backup_file {
            backup_file.url.to_string()
        } else {
            let Some(backup_id) = backup_id else {
                return Err("Failed to get backup id".into());
            };

            // Get the task
            let task = jobserver::Task::from_id(backup_id.parse::<Uuid>()?, &ctx.data().pool)
                .await
                .map_err(|e| format!("Failed to get backup task: {}", e))?;

            if task.format_task_for_simplex() != format!("g/{}", guild_id) {
                return Err("Backup task is not for this guild".into());
            }

            let Some(path) = task.get_file_path() else {
                return Err("Failed to get backup path".into());
            };

            format!("task:///{}", path)
        }
    };

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

    let mut base_message = ctx
        .send(
            poise::CreateReply::default().embed(
                CreateEmbed::default()
                    .title("Restoring Backup...")
                    .description(":yellow_circle: Please wait, starting backup task..."),
            ),
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
            "BackupSource": backup_url,
            "Decrypt": password.unwrap_or_default(),
            "ChannelRestoreMode": channel_restore_mode.unwrap_or(ChannelRestoreMode::Full).to_string(),
            "RoleRestoreMode": role_restore_mode.unwrap_or(RoleRestoreMode::Full).to_string(),
        },
    });

    // Restore backup
    let jobserver_cluster_id = shard_id(guild_id, data.props.shard_count().await?.try_into()?);
    let resp = data
        .reqwest
        .post(format!(
            "{}:{}/spawn-task",
            config::CONFIG.base_ports.jobserver_base_addr.get(),
            config::CONFIG.base_ports.jobserver.get() + jobserver_cluster_id
        ))
        .json(&splashcore_rs::jobserver::JobserverSpawnTaskRequest {
            name: "guild_restore_backup".to_string(),
            data: json,
            create: true,
            execute: true,
            id: None,
            user_id: ctx.author().id.to_string(),
        })
        .send()
        .await
        .map_err(|e| format!("Failed to create backup task: {}", e))?
        .error_for_status()
        .map_err(|e| format!("Failed to create backup task: {}", e))?;

    let restore_id = resp
        .json::<splashcore_rs::jobserver::JobserverSpawnTaskResponse>()
        .await?
        .id;

    base_message
        .edit(
            &ctx,
            serenity::all::EditMessage::default().embed(
                CreateEmbed::default()
                    .title("Restoring Backup...")
                    .description(format!(
                        ":yellow_circle: Created task with Task ID of {}",
                        restore_id
                    )),
            ),
        )
        .await?;

    let ch = botox::cache::CacheHttpImpl {
        cache: ctx.serenity_context().cache.clone(),
        http: ctx.serenity_context().http.clone(),
    };

    async fn update_base_message(
        cache_http: botox::cache::CacheHttpImpl,
        mut base_message: serenity::model::channel::Message,
        task: Arc<jobserver::Task>,
    ) -> Result<(), Error> {
        let new_task_msg = embed_task(&config::CONFIG.sites.api.get(), &task, vec![], true)?;

        base_message
            .edit(
                &cache_http,
                new_task_msg.to_prefix_edit(serenity::all::EditMessage::default()),
            )
            .await?;

        Ok(())
    }

    // Use jobserver::reactive to keep updating the message
    jobserver::taskpoll::reactive(
        &ch,
        &ctx.data().pool,
        restore_id.as_str(),
        |cache_http, task| {
            Box::pin(update_base_message(
                cache_http.clone(),
                base_message.clone(),
                task.clone(),
            ))
        },
        jobserver::taskpoll::PollTaskOptions::default(),
    )
    .await?;

    Ok(())
}
