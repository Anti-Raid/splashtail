pub use mlua::prelude::*;

/// An `Event` is an object that can be passed to a Lua template
pub struct Event {
    /// The title name of the event
    title: String,
    /// The name of the event
    name: String,
    /// The inner data of the object
    data: Box<dyn erased_serde::Serialize + Send + Sync>,
    /// Whether or not further processing of the action that triggered the event can be denied
    is_deniable: bool,
    /// Template data associated with the event. Not always present for every template (e.g. template exec)
    guild_template: Option<crate::GuildTemplate>,
}

impl Event {
    /// Creates a new event
    pub fn new<T: erased_serde::Serialize + Send + Sync + 'static>(
        title: String,
        name: String,
        data: T,
        is_deniable: bool,
        guild_template: Option<crate::GuildTemplate>,
    ) -> Self {
        Self {
            title,
            name,
            data: Box::new(data),
            is_deniable,
            guild_template,
        }
    }
}

impl LuaUserData for Event {
    fn add_fields<F: LuaUserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("title", |lua, this| {
            let title = lua.to_value(&this.title)?;
            Ok(title)
        });
        fields.add_field_method_get("name", |lua, this| {
            let name = lua.to_value(&this.name)?;
            Ok(name)
        });
        fields.add_field_method_get("data", |lua, this| {
            let v = lua.to_value(&*this.data)?;
            Ok(v)
        });
        fields.add_field_method_get("is_deniable", |_, this| Ok(this.is_deniable));
        fields.add_field_method_get("guild_template", |lua, this| {
            let v = lua.to_value(&this.guild_template)?;
            Ok(v)
        });
    }
}
