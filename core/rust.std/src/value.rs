use sqlx::{postgres::PgRow, postgres::PgTypeKind, Column, Row, TypeInfo};
use std::hash::{Hash, Hasher};

#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
/// Represents a supported value type
pub enum Value {
    /// A uuid value
    Uuid(sqlx::types::Uuid),

    /// A string value
    String(String),

    /// A timestamp value
    Timestamp(chrono::NaiveDateTime),

    /// A timestamp value with timezone
    TimestampTz(chrono::DateTime<chrono::Utc>),

    /// An interval value
    Interval(chrono::Duration),

    /// An integer value
    Integer(i64),

    /// A float value
    Float(f64),

    /// A boolean value
    Boolean(bool),

    /// A list of values
    List(Vec<Value>),

    /// A (indexmap) of values
    Map(indexmap::IndexMap<String, Value>),

    /// Raw JSON data
    Json(serde_json::Value),

    /// None
    None,
}

impl Hash for Value {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Value::Uuid(u) => u.hash(state),
            Value::String(s) => s.hash(state),
            Value::Timestamp(t) => t.hash(state),
            Value::TimestampTz(t) => t.hash(state),
            Value::Interval(i) => i.hash(state),
            Value::Integer(i) => i.hash(state),
            Value::Float(f) => f.to_bits().hash(state),
            Value::Boolean(b) => b.hash(state),
            Value::List(l) => l.hash(state),
            Value::Map(m) => {
                for (k, v) in m {
                    k.hash(state);
                    v.hash(state);
                }
            }
            Value::Json(j) => j.to_string().hash(state),
            Value::None => None::<u8>.hash(state),
        }
    }
}

impl Value {
    /// Convert the Value to a serde_json::Value
    #[allow(dead_code)]
    pub fn to_json(&self) -> serde_json::Value {
        match self {
            Value::Uuid(u) => serde_json::Value::String(u.to_string()),
            Value::String(s) => serde_json::Value::String(s.clone()),
            Value::Timestamp(t) => serde_json::Value::String(t.to_string()),
            Value::TimestampTz(t) => serde_json::Value::String(t.to_string()),
            Value::Integer(i) => serde_json::Value::Number(serde_json::Number::from(*i)),
            Value::Interval(i) => {
                serde_json::Value::Number(serde_json::Number::from(i.num_seconds()))
            }
            Value::Float(f) => serde_json::Value::Number(
                serde_json::Number::from_f64(*f).unwrap_or(serde_json::Number::from(0)),
            ),
            Value::Boolean(b) => serde_json::Value::Bool(*b),
            Value::List(l) => serde_json::Value::Array(l.iter().map(|v| v.to_json()).collect()),
            Value::Map(m) => {
                let mut obj = serde_json::Map::new();
                for (k, v) in m {
                    obj.insert(k.clone(), v.to_json());
                }
                serde_json::Value::Object(obj)
            }
            Value::Json(j) => j.clone(),
            Value::None => serde_json::Value::Null,
        }
    }

    /// Convert a serde_json::Value to a Value
    #[allow(dead_code)]
    pub fn from_json(value: &serde_json::Value) -> Self {
        match value {
            serde_json::Value::String(s) => {
                let t = chrono::NaiveDateTime::parse_from_str(&s, "%Y-%m-%d %H:%M:%S");

                if let Ok(t) = t {
                    return Self::Timestamp(t);
                }

                let value = chrono::DateTime::parse_from_rfc3339(&s);

                if let Ok(value) = value {
                    return Self::TimestampTz(value.into());
                }

                Self::String(s.clone())
            }
            serde_json::Value::Number(n) => {
                if n.is_i64() {
                    Self::Integer(n.as_i64().unwrap())
                } else {
                    Self::Float(n.as_f64().unwrap())
                }
            }
            serde_json::Value::Bool(b) => Self::Boolean(*b),
            serde_json::Value::Array(a) => Self::List(a.iter().map(Value::from_json).collect()),
            serde_json::Value::Object(o) => {
                let mut m = indexmap::IndexMap::new();
                for (k, v) in o {
                    m.insert(k.clone(), Self::from_json(v));
                }
                Self::Map(m)
            }
            serde_json::Value::Null => Self::None,
        }
    }

    /// Converts a PgColumn to a Value
    #[allow(dead_code)]
    pub fn from_sqlx(row: &PgRow, index: usize) -> Result<Self, crate::Error> {
        let typ_info = row.try_column(index)?.type_info();
        let typ = typ_info.name().to_lowercase();

        match typ_info.kind() {
            PgTypeKind::Simple => match typ.as_str() {
                "text" | "citext" => {
                    let Some(v) = row.try_get::<Option<String>, _>(index)? else {
                        return Ok(Value::None);
                    };
                    Ok(Value::String(v))
                }
                "int4" => {
                    let Some(v) = row.try_get::<Option<i32>, _>(index)? else {
                        return Ok(Value::None);
                    };
                    Ok(Value::Integer(v.into()))
                }
                "int8" | "bigint" => {
                    let Some(v) = row.try_get::<Option<i64>, _>(index)? else {
                        return Ok(Value::None);
                    };
                    Ok(Value::Integer(v))
                }
                "float4" => {
                    let Some(v) = row.try_get::<Option<f32>, _>(index)? else {
                        return Ok(Value::None);
                    };
                    Ok(Value::Float(v as f64))
                }
                "float8" => {
                    let Some(v) = row.try_get::<Option<f64>, _>(index)? else {
                        return Ok(Value::None);
                    };
                    Ok(Value::Float(v))
                }
                "bool" => {
                    let Some(v) = row.try_get::<Option<bool>, _>(index)? else {
                        return Ok(Value::None);
                    };
                    Ok(Value::Boolean(v))
                }
                "json" | "jsonb" => {
                    let Some(v) = row.try_get::<Option<serde_json::Value>, _>(index)? else {
                        return Ok(Value::None);
                    };
                    Ok(Value::from_json(&v))
                }
                "uuid" => {
                    let Some(v) = row.try_get::<Option<sqlx::types::Uuid>, _>(index)? else {
                        return Ok(Value::None);
                    };
                    Ok(Value::Uuid(v))
                }
                "timestamp" => {
                    let Some(v) =
                        row.try_get::<Option<sqlx::types::chrono::NaiveDateTime>, _>(index)?
                    else {
                        return Ok(Value::None);
                    };
                    Ok(Value::Timestamp(v))
                }
                "timestamptz" => {
                    let Some(v) = row
                        .try_get::<Option<sqlx::types::chrono::DateTime<chrono::Utc>>, _>(index)?
                    else {
                        return Ok(Value::None);
                    };
                    Ok(Value::TimestampTz(v))
                }
                "interval" => {
                    let Some(v) =
                        row.try_get::<Option<sqlx::postgres::types::PgInterval>, _>(index)?
                    else {
                        return Ok(Value::None);
                    };
                    let secs = crate::utils::pg_interval_to_secs(v);
                    Ok(Value::Interval(chrono::Duration::seconds(secs)))
                }
                _ => Err("Unsupported type".into()),
            },
            PgTypeKind::Array(ar) => {
                let typ = ar.name().to_lowercase();
                match typ.as_str() {
                    "text" | "citext" => {
                        let Some(v) = row.try_get::<Option<Vec<String>>, _>(index)? else {
                            return Ok(Value::None);
                        };
                        Ok(Value::List(v.into_iter().map(Value::String).collect()))
                    }
                    "int4" => {
                        let Some(v) = row.try_get::<Option<Vec<i32>>, _>(index)? else {
                            return Ok(Value::None);
                        };
                        Ok(Value::List(
                            v.into_iter().map(|x| Value::Integer(x.into())).collect(),
                        ))
                    }
                    "int8" | "bigint" => {
                        let Some(v) = row.try_get::<Option<Vec<i64>>, _>(index)? else {
                            return Ok(Value::None);
                        };
                        Ok(Value::List(v.into_iter().map(Value::Integer).collect()))
                    }
                    "float4" => {
                        let Some(v) = row.try_get::<Option<Vec<f32>>, _>(index)? else {
                            return Ok(Value::None);
                        };
                        Ok(Value::List(
                            v.into_iter().map(|f| Value::Float(f as f64)).collect(),
                        ))
                    }
                    "float8" => {
                        let Some(v) = row.try_get::<Option<Vec<f64>>, _>(index)? else {
                            return Ok(Value::None);
                        };
                        Ok(Value::List(v.into_iter().map(Value::Float).collect()))
                    }
                    "bool" => {
                        let Some(v) = row.try_get::<Option<Vec<bool>>, _>(index)? else {
                            return Ok(Value::None);
                        };
                        Ok(Value::List(v.into_iter().map(Value::Boolean).collect()))
                    }
                    "json" | "jsonb" => {
                        let Some(v) = row.try_get::<Option<Vec<serde_json::Value>>, _>(index)?
                        else {
                            return Ok(Value::None);
                        };
                        let mut m: Vec<Value> = Vec::new();

                        for i in v {
                            m.push(Value::from_json(&i));
                        }

                        Ok(Value::List(m))
                    }
                    "uuid" => {
                        let Some(v) = row.try_get::<Option<Vec<sqlx::types::Uuid>>, _>(index)?
                        else {
                            return Ok(Value::None);
                        };
                        Ok(Value::List(v.into_iter().map(Value::Uuid).collect()))
                    }
                    "timestamp" => {
                        let Some(v) = row
                            .try_get::<Option<Vec<sqlx::types::chrono::NaiveDateTime>>, _>(index)?
                        else {
                            return Ok(Value::None);
                        };
                        let mut m: Vec<Value> = Vec::new();

                        for i in v {
                            m.push(Value::Timestamp(i));
                        }

                        Ok(Value::List(m))
                    }
                    "timestamptz" => {
                        let Some(v) = row
                            .try_get::<Option<Vec<sqlx::types::chrono::DateTime<chrono::Utc>>>, _>(
                                index,
                            )?
                        else {
                            return Ok(Value::None);
                        };

                        let mut m: Vec<Value> = Vec::new();

                        for i in v {
                            m.push(Value::TimestampTz(i));
                        }

                        Ok(Value::List(m))
                    }
                    "interval" => {
                        let Some(v) = row
                            .try_get::<Option<Vec<sqlx::postgres::types::PgInterval>>, _>(index)?
                        else {
                            return Ok(Value::None);
                        };

                        let mut m: Vec<Value> = Vec::new();

                        for i in v {
                            let secs = crate::utils::pg_interval_to_secs(i);
                            m.push(Value::Interval(chrono::Duration::seconds(secs)));
                        }

                        Ok(Value::List(m))
                    }
                    _ => Err("Unsupported type".into()),
                }
            }
            _ => Err("Unsupported type".into()),
        }
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Uuid(u) => write!(f, "{}", u),
            Value::String(s) => write!(f, "{}", s),
            Value::Timestamp(t) => write!(f, "{}", t),
            Value::TimestampTz(t) => write!(f, "{}", t),
            Value::Integer(i) => write!(f, "{}", i),
            Value::Interval(i) => {
                // We format to a string in the format of "1d 2h 3m 4s"
                let mut secs = i.num_seconds();
                let mut mins = secs / 60;
                secs %= 60;
                let mut hours = mins / 60;
                mins %= 60;
                let days = hours / 24;
                hours %= 24;

                if days > 0 {
                    write!(f, "{}d ", days)?;
                }

                if hours > 0 {
                    write!(f, "{}h ", hours)?;
                }

                if mins > 0 {
                    write!(f, "{}m ", mins)?;
                }

                if secs > 0 {
                    write!(f, "{}s", secs)?;
                }

                Ok(())
            }
            Value::Float(fl) => write!(f, "{}", fl),
            Value::Boolean(b) => write!(f, "{}", b),
            Value::List(l) => {
                write!(f, "[")?;
                for (i, v) in l.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", v)?;
                }
                write!(f, "]")
            }
            Value::Json(j) => write!(f, "{}", j),
            Value::Map(m) => {
                write!(f, "{{")?;
                for (i, (k, v)) in m.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}: {}", k, v)?;
                }
                write!(f, "}}")
            }
            Value::None => write!(f, "None"),
        }
    }
}

// serde_json as_TYPE methods
impl Value {
    pub fn as_u64(&self) -> Option<u64> {
        match self {
            Value::Integer(i) => Some(*i as u64),
            Value::Float(f) => Some(*f as u64),
            _ => None,
        }
    }

    pub fn as_i64(&self) -> Option<i64> {
        match self {
            Value::Integer(i) => Some(*i),
            Value::Float(f) => Some(*f as i64),
            _ => None,
        }
    }

    pub fn as_f64(&self) -> Option<f64> {
        match self {
            Value::Integer(i) => Some(*i as f64),
            Value::Float(f) => Some(*f),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Value::Boolean(b) => Some(*b),
            _ => None,
        }
    }

    pub fn as_string(&self) -> Option<&str> {
        match self {
            Value::String(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_array(&self) -> Option<&Vec<Value>> {
        match self {
            Value::List(l) => Some(l),
            _ => None,
        }
    }

    pub fn as_map(&self) -> Option<&indexmap::IndexMap<String, Value>> {
        match self {
            Value::Map(m) => Some(m),
            _ => None,
        }
    }

    /// as_object is an alias to as_map
    pub fn as_object(&self) -> Option<&indexmap::IndexMap<String, Value>> {
        self.as_map()
    }

    pub fn as_uuid(&self) -> Option<&sqlx::types::Uuid> {
        match self {
            Value::Uuid(u) => Some(u),
            _ => None,
        }
    }

    pub fn as_timestamp(&self) -> Option<&chrono::NaiveDateTime> {
        match self {
            Value::Timestamp(t) => Some(t),
            _ => None,
        }
    }

    pub fn as_timestamp_tz(&self) -> Option<&chrono::DateTime<chrono::Utc>> {
        match self {
            Value::TimestampTz(t) => Some(t),
            _ => None,
        }
    }

    pub fn as_interval(&self) -> Option<&chrono::Duration> {
        match self {
            Value::Interval(i) => Some(i),
            _ => None,
        }
    }

    pub fn as_none(&self) -> bool {
        matches!(self, Value::None)
    }
}

impl serde::Serialize for Value {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let value = self.to_json();

        value.serialize(serializer)
    }
}

impl<'de> serde::Deserialize<'de> for Value {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = serde_json::Value::deserialize(deserializer)?;

        Ok(Value::from_json(&value))
    }
}

impl From<serde_json::Value> for Value {
    fn from(value: serde_json::Value) -> Self {
        Value::from_json(&value)
    }
}
