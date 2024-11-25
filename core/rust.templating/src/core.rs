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
    pub struct CreateMessageEmbedField {
        /// The name of the field
        pub name: String,
        /// The value of the field
        pub value: String,
        /// Whether the field is inline
        pub inline: bool,
    }

    /// Represents an embed author
    #[derive(Serialize, Deserialize, Debug, Default, Clone)]
    pub struct CreateMessageEmbedAuthor {
        /// The name of the author
        pub name: String,
        /// The URL of the author, must be a valid URL
        pub url: Option<String>,
        /// The icon URL of the author, must be a valid URL
        pub icon_url: Option<String>,
    }

    /// Represents an embed footer
    #[derive(Serialize, Deserialize, Debug, Default, Clone)]
    pub struct CreateMessageEmbedFooter {
        /// The text of the footer
        pub text: String,
        /// The icon URL of the footer, must be a valid URL
        pub icon_url: Option<String>,
    }

    /// Represents a message embed
    #[derive(Serialize, Deserialize, Debug, Default, Clone)]
    pub struct CreateMessageEmbed {
        /// The title set by the template
        pub title: Option<String>,
        /// The description set by the template
        pub description: Option<String>,
        /// The URL the embed should link to
        pub url: Option<String>,
        /// The timestamp to display on the embed
        pub timestamp: Option<String>,
        /// The color of the embed
        pub color: Option<serenity::all::Color>,
        /// The footer of the embed
        pub footer: Option<CreateMessageEmbedFooter>,
        /// The image URL for the embed
        pub image: Option<String>,
        /// The thumbnail URL for the embed
        pub thumbnail: Option<String>,
        /// The author of the embed
        pub author: Option<CreateMessageEmbedAuthor>,
        /// The fields that were set by the template
        pub fields: Option<Vec<CreateMessageEmbedField>>,
    }

    /// Message attachment
    #[derive(Serialize, Deserialize, Debug, Default, Clone)]
    pub struct CreateMessageAttachment {
        pub filename: String,
        pub description: Option<String>,
        pub content: Vec<u8>,
    }

    /// Represents a message that can be created by templates
    #[derive(Serialize, Deserialize, Debug, Default, Clone)]
    pub struct CreateMessage {
        /// Embeds [current_index, embeds]
        pub embeds: Option<Vec<CreateMessageEmbed>>,
        /// What content to set on the message
        pub content: Option<String>,
        /// The attachments
        pub attachments: Option<Vec<CreateMessageAttachment>>,
    }

    /// Converts a templated message to a discord reply
    ///
    /// This method also handles all of the various discord message+embed limits as well, returning an error if unable to comply
    pub fn to_discord_reply<'a>(message: CreateMessage) -> Result<DiscordReply<'a>, crate::Error> {
        let mut total_chars = 0;
        let mut total_content_chars = 0;
        let mut embeds = Vec::new();

        if let Some(t_embeds) = message.embeds {
            for template_embed in t_embeds {
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

                if let Some(url) = &template_embed.url {
                    if url.is_empty() {
                        return Err("URL cannot be empty".into());
                    }

                    if !url.starts_with("http://") && !url.starts_with("https://") {
                        return Err("URL must start with http:// or https://".into());
                    }

                    embed = embed.url(url.clone());
                    set = true;
                }

                if let Some(timestamp) = &template_embed.timestamp {
                    let timestamp = chrono::DateTime::parse_from_rfc3339(timestamp)
                        .map_err(|e| format!("Invalid timestamp provided to embed: {}", e))?;
                    embed = embed.timestamp(timestamp);
                    set = true;
                }

                if let Some(color) = template_embed.color {
                    embed = embed.color(color);
                    set = true;
                }

                if let Some(footer) = &template_embed.footer {
                    let text = slice_chars(
                        &footer.text,
                        &mut total_chars,
                        embed_limits::EMBED_FOOTER_TEXT_LIMIT,
                        embed_limits::EMBED_TOTAL_LIMIT,
                    );

                    let mut cef = serenity::all::CreateEmbedFooter::new(text);

                    if let Some(footer_icon_url) = &footer.icon_url {
                        if footer_icon_url.is_empty() {
                            return Err("Footer icon URL cannot be empty".into());
                        }

                        if !footer_icon_url.starts_with("http://")
                            && !footer_icon_url.starts_with("https://")
                        {
                            return Err(
                                "Footer icon URL must start with http:// or https://".into()
                            );
                        }

                        cef = cef.icon_url(footer_icon_url.clone());
                    }

                    embed = embed.footer(cef);

                    set = true;
                }

                if let Some(image) = &template_embed.image {
                    if image.is_empty() {
                        return Err("Image URL cannot be empty".into());
                    }

                    if !image.starts_with("http://") && !image.starts_with("https://") {
                        return Err("Image URL must start with http:// or https://".into());
                    }

                    embed = embed.image(image.clone());
                    set = true;
                }

                if let Some(thumbnail) = &template_embed.thumbnail {
                    if thumbnail.is_empty() {
                        return Err("Thumbnail URL cannot be empty".into());
                    }

                    if !thumbnail.starts_with("http://") && !thumbnail.starts_with("https://") {
                        return Err("Thumbnail URL must start with http:// or https://".into());
                    }

                    embed = embed.thumbnail(thumbnail.clone());
                    set = true;
                }

                if let Some(author) = &template_embed.author {
                    let name = slice_chars(
                        &author.name,
                        &mut total_chars,
                        embed_limits::EMBED_AUTHOR_NAME_LIMIT,
                        embed_limits::EMBED_TOTAL_LIMIT,
                    );

                    let mut cea = serenity::all::CreateEmbedAuthor::new(name);

                    if let Some(url) = &author.url {
                        if url.is_empty() {
                            return Err("Author URL cannot be empty".into());
                        }

                        if !url.starts_with("http://") && !url.starts_with("https://") {
                            return Err("Author URL must start with http:// or https://".into());
                        }

                        cea = cea.url(url.clone());
                    }

                    if let Some(icon_url) = &author.icon_url {
                        if icon_url.is_empty() {
                            return Err("Author icon URL cannot be empty".into());
                        }

                        if !icon_url.starts_with("http://") && !icon_url.starts_with("https://") {
                            return Err(
                                "Author icon URL must start with http:// or https://".into()
                            );
                        }

                        cea = cea.icon_url(icon_url.clone());
                    }

                    embed = embed.author(cea);

                    set = true;
                }

                if let Some(fields) = template_embed.fields {
                    if !fields.is_empty() {
                        set = true;
                    }

                    for (count, field) in fields.into_iter().enumerate() {
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
                }

                if set {
                    embeds.push(embed);
                }
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

        // Lastly handle attachments
        let mut attachments = Vec::new();

        if let Some(attach) = message.attachments {
            if attach.len() > message_limits::MESSAGE_MAX_ATTACHMENT_COUNT {
                return Err(format!(
                    "Too many attachments, limit is {}",
                    message_limits::MESSAGE_MAX_ATTACHMENT_COUNT
                )
                .into());
            }

            for attachment in attach {
                let desc = attachment.description.unwrap_or_default();
                if desc.len() > message_limits::MESSAGE_ATTACHMENT_DESCRIPTION_LIMIT {
                    return Err(format!(
                        "Attachment description exceeds limit of {}",
                        message_limits::MESSAGE_ATTACHMENT_DESCRIPTION_LIMIT
                    )
                    .into());
                }

                let content = attachment.content;

                if content.is_empty() {
                    return Err("Attachment content cannot be empty".into());
                }

                if content.len() > message_limits::MESSAGE_ATTACHMENT_CONTENT_BYTES_LIMIT {
                    return Err(format!(
                        "Attachment content exceeds limit of {} bytes",
                        message_limits::MESSAGE_ATTACHMENT_CONTENT_BYTES_LIMIT
                    )
                    .into());
                }

                let mut ca = serenity::all::CreateAttachment::bytes(content, attachment.filename);

                if !desc.is_empty() {
                    ca = ca.description(desc);
                }

                attachments.push(ca);
            }
        }

        if content.is_none() && embeds.is_empty() && attachments.is_empty() {
            return Err("No content/embeds/attachments set".into());
        }

        Ok(DiscordReply {
            embeds,
            content,
            attachments,
        })
    }

    #[derive(Default)]
    pub struct DiscordReply<'a> {
        pub content: Option<String>,
        pub embeds: Vec<serenity::all::CreateEmbed<'a>>,
        pub attachments: Vec<serenity::all::CreateAttachment<'a>>,
    }

    impl<'a> DiscordReply<'a> {
        pub fn to_create_message(self) -> serenity::all::CreateMessage<'a> {
            let mut message = serenity::all::CreateMessage::default();

            if let Some(content) = self.content {
                message = message.content(content);
            }

            message = message.embeds(self.embeds);

            for attachment in self.attachments {
                message = message.add_file(attachment);
            }

            message
        }

        pub fn to_edit_message(self) -> serenity::all::EditMessage<'a> {
            let mut message = serenity::all::EditMessage::default();

            if let Some(content) = self.content {
                message = message.content(content);
            }

            message = message.embeds(self.embeds);

            // NOTE: This resets old attachments
            for attachment in self.attachments {
                message = message.new_attachment(attachment);
            }

            message
        }
    }
}

pub mod captcha {
    #[derive(serde::Serialize, serde::Deserialize)]
    pub struct Captcha {
        pub text: String,
        pub content: Option<String>, // Message content
        pub image: Option<Vec<u8>>,  // Image data
    }
}

pub mod templating_core {
    const MAX_CAPS: usize = 50;
    const MAX_PRAGMA_SIZE: usize = 2048;

    use std::str::FromStr;

    use silverpelt::Error;

    #[derive(Clone, serde::Serialize, serde::Deserialize, Default)]
    pub struct TemplatePragma {
        pub lang: TemplateLanguage,

        #[serde(default)]
        pub allowed_caps: Vec<String>,

        #[serde(flatten)]
        pub extra_info: indexmap::IndexMap<String, serde_json::Value>,
    }

    impl TemplatePragma {
        pub fn parse(template: &str) -> Result<(&str, Self), Error> {
            let (first_line, rest) = match template.find('\n') {
                Some(i) => template.split_at(i),
                None => return Ok((template, Self::default())),
            };

            // Unravel any comments before the @pragma
            let first_line = first_line.trim_start_matches("--").trim();

            if !first_line.contains("@pragma ") {
                return Ok((template, Self::default()));
            }

            // Remove out the @pragma and serde parse it
            let first_line = first_line.replace("@pragma ", "");

            if first_line.as_bytes().len() > MAX_PRAGMA_SIZE {
                return Err("Pragma too large".into());
            }

            let pragma: TemplatePragma = serde_json::from_str(&first_line)?;

            if pragma.allowed_caps.len() > MAX_CAPS {
                return Err("Too many allowed capabilities specified".into());
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

    /// Parses a shop template of form template_name#version
    pub fn parse_shop_template(s: &str) -> Result<(String, String), Error> {
        let s = s.trim_start_matches("$shop/");
        let (template, version) = match s.split_once('#') {
            Some((template, version)) => (template, version),
            None => return Err("Invalid shop template".into()),
        };

        Ok((template.to_string(), version.to_string()))
    }

    /// Creates a shop template string given name and version
    pub fn create_shop_template(template: &str, version: &str) -> String {
        format!("$shop/{}#{}", template, version)
    }

    #[derive(serde::Deserialize, serde::Serialize, Clone, Debug)]
    pub struct GuildTemplate {
        pub name: String,
        pub description: Option<String>,
        pub shop_name: Option<String>,
        pub events: Option<Vec<String>>,
        pub content: String,
        pub created_by: String,
        pub created_at: chrono::DateTime<chrono::Utc>,
        pub updated_by: String,
        pub updated_at: chrono::DateTime<chrono::Utc>,
    }

    #[derive(Clone, Hash, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
    #[serde(tag = "type")]
    pub enum Template {
        Raw(String),
        Named(String),
    }
}

pub mod page {
    pub const MAX_PAGE_ID_LENGTH: usize = 128;

    pub struct Page {
        pub page_id: String,
        pub title: String,
        pub description: String,
        pub template: crate::Template,
        pub settings: Vec<ar_settings::types::Setting>,
    }
}
