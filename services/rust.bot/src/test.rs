/// This test ensures that all modules can be parsed
#[cfg(test)]
pub mod test_module_parse {
    #[test]
    fn test_module_parse() {
        let _ = modules::modules();
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

        let pg_pool = sqlx::postgres::PgPoolOptions::new()
            .connect(&config::CONFIG.meta.postgres_url)
            .await
            .expect("Could not initialize connection");

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
                let reqwest_client = reqwest::Client::new();

                let mut data_store = config_opt
                    .data_store
                    .create(
                        &config_opt,
                        &cache_http,
                        &reqwest_client,
                        &pg_pool,
                        serenity::all::GuildId::new(1),
                        serenity::all::UserId::new(1),
                        &base_data::permodule::DummyPermoduleFunctionExecutor {},
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
