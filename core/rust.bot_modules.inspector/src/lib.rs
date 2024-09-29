pub mod cache; // Used by root module
mod cmd;
mod core;
mod dehoist;
pub mod events; // Events is a public interface
mod guildprotect;
mod settings;
pub mod types;

pub struct Module;

impl silverpelt::module::Module for Module {
    fn id(&self) -> &'static str {
        "inspector"
    }

    fn name(&self) -> &'static str {
        "Inspector"
    }

    fn description(&self) -> &'static str {
        "Provides passive anti-spam options"
    }

    fn raw_commands(&self) -> Vec<silverpelt::module::CommandObj> {
        vec![
            (
                cmd::inspector_global(),
                indexmap::indexmap! {
                    "list" => silverpelt::types::CommandExtendedData::kittycat_or_admin("inspector_global", "list"),
                    "setup" => silverpelt::types::CommandExtendedData::kittycat_or_admin("inspector_global", "setup"),
                    "update" => silverpelt::types::CommandExtendedData::kittycat_or_admin("inspector_global", "setup"),
                    "delete" => silverpelt::types::CommandExtendedData::kittycat_or_admin("inspector_global", "setup"),
                },
            ),
            (
                cmd::inspector_specific(),
                indexmap::indexmap! {
                    "list" => silverpelt::types::CommandExtendedData::kittycat_or_admin("inspector_specific", "list"),
                    "setup" => silverpelt::types::CommandExtendedData::kittycat_or_admin("inspector_specific", "setup"),
                    "update" => silverpelt::types::CommandExtendedData::kittycat_or_admin("inspector_specific", "setup"),
                    "delete" => silverpelt::types::CommandExtendedData::kittycat_or_admin("inspector_specific", "setup"),
                },
            ),
        ]
    }

    fn event_listeners(&self) -> Option<Box<dyn silverpelt::module::ModuleEventListeners>> {
        Some(Box::new(EventListener))
    }

    fn sting_sources(&self) -> Vec<std::sync::Arc<dyn silverpelt::sting_sources::StingSource>> {
        vec![std::sync::Arc::new(core::InspectorPunishmentsStingSource)]
    }

    fn s3_paths(&self) -> Vec<String> {
        vec!["inspector/guild_icons/{guild_id}".to_string()]
    }

    fn config_options(&self) -> Vec<module_settings::types::ConfigOption> {
        vec![
            (*settings::INSPECTOR_GLOBAL_OPTIONS).clone(),
            (*settings::INSPECTOR_SPECIFIC_OPTIONS).clone(),
        ]
    }
}

struct EventListener;

#[async_trait::async_trait]
impl silverpelt::module::ModuleEventListeners for EventListener {
    async fn on_startup(&self, data: &silverpelt::data::Data) -> Result<(), silverpelt::Error> {
        cache::setup_cache_initial(&data.pool).await?;
        cache::setup_am_toggle(data).await?;
        cache::setup_fake_bots_cache(&data.pool).await?;
        Ok(())
    }

    async fn event_handler(
        &self,
        _ctx: &silverpelt::EventHandlerContext,
    ) -> Result<(), silverpelt::Error> {
        events::event_listener(_ctx).await
    }
}
