pub use mlua::prelude::*;
use std::{ops::Deref, sync::Arc};

pub enum ArcOrNormal<T: Sized> {
    Arc(Arc<T>),
    Normal(T),
}

impl<T: Sized> Deref for ArcOrNormal<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        match self {
            ArcOrNormal::Arc(a) => a.as_ref(),
            ArcOrNormal::Normal(b) => &b,
        }
    }
}

impl<T: serde::Serialize> serde::Serialize for ArcOrNormal<T> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            ArcOrNormal::Arc(a) => serde::Serialize::serialize(a, serializer),
            ArcOrNormal::Normal(b) => serde::Serialize::serialize(b, serializer),
        }
    }
}

impl<'de, T: serde::de::Deserialize<'de>> serde::de::Deserialize<'de> for ArcOrNormal<T> {
    fn deserialize<D>(deserializer: D) -> Result<ArcOrNormal<T>, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let v = T::deserialize(deserializer)?;
        Ok(ArcOrNormal::Normal(v))
    }
}

impl<T: Clone> Clone for ArcOrNormal<T> {
    fn clone(&self) -> Self {
        match self {
            ArcOrNormal::Arc(a) => ArcOrNormal::Arc(a.clone()),
            ArcOrNormal::Normal(b) => ArcOrNormal::Normal(b.clone()),
        }
    }
}

/// An `Event` is an object that can be passed to a Lua template
#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct Event {
    /// The title name of the event
    title: String,
    /// The name of the base event
    base_name: String,
    /// The name of the event
    name: String,
    /// The inner data of the object
    data: ArcOrNormal<serde_json::Value>,
    /// Whether or not further processing of the action that triggered the event can be denied
    is_deniable: bool,
    /// The random identifier of the event
    uid: sqlx::types::Uuid,
    /// The author, if any, of the event
    author: Option<String>,
}

impl Event {
    /// Create a new Event
    pub fn new(
        title: String,
        base_name: String,
        name: String,
        data: ArcOrNormal<serde_json::Value>,
        is_deniable: bool,
        author: Option<String>,
    ) -> Self {
        Self {
            title,
            base_name,
            name,
            data,
            is_deniable,
            uid: sqlx::types::Uuid::new_v4(),
            author,
        }
    }
}

impl Event {
    /// Returns the base name of the event
    pub fn base_name(&self) -> &str {
        &self.base_name
    }

    /// Returns the name (NOT the base name) of the event
    pub fn name(&self) -> &str {
        &self.name
    }
}

impl LuaUserData for Event {
    fn add_fields<F: LuaUserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("title", |lua, this| {
            let title = lua.to_value(&this.title)?;
            Ok(title)
        });
        fields.add_field_method_get("base_name", |lua, this| {
            let base_name = lua.to_value(&this.base_name)?;
            Ok(base_name)
        });
        fields.add_field_method_get("name", |lua, this| {
            let name = lua.to_value(&this.name)?;
            Ok(name)
        });
        fields.add_field_method_get("data", |lua, this| {
            log::info!("Event: Serializing data");
            let v = lua.to_value(&this.data)?;
            Ok(v)
        });
        fields.add_field_method_get("is_deniable", |_, this| Ok(this.is_deniable));
        fields.add_field_method_get("uid", |lua, this| {
            let uid = lua.to_value(&this.uid)?;
            Ok(uid)
        });
        fields.add_field_method_get("author", |lua, this| {
            let author = lua.to_value(&this.author)?;
            Ok(author)
        });
    }
}
