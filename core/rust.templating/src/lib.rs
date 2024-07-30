pub mod core;
mod lang_javascript_quickjs;
mod lang_javascript_v8;
mod lang_lua;
mod lang_rhai;
mod lang_tera;

use once_cell::sync::Lazy;
use splashcore_rs::types::silverpelt::PermissionResult;
use std::str::FromStr;

static TEMPLATING_ENVVAR: Lazy<Vec<String>> = Lazy::new(|| {
    let v = std::env::var("ANTIRAID_SUPPORTED_TEMPLATED_ENGINES");

    match v {
        Ok(v) => v.split(',').map(|s| s.trim().to_string()).collect(),
        Err(_) => Vec::new(),
    }
});

pub struct CompileTemplateOptions {
    /// Cache the result of the template compilation
    pub cache_result: bool,
    /// Ignore the cache and compile the template again
    pub ignore_cache: bool,
}

pub enum TemplateLanguageSupportTier {
    TierOne,   // Fully supported without an environment variable, full sandboxing built in
    TierTwo,   // Supported without an environment variable, may have limited sandboxing
    TierThree, // Supported with an environment variable, may have limited sandboxing and may be broken
}

impl TemplateLanguageSupportTier {
    pub fn can_execute_without_env_var(&self) -> bool {
        match self {
            Self::TierOne | Self::TierTwo => true,
            Self::TierThree => false,
        }
    }
}

pub enum TemplateLanguage {
    Lua,
    Rhai,
    Tera,
}

impl FromStr for TemplateLanguage {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "lang_lua" => Ok(Self::Lua),
            "lang_rhai" => Ok(Self::Rhai),
            "lang_tera" => Ok(Self::Tera),
            _ => Err(()),
        }
    }
}

impl std::fmt::Display for TemplateLanguage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Lua => write!(f, "lang_lua"),
            Self::Rhai => write!(f, "lang_rhai"),
            Self::Tera => write!(f, "lang_tera"),
        }
    }
}

impl TemplateLanguage {
    pub fn support_tier(&self) -> TemplateLanguageSupportTier {
        match self {
            Self::Lua => TemplateLanguageSupportTier::TierTwo, // Untested but could become TierOne
            Self::Rhai => TemplateLanguageSupportTier::TierTwo,
            Self::Tera => TemplateLanguageSupportTier::TierTwo,
        }
    }

    pub fn can_execute(&self) -> bool {
        let tier = self.support_tier();

        if tier.can_execute_without_env_var() {
            return true;
        }

        TEMPLATING_ENVVAR.contains(&self.to_string())
    }

    pub fn from_pragma(pragma: &str) -> Option<Self> {
        let comment = pragma.trim();

        if comment.starts_with("//lang:") {
            let lang = comment.split(':').nth(1)?;

            match Self::from_str(lang) {
                Ok(lang) => Some(lang),
                Err(_) => None,
            }
        } else {
            None
        }
    }
}

pub async fn compile_template(
    guild_id: serenity::all::GuildId,
    template: &str,
    opts: CompileTemplateOptions,
) -> Result<(), base_data::Error> {
    let (first_line, rest) = match template.find('\n') {
        Some(i) => template.split_at(i),
        None => return Err("No/unknown template language specified".into()),
    };

    let Some(lang) = TemplateLanguage::from_pragma(first_line) else {
        return Err("No/unknown template language specified".into());
    };

    match lang {
        TemplateLanguage::Lua => {
            lang_lua::compile_template(guild_id, rest).await?;
        }
        TemplateLanguage::Rhai => {
            let mut engine = lang_rhai::create_engine();
            lang_rhai::apply_sandboxing(&mut engine);
            lang_rhai::compile(&engine, rest, opts)?;
        }
        TemplateLanguage::Tera => {
            lang_tera::compile_template(rest, opts).await?;
        }
    }

    Ok(())
}

/// Renders a message template
pub async fn render_message_template(
    guild_id: serenity::all::GuildId,
    template: &str,
    args: crate::core::MessageTemplateContext,
    opts: CompileTemplateOptions,
) -> Result<core::DiscordReply, base_data::Error> {
    let (first_line, rest) = match template.find('\n') {
        Some(i) => template.split_at(i),
        None => return Err("No/unknown template language specified".into()),
    };

    let Some(lang) = TemplateLanguage::from_pragma(first_line) else {
        return Err("No/unknown template language specified".into());
    };

    match lang {
        TemplateLanguage::Lua => {
            let msg_exec_template = lang_lua::render_message_template(guild_id, rest, args).await?;
            lang_lua::plugins::message::to_discord_reply(msg_exec_template)
        }
        TemplateLanguage::Rhai => {
            let mut engine = lang_rhai::create_engine();
            lang_rhai::apply_sandboxing(&mut engine);
            let ast = lang_rhai::compile(&engine, rest, opts)?;

            let mut scope = lang_rhai::plugins::message::create_message_scope(args)?;
            let result: lang_rhai::plugins::message::plugin::Message =
                engine.eval_ast_with_scope(&mut scope, &ast)?;

            lang_rhai::plugins::message::to_discord_reply(result)
        }
        TemplateLanguage::Tera => {
            let mut tera = lang_tera::compile_template(rest, opts).await?;
            let msg_exec_template =
                lang_tera::message::execute_template_for_message(&mut tera, args).await?;
            msg_exec_template.discord_reply()
        }
    }
}

/// Renders a permissions template
pub async fn render_permissions_template(
    guild_id: serenity::all::GuildId,
    template: &str,
    pctx: crate::core::PermissionTemplateContext,
    opts: CompileTemplateOptions,
) -> PermissionResult {
    let (first_line, rest) = match template.find('\n') {
        Some(i) => template.split_at(i),
        None => {
            return PermissionResult::GenericError {
                error: "No/unknown template language specified".into(),
            }
        }
    };

    let Some(lang) = TemplateLanguage::from_pragma(first_line) else {
        return PermissionResult::GenericError {
            error: "No/unknown template language specified".into(),
        };
    };

    match lang {
        TemplateLanguage::Lua => {
            match lang_lua::render_permissions_template(guild_id, rest, pctx).await {
                Ok(result) => result,
                Err(e) => PermissionResult::GenericError {
                    error: format!("Failed to render: {:?}", e),
                },
            }
        }
        TemplateLanguage::Rhai => {
            let mut engine = lang_rhai::create_engine();
            lang_rhai::apply_sandboxing(&mut engine);

            let ast = match lang_rhai::compile(&engine, rest, opts) {
                Ok(ast) => ast,
                Err(e) => {
                    return PermissionResult::GenericError {
                        error: format!("Failed to compile: {:?}", e),
                    }
                }
            };

            let mut scope = match lang_rhai::plugins::permissions::create_permission_scope(pctx) {
                Ok(scope) => scope,
                Err(e) => {
                    return PermissionResult::GenericError {
                        error: format!("Failed to create scope: {:?}", e),
                    }
                }
            };

            let result: PermissionResult = match engine.eval_ast_with_scope(&mut scope, &ast) {
                Ok(result) => result,
                Err(e) => {
                    return PermissionResult::GenericError {
                        error: format!("Failed to eval: {:?}", e),
                    }
                }
            };

            result
        }
        TemplateLanguage::Tera => {
            let mut tera = match lang_tera::compile_template(rest, opts).await {
                Ok(tera) => tera,
                Err(e) => {
                    return PermissionResult::GenericError {
                        error: format!("Failed to compile: {:?}", e),
                    }
                }
            };

            lang_tera::permissions::execute_permissions_template(&mut tera, pctx).await
        }
    }
}
