use super::{
    canonical_module::CanonicalModule, permissions::PermissionResult, CommandExtendedDataMap,
    Module, ModuleEventHandler,
};
use indexmap::IndexMap;
use moka::future::Cache;
use once_cell::sync::Lazy;
use serenity::all::{GuildId, UserId};

/// The silverpelt cache is a structure that contains the core state for the bot
pub struct SilverpeltCache {
    /// Cache of whether a (GuildId, String) pair has said module enabled or disabled
    pub module_enabled_cache: Cache<(GuildId, String), bool>,

    /// Cache of whether a (GuildId, UserId) pair has the permission to run a command
    pub command_permission_cache: Cache<
        (GuildId, UserId, super::cmd::CheckCommandOptions),
        IndexMap<String, PermissionResult>,
    >,

    /// Cache of the extended data given a command (the extended data map stores the default base permissions and other data per command)
    pub command_extra_data_map: dashmap::DashMap<String, CommandExtendedDataMap>,

    /// A commonly needed operation is mapping a module id to its respective module
    ///
    /// Module_id_cache is a cache of module id to module
    pub module_id_cache: dashmap::DashMap<String, Module>,

    /// Command ID to module map
    ///
    /// This uses an indexmap for now to avoid sending values over await point
    pub command_id_module_map: indexmap::IndexMap<String, String>,

    /// Cache of the canonical forms of all modules
    pub canonical_module_cache: dashmap::DashMap<String, CanonicalModule>,

    /// Cache of all regexes and their parsed forms
    pub regex_cache: Cache<String, regex::Regex>,

    /// Cache of all regexes and their pat as a (String, String) to a boolean indicating success
    pub regex_match_cache: Cache<(String, String), bool>,

    /// Cache of all event listeners for a given module
    pub module_event_listeners_cache: indexmap::IndexMap<String, Vec<ModuleEventHandler>>,
}

impl SilverpeltCache {
    pub fn new() -> Self {
        log::info!("Making new SilverpeltCache");
        Self {
            command_permission_cache: Cache::builder()
                .support_invalidation_closures()
                .time_to_live(std::time::Duration::from_secs(60))
                .build(),
            module_enabled_cache: Cache::builder().support_invalidation_closures().build(),
            command_extra_data_map: {
                let map = dashmap::DashMap::new();

                for module in crate::modules::modules() {
                    for (command, extended_data) in module.commands {
                        map.insert(command.name.clone(), extended_data);
                    }
                }

                map
            },
            module_id_cache: {
                let map = dashmap::DashMap::new();

                for module in crate::modules::modules() {
                    map.insert(module.id.to_string(), module);
                }

                map
            },
            command_id_module_map: {
                let mut map = indexmap::IndexMap::new();

                for module in crate::modules::modules() {
                    for command in module.commands.iter() {
                        map.insert(command.0.name.to_string(), module.id.to_string());

                        for sub in command.0.subcommands.iter() {
                            map.insert(sub.name.to_string(), module.id.to_string());
                        }
                    }
                }

                map
            },
            canonical_module_cache: {
                let map = dashmap::DashMap::new();

                for module in crate::modules::modules() {
                    map.insert(module.id.to_string(), CanonicalModule::from(module));
                }

                map
            },
            regex_cache: Cache::builder().support_invalidation_closures().build(),
            regex_match_cache: Cache::builder().support_invalidation_closures().build(),
            module_event_listeners_cache: {
                let mut map = indexmap::IndexMap::new();

                for module in crate::modules::modules() {
                    map.insert(module.id.to_string(), module.event_handlers);
                }

                map
            },
        }
    }

    // This method attempts to match on a regex while using the cache where possible
    pub async fn regex_match(&self, regex: &str, pat: &str) -> Result<bool, crate::Error> {
        if let Some(m) = self
            .regex_match_cache
            .get(&(regex.to_string(), pat.to_string()))
            .await
        {
            return Ok(m);
        }

        let compiled_regex = if let Some(compiled_regex) = self.regex_cache.get(regex).await {
            compiled_regex
        } else {
            let regex_compiled = regex::Regex::new(regex)?;
            self.regex_cache
                .insert(regex.to_string(), regex_compiled.clone())
                .await;

            regex_compiled
        };

        let result = compiled_regex.is_match(pat);

        self.regex_match_cache
            .insert((regex.to_string(), pat.to_string()), result)
            .await;

        Ok(result)
    }
}

pub static SILVERPELT_CACHE: Lazy<SilverpeltCache> = Lazy::new(SilverpeltCache::new);
