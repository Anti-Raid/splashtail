/// This test ensures that all modules can be parsed
#[cfg(test)]
pub mod test_module_parse {
    use std::sync::Arc;

    #[test]
    fn test_module_parse() {
        let _ = modules::modules();
    }

    async fn new_dummy_basedatadata() -> silverpelt::data::Data {
        const POSTGRES_MAX_CONNECTIONS: u32 = 3; // max connections to the database, we don't need too many here
        const REDIS_MAX_CONNECTIONS: u32 = 10; // max connections to the redis

        let pool = fred::prelude::Builder::from_config(
            fred::prelude::RedisConfig::from_url(&config::CONFIG.meta.bot_redis_url)
                .expect("Could not initialize Redis config"),
        )
        .build_pool(REDIS_MAX_CONNECTIONS.try_into().unwrap())
        .expect("Could not initialize Redis pool");

        let pg_pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(POSTGRES_MAX_CONNECTIONS)
            .connect(&config::CONFIG.meta.postgres_url)
            .await
            .expect("Could not initialize connection");

        silverpelt::data::Data {
            object_store: Arc::new(
                config::CONFIG
                    .object_storage
                    .build()
                    .expect("Could not initialize object store"),
            ),
            pool: pg_pool.clone(),
            reqwest: reqwest::Client::new(),
            extra_data: dashmap::DashMap::new(),
            props: Arc::new(crate::Props {
                mewld_ipc: Arc::new(crate::ipc::mewld::MewldIpcClient {
                    redis_pool: pool.clone(),
                    cache: Arc::new(crate::ipc::mewld::MewldIpcCache::default()),
                    pool: pg_pool.clone(),
                }),
                animus_magic_ipc: std::sync::OnceLock::new(),
                pool: pg_pool.clone(),
                proxy_support_data: tokio::sync::RwLock::new(None),
            }),
            silverpelt_cache: (*crate::SILVERPELT_CACHE).clone(),
        }
    }

    #[tokio::test]
    async fn check_modules_test() {
        // Check for env var CHECK_MODULES_TEST_ENABLED
        if std::env::var("CHECK_MODULES_TEST_ENABLED").unwrap_or_default() == "true" {
            return;
        }

        // Set current directory to ../../
        let current_dir = std::env::current_dir().unwrap();

        if current_dir.ends_with("services/rust.bot") {
            std::env::set_current_dir("../../").unwrap();
        }

        let data = new_dummy_basedatadata().await;
        for module in modules::modules() {
            assert!(module.is_parsed());

            // Ensure that all settings have all columns
            for config_opt in module.config_options {
                let mut missing_columns = Vec::new();

                for column in config_opt.columns.iter() {
                    missing_columns.push(column.id.to_string());
                }

                let cache = serenity::all::Cache::new();
                let http = serenity::all::Http::new("DUMMY");
                let cache_http = botox::cache::CacheHttpImpl {
                    cache: cache.into(),
                    http: http.into(),
                };

                let mut data_store = config_opt
                    .data_store
                    .create(
                        &config_opt,
                        serenity::all::GuildId::new(1),
                        serenity::all::UserId::new(1),
                        &data.settings_data(cache_http),
                        indexmap::IndexMap::new(),
                    )
                    .await
                    .unwrap();

                let columns = data_store.columns().await.unwrap();

                println!(
                    "Module: {}, Config Opt: {}, Columns: {:?}",
                    module.id, config_opt.id, columns
                );

                for column in columns {
                    if let Some(index) = missing_columns.iter().position(|x| x == &column) {
                        missing_columns.remove(index);
                    }
                }

                if !missing_columns.is_empty() {
                    panic!(
                        "Module {} has a config option {} with missing columns: {}",
                        module.id,
                        config_opt.id,
                        missing_columns.join(", ")
                    );
                }
            }
        }
    }
}
