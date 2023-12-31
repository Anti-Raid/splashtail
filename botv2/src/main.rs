mod impls;
mod cmds;
mod config;
mod tasks;
mod modules;
mod ipc;

use std::sync::Arc;

use log::{error, info};
use poise::serenity_prelude::FullEvent;
use poise::CreateReply;
use sqlx::postgres::PgPoolOptions;
use std::io::Write;

use crate::impls::cache::CacheHttpImpl;

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

// User data, which is stored and accessible in all command invocations
pub struct Data {
    pub pool: sqlx::PgPool,
    pub ipc: Arc<ipc::client::IpcClient>,
    pub mewld_args: Arc<crate::ipc::argparse::MewldCmdArgs>,
    pub cache_http: crate::impls::cache::CacheHttpImpl,
    pub shards_ready: Arc<dashmap::DashMap<u16, bool>>,
}

#[poise::command(prefix_command)]
async fn register(ctx: Context<'_>) -> Result<(), Error> {
    poise::builtins::register_application_commands_buttons(ctx).await?;
    Ok(())
}

async fn on_error(error: poise::FrameworkError<'_, Data, Error>) {
    // This is our custom error handler
    // They are many errors that can occur, so we only handle the ones we want to customize
    // and forward the rest to the default handler
    match error {
        poise::FrameworkError::Setup { error, .. } => panic!("Failed to start bot: {:?}", error),
        poise::FrameworkError::Command { error, ctx, .. } => {
            error!("Error in command `{}`: {:?}", ctx.command().name, error,);
            let err = ctx
                .say(format!(
                    "There was an error running this command: {}",
                    error
                ))
                .await;

            if let Err(e) = err {
                error!("SQLX Error: {}", e);
            }
        }
        poise::FrameworkError::CommandCheckFailed { error, ctx, .. } => {
            error!(
                "[Possible] error in command `{}`: {:?}",
                ctx.command().name,
                error,
            );
            if let Some(error) = error {
                error!("Error in command `{}`: {:?}", ctx.command().name, error,);
                let err = ctx.say(format!("**{}**", error)).await;

                if let Err(e) = err {
                    error!("Error while sending error message: {}", e);
                }
            }
        }
        error => {
            if let Err(e) = poise::builtins::on_error(error).await {
                error!("Error while handling error: {}", e);
            }
        }
    }
}

async fn event_listener(ctx: &serenity::client::Context, event: &FullEvent, user_data: &Data) -> Result<(), Error> {
    match event {
        FullEvent::InteractionCreate {
            interaction,
        } => {
            info!("Interaction received: {:?}", interaction.id());

            let ic = match interaction {
                serenity::all::Interaction::Command(ic) => ic,
                _ => return Ok(()),
            };
                 
            if !config::CONFIG.discord_auth.can_use_bot.contains(&ic.user.id) {
                let primary = poise::serenity_prelude::CreateEmbed::default()
                    .color(0xff0000)
                    .title("AntiRaid")
                    .url("https://discord.gg/Qa52e2bNms")
                    .description("Unfortunately, AntiRaid is currently unavailable due to poor code management and changes with the Discord API. We are currently in the works of V6, and hope to have it out by next month. All use of our services will not be available, and updates will be pushed here. We are extremely sorry for the inconvenience.\nFor more information you can also join our [Support Server](https://discord.gg/Qa52e2bNms)!");

                let changes = ["We are working extremely hard on Antiraid v6, and have completed working on half of the bot. We should have this update out by Q1/Q2 2024! Delays may occur due to the sheer scope of the unique features we want to provide!"];

                let updates = poise::serenity_prelude::CreateEmbed::default()
                    .color(0x0000ff)
                    .title("Updates")
                    .description(changes.join("\t-"));
    
                let statistics = poise::serenity_prelude::CreateEmbed::default()
                    .color(0xff0000)
                    .description(format!(
                        "**Server Count:** {}\n**Shard Count:** {}\n**Cluster Count:** {}\n**Cluster ID:** {}\n**Cluster Name:** {}\n**Uptime:** {}",
                        user_data.ipc.cache.total_guilds(),
                        user_data.ipc.mewld_args.shard_count,
                        user_data.ipc.mewld_args.cluster_count,
                        user_data.ipc.mewld_args.cluster_id,
                        user_data.ipc.mewld_args.cluster_name,
                        {
                            let duration: std::time::Duration = std::time::Duration::from_secs((chrono::Utc::now().timestamp() - crate::config::CONFIG.bot_start_time) as u64);
        
                            let seconds = duration.as_secs() % 60;
                            let minutes = (duration.as_secs() / 60) % 60;
                            let hours = (duration.as_secs() / 60) / 60;
        
                            format!("{}h{}m{}s", hours, minutes, seconds)
                        }
                    ));

                ic.create_response(
                    &ctx,
                    serenity::all::CreateInteractionResponse::Message(
                        serenity::all::CreateInteractionResponseMessage::default()
                        .flags(serenity::all::InteractionResponseFlags::EPHEMERAL)
                        .add_embed(primary)
                        .add_embed(updates)
                        .add_embed(statistics)
                    )
                )
                .await
                .map_err(|e| format!("Error sending reply: {}", e))?;
            }
        }
        FullEvent::Ready {
            data_about_bot,
        } => {
            info!("{} is ready on shard {}", data_about_bot.user.name, ctx.shard_id);
            
            tokio::task::spawn(crate::tasks::taskcat::start_all_tasks(
                user_data.pool.clone(),
                user_data.cache_http.clone(),
                ctx.clone(),
            ));

            if ctx.shard_id.0 == *user_data.mewld_args.shards.last().unwrap() {
                info!("All shards ready, launching next cluster");
                if let Err(e) = user_data.ipc.publish_ipc_launch_next().await {
                    error!("Error publishing IPC launch next: {}", e);
                    return Err(e);
                }

                user_data.shards_ready.insert(ctx.shard_id.0, true);

                info!("Published IPC launch next to channel {}", user_data.mewld_args.mewld_redis_channel);
            }
        }
        _ => {}
    }

    // Add all event listeners for key modules here
    if let Err(e) = crate::modules::limits::events::event_listener(ctx, event, user_data).await {
        error!("Error in limits event listener: {}", e);
    }

    Ok(())
}

#[tokio::main]
async fn main() {
    const POSTGRES_MAX_CONNECTIONS: u32 = 3; // max connections to the database, we don't need too many here
    const REDIS_MAX_CONNECTIONS: u32 = 10; // max connections to the redis

    let args = std::env::args().collect::<Vec<_>>();
    let mewld_args = Arc::new(crate::ipc::argparse::MewldCmdArgs::parse_argv(&args).unwrap());

    // Setup logging
    let cluster_id = mewld_args.cluster_id;
    let cluster_name = mewld_args.cluster_name.clone();
    let cluster_count = mewld_args.cluster_count;
    let shards = mewld_args.shards.clone();
    let shard_count = mewld_args.shard_count;
    env_logger::builder()
    .format(move |buf, record| writeln!(buf, "[{} ({} of {})] {} - {}", cluster_name, cluster_id, cluster_count-1, record.level(), record.args()))
    .filter(Some("botv2"), log::LevelFilter::Info)
    .filter(None, log::LevelFilter::Error)
    .init();

    info!("{:#?}", mewld_args);

    let proxy_url = config::CONFIG.meta.proxy.clone();

    info!("Proxy URL: {}", proxy_url);

    let http = serenity::all::HttpBuilder::new(&config::CONFIG.discord_auth.token)
        .proxy(proxy_url)
        .ratelimiter_disabled(true)
        .build();

    let mut intents = serenity::all::GatewayIntents::all();

    // The really spammy intents
    intents.remove(serenity::all::GatewayIntents::GUILD_PRESENCES); // Don't even have the privileged gateway intent for this
    intents.remove(serenity::all::GatewayIntents::GUILD_MESSAGE_TYPING); // Don't care about typing
    intents.remove(serenity::all::GatewayIntents::DIRECT_MESSAGE_TYPING); // Don't care about typing
    intents.remove(serenity::all::GatewayIntents::DIRECT_MESSAGES); // Don't care about DMs

    let client_builder = serenity::all::ClientBuilder::new_with_http(
        http,
        intents,
    );

    let framework = poise::Framework::new(
        poise::FrameworkOptions {
            initialize_owners: true,
            prefix_options: poise::PrefixFrameworkOptions {
                prefix: Some("%".into()),
                ..poise::PrefixFrameworkOptions::default()
            },
            event_handler: |ctx, event, _fc, user_data| Box::pin(event_listener(ctx, event, user_data)),
            commands: {
                let mut cmds = vec![
                    register(),
                ];

                cmds.extend(cmds::core::commands()); // Core plugin
                cmds.extend(cmds::limits::commands()); // Limits plugin

                cmds
            },
            command_check: Some(|ctx| {
                Box::pin(async move {
                    if !config::CONFIG.discord_auth.can_use_bot.contains(&ctx.author().id) {
                        // We already send in the event handler
                        if let poise::Context::Application(_) = ctx { 
                            return Ok(false) 
                        }

                        let data = ctx.data();
                        let primary = poise::serenity_prelude::CreateEmbed::default()
                            .color(0xff0000)
                            .title("AntiRaid")
                            .url("https://discord.gg/Qa52e2bNms")
                            .description("Unfortunately, AntiRaid is currently unavailable due to poor code management and changes with the Discord API. We are currently in the works of V6, and hope to have it out by next month. All use of our services will not be available, and updates will be pushed here. We are extremely sorry for the inconvenience.\nFor more information you can also join our [Support Server](https://discord.gg/Qa52e2bNms)!");

                        let changes = ["We are working extremely hard on Antiraid v6, and have completed working on half of the bot. We should have this update out by Q1/Q2 2024! Delays may occur due to the sheer scope of the unique features we want to provide!"];

                        let updates = poise::serenity_prelude::CreateEmbed::default()
                            .color(0x0000ff)
                            .title("Updates")
                            .description(changes.join("\t-"));

                        let statistics = poise::serenity_prelude::CreateEmbed::default()
                            .color(0xff0000)
                            .description(format!(
                                "**Server Count:** {}\n**Shard Count:** {}\n**Cluster Count:** {}\n**Cluster ID:** {}\n**Cluster Name:** {}\n**Uptime:** {}",
                                data.ipc.cache.total_guilds(),
                                data.ipc.mewld_args.shard_count,
                                data.ipc.mewld_args.cluster_count,
                                data.ipc.mewld_args.cluster_id,
                                data.ipc.mewld_args.cluster_name,
                                {
                                    let duration: std::time::Duration = std::time::Duration::from_secs((chrono::Utc::now().timestamp() - crate::config::CONFIG.bot_start_time) as u64);
                
                                    let seconds = duration.as_secs() % 60;
                                    let minutes = (duration.as_secs() / 60) % 60;
                                    let hours = (duration.as_secs() / 60) / 60;
                
                                    format!("{}h{}m{}s", hours, minutes, seconds)
                                }
                            ));

                        ctx.send(
                            CreateReply::default()
                            .embed(primary)
                            .embed(updates)
                            .embed(statistics)
                        )
                        .await
                        .map_err(|e| format!("Error sending reply: {}", e))?;

                        return Ok(false)
                    }

                    // Look for guild
                    if let Some(guild_id) = ctx.guild_id() {
                        if ["register", "setup"].contains(&ctx.command().name.as_str()) {
                            return Ok(true);
                        }

                        let data = ctx.data();

                        let guild = sqlx::query!(
                            "SELECT COUNT(*) FROM guilds WHERE id = $1",
                            guild_id.to_string()
                        )
                        .fetch_one(&data.pool)
                        .await?;

                        if guild.count.unwrap_or_default() == 0 {
                            // Guild not found, create it
                            sqlx::query!(
                                "INSERT INTO guilds (id) VALUES ($1)", 
                                guild_id.to_string()
                            )
                            .execute(&data.pool)
                            .await?;
                        }

                        Ok(true)
                    } else {
                        Err("This command can only be run from servers".into())
                    }
                })
            }),
            pre_command: |ctx| {
                Box::pin(async move {
                    info!(
                        "Executing command {} for user {} ({})...",
                        ctx.command().qualified_name,
                        ctx.author().name,
                        ctx.author().id
                    );
                })
            },
            post_command: |ctx| {
                Box::pin(async move {
                    info!(
                        "Done executing command {} for user {} ({})...",
                        ctx.command().qualified_name,
                        ctx.author().name,
                        ctx.author().id
                    );
                })
            },
            on_error: |error| Box::pin(on_error(error)),
            ..Default::default()
        },
        move |ctx, _ready, framework| {
            Box::pin(async move {
                info!("Initializing data for shard {}", ctx.shard_id);

                let data = Data {
                    cache_http: CacheHttpImpl {
                        cache: ctx.cache.clone(),
                        http: ctx.http.clone(),
                    },
                    ipc: Arc::new(crate::ipc::client::IpcClient {
                        redis_pool: fred::prelude::Builder::default_centralized()
                            .build_pool(REDIS_MAX_CONNECTIONS.try_into().unwrap())
                            .expect("Could not initialize Redis pool"),
                        shard_manager: framework.shard_manager().clone(),
                        mewld_args: mewld_args.clone(),
                        serenity_cache: CacheHttpImpl {
                            cache: ctx.cache.clone(),
                            http: ctx.http.clone(),
                        },
                        cache: Arc::new(crate::ipc::client::IpcCache::default()),
                    }),
                    mewld_args: mewld_args.clone(),
                    pool: PgPoolOptions::new()
                        .max_connections(POSTGRES_MAX_CONNECTIONS)
                        .connect(&config::CONFIG.meta.postgres_url)
                        .await
                        .expect("Could not initialize connection"),
                    shards_ready: Arc::new(dashmap::DashMap::new()),
                };

                let ipc_ref = data.ipc.clone();
                tokio::task::spawn(async move {
                    let ipc_ref = ipc_ref;
                    ipc_ref.start_ipc_listener().await;
                });

                Ok(data)
            })
        },
    );

    let mut client = client_builder
        .framework(framework)
        .await
        .expect("Error creating client");

    let shard_range = std::ops::Range {
        start: shards[0],
        end: *shards.last().unwrap(),
    };

    info!("Starting shard range: {:?}", shard_range);

    if let Err(why) = client.start_shard_range(shard_range, shard_count).await {
        error!("Client error: {:?}", why);
    }

    std::process::exit(1); // Clean exit with status code of 1
}
