mod core;
mod gitlogs;
mod limits;
mod root;
mod server_backups;
mod server_member_backups;

/// List of enabled modules
/// 
/// Add to this list to create a module
pub fn enabled_modules() -> Vec<crate::silverpelt::Module> {
    vec![
        core::module(),
        gitlogs::module(),
        limits::module(),
        root::module(),
        server_backups::module(),
        server_member_backups::module(),
    ]
}
