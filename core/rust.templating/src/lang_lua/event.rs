pub use mlua::prelude::*;
use std::{ops::Deref, sync::Arc};

pub enum ArcOrBox<T: ?Sized> {
    Arc(Arc<T>),
    Box(Box<T>),
}

impl Deref for ArcOrBox<dyn erased_serde::Serialize + Send + Sync> {
    type Target = dyn erased_serde::Serialize + Send + Sync;

    fn deref(&self) -> &Self::Target {
        match self {
            ArcOrBox::Arc(a) => a.as_ref(),
            ArcOrBox::Box(b) => b.as_ref(),
        }
    }
}

/// A create event struct that can be cloned and/or turned into an Event
pub struct CreateEventArc {
    pub title: String,
    pub base_name: String,
    pub name: String,
    pub data: Arc<dyn erased_serde::Serialize + Send + Sync>,
    pub is_deniable: bool,
}

impl CreateEventArc {
    /// Creates a new CreateEventArc with data already wrapped in an Arc
    pub fn new_arc<T: erased_serde::Serialize + Send + Sync + 'static>(
        title: String,
        base_name: String,
        name: String,
        data: Arc<T>,
        is_deniable: bool,
    ) -> Self {
        Self {
            title,
            base_name,
            name,
            data,
            is_deniable,
        }
    }

    /// Creates a new CreateEventArc with data that will be wrapped in an Arc
    pub fn new<T: erased_serde::Serialize + Send + Sync + 'static>(
        title: String,
        base_name: String,
        name: String,
        data: T,
        is_deniable: bool,
    ) -> Self {
        Self::new_arc(title, base_name, name, Arc::new(data), is_deniable)
    }

    pub fn into_event(&self) -> Event {
        Event::new(
            self.title.clone(),
            self.base_name.clone(),
            self.name.clone(),
            ArcOrBox::Arc(self.data.clone()),
            self.is_deniable,
        )
    }
}

/// An `Event` is an object that can be passed to a Lua template
pub struct Event {
    /// The title name of the event
    title: String,
    /// The name of the base event
    base_name: String,
    /// The name of the event
    name: String,
    /// The inner data of the object
    data: ArcOrBox<dyn erased_serde::Serialize + Send + Sync>,
    /// Whether or not further processing of the action that triggered the event can be denied
    is_deniable: bool,
    /// The random identifier of the event
    uid: sqlx::types::Uuid,
    /// The cached serialized value of the data
    cached_data: std::sync::Mutex<Option<LuaValue>>,
}

impl Event {
    /// Creates a new event using boxing
    pub fn new_boxed<T: erased_serde::Serialize + Send + Sync + 'static>(
        title: String,
        base_name: String,
        name: String,
        data: T,
        is_deniable: bool,
    ) -> Self {
        Self::new(
            title,
            base_name,
            name,
            ArcOrBox::Box(Box::new(data)),
            is_deniable,
        )
    }

    /// Creates a new event using an Arc
    pub fn new_arc<T: erased_serde::Serialize + Send + Sync + 'static>(
        title: String,
        base_name: String,
        name: String,
        data: Arc<T>,
        is_deniable: bool,
    ) -> Self {
        Self::new(title, base_name, name, ArcOrBox::Arc(data), is_deniable)
    }

    /// Create from ArcOrBox
    pub fn new(
        title: String,
        base_name: String,
        name: String,
        data: ArcOrBox<dyn erased_serde::Serialize + Send + Sync>,
        is_deniable: bool,
    ) -> Self {
        Self {
            title,
            base_name,
            name,
            data,
            is_deniable,
            uid: sqlx::types::Uuid::new_v4(),
            cached_data: std::sync::Mutex::new(None),
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
            let mut cached_data = this
                .cached_data
                .lock()
                .map_err(|e| LuaError::external(e.to_string()))?;

            if let Some(v) = cached_data.as_ref() {
                return Ok(v.clone());
            }

            log::info!("Event: Serializing data");
            let v = lua.to_value(&*this.data)?;

            *cached_data = Some(v.clone());

            Ok(v)
        });
        fields.add_field_method_get("is_deniable", |_, this| Ok(this.is_deniable));
        fields.add_field_method_get("uid", |lua, this| {
            let uid = lua.to_value(&this.uid)?;
            Ok(uid)
        });
    }
}
