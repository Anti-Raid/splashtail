pub mod r#async;
pub mod discord;
pub mod img_captcha;
pub mod interop;
pub mod kv;
pub mod lune;
pub mod page;
pub mod permissions;
pub mod stings;
pub mod typesext;

use crate::lang_lua::state;
use mlua::prelude::*;
use std::sync::LazyLock;

// Modules can load their own plugins
pub static PLUGINS: LazyLock<indexmap::IndexMap<String, (ModuleFn, Option<ModuleDocFn>)>> =
    LazyLock::new(|| {
        indexmap::indexmap! {
            "@antiraid/async".to_string() => (r#async::init_plugin as ModuleFn, Some(r#async::plugin_docs as ModuleDocFn)),
            "@antiraid/discord".to_string() => (discord::init_plugin as ModuleFn, Some(discord::plugin_docs as ModuleDocFn)),
            "@antiraid/interop".to_string() => (interop::init_plugin as ModuleFn, Some(interop::plugin_docs as ModuleDocFn)),
            "@antiraid/img_captcha".to_string() => (img_captcha::init_plugin as ModuleFn, Some(img_captcha::plugin_docs as ModuleDocFn)),
            "@antiraid/kv".to_string() => (kv::init_plugin as ModuleFn, Some(kv::plugin_docs as ModuleDocFn)),
            "@antiraid/page".to_string() => (page::init_plugin as ModuleFn, Some(page::plugin_docs as ModuleDocFn)),
            "@antiraid/permissions".to_string() => (permissions::init_plugin as ModuleFn, Some(permissions::plugin_docs as ModuleDocFn)),
            "@antiraid/stings".to_string() => (stings::init_plugin as ModuleFn, None as Option<ModuleDocFn>),
            "@antiraid/typesext".to_string() => (typesext::init_plugin as ModuleFn, Some(typesext::plugin_docs as ModuleDocFn)),

            // External plugins
            "@lune/datetime".to_string() => (lune::datetime::init_plugin as ModuleFn, None as Option<ModuleDocFn>),
            "@lune/regex".to_string() => (lune::regex::init_plugin as ModuleFn, None as Option<ModuleDocFn>),
            "@lune/serde".to_string() => (lune::serde::init_plugin as ModuleFn, None as Option<ModuleDocFn>),
        }
    });

type ModuleFn = fn(&Lua) -> LuaResult<LuaTable>;
type ModuleDocFn = fn() -> templating_docgen::Plugin;

#[derive(serde::Serialize, serde::Deserialize, Default)]
pub struct RequirePluginArgs {
    pub plugin_cache: Option<bool>,
}

#[derive(serde::Serialize, serde::Deserialize, Default)]
pub struct RequireTemplateImportArgs {
    pub current_path: Option<String>,
    pub custom_prefix: Option<String>,
}

pub async fn require(lua: Lua, (plugin_name, args): (String, LuaValue)) -> LuaResult<LuaTable> {
    // Relative imports are special, they include the template stored in guild_templates
    if plugin_name.starts_with("./")
        || plugin_name.starts_with("../")
        || plugin_name.starts_with("$shop/")
    {
        let (pool, guild_id, compiler, vm_bytecode_cache) = {
            let Some(data) = lua.app_data_ref::<state::LuaUserData>() else {
                return Err(LuaError::external("No app data found"));
            };

            (
                data.pool.clone(),
                data.guild_id,
                data.compiler.clone(),
                data.vm_bytecode_cache.clone(),
            )
        };

        let args: RequireTemplateImportArgs = lua
            .from_value::<Option<RequireTemplateImportArgs>>(args)?
            .unwrap_or_default();

        // Get the current path if token is specified
        let current_path = {
            if let Some(current_path) = args.current_path {
                current_path
            } else {
                // Root is the current path
                "".to_string()
            }
        };
        let resolved_path = resolve_template_import_path(
            &current_path,
            &plugin_name,
            &args.custom_prefix.unwrap_or("/".to_string()),
        );

        let cache_key = format!("requireTemplate:{}", resolved_path);

        if let Ok(table) = lua.named_registry_value::<LuaTable>(&cache_key) {
            return Ok(table);
        }

        // Get template content
        let template_content = crate::get_template(guild_id, &resolved_path, &pool)
            .await
            .map_err(|_| LuaError::external("Failed to get template"))?;

        let template_bytecode = crate::lang_lua::resolve_template_to_bytecode(
            template_content.content,
            crate::Template::Named(resolved_path.clone()),
            &vm_bytecode_cache,
            &compiler,
        )
        .await
        .map_err(|_| LuaError::external("Failed to compile template"))?;

        let table: LuaTable = lua
            .load(&template_bytecode)
            .set_name(resolved_path)
            .set_mode(mlua::ChunkMode::Binary) // Ensure auto-detection never selects binary mode
            .call_async(())
            .await?;

        lua.set_named_registry_value(&cache_key, table.clone())?;

        return Ok(table);
    }

    match PLUGINS.get(plugin_name.as_str()) {
        Some(plugin) => {
            let args: RequirePluginArgs = lua
                .from_value::<Option<RequirePluginArgs>>(args)?
                .unwrap_or_default();

            if args.plugin_cache.unwrap_or(true) {
                // Get table from vm cache
                if let Ok(table) = lua.named_registry_value::<LuaTable>(&plugin_name) {
                    return Ok(table);
                }
            }

            let res = plugin.0(&lua);

            if args.plugin_cache.unwrap_or(true) {
                if let Ok(table) = &res {
                    lua.set_named_registry_value(&plugin_name, table.clone())?;
                }
            }

            res
        }
        None => {
            if let Ok(table) = lua.globals().get::<LuaTable>(plugin_name.clone()) {
                return Ok(table);
            }

            Err(LuaError::runtime(format!(
                "module '{}' not found",
                plugin_name
            )))
        }
    }
}

/// Resolves a path given the current_path and the path to resolve
///
/// Rules:
/// The path starts at current path and the 'instructions' in path are applied to it
///
/// The caller should then, if requested in args, make a new template token and inherit the pragma (or use Null for token if caller did not request) for the imported template
fn resolve_template_import_path(current_path: &str, path: &str, prefix: &str) -> String {
    if path.starts_with("$shop/") {
        return path.to_string();
    }

    /*
    Potentially useful in the future
    // Get cwd and the file
    let (cwd, cwf) = if current_path.is_empty() {
        ("", current_path) // If current_path is empty, we are at the root
    } else {
        // Split by prefix and take all but last element as cwd and last as file
        let (cwd, file) = current_path.rsplit_once(prefix).unwrap_or(("", current_path));
        (cwd, file)
    };*/

    let mut new_path = current_path.to_string();

    for (i, inst) in path.split(prefix).enumerate() {
        match (i, inst) {
            (0, "~") => new_path.clear(),
            (_, "") => {}
            (_, ".") => {
                // UNIX notes: Note that the current path is a file, everything else is considered a directory
                //
                // Using '/' as a prefix 'casts' the file path to a directory
                if new_path == current_path && !new_path.starts_with(prefix) {
                    new_path = new_path
                        .rsplit_once(prefix)
                        .map(|(parent, _)| parent)
                        .unwrap_or("")
                        .to_string();
                }
            }
            (_, "..") => {
                new_path = new_path
                    .rsplit_once(prefix)
                    .map(|(parent, _)| parent)
                    .unwrap_or("")
                    .to_string();
            }
            (_, _) => {
                if new_path.is_empty() {
                    new_path.push_str(inst);
                } else {
                    new_path.push_str(prefix);
                    new_path.push_str(inst);
                }
            }
        }
    }

    new_path
        .trim_start_matches(prefix)
        .replace("//", "/")
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::resolve_template_import_path;

    #[test]
    fn test_resolve_template_import_path() {
        assert_eq!(resolve_template_import_path("myscripts/foo", "~/", "/"), "");
        assert_eq!(
            resolve_template_import_path("", "myscripts/abc", "/"),
            "myscripts/abc"
        );
        assert_eq!(
            resolve_template_import_path("myscripts", "../abc", "/"),
            "abc",
        );
        assert_eq!(
            resolve_template_import_path("myscripts", "../abcgh/d", "/"),
            "abcgh/d"
        );
        assert_eq!(
            resolve_template_import_path("myscripts/abc", "../def", "/"),
            "myscripts/def"
        );
        assert_eq!(
            resolve_template_import_path("myscripts/abc", "./def", "/"),
            "myscripts/def"
        );
        assert_eq!(
            resolve_template_import_path("myscripts/abc", "def/..", "/"),
            "myscripts/abc"
        );
        assert_eq!(
            resolve_template_import_path("myscripts/abc/", "def", "/"),
            "myscripts/abc/def"
        );
        assert_eq!(
            resolve_template_import_path("/myscripts/abc", "def", "/"),
            "myscripts/abc/def"
        );
        assert_eq!(
            resolve_template_import_path("/myscripts/abc/", "/def", "/"),
            "myscripts/abc/def"
        );
    }
}
