use crate::{canonical_module::CanonicalModule, CommandExtendedDataMap, Module};
use moka::future::Cache;
use serenity::all::GuildId;
use std::sync::Arc;

/// The silverpelt cache is a structure that contains the core state for the bot
pub struct SilverpeltCache {
    /// Cache of whether a (GuildId, String) pair has said module enabled or disabled
    pub module_enabled_cache: Cache<(GuildId, String), bool>,

    /// Cache of the extended data given a command (the extended data map stores the default base permissions and other data per command)
    pub command_extra_data_map: dashmap::DashMap<String, CommandExtendedDataMap>,

    /// A commonly needed operation is mapping a module id to its respective module
    ///
    /// module_cache is a cache of module id to module
    ///
    /// We use indexmap here to avoid the 'static restriction
    pub module_cache: indexmap::IndexMap<String, Arc<Module>>,

    /// Command ID to module map
    ///
    /// This uses an indexmap for now to avoid sending values over await point
    pub command_id_module_map: dashmap::DashMap<String, String>,

    /// Cache of the canonical forms of all modules
    pub canonical_module_cache: dashmap::DashMap<String, CanonicalModule>,

    /// Cache of all regexes and their parsed forms
    pub regex_cache: Cache<String, regex::Regex>,

    /// Cache of all regexes and their pat as a (String, String) to a boolean indicating success
    pub regex_match_cache: Cache<(String, String), bool>,
}

impl Default for SilverpeltCache {
    fn default() -> Self {
        Self {
            module_enabled_cache: Cache::builder().support_invalidation_closures().build(),
            command_extra_data_map: dashmap::DashMap::new(),
            module_cache: indexmap::IndexMap::new(),
            command_id_module_map: dashmap::DashMap::new(),
            canonical_module_cache: dashmap::DashMap::new(),
            regex_cache: Cache::builder().support_invalidation_closures().build(),
            regex_match_cache: Cache::builder().support_invalidation_closures().build(),
        }
    }
}

impl SilverpeltCache {
    pub fn add_module(&mut self, module: Module) {
        let module = Arc::new(module);

        // Add the commands to cache
        for (command, extended_data) in module.commands.iter() {
            self.command_id_module_map
                .insert(command.name.clone(), module.id.to_string());
            self.command_extra_data_map
                .insert(command.name.clone(), extended_data.clone());
        }

        // Add to canonical cache
        let module_ref: &Module = &module;
        self.canonical_module_cache
            .insert(module.id.to_string(), CanonicalModule::from(module_ref));

        // Add the module to cache
        self.module_cache.insert(module.id.to_string(), module);
    }

    pub fn remove_module(&mut self, module_id: &str) {
        if let Some(module) = self.module_cache.shift_remove(module_id) {
            for (command, _) in module.commands.iter() {
                self.command_id_module_map.remove(&command.name);
                self.command_extra_data_map.remove(&command.name);
            }

            self.canonical_module_cache.remove(module_id);
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
