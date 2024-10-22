mod atomicinstant;
pub mod core;

mod lang_lua;
pub use lang_lua::state::LuaKVConstraints;

type Error = Box<dyn std::error::Error + Send + Sync>; // This is constant and should be copy pasted

use std::str::FromStr;

const MAX_ACTIONS: usize = 50;
const MAX_KV_OPS: usize = 50;
const MAX_PRAGMA_SIZE: usize = 2048;

#[derive(Clone, serde::Serialize, serde::Deserialize, Default)]
pub struct TemplatePragma {
    pub lang: TemplateLanguage,

    #[serde(default)]
    pub actions: Vec<String>,

    #[serde(default)]
    pub kv_ops: Vec<String>,

    #[serde(flatten)]
    pub extra_info: indexmap::IndexMap<String, serde_json::Value>,
}

impl TemplatePragma {
    pub fn parse(template: &str) -> Result<(&str, Self), Error> {
        let (first_line, rest) = match template.find('\n') {
            Some(i) => template.split_at(i),
            None => return Ok((template, Self::default())),
        };

        if !first_line.contains("@pragma ") {
            return Ok((template, Self::default()));
        }

        // Remove out the @pragma and serde parse it
        let first_line = first_line.replace("@pragma ", "");

        if first_line.as_bytes().len() > MAX_PRAGMA_SIZE {
            return Err("Pragma too large".into());
        }

        let pragma: TemplatePragma = serde_json::from_str(&first_line)?;

        if pragma.actions.len() > MAX_ACTIONS {
            return Err("Too many actions specified".into());
        }

        if pragma.kv_ops.len() > MAX_KV_OPS {
            return Err("Too many kv ops specified".into());
        }

        Ok((rest, pragma))
    }
}

#[derive(Clone, serde::Serialize, serde::Deserialize, Default)]
pub enum TemplateLanguage {
    #[cfg(feature = "lua")]
    #[serde(rename = "lua")]
    #[default]
    Lua,
}

impl FromStr for TemplateLanguage {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            #[cfg(feature = "lua")]
            "lang_lua" => Ok(Self::Lua),
            _ => Err(()),
        }
    }
}

impl std::fmt::Display for TemplateLanguage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            #[cfg(feature = "lua")]
            Self::Lua => write!(f, "lang_lua"),
        }
    }
}

async fn get_template(
    guild_id: serenity::all::GuildId,
    template: &str,
    pool: &sqlx::PgPool,
) -> Result<String, Error> {
    let rec = sqlx::query!(
        "SELECT content FROM guild_templates WHERE guild_id = $1 AND name = $2",
        guild_id.to_string(),
        template
    )
    .fetch_optional(pool)
    .await?;

    match rec {
        Some(rec) => Ok(rec.content),
        None => Err("Template not found".into()),
    }
}

#[derive(Clone, Hash, PartialEq, Eq)]
pub enum Template {
    Raw(String),
    Named(String),
}

#[allow(unused_variables)]
pub async fn parse(
    guild_id: serenity::all::GuildId,
    template: Template,
    pool: sqlx::PgPool,
    cache_http: botox::cache::CacheHttpImpl,
    reqwest_client: reqwest::Client,
) -> Result<(), Error> {
    let template_content = match template {
        Template::Raw(ref template) => template.clone(),
        Template::Named(ref template) => get_template(guild_id, template, &pool).await?,
    };

    let (template_content, pragma) = TemplatePragma::parse(&template_content)?;

    Ok(())
}

#[typetag::serde(tag = "type")]
pub trait Context: Send + Sync {}

/// Executes a template
pub async fn execute<C: Context + serde::Serialize, RenderResult: serde::de::DeserializeOwned>(
    guild_id: serenity::all::GuildId,
    template: Template,
    pool: sqlx::PgPool,
    cache_http: botox::cache::CacheHttpImpl,
    reqwest_client: reqwest::Client,
    ctx: C,
) -> Result<RenderResult, Error> {
    let template_content = match template {
        Template::Raw(ref template) => template.clone(),
        Template::Named(ref template) => get_template(guild_id, template, &pool).await?,
    };

    let (template_content, pragma) = TemplatePragma::parse(&template_content)?;

    match pragma.lang {
        #[cfg(feature = "lua")]
        TemplateLanguage::Lua => lang_lua::render_template(
            ctx,
            lang_lua::ParseCompileState {
                cache_http,
                reqwest_client,
                guild_id,
                template,
                pragma,
                template_content: template_content.to_string(),
                pool,
            },
        )
        .await
        .map_err(|e| e.into()),
    }
}

#[cfg(feature = "lua")]
pub mod luau_utils {
    pub fn wrap_main_in_entrypoint(template: &str) -> String {
        format!(
            r#"@pragma {{"lang":"lua"}}
function (args) 
    if 1==1 then
        {}
        return _main(args)
    end
end"#,
            template
        )
    }
}
