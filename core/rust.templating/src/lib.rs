use base_data::limits::embed_limits;
use moka::future::Cache;
use once_cell::sync::Lazy;
use std::error::Error;
use std::sync::{Arc, RwLock};
use std::time::Duration;
use tera::Tera;

// Re-export Tera as Engine
pub mod engine {
    pub use tera::Filter;
    pub use tera::Function;
    pub use tera::Result;
    pub use tera::Tera as Engine;
    pub use tera::Value;
}

/// Maximum number of AST nodes in a template
pub const MAX_TEMPLATE_NODES: usize = 1024;

/// Timeout for template execution
pub const TEMPLATE_EXECUTION_TIMEOUT: Duration = Duration::from_millis(600);

/// Stores a cache of templates with the template content as key
static TEMPLATE_CACHE: Lazy<Cache<String, Tera>> = Lazy::new(|| {
    Cache::builder()
        .time_to_live(Duration::from_secs(60 * 60))
        .build()
});

pub struct CompileTemplateOptions {
    /// Cache the result of the template compilation
    pub cache_result: bool,
    /// Ignore the cache and compile the template again
    pub ignore_cache: bool,
}

pub fn make_templating_context() -> tera::Context {
    tera::Context::new()
}

pub async fn compile_template(
    template: &str,
    opts: CompileTemplateOptions,
) -> Result<Tera, base_data::Error> {
    if !opts.ignore_cache {
        // Check if in template
        if let Some(ref tera) = TEMPLATE_CACHE.get(template).await {
            return Ok(tera.clone());
        }
    }

    // Compile a new template
    let mut tera = Tera::default();

    tera.autoescape_on(vec![]);

    // Add main template
    tera.add_raw_template("main", template)?;

    let mut total_nodes = 0;
    for (_, t) in tera.templates.iter() {
        total_nodes += t.ast.len();

        if total_nodes > MAX_TEMPLATE_NODES {
            return Err("Template has too many nodes".into());
        }
    }

    if opts.cache_result {
        // Store the template in the cache
        TEMPLATE_CACHE
            .insert(template.to_string(), tera.clone())
            .await;
    }

    Ok(tera)
}

#[derive(Debug, Default)]
struct InternalTemplateExecuteState {
    /// The title set by the template
    title: RwLock<Option<String>>,
    /// The fields that were set by the template
    fields: RwLock<indexmap::IndexMap<String, (String, bool)>>,
}

// Set title of embed
struct TitleFunction {
    state: Arc<InternalTemplateExecuteState>,
}

impl tera::Function for TitleFunction {
    fn call(
        &self,
        args: &std::collections::HashMap<String, tera::Value>,
    ) -> tera::Result<tera::Value> {
        let title = args.get("title").ok_or("Title not provided")?;

        // Lock title for writing
        let mut title_writer = self
            .state
            .title
            .write()
            .map_err(|_| "Failed to write to title")?;

        // Insert the title, use a match to avoid quoting the string given
        *title_writer = Some(match title {
            tera::Value::String(s) => s.to_string(),
            _ => title.to_string(),
        });

        // Drop the lock
        drop(title_writer);

        Ok(tera::Value::Null)
    }
}

/// Set fields of embeds
struct FieldFunction {
    state: Arc<InternalTemplateExecuteState>,
}

impl tera::Function for FieldFunction {
    fn call(
        &self,
        args: &std::collections::HashMap<String, tera::Value>,
    ) -> tera::Result<tera::Value> {
        let field_name = args.get("name").ok_or("Field name not provided")?;
        let field_value = args.get("value").ok_or("Field not found")?;

        // Inline defaults to false if unset
        let field_is_inline = args
            .get("inline")
            .map_or(false, |v| v.as_bool().unwrap_or(false));

        // Lock fields for writing
        let mut fields_writer = self
            .state
            .fields
            .write()
            .map_err(|_| "Failed to write to fields")?;

        let field_value_str = match field_value {
            tera::Value::String(s) => s.to_string(),
            _ => field_value.to_string(),
        };

        // Insert the field
        fields_writer.insert(field_name.to_string(), (field_value_str, field_is_inline));

        // Drop the lock
        drop(fields_writer);

        Ok(tera::Value::Null)
    }
}

#[allow(dead_code)]
struct StubFunction {}

impl tera::Function for StubFunction {
    fn call(
        &self,
        _args: &std::collections::HashMap<String, tera::Value>,
    ) -> tera::Result<tera::Value> {
        Ok(tera::Value::Null)
    }
}

/// Better title filter
struct BetterTitleFilter {}

impl tera::Filter for BetterTitleFilter {
    fn filter(
        &self,
        val: &tera::Value,
        _args: &std::collections::HashMap<String, tera::Value>,
    ) -> tera::Result<tera::Value> {
        let val = val.as_str().ok_or("Title not a string")?;

        let title = val
            .split('_')
            .map(|s| {
                let mut c = s.chars();
                match c.next() {
                    None => String::new(),
                    Some(f) => f.to_uppercase().chain(c).collect(),
                }
            })
            .collect::<Vec<String>>()
            .join(" ");

        Ok(tera::Value::String(
            title.to_string().to_uppercase().replace("_", " "),
        ))
    }
}

pub struct ExecutedTemplate {
    pub title: Option<String>,
    pub description: String,
    pub fields: indexmap::IndexMap<String, (String, bool)>,
}

/// Executes a template with the given context
pub async fn execute_template(
    tera: &mut Tera,
    context: Arc<tera::Context>,
) -> Result<ExecutedTemplate, base_data::Error> {
    let ites = Arc::new(InternalTemplateExecuteState::default());

    // Add title function
    tera.register_function(
        "title",
        TitleFunction {
            state: ites.clone(),
        },
    );

    // Add field function
    tera.register_function(
        "field",
        FieldFunction {
            state: ites.clone(),
        },
    );

    // Add bettertitle filter
    tera.register_filter("bettertitle", BetterTitleFilter {});

    // Render the template
    let rendered = tokio::time::timeout(
        TEMPLATE_EXECUTION_TIMEOUT,
        tera.render_async("main", &context),
    )
    .await
    .map_err(|_| "Template execution timed out")?;

    if let Err(e) = rendered {
        return Err(format!("Error: {}, Source: {:?}", e, e.source()).into());
    }

    let rendered = rendered.unwrap();

    // Read the outputted template specials
    let title_reader = ites.title.read().map_err(|_| "Failed to read title")?;
    let fields_reader = ites.fields.read().map_err(|_| "Failed to read fields")?;
    Ok(ExecutedTemplate {
        title: (*title_reader).clone(),
        description: rendered,
        fields: (*fields_reader).clone(),
    })
}

pub fn to_embed<'a>(executed_template: ExecutedTemplate) -> serenity::all::CreateEmbed<'a> {
    let mut embed = serenity::all::CreateEmbed::default();

    let mut total_chars: usize = 0;

    fn _get_char_limit(total_chars: usize, limit: usize, max_chars: usize) -> usize {
        if max_chars <= total_chars {
            return 0;
        }

        // If limit is 6000 and max_chars - total_chars is 1000, return 1000 etc.
        std::cmp::min(limit, max_chars - total_chars)
    }

    fn _slice_chars(s: &str, total_chars: &mut usize, limit: usize, max_chars: usize) -> String {
        let char_limit = _get_char_limit(*total_chars, limit, max_chars);

        if char_limit == 0 {
            return String::new();
        }

        *total_chars += char_limit;

        s.chars().take(char_limit).collect()
    }

    if let Some(title) = &executed_template.title {
        // Slice title to EMBED_TITLE_LIMIT
        embed = embed.title(_slice_chars(
            title,
            &mut total_chars,
            embed_limits::EMBED_TITLE_LIMIT,
            embed_limits::EMBED_TOTAL_LIMIT,
        ));
    }

    embed = embed.description(
        _slice_chars(
            &executed_template.description,
            &mut total_chars,
            embed_limits::EMBED_DESCRIPTION_LIMIT,
            embed_limits::EMBED_TOTAL_LIMIT,
        )
        .to_string(),
    );

    for (count, (name, (value, inline))) in executed_template.fields.into_iter().enumerate() {
        if count >= embed_limits::EMBED_FIELDS_MAX_COUNT {
            break;
        }

        // Slice field name to EMBED_FIELD_NAME_LIMIT
        let name = _slice_chars(
            &name,
            &mut total_chars,
            embed_limits::EMBED_FIELD_NAME_LIMIT,
            embed_limits::EMBED_TOTAL_LIMIT,
        );

        // Slice field value to EMBED_FIELD_VALUE_LIMIT
        let value = _slice_chars(
            &value,
            &mut total_chars,
            embed_limits::EMBED_FIELD_VALUE_LIMIT,
            embed_limits::EMBED_TOTAL_LIMIT,
        );

        embed = embed.field(name, value, inline);
    }

    embed
}
