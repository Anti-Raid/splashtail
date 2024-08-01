pub mod core;

#[cfg(feature = "quickjs")]
mod lang_javascript_quickjs;
#[cfg(feature = "v8")]
mod lang_javascript_v8;
#[cfg(feature = "lua")]
mod lang_lua;
#[cfg(feature = "rhai")]
mod lang_rhai;
#[cfg(feature = "tera")]
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
    #[cfg(feature = "lua")]
    Lua,
    #[cfg(feature = "rhai")]
    Rhai,
    #[cfg(feature = "tera")]
    Tera,
}

impl FromStr for TemplateLanguage {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            #[cfg(feature = "lua")]
            "lang_lua" => Ok(Self::Lua),
            #[cfg(feature = "rhai")]
            "lang_rhai" => Ok(Self::Rhai),
            #[cfg(feature = "tera")]
            "lang_tera" => Ok(Self::Tera),
            _ => Err(()),
        }
    }
}

impl std::fmt::Display for TemplateLanguage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            #[cfg(feature = "lua")]
            Self::Lua => write!(f, "lang_lua"),
            #[cfg(feature = "rhai")]
            Self::Rhai => write!(f, "lang_rhai"),
            #[cfg(feature = "tera")]
            Self::Tera => write!(f, "lang_tera"),
        }
    }
}

impl TemplateLanguage {
    pub fn support_tier(&self) -> TemplateLanguageSupportTier {
        match self {
            #[cfg(feature = "lua")]
            Self::Lua => TemplateLanguageSupportTier::TierOne, 
            #[cfg(feature = "rhai")]
            Self::Rhai => TemplateLanguageSupportTier::TierTwo,
            #[cfg(feature = "tera")]
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

#[allow(unused_variables)]
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
        #[cfg(feature = "lua")]
        TemplateLanguage::Lua => {
            lang_lua::compile_template(guild_id, rest).await?;
        }
        #[cfg(feature = "rhai")]
        TemplateLanguage::Rhai => {
            let mut engine = lang_rhai::create_engine();
            lang_rhai::apply_sandboxing(&mut engine);
            lang_rhai::compile(&engine, rest, opts)?;
        }
        #[cfg(feature = "tera")]
        TemplateLanguage::Tera => {
            lang_tera::compile_template(rest, opts).await?;
        }
    }

    Ok(())
}

/// Renders a message template
#[allow(unused_variables)]
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
        #[cfg(feature = "lua")]
        TemplateLanguage::Lua => {
            let msg_exec_template = lang_lua::render_message_template(guild_id, rest, args).await?;
            lang_lua::plugins::message::to_discord_reply(msg_exec_template)
        }
        #[cfg(feature = "rhai")]
        TemplateLanguage::Rhai => {
            let mut engine = lang_rhai::create_engine();
            lang_rhai::apply_sandboxing(&mut engine);
            let ast = lang_rhai::compile(&engine, rest, opts)?;

            let mut scope = lang_rhai::plugins::message::create_message_scope(args)?;
            let result: lang_rhai::plugins::message::plugin::Message =
                engine.eval_ast_with_scope(&mut scope, &ast)?;

            lang_rhai::plugins::message::to_discord_reply(result)
        }
        #[cfg(feature = "tera")]
        TemplateLanguage::Tera => {
            let mut tera = lang_tera::compile_template(rest, opts).await?;
            let msg_exec_template =
                lang_tera::message::execute_template_for_message(&mut tera, args).await?;
            msg_exec_template.discord_reply()
        }
    }
}

/// Renders a permissions template
#[allow(unused_variables)]
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
        #[cfg(feature = "lua")]
        TemplateLanguage::Lua => {
            match lang_lua::render_permissions_template(guild_id, rest, pctx).await {
                Ok(result) => result,
                Err(e) => PermissionResult::GenericError {
                    error: format!("Failed to render: {:?}", e),
                },
            }
        }
        #[cfg(feature = "rhai")]
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
        #[cfg(feature = "tera")]
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
