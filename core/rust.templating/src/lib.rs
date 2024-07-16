use base_data::limits::{embed_limits, message_limits};
use moka::future::Cache;
use once_cell::sync::Lazy;
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
struct InternalTemplateEmbedsState {
    /// The title set by the template
    title: Option<String>,
    /// The description set by the template
    description: Option<String>,
    /// The fields that were set by the template
    fields: indexmap::IndexMap<String, (String, bool)>,
}

#[derive(Debug, Default)]
struct InternalTemplateExecuteState {
    /// Embeds [current_index, embeds]
    embeds: RwLock<(usize, Vec<InternalTemplateEmbedsState>)>,
    /// What content to set on the message
    content: RwLock<Option<String>>,
}
// Set title of embed
struct EmbedTitleFunction {
    state: Arc<InternalTemplateExecuteState>,
}

// title(title="title")
impl tera::Function for EmbedTitleFunction {
    fn call(
        &self,
        args: &std::collections::HashMap<String, tera::Value>,
    ) -> tera::Result<tera::Value> {
        let title = args.get("title").ok_or("Title not provided")?;

        let mut embeds = self
            .state
            .embeds
            .write()
            .map_err(|_| "Failed to read embeds")?;

        if embeds.1.is_empty() {
            embeds.1.push(InternalTemplateEmbedsState::default()); // We do not increment the current embed here as it will be zero initially
        }

        let current_idx = embeds.0;
        let current_embed = embeds
            .1
            .get_mut(current_idx)
            .ok_or("Failed to get current embed")?;

        // Insert the title, use a match to avoid quoting the string given
        current_embed.title = Some(match title {
            tera::Value::String(s) => s.to_string(),
            _ => title.to_string(),
        });

        // Drop the lock
        drop(embeds);

        Ok(tera::Value::Null)
    }
}

/// Set fields of embeds
struct EmbedFieldFunction {
    state: Arc<InternalTemplateExecuteState>,
}

// field(name="name", value="value", inline=true/false)
impl tera::Function for EmbedFieldFunction {
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

        let mut embeds = self
            .state
            .embeds
            .write()
            .map_err(|_| "Failed to read embeds")?;

        if embeds.1.is_empty() {
            embeds.1.push(InternalTemplateEmbedsState::default()); // We do not increment the current embed here as it will be zero initially
        }

        let current_idx = embeds.0;
        let current_embed = embeds
            .1
            .get_mut(current_idx)
            .ok_or("Failed to get current embed")?;

        let field_value_str = match field_value {
            tera::Value::String(s) => s.to_string(),
            _ => field_value.to_string(),
        };

        // Insert the field
        current_embed
            .fields
            .insert(field_name.to_string(), (field_value_str, field_is_inline));

        Ok(tera::Value::Null)
    }
}

/// Set embed description, we use a filter here to make multiline embed descriptions easier
struct EmbedDescriptionFilter {
    state: Arc<InternalTemplateExecuteState>,
}

/// {% filter description %}
/// My description here
/// {% endfilter %}
impl tera::Filter for EmbedDescriptionFilter {
    fn filter(
        &self,
        value: &tera::Value,
        _args: &std::collections::HashMap<String, tera::Value>,
    ) -> tera::Result<tera::Value> {
        let text = match value {
            tera::Value::String(s) => Some(s.to_string()),
            tera::Value::Bool(true) => return Ok(tera::Value::Null), // Ignore true
            tera::Value::Bool(false) => None,
            tera::Value::Null => None,
            _ => Some(value.to_string()),
        };

        // Lock fields for writing
        let mut embeds_writer = self
            .state
            .embeds
            .write()
            .map_err(|_| "Failed to write to use_embed")?;

        // Set the state
        if embeds_writer.1.is_empty() {
            embeds_writer.1.push(InternalTemplateEmbedsState::default()); // We do not increment the current embed here as it will be zero initially
        }

        let current_idx = embeds_writer.0;
        let current_embed = embeds_writer
            .1
            .get_mut(current_idx)
            .ok_or("Failed to get current embed")?;

        // Insert the description
        current_embed.description = text;

        // Drop the lock
        drop(embeds_writer);

        Ok(tera::Value::Null)
    }
}

/// Set content of message, we use a filter here to make multiline content easier
struct ContentFilter {
    state: Arc<InternalTemplateExecuteState>,
}

/// {% filter content %}
/// My content here
/// {% endfilter %}
impl tera::Filter for ContentFilter {
    fn filter(
        &self,
        value: &tera::Value,
        _args: &std::collections::HashMap<String, tera::Value>,
    ) -> tera::Result<tera::Value> {
        let text = match value {
            tera::Value::String(s) => Some(s.to_string()),
            tera::Value::Bool(true) => return Ok(tera::Value::Null), // Ignore true
            tera::Value::Bool(false) => None,
            tera::Value::Null => None,
            _ => Some(value.to_string()),
        };

        // Lock fields for writing
        let mut content_writer = self
            .state
            .content
            .write()
            .map_err(|_| "Failed to write to use_embed")?;

        // Set the state
        *content_writer = text;

        // Drop the lock
        drop(content_writer);

        Ok(tera::Value::Null)
    }
}

// Add a new embed to the template
struct NewEmbedFunction {
    state: Arc<InternalTemplateExecuteState>,
}

// new_embed(title="" [optional], description="" [optional])
impl tera::Function for NewEmbedFunction {
    fn call(
        &self,
        args: &std::collections::HashMap<String, tera::Value>,
    ) -> tera::Result<tera::Value> {
        let title = args.get("title");
        let description = args.get("description");

        let mut embeds = self
            .state
            .embeds
            .write()
            .map_err(|_| "Failed to read embeds")?;

        // Add a new embed
        embeds.1.push(InternalTemplateEmbedsState {
            title: match title {
                Some(tera::Value::String(s)) => Some(s.to_string()),
                _ => None,
            },
            description: match description {
                Some(tera::Value::String(s)) => Some(s.to_string()),
                _ => None,
            },
            fields: indexmap::IndexMap::new(),
        });

        // Set the current embed index to embeds.len() - 1
        embeds.0 += embeds.1.len() - 1;

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

#[derive(Debug, Default)]
pub struct TemplateEmbed {
    /// The title set by the template
    pub title: Option<String>,
    /// The description set by the template
    pub description: Option<String>,
    /// The fields that were set by the template
    pub fields: indexmap::IndexMap<String, (String, bool)>,
}

pub struct ExecutedTemplate {
    embeds: Vec<TemplateEmbed>,
    /// What content to set on the message
    content: Option<String>,
}

impl ExecutedTemplate {
    pub fn to_discord_reply<'a>(self) -> DiscordReply<'a> {
        let mut total_chars: usize = 0;
        let mut total_content_chars = 0;

        fn _get_char_limit(total_chars: usize, limit: usize, max_chars: usize) -> usize {
            if max_chars <= total_chars {
                return 0;
            }

            // If limit is 6000 and max_chars - total_chars is 1000, return 1000 etc.
            std::cmp::min(limit, max_chars - total_chars)
        }

        fn _slice_chars(
            s: &str,
            total_chars: &mut usize,
            limit: usize,
            max_chars: usize,
        ) -> String {
            let char_limit = _get_char_limit(*total_chars, limit, max_chars);

            if char_limit == 0 {
                return String::new();
            }

            *total_chars += char_limit;

            s.chars().take(char_limit).collect()
        }

        let mut embeds = Vec::new();
        for template_embed in self.embeds {
            let mut embed = serenity::all::CreateEmbed::default();

            if let Some(title) = &template_embed.title {
                // Slice title to EMBED_TITLE_LIMIT
                embed = embed.title(_slice_chars(
                    title,
                    &mut total_chars,
                    embed_limits::EMBED_TITLE_LIMIT,
                    embed_limits::EMBED_TOTAL_LIMIT,
                ));
            }

            if let Some(description) = &template_embed.description {
                // Slice description to EMBED_DESCRIPTION_LIMIT
                embed = embed.description(
                    _slice_chars(
                        description,
                        &mut total_chars,
                        embed_limits::EMBED_DESCRIPTION_LIMIT,
                        embed_limits::EMBED_TOTAL_LIMIT,
                    )
                    .to_string(),
                );
            }

            for (count, (name, (value, inline))) in template_embed.fields.into_iter().enumerate() {
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

            embeds.push(embed);
        }

        // Now handle content
        let content = self.content.map(|c| {
            _slice_chars(
                &c,
                &mut total_content_chars,
                message_limits::MESSAGE_CONTENT_LIMIT,
                message_limits::MESSAGE_CONTENT_LIMIT,
            )
        });

        DiscordReply { embeds, content }
    }
}

/// Executes a template with the given context returning the resultant string
///
/// Note that for message templates, the `execute_template_for_message` function should be used instead
pub async fn execute_template(
    tera: &mut Tera,
    context: Arc<tera::Context>,
) -> Result<String, base_data::Error> {
    // Render the template
    Ok(tokio::time::timeout(
        TEMPLATE_EXECUTION_TIMEOUT,
        tera.render_async("main", &context),
    )
    .await
    .map_err(|_| "Template execution timed out")??)
}

/// Executes a template with the given context with the expectation that the template returns a message
pub async fn execute_template_for_message(
    tera: &mut Tera,
    context: Arc<tera::Context>,
) -> Result<ExecutedTemplate, base_data::Error> {
    let ites = Arc::new(InternalTemplateExecuteState::default());

    // Add embed_title function
    tera.register_function(
        "embed_title",
        EmbedTitleFunction {
            state: ites.clone(),
        },
    );

    // Add embed_field function
    tera.register_function(
        "embed_field",
        EmbedFieldFunction {
            state: ites.clone(),
        },
    );

    // Add embed_description filter
    tera.register_filter(
        "embed_description",
        EmbedDescriptionFilter {
            state: ites.clone(),
        },
    );

    // Add new_embed function
    tera.register_function(
        "new_embed",
        NewEmbedFunction {
            state: ites.clone(),
        },
    );

    // Add content filter
    tera.register_filter(
        "content",
        ContentFilter {
            state: ites.clone(),
        },
    );

    // Add bettertitle filter
    tera.register_filter("bettertitle", BetterTitleFilter {});

    // Execute the template
    execute_template(tera, context).await?;

    // Read the outputted template embeds
    let embeds_reader = ites.embeds.read().map_err(|_| "Failed to read embeds")?;
    let mut template_embeds = Vec::new();

    for embed in embeds_reader.1.iter() {
        template_embeds.push(TemplateEmbed {
            title: embed.title.clone(),
            description: embed.description.clone(),
            fields: embed.fields.clone(),
        });
    }

    // Add the rendered content to the content
    let content_reader = ites.content.read().map_err(|_| "Failed to read content")?;

    Ok(ExecutedTemplate {
        embeds: template_embeds,
        content: (*content_reader).clone(),
    })
}

#[derive(Default, serde::Serialize)]
/// A DiscordReply is guaranteed to map 1-1 to discords API
pub struct DiscordReply<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    pub embeds: Vec<serenity::all::CreateEmbed<'a>>,
}
