mod atomicinstant;
pub mod core;

#[cfg(feature = "lua")]
mod lang_lua;
#[cfg(feature = "lua")]
pub use lang_lua::state::LuaKVConstraints;

type Error = Box<dyn std::error::Error + Send + Sync>; // This is constant and should be copy pasted

use std::str::FromStr;

#[derive(Clone, serde::Serialize, serde::Deserialize)]
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
            None => return Err("No/unknown template language specified".into()),
        };

        if !first_line.contains("@pragma ") {
            return Err("No/unknown template language specified".into());
        }

        // Remove out the @pragma and serde parse it
        let first_line = first_line.replace("@pragma ", "");

        Ok((rest, serde_json::from_str(&first_line)?))
    }
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub enum TemplateLanguage {
    #[cfg(feature = "lua")]
    #[serde(rename = "lua")]
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
    let template = match template {
        Template::Raw(template) => template,
        Template::Named(template) => get_template(guild_id, &template, &pool).await?,
    };

    let (template, pragma) = TemplatePragma::parse(&template)?;

    match pragma.lang {
        #[cfg(feature = "lua")]
        TemplateLanguage::Lua => {
            lang_lua::parse(cache_http, reqwest_client, guild_id, pragma, template, pool).await?;
        }
    }

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
    let template = match template {
        Template::Raw(template) => template,
        Template::Named(template) => get_template(guild_id, &template, &pool).await?,
    };

    let (template, pragma) = TemplatePragma::parse(&template)?;

    match pragma.lang {
        #[cfg(feature = "lua")]
        TemplateLanguage::Lua => {
            let v = lang_lua::render_template(
                cache_http,
                reqwest_client,
                guild_id,
                pragma,
                template,
                pool,
                ctx,
            )
            .await?;

            Ok(v)
        }
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
