use poise::{ChoiceParameter, CommandParameterChoice};
use std::collections::HashMap;
use crate::silverpelt::SILVERPELT_CACHE;
use once_cell::sync::Lazy;

static MODULE_LIST_CPC: Lazy<Vec<CommandParameterChoice>> = Lazy::new(|| {
    let mut cpc = Vec::new();

    for module in crate::modules::module_ids() {
        cpc.push(CommandParameterChoice {
            name: module.into(),
            localizations: HashMap::new(),
            __non_exhaustive: (), // Poise moment
        });
    }

    cpc
});

/// Helper struct to allow the user to select a module from a list of modules
///
/// Note that this currently only works based on the ID and not the module name due to technical issues regarding a loop in silverpelt_cache
pub struct ModuleList {
    /// The id of the module they have chosen
    pub chosen_id: String,
}

impl ChoiceParameter for ModuleList {
    fn list() -> Vec<CommandParameterChoice> {
        MODULE_LIST_CPC.clone()
    }

    fn from_index(index: usize) -> Option<Self> {        
        let module_name = MODULE_LIST_CPC.get(index)?.name.clone().into_owned();
        //let chosen_module_id = SILVERPELT_CACHE.module_id_name_cache.get(&module_name)?;

        Some(ModuleList {
            chosen_id: module_name.clone(),
        })
    }

    fn from_name(name: &str) -> Option<Self> {
        let chosen_module_id = SILVERPELT_CACHE.module_id_name_cache.get(name)?;

        Some(ModuleList {
            chosen_id: chosen_module_id.clone(),
        })
    }

    fn name(&self) -> &'static str {
        let module = SILVERPELT_CACHE.module_id_cache.get(&self.chosen_id);

        if let Some(module) = module {
            module.name
        } else {
            unreachable!("Module should always be found in cache, so this should never be reachable");
        }   
    }

    fn localized_name(&self, _locale: &str) -> Option<&'static str> {
        None
    }
}