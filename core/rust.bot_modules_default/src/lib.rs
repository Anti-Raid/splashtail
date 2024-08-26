/// List of modules to load
pub fn modules() -> Vec<silverpelt::Module> {
    let base_modules = vec![
        bot_modules_afk::module().parse(),
        bot_modules_auditlogs::module().parse(),
        bot_modules_core::module().parse(),
        bot_modules_gitlogs::module().parse(),
        bot_modules_inspector::module().parse(),
        bot_modules_limits::module().parse(),
        bot_modules_lockdown::module().parse(),
        bot_modules_moderation::module().parse(),
        bot_modules_punishments::module().parse(),
        bot_modules_server_backups::module().parse(),
        bot_modules_server_member_backups::module().parse(),
        bot_modules_settings::module().parse(),
        bot_modules_temporary_punishments::module().parse(),
        bot_modules_root::module().parse(),
    ];

    // Add ACL module
    let mut module_ids = Vec::new();

    for module in base_modules.iter() {
        module_ids.push(module.id);
    }

    let mut modules = Vec::new();

    modules.push(bot_modules_acl::module(module_ids).parse());

    // Add all base modules
    modules.extend(base_modules);

    modules
}
