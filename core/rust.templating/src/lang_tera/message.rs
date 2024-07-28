use crate::core::{slice_chars, DiscordReply};
use base_data::limits::{embed_limits, message_limits};
use gwevent::field::{CategorizedField, Field};
use std::sync::{Arc, RwLock};
use tera::Tera;

#[derive(Debug, Default)]
struct InternalTemplateEmbedsState {
    /// The title set by the template
    title: Option<String>,
    /// The description set by the template
    description: Option<String>,
    /// The fields that were set by the template
    fields: Vec<(String, String, bool)>,
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

        let field_name_str = match field_name {
            tera::Value::String(s) => s.to_string(),
            _ => field_name.to_string(),
        };

        let field_value_str = match field_value {
            tera::Value::String(s) => s.to_string(),
            _ => field_value.to_string(),
        };

        // Insert the field
        current_embed
            .fields
            .push((field_name_str, field_value_str, field_is_inline));

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
            fields: Vec::new(),
        });

        // Set the current embed index to embeds.len() - 1
        embeds.0 += embeds.1.len() - 1;

        Ok(tera::Value::Null)
    }
}

/// Field formatter
pub struct FieldFormatterFilter {
    /// Whether or not the template defaults to a CategorizedField versus a simple Field
    pub is_categorized_default: bool,
}

impl tera::Filter for FieldFormatterFilter {
    fn filter(
        &self,
        val: &tera::Value,
        args: &std::collections::HashMap<String, tera::Value>,
    ) -> tera::Result<tera::Value> {
        let is_categorized = args
            .get("is_categorized")
            .map_or(self.is_categorized_default, |x| {
                x.as_bool().unwrap_or(self.is_categorized_default)
            });

        if is_categorized {
            let field: CategorizedField = serde_json::from_value(val.clone())
                .map_err(|e| format!("Failed to parse categorized field: {:?}", e))?;

            let formatted = field
                .template_format()
                .map_err(|e| format!("Failed to format categorized field: {:?}", e))?;

            Ok(tera::Value::String(formatted))
        } else {
            let field: Field = serde_json::from_value(val.clone())
                .map_err(|e| format!("Failed to parse field: {:?}", e))?;

            let formatted = field
                .template_format()
                .map_err(|e| format!("Failed to format field: {:?}", e))?;

            Ok(tera::Value::String(formatted))
        }
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

#[derive(Debug, Default)]
pub struct TemplateEmbed {
    /// The title set by the template
    pub title: Option<String>,
    /// The description set by the template
    pub description: Option<String>,
    /// The fields that were set by the template
    pub fields: Vec<(String, String, bool)>,
}

pub struct ExecutedTemplate {
    /// The embeds that were set by the template
    embeds: Vec<TemplateEmbed>,
    /// What content to set on the message
    content: Option<String>,
}

impl ExecutedTemplate {
    pub fn to_discord_reply<'a>(self) -> Result<DiscordReply<'a>, base_data::Error> {
        let mut total_chars: usize = 0;
        let mut total_content_chars = 0;

        let mut embeds = Vec::new();
        for template_embed in self.embeds {
            let mut set = false; // Is something set on the embed?
            let mut embed = serenity::all::CreateEmbed::default();

            if let Some(title) = &template_embed.title {
                // Slice title to EMBED_TITLE_LIMIT
                embed = embed.title(slice_chars(
                    title,
                    &mut total_chars,
                    embed_limits::EMBED_TITLE_LIMIT,
                    embed_limits::EMBED_TOTAL_LIMIT,
                ));
                set = true;
            }

            if let Some(description) = &template_embed.description {
                // Slice description to EMBED_DESCRIPTION_LIMIT
                embed = embed.description(
                    slice_chars(
                        description,
                        &mut total_chars,
                        embed_limits::EMBED_DESCRIPTION_LIMIT,
                        embed_limits::EMBED_TOTAL_LIMIT,
                    )
                    .to_string(),
                );
                set = true;
            }

            if !template_embed.fields.is_empty() {
                set = true;
            }

            for (count, (name, value, inline)) in template_embed.fields.into_iter().enumerate() {
                if count >= embed_limits::EMBED_FIELDS_MAX_COUNT {
                    break;
                }

                let name = name.trim();
                let value = value.trim();

                if name.is_empty() || value.is_empty() {
                    continue;
                }

                // Slice field name to EMBED_FIELD_NAME_LIMIT
                let name = slice_chars(
                    name,
                    &mut total_chars,
                    embed_limits::EMBED_FIELD_NAME_LIMIT,
                    embed_limits::EMBED_TOTAL_LIMIT,
                );

                // Slice field value to EMBED_FIELD_VALUE_LIMIT
                let value = slice_chars(
                    value,
                    &mut total_chars,
                    embed_limits::EMBED_FIELD_VALUE_LIMIT,
                    embed_limits::EMBED_TOTAL_LIMIT,
                );

                embed = embed.field(name, value, inline);
            }

            if set {
                embeds.push(embed);
            }
        }

        // Now handle content
        let content = self.content.map(|c| {
            slice_chars(
                &c,
                &mut total_content_chars,
                message_limits::MESSAGE_CONTENT_LIMIT,
                message_limits::MESSAGE_CONTENT_LIMIT,
            )
        });

        if content.is_none() && embeds.is_empty() {
            return Err("No content or embeds set".into());
        }

        Ok(DiscordReply { embeds, content })
    }
}

/// Executes a template with the given context with the expectation that the template returns a message
pub async fn execute_template_for_message(
    tera: &mut Tera,
    args: crate::core::MessageTemplateContext,
) -> Result<ExecutedTemplate, base_data::Error> {
    let mut ctx = tera::Context::new();
    ctx.insert("args", &args)?;

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

    // Add gwevent templater
    tera.register_filter(
        "formatter__gwevent_field",
        FieldFormatterFilter {
            is_categorized_default: true,
        },
    );

    // Execute the template
    super::execute_template(tera, &ctx).await?;

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
