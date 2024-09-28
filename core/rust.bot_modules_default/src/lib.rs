/// List of modules to load
pub fn modules() -> Vec<Box<dyn silverpelt::module::Module>> {
    // List of base modules (wrapped in an Box::new, not a macro)
    let base_modules: Vec<Box<dyn silverpelt::module::Module>> = vec![
        Box::new(bot_modules_afk::Module),
        Box::new(bot_modules_auditlogs::Module),
        Box::new(bot_modules_core::Module),
        Box::new(bot_modules_gitlogs::Module),
        Box::new(bot_modules_inspector::Module),
        Box::new(bot_modules_limits::Module),
        Box::new(bot_modules_lockdown::Module),
        Box::new(bot_modules_moderation::Module),
        Box::new(bot_modules_punishments::Module),
        Box::new(bot_modules_server_backups::Module),
        Box::new(bot_modules_server_member_backups::Module),
        Box::new(bot_modules_settings::Module),
        Box::new(bot_modules_sting_sources::Module),
        Box::new(bot_modules_temporary_punishments::Module),
        Box::new(bot_modules_root::Module),
    ];

    // Add ACL module
    let mut module_ids = Vec::new();

    for module in base_modules.iter() {
        module_ids.push(module.id());
    }

    let mut modules: Vec<Box<dyn silverpelt::module::Module>> = Vec::new();

    modules.push(Box::new(bot_modules_acl::Module { module_ids }));

    // Add all base modules
    modules.extend(base_modules);

    modules
}
