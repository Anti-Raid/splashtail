mod limits;
mod server_backups;
mod server_member_backups;
mod core;
mod root;

/// List of enabled modules
/// 
/// Add to this list to create a module
pub fn enabled_modules() -> Vec<crate::silverpelt::Module> {
    vec![
        core::module(),
        limits::module(),
        server_backups::module(),
        root::module(),
        server_member_backups::module(),
    ]
}