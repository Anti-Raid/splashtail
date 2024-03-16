use serenity::all::{UserId, ChannelId, RoleId, MessageId, GuildId, EmojiId};
use small_fixed_array::FixedString;
use serde::ser::SerializeStruct;

/// Contains variant with inner as default value
/// Should be serde serialized as {"type": "String", "default": "value"}
#[derive(Debug, Clone, PartialEq)]
pub enum WebFieldType {
    /// A string
    String(String),

    /// A user id
    User(UserId),

    /// A channel id
    Channel(ChannelId),

    /// A role id
    Role(RoleId),

    /// A message id
    Message(MessageId),

    /// A guild id
    Guild(GuildId),

    /// An emoji id
    Emoji(EmojiId),
}

// Macro to construct the serde serializer for WebFieldType
// This is a custom serializer because we want to serialize the enum as a struct
// with the type and default fields
// Macro to simplify the serialization process
#[macro_export]
macro_rules! serialize_web_field_type {
    ($serializer:expr, $variant:expr, $value:expr) => {
        {
            let mut state = $serializer.serialize_struct("WebFieldType", 2)?;
            state.serialize_field("type", $variant)?;
            state.serialize_field("default", $value)?;
            state.end()
        }
    };
}

// Custom serializer for WebFieldType
impl serde::Serialize for WebFieldType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        match self {
            WebFieldType::String(s) => serialize_web_field_type!(serializer, "String", s),
            WebFieldType::User(s) => serialize_web_field_type!(serializer, "User", s),
            WebFieldType::Channel(s) => serialize_web_field_type!(serializer, "Channel", s),
            WebFieldType::Role(s) => serialize_web_field_type!(serializer, "Role", s),
            WebFieldType::Message(s) => serialize_web_field_type!(serializer, "Message", s),
            WebFieldType::Guild(s) => serialize_web_field_type!(serializer, "Guild", s),
            WebFieldType::Emoji(s) => serialize_web_field_type!(serializer, "Emoji", s),
        }
    }
}

// Custom deserializer for WebFieldType
impl<'de> serde::Deserialize<'de> for WebFieldType {
    fn deserialize<D>(deserializer: D) -> Result<WebFieldType, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        let value = serde_json::Value::deserialize(deserializer)?;

        let field_type = value["type"].as_str().ok_or(serde::de::Error::custom("Missing type field"))?;
        let default = value["default"].as_str().ok_or(serde::de::Error::custom("Missing default field"))?;

        match field_type {
            "String" => Ok(WebFieldType::String(default.to_string())),
            "User" => Ok(WebFieldType::User(default.parse().map_err(serde::de::Error::custom)?)),
            "Channel" => Ok(WebFieldType::Channel(default.parse().map_err(serde::de::Error::custom)?)),
            "Role" => Ok(WebFieldType::Role(default.parse().map_err(serde::de::Error::custom)?)),
            "Message" => Ok(WebFieldType::Message(default.parse().map_err(serde::de::Error::custom)?)),
            "Guild" => Ok(WebFieldType::Guild(default.parse().map_err(serde::de::Error::custom)?)),
            "Emoji" => Ok(WebFieldType::Emoji(default.parse().map_err(serde::de::Error::custom)?)),
            _ => Err(serde::de::Error::custom("Invalid type field")),
        }
    }
}

impl From<String> for WebFieldType {
    fn from(s: String) -> Self {
        Self::String(s)
    }
}

impl From<FixedString<u16>> for WebFieldType {
    fn from(s: FixedString<u16>) -> Self {
        Self::String(s.to_string())
    }
}

impl From<FixedString<u32>> for WebFieldType {
    fn from(s: FixedString<u32>) -> Self {
        Self::String(s.to_string())
    }
}

impl From<UserId> for WebFieldType {
    fn from(s: UserId) -> Self {
        Self::User(s)
    }
}

impl From<ChannelId> for WebFieldType {
    fn from(s: ChannelId) -> Self {
        Self::Channel(s)
    }
}

impl From<RoleId> for WebFieldType {
    fn from(s: RoleId) -> Self {
        Self::Role(s)
    }
}

impl From<MessageId> for WebFieldType {
    fn from(s: MessageId) -> Self {
        Self::Message(s)
    }
}

impl From<GuildId> for WebFieldType {
    fn from(s: GuildId) -> Self {
        Self::Guild(s)
    }
}

impl From<EmojiId> for WebFieldType {
    fn from(s: EmojiId) -> Self {
        Self::Emoji(s)
    }
}

pub struct ConfigOption {
    /// The ID of the option
    pub id: String,
    /// The name of the option
    pub name: String,
    /// The description of the option
    pub field_type: WebFieldType,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    pub fn serialize_web_field_type() {
        let s = serde_json::to_string(&WebFieldType::String("test".to_string())).unwrap();
        assert_eq!(s, r#"{"type":"String","default":"test"}"#);
    }

    #[test]
    pub fn deserialize_web_field_type() {
        let s = r#"{"type":"String","default":"test"}"#;
        let deserialized: WebFieldType = serde_json::from_str(s).unwrap();
        assert_eq!(deserialized, WebFieldType::String("test".to_string()));
    }
}
