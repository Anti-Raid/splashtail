use axum::{http::Request, routing::get, Router};
use hyper::body::Incoming;
use hyper_util::{
    rt::{TokioExecutor, TokioIo},
    server,
};
use std::{convert::Infallible, path::PathBuf, sync::Arc};
use tokio::net::UnixListener;
use tower_service::Service;

#[derive(Debug, Clone)]
pub enum CreateRpcServerBind {
    /// Bind to a specific address
    Address(String),
    /// Bind to a unix socket
    #[cfg(unix)]
    UnixSocket(String),
}

#[derive(Debug, Clone)]
pub struct CreateRpcServerOptions {
    /// The bind address for the RPC server
    pub bind: CreateRpcServerBind,
}

#[derive(Clone)]
pub struct AppData {
    pub data: Arc<silverpelt::data::Data>,
    pub cache_http: Arc<botox::cache::CacheHttpImpl>,
    pub serenity_context: serenity::all::Context,
}

impl AppData {
    pub fn new(data: Arc<silverpelt::data::Data>, ctx: &serenity::all::Context) -> Self {
        Self {
            data,
            serenity_context: ctx.clone(),
            cache_http: Arc::new(botox::cache::CacheHttpImpl::from_ctx(ctx)),
        }
    }
}

pub fn create_blank_rpc_server() -> Router<AppData> {
    Router::new()
        .layer(tower_http::trace::TraceLayer::new_for_http())
        .route("/", get(|| async { "bot" }))
}

pub async fn start_rpc_server(
    opts: CreateRpcServerOptions,
    mut make_service: axum::routing::IntoMakeService<Router>,
) -> Result<(), silverpelt::Error> {
    match opts.bind {
        CreateRpcServerBind::Address(addr) => {
            let listener = tokio::net::TcpListener::bind(addr).await?;

            log::info!("Listening on {}", listener.local_addr()?);

            loop {
                let (socket, _remote_addr) = match listener.accept().await {
                    Ok(ok) => ok,
                    Err(err) => {
                        log::error!("failed to accept connection: {err:#}");
                        continue;
                    }
                };

                let tower_service = unwrap_infallible(make_service.call(&socket).await);

                tokio::spawn(async move {
                    let socket = TokioIo::new(socket);

                    let hyper_service =
                        hyper::service::service_fn(move |request: Request<Incoming>| {
                            tower_service.clone().call(request)
                        });

                    if let Err(err) = server::conn::auto::Builder::new(TokioExecutor::new())
                        .serve_connection_with_upgrades(socket, hyper_service)
                        .await
                    {
                        log::error!("failed to serve connection: {err:#}");
                    }
                });
            }
        }
        #[cfg(unix)]
        CreateRpcServerBind::UnixSocket(path) => {
            let path = PathBuf::from(path);

            let _ = tokio::fs::remove_file(&path).await;

            tokio::fs::create_dir_all(path.parent().unwrap()).await?;

            let uds = UnixListener::bind(path.clone()).unwrap();

            loop {
                let (socket, _remote_addr) = match uds.accept().await {
                    Ok(ok) => ok,
                    Err(err) => {
                        log::error!("failed to accept connection: {err:#}");
                        continue;
                    }
                };

                let tower_service = unwrap_infallible(make_service.call(&socket).await);

                tokio::spawn(async move {
                    let socket = TokioIo::new(socket);

                    let hyper_service =
                        hyper::service::service_fn(move |request: Request<Incoming>| {
                            tower_service.clone().call(request)
                        });

                    if let Err(err) = server::conn::auto::Builder::new(TokioExecutor::new())
                        .serve_connection_with_upgrades(socket, hyper_service)
                        .await
                    {
                        log::error!("failed to serve connection: {err:#}");
                    }
                });
            }
        }
    }
}

fn unwrap_infallible<T>(result: Result<T, Infallible>) -> T {
    match result {
        Ok(value) => value,
        #[allow(unreachable_patterns)]
        Err(never) => match never {},
    }
}
