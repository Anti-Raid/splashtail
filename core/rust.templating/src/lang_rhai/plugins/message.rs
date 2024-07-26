use crate::slice_chars;
use base_data::limits::{embed_limits, message_limits};
use rhai::plugin::*;
use serde::{Deserialize, Serialize};

#[export_module]
pub mod plugin {
    /// Represents an embed field
    #[derive(Serialize, Deserialize, Debug, Default, Clone)]
    pub struct MessageEmbedField {
        /// The name of the field
        pub name: String,
        /// The value of the field
        pub value: String,
        /// Whether the field is inline
        pub inline: bool,
    }

    /// Creates a new field
    #[rhai_fn(name = "new_field")]
    pub fn new_field(name: &str, value: &str, inline: bool) -> MessageEmbedField {
        MessageEmbedField {
            name: name.to_string(),
            value: value.to_string(),
            inline,
        }
    }

    #[derive(Serialize, Deserialize, Debug, Default, Clone)]
    pub struct MessageEmbed {
        /// The title set by the template
        pub title: Option<String>,
        /// The description set by the template
        pub description: Option<String>,
        /// The fields that were set by the template
        pub fields: Vec<MessageEmbedField>,
    }

    /// Creates a new empty embed
    #[rhai_fn(name = "new_embed")]
    pub fn new_embed() -> MessageEmbed {
        MessageEmbed::default()
    }

    /// Sets the title of an embed
    #[rhai_fn(name = "set_title", global)]
    pub fn set_title(embed: &mut MessageEmbed, title: &str) -> Result<(), String> {
        if title.is_empty() {
            return Err("Title cannot be empty".to_string());
        }
        embed.title = Some(title.to_string());

        Ok(())
    }

    /// Sets the description of an embed
    #[rhai_fn(name = "set_description", global)]
    pub fn set_description(embed: &mut MessageEmbed, description: &str) -> Result<(), String> {
        if description.is_empty() {
            return Err("Description cannot be empty".to_string());
        }
        embed.description = Some(description.to_string());

        Ok(())
    }

    /// Returns the length of the fields in an embed
    #[rhai_fn(name = "field_count", pure)]
    pub fn field_count(embed: &mut MessageEmbed) -> usize {
        embed.fields.len()
    }

    /// Takes out the fields from an embed
    ///
    /// Note that this *removes* the fields from the embed
    /// and returns it to the caller. It is up to the caller to then
    /// set the fields back on the embed if needed.
    #[rhai_fn(name = "take_fields", global)]
    pub fn take_fields(embed: &mut MessageEmbed) -> Vec<MessageEmbedField> {
        std::mem::take(&mut embed.fields)
    }

    /// Appends a field to an embed
    #[rhai_fn(name = "add_field", global)]
    pub fn add_field(embed: &mut MessageEmbed, field: MessageEmbedField) -> Result<(), String> {
        if embed.fields.len() >= embed_limits::EMBED_FIELDS_MAX_COUNT {
            return Err("Maximum number of fields exceeded".to_string());
        }
        embed.fields.push(field);
        Ok(())
    }

    /// Inserts a single field to an embed at an index
    #[rhai_fn(name = "insert_field", global)]
    pub fn insert_field(
        embed: &mut MessageEmbed,
        index: i64,
        field: MessageEmbedField,
    ) -> Result<(), String> {
        if embed.fields.len() >= embed_limits::EMBED_FIELDS_MAX_COUNT {
            return Err("Maximum number of fields exceeded".to_string());
        }

        let index = usize::try_from(index).map_err(|_| "Index out of bounds".to_string())?;

        if index > embed.fields.len() {
            return Err("Index out of bounds".to_string());
        }

        embed.fields.insert(index, field);
        Ok(())
    }

    /// Removes a field from an embed by index returning the field
    #[rhai_fn(name = "remove_field", global)]
    pub fn remove_field(
        embed: &mut MessageEmbed,
        index: usize,
    ) -> Result<MessageEmbedField, String> {
        if index >= embed.fields.len() {
            return Err("Index out of bounds".to_string());
        }

        Ok(embed.fields.remove(index))
    }

    #[derive(Debug, Default, Clone)]
    pub struct Message {
        /// Embeds [current_index, embeds]
        pub embeds: Vec<MessageEmbed>,
        /// What content to set on the message
        pub content: Option<String>,
    }

    /// Creates a new empty message
    #[rhai_fn(name = "new_message")]
    pub fn new_message() -> Message {
        Message::default()
    }

    /// Sets the content of a message
    #[rhai_fn(name = "set_content", global)]
    pub fn set_content(message: &mut Message, content: &str) -> Result<(), String> {
        if content.is_empty() {
            return Err("Content cannot be empty".to_string());
        }
        message.content = Some(content.to_string());

        Ok(())
    }

    /// Takes out the embeds from a message
    /// Note that this *removes* the embeds from the message
    /// and returns it to the caller. It is up to the caller to then
    /// set the embeds back on the message if needed.
    #[rhai_fn(name = "take_embeds", global)]
    pub fn take_embeds(message: &mut Message) -> Vec<MessageEmbed> {
        std::mem::take(&mut message.embeds)
    }

    /// Appends an embed to a message
    /// Returns an error if the maximum number of embeds is exceeded
    #[rhai_fn(name = "add_embed", global)]
    pub fn add_embed(message: &mut Message, embed: MessageEmbed) -> Result<(), String> {
        if message.embeds.len() >= embed_limits::EMBED_MAX_COUNT {
            return Err("Maximum number of embeds exceeded".to_string());
        }
        message.embeds.push(embed);
        Ok(())
    }

    /// Inserts an embed to a message at an index
    /// Returns an error if the index is out of bounds
    #[rhai_fn(name = "insert_embed", global)]
    pub fn insert_embed(
        message: &mut Message,
        index: i64,
        embed: MessageEmbed,
    ) -> Result<(), String> {
        if message.embeds.len() >= embed_limits::EMBED_MAX_COUNT {
            return Err("Maximum number of embeds exceeded".to_string());
        }

        let index = usize::try_from(index).map_err(|_| "Index out of bounds".to_string())?;

        if index > message.embeds.len() {
            return Err("Index out of bounds".to_string());
        }

        message.embeds.insert(index, embed);

        Ok(())
    }

    /// Removes an index from a message by index
    #[rhai_fn(name = "remove_embed", global)]
    pub fn remove_embed(message: &mut Message, index: usize) -> Result<MessageEmbed, String> {
        if index >= message.embeds.len() {
            return Err("Index out of bounds".to_string());
        }
        Ok(message.embeds.remove(index))
    }
}

#[derive(Default, serde::Serialize)]
/// A DiscordReply is guaranteed to map 1-1 to discords API
pub struct DiscordReply<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    pub embeds: Vec<serenity::all::CreateEmbed<'a>>,
}

pub fn to_discord_reply<'a>(
    message: plugin::Message,
) -> Result<DiscordReply<'a>, base_data::Error> {
    let mut total_chars = 0;
    let mut total_content_chars = 0;
    let mut embeds = Vec::new();
    for template_embed in message.embeds {
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

        for (count, field) in template_embed.fields.into_iter().enumerate() {
            if count >= embed_limits::EMBED_FIELDS_MAX_COUNT {
                break;
            }

            let name = field.name.trim();
            let value = field.value.trim();

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

            embed = embed.field(name, value, field.inline);
        }

        if set {
            embeds.push(embed);
        }
    }

    // Now handle content
    let content = message.content.map(|c| {
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

#[cfg(test)]
mod tests {
    use super::*;
    use rhai::module_resolvers::StaticModuleResolver;

    #[test]
    fn test_message() {
        let mut engine = Engine::new();
        let module = exported_module!(plugin);

        let mut resolver = StaticModuleResolver::new();
        resolver.insert("message", module);

        engine.set_module_resolver(resolver);

        // Add the event object for testing to scope
        let mut scope = rhai::Scope::new();

        let dyn_val: rhai::Dynamic = serde_json::from_value(serde_json::json!({
            "a": 123,
            "b": "c"
        }))
        .unwrap();

        scope.set_value("a", dyn_val);

        // a is now defined in the template as the object map #{"a": 123, "b": "c"}

        let script = r#"import "message" as message;
            let message = message::new_message();
            message.set_content("Hello, World!");

            // Embeds
            let cool_embed = message::new_embed();
            cool_embed.set_title("Cool Embed");
            cool_embed.set_description("This is a cool embed");
            // Add some fields
            cool_embed.add_field(message::new_field("Field 1", "Value 1", false));
            cool_embed.add_field(message::new_field("Field 2", "Value 2", true));
            // Add to message
            message.add_embed(cool_embed);
            message
        "#;

        let result: plugin::Message = engine.eval_with_scope(&mut scope, script).unwrap();

        assert_eq!(result.content, Some("Hello, World!".to_string()));
        assert_eq!(
            format!("{:?}", result),
            r#"Message { embeds: [MessageEmbed { title: Some("Cool Embed"), description: Some("This is a cool embed"), fields: [MessageEmbedField { name: "Field 1", value: "Value 1", inline: false }, MessageEmbedField { name: "Field 2", value: "Value 2", inline: true }] }], content: Some("Hello, World!") }"#
        );
    }
}
