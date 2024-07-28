pub mod core;
mod lang_javascript_quickjs;
mod lang_javascript_v8;
mod lang_rhai;
mod lang_tera;

use splashcore_rs::types::silverpelt::PermissionResult;
use std::str::FromStr;

pub struct CompileTemplateOptions {
    /// Cache the result of the template compilation
    pub cache_result: bool,
    /// Ignore the cache and compile the template again
    pub ignore_cache: bool,
}

pub enum TemplateLanguage {
    Rhai,
    Tera,
}

impl FromStr for TemplateLanguage {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "lang_rhai" => Ok(Self::Rhai),
            "lang_tera" => Ok(Self::Tera),
            _ => Err(()),
        }
    }
}

impl TemplateLanguage {
    pub fn from_comment(comment: &str) -> Option<Self> {
        let comment = comment.trim();

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
    template: &str,
    opts: CompileTemplateOptions,
) -> Result<(), base_data::Error> {
    let (first_line, rest) = match template.find('\n') {
        Some(i) => template.split_at(i),
        None => (template, ""),
    };

    let Some(lang) = TemplateLanguage::from_comment(first_line) else {
        return Err("No/unknown template language specified".into());
    };

    match lang {
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
    template: &str,
    args: crate::core::MessageTemplateContext,
    opts: CompileTemplateOptions,
) -> Result<core::DiscordReply, base_data::Error> {
    let (first_line, rest) = match template.find('\n') {
        Some(i) => template.split_at(i),
        None => (template, ""),
    };

    let Some(lang) = TemplateLanguage::from_comment(first_line) else {
        return Err("No/unknown template language specified".into());
    };

    match lang {
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
    template: &str,
    pctx: crate::core::PermissionTemplateContext,
    opts: CompileTemplateOptions,
) -> PermissionResult {
    let (first_line, rest) = match template.find('\n') {
        Some(i) => template.split_at(i),
        None => (template, ""),
    };

    let Some(lang) = TemplateLanguage::from_comment(first_line) else {
        return PermissionResult::GenericError {
            error: "No/unknown template language specified".into(),
        };
    };

    match lang {
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
