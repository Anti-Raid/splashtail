use log::{error, info};
use poise::serenity_prelude::{self as serenity, FullEvent};
use sqlx::postgres::PgPoolOptions;

use crate::impls::cache::CacheHttpImpl;

mod admin;
mod checks;
mod config;
mod help;
mod impls;
mod panelapi;
mod rpc;
mod rpc_command;
mod staff;
mod stats;
mod tasks;
mod test;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;
// User data, which is stored and accessible in all command invocations
pub struct Data {
    pool: sqlx::PgPool,
    cache_http: CacheHttpImpl,
}

/// Displays your or another user's account creation date
#[poise::command(slash_command, prefix_command)]
async fn age(
    ctx: Context<'_>,
    #[description = "Selected user"] user: Option<serenity::User>,
) -> Result<(), Error> {
    let u = user.as_ref().unwrap_or_else(|| ctx.author());
    let response = format!("{}'s account was created at {}", u.name, u.created_at());
    ctx.say(response).await?;
    Ok(())
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
                let err = ctx
                    .say(format!(
                        "Whoa there, do you have permission to do this?: {}",
                        error
                    ))
                    .await;

                if let Err(e) = err {
                    error!("Error while sending error message: {}", e);
                }
            } else {
                let err = ctx
                    .say("You don't have permission to do this but we couldn't figure out why...")
                    .await;

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

async fn event_listener(event: &FullEvent, user_data: &Data) -> Result<(), Error> {
    match event {
        FullEvent::InteractionCreate {
            interaction,
        } => {
            info!("Interaction received: {:?}", interaction.id());
        }
        FullEvent::CacheReady { guilds } => {
            info!("Cache ready with {} guilds", guilds.len());
        }
        FullEvent::Ready {
            data_about_bot,
        } => {
            info!(
                "{} is ready! Doing some minor DB fixes",
                data_about_bot.user.name
            );

            // Start RPC
            tokio::task::spawn(panelapi::server::init_panelapi(
                user_data.pool.clone(),
                user_data.cache_http.clone(),
            ));

            tokio::task::spawn(crate::tasks::taskcat::start_all_tasks(
                user_data.pool.clone(),
                user_data.cache_http.clone(),
            ));
        },
        _ => {}
    }

    Ok(())
}

#[tokio::main]
async fn main() {
    const MAX_CONNECTIONS: u32 = 6; // max connections to the database, we don't need too many here

    std::env::set_var("RUST_LOG", "bot=info, moka=error");

    env_logger::init();

    info!("Proxy URL: {}", config::CONFIG.proxy_url);

    let http = serenity::HttpBuilder::new(&config::CONFIG.token)
        .proxy(config::CONFIG.proxy_url.clone())
        .ratelimiter_disabled(true)
        .build();

    let client_builder =
        serenity::ClientBuilder::new_with_http(http, serenity::GatewayIntents::all());

    let framework = poise::Framework::new(
        poise::FrameworkOptions {
            initialize_owners: true,
            prefix_options: poise::PrefixFrameworkOptions {
                prefix: Some("ibb!".into()),
                ..poise::PrefixFrameworkOptions::default()
            },
            event_handler: |_ctx, event, _fc, user_data| Box::pin(event_listener(event, user_data)),
            commands: vec![
                age(),
                register(),
                help::simplehelp(),
                help::help(),
                staff::staff(),
                stats::stats(),
                rpc_command::rpc(),
                rpc_command::rpclist(),
                test::modaltest(),
            ],
            // This code is run before every command
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
            // This code is run after every command returns Ok
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
                        .connect(&config::CONFIG.database_url)
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
