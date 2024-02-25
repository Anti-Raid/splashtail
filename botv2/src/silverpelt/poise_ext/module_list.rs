use crate::silverpelt::SILVERPELT_CACHE;
use serenity::all::AutocompleteChoice;
use crate::Context;

pub async fn autocomplete<'a>(
    _ctx: Context<'_>,
    partial: &'a str,
) -> Vec<AutocompleteChoice<'a>> {
    let mut ac = Vec::new();

    for mv in SILVERPELT_CACHE.module_id_cache.iter() {
        let module = mv.value();

        if module.name.to_lowercase().contains(&partial.to_lowercase()) || module.id.to_lowercase().contains(&partial.to_lowercase()) {
            ac.push(AutocompleteChoice::new(module.name, module.id));
        }
    }

    ac
}