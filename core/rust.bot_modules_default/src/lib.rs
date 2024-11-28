use silverpelt::module::Module;

/// List of modules to load
pub fn modules() -> Vec<Box<dyn Module>> {
    vec![
        Box::new(bot_modules_core::Module),
        Box::new(bot_modules_lockdown::Module),
        Box::new(bot_modules_moderation::Module),
        Box::new(bot_modules_server_backups::Module),
        Box::new(bot_modules_temporary_punishments::Module),
    ]
}
