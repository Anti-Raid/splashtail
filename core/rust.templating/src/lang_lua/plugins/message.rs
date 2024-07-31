use crate::core::{slice_chars, DiscordReply};
use base_data::limits::{embed_limits, message_limits};
use gwevent::field::{CategorizedField, Field};
use mlua::prelude::*;
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

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct MessageEmbed {
    /// The title set by the template
    pub title: Option<String>,
    /// The description set by the template
    pub description: Option<String>,
    /// The fields that were set by the template
    pub fields: Vec<MessageEmbedField>,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Message {
    /// Embeds [current_index, embeds]
    pub embeds: Vec<MessageEmbed>,
    /// What content to set on the message
    pub content: Option<String>,
}

pub fn to_discord_reply<'a>(message: Message) -> Result<DiscordReply<'a>, base_data::Error> {
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

pub fn init_plugin(lua: &Lua) -> LuaResult<LuaTable> {
    let module = lua.create_table()?;
    module.set(
        "new_message",
        lua.create_function(|_, ()| {
            let message = Message::default();

            Ok(match serde_json::to_string(&message) {
                Ok(s) => super::core::ArLuaResult(Ok(s)),
                Err(e) => super::core::ArLuaResult(Err(e.to_string())),
            })
        })?,
    )?;
    module.set(
        "new_message_embed",
        lua.create_function(|_, ()| {
            let embed = MessageEmbed::default();

            Ok(match serde_json::to_string(&embed) {
                Ok(s) => super::core::ArLuaResult(Ok(s)),
                Err(e) => super::core::ArLuaResult(Err(e.to_string())),
            })
        })?,
    )?;

    module.set(
        "new_message_embed_field",
        lua.create_function(|_, ()| {
            let field = MessageEmbedField::default();

            Ok(match serde_json::to_string(&field) {
                Ok(s) => super::core::ArLuaResult(Ok(s)),
                Err(e) => super::core::ArLuaResult(Err(e.to_string())),
            })
        })?,
    )?;

    module.set(
        "format_gwevent_field",
        lua.create_function(|lua, (field,): (LuaValue,)| {
            log::info!("format_gwevent_field: {:?}", field);

            // Cast it to a normal field
            let field = match lua.from_value::<Field>(field) {
                Ok(f) => f,
                Err(e) => {
                    return Ok(super::core::ArLuaResult(Err(format!(
                        "Failed to cast field: {}",
                        e
                    ))));
                }
            };

            Ok(match field.template_format() {
                Ok(s) => super::core::ArLuaResult(Ok(s)),
                Err(e) => super::core::ArLuaResult(Err(e.to_string())),
            })
        })?,
    )?;

    module.set(
        "format_gwevent_categorized_field",
        lua.create_function(|lua, (field,): (LuaValue,)| {
            log::info!("format_gwevent_categorized_field: {:?}", field);
            // Cast it to a normal field
            let cat_field = match lua.from_value::<CategorizedField>(field) {
                Ok(f) => f,
                Err(e) => {
                    return Ok(super::core::ArLuaResult(Err(format!(
                        "Failed to cast field: {}",
                        e
                    ))));
                }
            };

            Ok(match cat_field.template_format() {
                Ok(s) => super::core::ArLuaResult(Ok(s)),
                Err(e) => super::core::ArLuaResult(Err(e.to_string())),
            })
        })?,
    )?;

    Ok(module)
}
