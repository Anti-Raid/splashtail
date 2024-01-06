mod impls;
mod cmds;
mod config;
mod tasks;
mod modules;

use std::collections::HashSet;

use log::{error, info};
use poise::serenity_prelude::{FullEvent, RoleAction, UserId};
use poise::CreateReply;
use serenity::model::guild::audit_log::{Action, ChannelAction};
use sqlx::postgres::PgPoolOptions;

use crate::impls::cache::CacheHttpImpl;

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

// User data, which is stored and accessible in all command invocations
pub struct Data {
    pool: sqlx::PgPool,
    cache_http: crate::impls::cache::CacheHttpImpl,
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
        }
        FullEvent::Ready {
            data_about_bot,
        } => {
            info!("{} is ready!", data_about_bot.user.name);

            tokio::task::spawn(crate::tasks::taskcat::start_all_tasks(
                user_data.pool.clone(),
                user_data.cache_http.clone(),
                ctx.clone(),
            ));
        }
        FullEvent::GuildAuditLogEntryCreate {
            entry,
            guild_id,
        } => {
            info!("Audit log created: {:?}. Guild: {}", entry, guild_id);

            let res = match entry.action {
                Action::Channel(ch) => {
                    let ch_id = entry.target_id.ok_or("No channel ID found")?;

                    match ch {
                        ChannelAction::Create => {
                            info!("Channel created: {}", ch_id);

                            modules::limits::handler::handle_mod_action(
                                *guild_id,
                                entry.user_id,
                                &user_data.pool,
                                &user_data.cache_http,
                                modules::limits::core::UserLimitTypes::ChannelAdd,
                                ch_id.to_string(),
                            )
                            .await
                        }
                        ChannelAction::Delete => {
                            info!("Channel deleted: {}", ch_id);

                            modules::limits::handler::handle_mod_action(
                                *guild_id,
                                entry.user_id,
                                &user_data.pool,
                                &user_data.cache_http,
                                modules::limits::core::UserLimitTypes::ChannelRemove,
                                ch_id.to_string(),
                            )
                            .await
                        }
                        ChannelAction::Update => {
                            info!("Channel updated: {}", ch_id);

                            modules::limits::handler::handle_mod_action(
                                *guild_id,
                                entry.user_id,
                                &user_data.pool,
                                &user_data.cache_http,
                                modules::limits::core::UserLimitTypes::ChannelUpdate,
                                ch_id.to_string(),
                            )
                            .await
                        }
                        _ => Ok(()),
                    }
                }
                Action::Role(ra) => {
                    let r_id = entry.target_id.ok_or("No role ID found")?;

                    match ra {
                        RoleAction::Create => {
                            info!("Role created: {}", r_id);

                            modules::limits::handler::handle_mod_action(
                                *guild_id,
                                entry.user_id,
                                &user_data.pool,
                                &user_data.cache_http,
                                modules::limits::core::UserLimitTypes::RoleAdd,
                                r_id.to_string(),
                            )
                            .await
                        }
                        RoleAction::Update => {
                            info!("Role updated: {}", r_id);

                            modules::limits::handler::handle_mod_action(
                                *guild_id,
                                entry.user_id,
                                &user_data.pool,
                                &user_data.cache_http,
                                modules::limits::core::UserLimitTypes::RoleUpdate,
                                r_id.to_string(),
                            )
                            .await
                        }
                        RoleAction::Delete => {
                            info!("Role deleted: {}", r_id);

                            modules::limits::handler::handle_mod_action(
                                *guild_id,
                                entry.user_id,
                                &user_data.pool,
                                &user_data.cache_http,
                                modules::limits::core::UserLimitTypes::RoleRemove,
                                r_id.to_string(),
                            )
                            .await
                        }
                        _ => Ok(()),
                    }
                }
                _ => Ok(()),
            };

            if let Err(res) = res {
                error!("Error while handling audit log: {}", res);
                return Err(res);
            }
        }
        _ => {}
    }

    Ok(())
}

#[tokio::main]
async fn main() {
    const MAX_CONNECTIONS: u32 = 3; // max connections to the database, we don't need too many here

    std::env::set_var("RUST_LOG", "botv2=info");

    env_logger::init();

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
                    cmds::core::help::help(),
                    cmds::core::help::simplehelp(),
                    cmds::core::stats::stats(),
                    cmds::core::ping::ping(),
                ];

                cmds.extend(cmds::limits::commands());

                cmds
            },
            command_check: Some(|ctx| {
                Box::pin(async move {
                    if !config::CONFIG.discord_auth.can_use_bot.contains(&ctx.author().id) {
                   /*
const primary = new EmbedBuilder()
                .setColor("Red")
                .setTitle("AntiRaid")
                .setURL("https://discord.gg/Qa52e2bNms")
                .setDescription("Unfortunately, AntiRaid is currently unavailable due to poor code management and changes with the Discord API. We are currently in the works of V6, and hope to have it out by next month. All use of our services will not be available, and updates will be pushed here. We are extremely sorry for the inconvenience.\nFor more information you can also join our [Support Server](https://discord.gg/Qa52e2bNms)!")

                const changes = ["We are working extremely hard on Antiraid v6, and have completed working on almost half of the bot. We should have this update out by January 5th, 2024."];
                const updates = new EmbedBuilder()
                .setColor("Blue")
                .setTitle("Updates")
                .setDescription(changes.join("\t-"))

                let guildCount = await getServerCount(this)
                let shardCount = await getShardCount(this)

                const statistics = new EmbedBuilder()
                .setColor("Red")
                .setDescription((`**Server Count:** ${guildCount}\n**Shard Count:** ${shardCount}\n**Cluster Count:** ${this.clusterCount}\n**Cluster ID:** ${this.clusterId}\n**Cluster Name:** ${this.clusterName}\n**Uptime:** ${uptimeToHuman(this.uptime)}`))

                ctx.reply({
                    embeds: [primary, updates, statistics]
                })

                return
            }
                     */

                        let primary = poise::serenity_prelude::CreateEmbed::default()
                            .color(0xff0000)
                            .title("AntiRaid")
                            .url("https://discord.gg/Qa52e2bNms")
                            .description("Unfortunately, AntiRaid is currently unavailable due to poor code management and changes with the Discord API. We are currently in the works of V6, and hope to have it out by next month. All use of our services will not be available, and updates will be pushed here. We are extremely sorry for the inconvenience.\nFor more information you can also join our [Support Server](https://discord.gg/Qa52e2bNms)!");

                        let changes = vec!["We are working extremely hard on Antiraid v6, and have completed working on half of the bot. We should have this update out by Q1/Q2 2024! Delays may occur due to the sheer scope of the unique features we want to provide!"];

                        let updates = poise::serenity_prelude::CreateEmbed::default()
                            .color(0x0000ff)
                            .title("Updates")
                            .description(changes.join("\t-"));

                        ctx.send(
                            CreateReply::default()
                            .embed(primary)
                            .embed(updates)
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
        move |ctx, _ready, _framework| {
            Box::pin(async move {
                Ok(Data {
                    cache_http: CacheHttpImpl {
                        cache: ctx.cache.clone(),
                        http: ctx.http.clone(),
                    },
                    pool: PgPoolOptions::new()
                        .max_connections(MAX_CONNECTIONS)
                        .connect(&config::CONFIG.meta.postgres_url)
                        .await
                        .expect("Could not initialize connection"),
                })
            })
        },
    );

    let mut client = client_builder
        .framework(framework)
        .await
        .expect("Error creating client");

    if let Err(why) = client.start().await {
        error!("Client error: {:?}", why);
    }
}
