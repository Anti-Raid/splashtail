use std::fmt::Display;
use std::sync::Arc;
use axum::response::{IntoResponse, Response};
use axum::routing::post;
use axum::{extract::State, http::StatusCode, Router};
use strum_macros::{Display, EnumVariantNames};
use axum::extract::DefaultBodyLimit;
use log::{info, error};
use serenity::all::GuildId;
use axum::Json;
use serde::{Deserialize, Serialize};

struct Error {
    status: StatusCode,
    message: String,
}

#[allow(dead_code)]
impl Error {
    fn new(e: impl Display) -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            message: e.to_string(),
        }
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        (self.status, self.message).into_response()
    }
}

pub struct AppState {
    pub cache_http: crate::impls::cache::CacheHttpImpl,
    pub shard_manager: Arc<serenity::all::ShardManager>
}

pub async fn start_iserver(
    serenity_cache: &crate::impls::cache::CacheHttpImpl,
    shard_manager: &Arc<serenity::all::ShardManager>
) -> ! {
    let app_state = Arc::new(AppState {
        cache_http: serenity_cache.clone(),
        shard_manager: shard_manager.clone()
    });

    let cluster = crate::ipc::argparse::MEWLD_ARGS.cluster_id;
    let base_port = crate::config::CONFIG.meta.bot_iserver_base_port.get();

    // Ensure we can bind without overflow
    if cluster > (65535 - base_port) {
        panic!("Cluster ID is too high! ({} > {})", cluster, 65535 - base_port);
    }

    let port = base_port + cluster;

    let app = Router::new()
        .route("/", post(query))
        .with_state(app_state)
        .layer(DefaultBodyLimit::max(1048576000));

    let addr = format!("127.0.0.1:{}", port)
        .parse()
        .expect("Invalid IServer address");

    info!("Starting IServer server on {} for cluster {}", addr, cluster);

    if let Err(e) = axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
    {
        panic!("IServer server error: {}", e);
    }

    error!("IServer server exited unexpectedly (port={}, cluster={}", port, cluster);
    unreachable!();
}

#[derive(Serialize, Deserialize, Display, Clone, EnumVariantNames)]
pub enum IServerQuery {
    /// Given a list of guild IDs, return whether or not they exist on the bot
    GuildsExist {
        guilds: Vec<GuildId>,
    },

    /// Returns the list of modules on the bot
    Modules {},
}

#[axum_macros::debug_handler]
async fn query(
    State(state): State<Arc<AppState>>,
    Json(req): Json<IServerQuery>,
) -> Result<impl IntoResponse, Error> {
    match req {
        IServerQuery::GuildsExist { guilds } => {
            let mut guilds_exist = Vec::with_capacity(guilds.len());

            for guild in guilds {
                guilds_exist.push({
                    if state.cache_http.cache.guild(guild).is_some() {
                        1
                    } else {
                        0
                    }
                });
            }

            Ok(Json(guilds_exist).into_response())
        },
        IServerQuery::Modules {} => {
            let mut modules = indexmap::IndexMap::new();

            for (id, module) in crate::silverpelt::CANONICAL_MODULE_CACHE.iter() {
                modules.insert(id.to_string(), module);
            }

            Ok(Json(modules).into_response())
        }
    }
}