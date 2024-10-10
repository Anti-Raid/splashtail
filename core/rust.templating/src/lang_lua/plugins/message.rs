use crate::core::messages::{Message, MessageEmbed, MessageEmbedField};
use gwevent::field::{CategorizedField, Field};
use mlua::prelude::*;

pub fn init_plugin(lua: &Lua) -> LuaResult<LuaTable> {
    let module = lua.create_table()?;

    module.set(
        "new_message",
        lua.create_function(|lua, ()| {
            let message = Message::default();
            lua.to_value(&message)
        })?,
    )?;
    module.set(
        "new_message_embed",
        lua.create_function(|lua, ()| {
            let embed = MessageEmbed::default();
            lua.to_value(&embed)
        })?,
    )?;

    module.set(
        "new_message_embed_field",
        lua.create_function(|lua, ()| {
            let field = MessageEmbedField::default();
            lua.to_value(&field)
        })?,
    )?;

    module.set(
        "format_gwevent_field",
        lua.create_function(|lua, (field,): (LuaValue,)| {
            let field: Field = lua.from_value(field)?;
            lua.to_value(&field.template_format().map_err(LuaError::external)?)
        })?,
    )?;

    module.set(
        "format_gwevent_categorized_field",
        lua.create_function(|lua, (field,): (LuaValue,)| {
            let cfield: CategorizedField = lua.from_value(field)?;
            lua.to_value(&cfield.template_format().map_err(LuaError::external)?)
        })?,
    )?;

    module.set_readonly(true); // Block any attempt to modify this table

    Ok(module)
}
