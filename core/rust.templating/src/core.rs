pub fn get_char_limit(total_chars: usize, limit: usize, max_chars: usize) -> usize {
    if max_chars <= total_chars {
        return 0;
    }

    // If limit is 6000 and max_chars - total_chars is 1000, return 1000 etc.
    std::cmp::min(limit, max_chars - total_chars)
}

pub fn slice_chars(s: &str, total_chars: &mut usize, limit: usize, max_chars: usize) -> String {
    let char_limit = get_char_limit(*total_chars, limit, max_chars);

    if char_limit == 0 {
        return String::new();
    }

    if s.len() > char_limit {
        *total_chars += char_limit;
        s.chars().take(char_limit).collect()
    } else {
        *total_chars += s.len();
        s.to_string()
    }
}

pub mod messages {
    use super::slice_chars;
    use limits::{embed_limits, message_limits};
    use serde::{Deserialize, Serialize};

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

    /// Represents a message embed
    #[derive(Serialize, Deserialize, Debug, Default, Clone)]
    pub struct MessageEmbed {
        /// The title set by the template
        pub title: Option<String>,
        /// The description set by the template
        pub description: Option<String>,
        /// The fields that were set by the template
        pub fields: Vec<MessageEmbedField>,
    }

    /// Represents a message that can be created by templates
    #[derive(Serialize, Deserialize, Debug, Default, Clone)]
    pub struct Message {
        /// Embeds [current_index, embeds]
        pub embeds: Vec<MessageEmbed>,
        /// What content to set on the message
        pub content: Option<String>,
    }

    /// Converts a templated message to a discord reply
    ///
    /// This method also handles all of the various discord message+embed limits as well, returning an error if unable to comply
    pub fn to_discord_reply<'a>(message: Message) -> Result<DiscordReply<'a>, crate::Error> {
        let mut total_chars = 0;
        let mut total_content_chars = 0;
        let mut embeds = Vec::new();
        for template_embed in message.embeds {
            if embeds.len() >= embed_limits::EMBED_MAX_COUNT {
                break;
            }

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

    #[derive(Default, serde::Serialize)]
    /// A DiscordReply is guaranteed to map 1-1 to discords API
    pub struct DiscordReply<'a> {
        #[serde(skip_serializing_if = "Option::is_none")]
        pub content: Option<String>,
        pub embeds: Vec<serenity::all::CreateEmbed<'a>>,
    }
}
