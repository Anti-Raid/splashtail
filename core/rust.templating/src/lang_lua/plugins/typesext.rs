use futures_util::stream::{Stream, StreamExt};
use mlua::prelude::*;
use serde::ser::SerializeMap;
use std::pin::Pin;

pub fn plugin_docs() -> templating_docgen::Plugin {
    templating_docgen::Plugin::default()
        .name("@antiraid/typesext")
        .description("Extra types used by Anti-Raid Lua templating subsystem to either add in common functionality such as streams or handle things like u64/i64 types performantly.")
        .type_mut(
            "LuaStream",
            "LuaStream<T> provides a stream implementation. This is returned by MessageHandle's await_component_interaction for instance for handling button clicks/select menu choices etc.",
            |t| {
                t
                .add_generic("T")
                .method_mut("next", |m| {
                    m.description("Returns the next item in the stream.")
                    .return_("item", |r| {
                        r.typ("<T>").description("The next item in the stream.")
                    })
                })
                .method_mut("for_each", |m| {
                    m.description("Executes a callback for every entry in the stream.")
                    .parameter("callback", |p| {
                        p.typ("function").description("The callback to execute for each entry.")
                    })
                })
            }
        )
        .type_mut("MultiOption", "MultiOption allows distinguishing between `null` and empty fields. Use the value to show both existence and value (`Some(Some(value))`) an empty object to show existence (``Some(None)``) or null to show neither (`None`)", |t| {
            t
            .add_generic("T")
        })
        .type_mut("U64", "U64 is a 64-bit unsigned integer type. Implements Add/Subtract/Multiply/Divide/Modulus/Power/Integer Division/Equality/Comparison (Lt/Le and its complements Gt/Ge) and ToString with a type name of U64", |mut t| {
            t
            .method_mut("to_ne_bytes", |m| {
                m.description("Converts the U64 to a little-endian byte array.")
                .return_("bytes", |r| {
                    r.typ("{u8}").description("The little-endian byte array.")
                })
            })
            .method_mut("from_ne_bytes", |m| {
                m.description("Converts a little-endian byte array to a U64.")
                .parameter("bytes", |p| {
                    p.typ("{u8}").description("The little-endian byte array.")
                })
                .return_("u64", |r| {
                    r.typ("U64").description("The U64 value.")
                })
            })
            .method_mut("to_le_bytes", |m| {
                m.description("Converts the U64 to a little-endian byte array.")
                .return_("bytes", |r| {
                    r.typ("{u8}").description("The little-endian byte array.")
                })
            })
            .method_mut("from_le_bytes", |m| {
                m.description("Converts a little-endian byte array to a U64.")
                .parameter("bytes", |p| {
                    p.typ("{u8}").description("The little-endian byte array.")
                })
                .return_("u64", |r| {
                    r.typ("U64").description("The U64 value.")
                })
            })
            .method_mut("to_be_bytes", |m| {
                m.description("Converts the U64 to a big-endian byte array.")
                .return_("bytes", |r| {
                    r.typ("{u8}").description("The big-endian byte array.")
                })
            })
            .method_mut("from_be_bytes", |m| {
                m.description("Converts a big-endian byte array to a U64.")
                .parameter("bytes", |p| {
                    p.typ("{u8}").description("The big-endian byte array.")
                })
                .return_("u64", |r| {
                    r.typ("U64").description("The U64 value.")
                })
            })
            .method_mut("to_i64", |m| {
                m.description("Converts the U64 to an i64.")
                .return_("i64", |r| {
                    r.typ("I64").description("The i64 value.")
                })
            })
        })
        .type_mut("I64", "I64 is a 64-bit signed integer type. Implements Add/Subtract/Multiply/Divide/Modulus/Power/Integer Division/Equality/Comparison (Lt/Le and its complements Gt/Ge) and ToString with a type name of I64", |mut t| {
            t
            .method_mut("to_ne_bytes", |m| {
                m.description("Converts the I64 to a little-endian byte array.")
                .return_("bytes", |r| {
                    r.typ("{u8}").description("The little-endian byte array.")
                })
            })
            .method_mut("from_ne_bytes", |m| {
                m.description("Converts a little-endian byte array to a I64.")
                .parameter("bytes", |p| {
                    p.typ("{u8}").description("The little-endian byte array.")
                })
                .return_("i64", |r| {
                    r.typ("I64").description("The I64 value.")
                })
            })
            .method_mut("to_le_bytes", |m| {
                m.description("Converts the I64 to a little-endian byte array.")
                .return_("bytes", |r| {
                    r.typ("{u8}").description("The little-endian byte array.")
                })
            })
            .method_mut("from_le_bytes", |m| {
                m.description("Converts a little-endian byte array to a I64.")
                .parameter("bytes", |p| {
                    p.typ("{u8}").description("The little-endian byte array.")
                })
                .return_("i64", |r| {
                    r.typ("I64").description("The I64 value.")
                })
            })
            .method_mut("to_be_bytes", |m| {
                m.description("Converts the I64 to a big-endian byte array.")
                .return_("bytes", |r| {
                    r.typ("{u8}").description("The big-endian byte array.")
                })
            })
            .method_mut("from_be_bytes", |m| {
                m.description("Converts a big-endian byte array to a I64.")
                .parameter("bytes", |p| {
                    p.typ("{u8}").description("The big-endian byte array.")
                })
                .return_("i64", |r| {
                    r.typ("I64").description("The I64 value.")
                })
            })
            .method_mut("to_u64", |m| {
                m.description("Converts the I64 to a U64.")
                .return_("u64", |r| {
                    r.typ("U64").description("The U64 value.")
                })
            })
        })
        .type_mut("bitu64", "[bit32](https://luau.org/library#bit32-library) but for U64 datatype. Note that bit64 is experimental and may not be properly documented at all times. When in doubt, reach for Luau's bit32 documentation and simply replace 31's with 63's", |mut t| {
            t
            .method_mut("band", |m| {
                m.description("Performs a bitwise AND operation on the given values.")
                .parameter("values", |p| {
                    p.typ("{U64}").description("The values to perform the operation on.")
                })
                .return_("result", |r| {
                    r.typ("U64").description("The result of the operation.")
                })
            })
            .method_mut("bnor", |m| {
                m.description("Performs a bitwise NOR operation on the given value.")
                .parameter("n", |p| {
                    p.typ("U64").description("The value to perform the operation on.")
                })
                .return_("result", |r| {
                    r.typ("U64").description("The result of the operation.")
                })
            })
            .method_mut("bor", |m| {
                m.description("Performs a bitwise OR operation on the given values.")
                .parameter("values", |p| {
                    p.typ("{U64}").description("The values to perform the operation on.")
                })
                .return_("result", |r| {
                    r.typ("U64").description("The result of the operation.")
                })
            })
            .method_mut("bxor", |m| {
                m.description("Performs a bitwise XOR operation on the given values.")
                .parameter("values", |p| {
                    p.typ("{U64}").description("The values to perform the operation on.")
                })
                .return_("result", |r| {
                    r.typ("U64").description("The result of the operation.")
                })
            })
            .method_mut("btest", |m| {
                m.description("Tests if the bitwise AND of the given values is not zero.")
                .parameter("values", |p| {
                    p.typ("{U64}").description("The values to perform the operation on.")
                })
                .return_("result", |r| {
                    r.typ("bool").description("True if the bitwise AND of the values is not zero, false otherwise.")
                })
            })
            .method_mut("extract", |m| {
                m.description("Extracts a field from a value.")
                .parameter("n", |p| {
                    p.typ("U64").description("The value to extract the field from.")
                })
                .parameter("f", |p| {
                    p.typ("u64").description("The field to extract.")
                })
                .parameter("w", |p| {
                    p.typ("u64").description("The width of the field to extract.")
                })
                .return_("result", |r| {
                    r.typ("U64").description("The extracted field.")
                })
            })
            .method_mut("lrotate", |m| {
                m.description("Rotates a value left or right.")
                .parameter("n", |p| {
                    p.typ("U64").description("The value to rotate.")
                })
                .parameter("i", |p| {
                    p.typ("i64").description("The amount to rotate by.")
                })
                .return_("result", |r| {
                    r.typ("U64").description("The rotated value.")
                })
            })
            .method_mut("lshift", |m| {
                m.description("Shifts a value left or right.")
                .parameter("n", |p| {
                    p.typ("U64").description("The value to shift.")
                })
                .parameter("i", |p| {
                    p.typ("i64").description("The amount to shift by.")
                })
                .return_("result", |r| {
                    r.typ("U64").description("The shifted value.")
                })
            })
            .method_mut("replace", |m| {
                m.description("Replaces a field in a value.")
                .parameter("n", |p| {
                    p.typ("U64").description("The value to replace the field in.")
                })
                .parameter("v", |p| {
                    p.typ("U64").description("The value to replace the field with.")
                })
                .parameter("f", |p| {
                    p.typ("u64").description("The field to replace.")
                })
                .parameter("w", |p| {
                    p.typ("u64").description("The width of the field to replace.")
                })
                .return_("result", |r| {
                    r.typ("U64").description("The value with the field replaced.")
                })
            })
            .method_mut("rrotate", |m| {
                m.description("Rotates a value left or right.")
                .parameter("n", |p| {
                    p.typ("U64").description("The value to rotate.")
                })
                .parameter("i", |p| {
                    p.typ("i64").description("The amount to rotate by.")
                })
                .return_("result", |r| {
                    r.typ("U64").description("The rotated value.")
                })
            })
            .method_mut("rshift", |m| {
                m.description("Shifts a value left or right.")
                .parameter("n", |p| {
                    p.typ("U64").description("The value to shift.")
                })
                .parameter("i", |p| {
                    p.typ("i64").description("The amount to shift by.")
                })
                .return_("result", |r| {
                    r.typ("U64").description("The shifted value.")
                })
            })
        })
        .method_mut("U64", |m| {
            m.description("Creates a new U64.")
            .parameter("value", |p| {
                p.typ("u64").description("The value of the U64.")
            })
            .return_("u64", |r| {
                r.typ("U64").description("The U64 value.")
            })
        })
        .method_mut("I64", |m| {
            m.description("Creates a new I64.")
            .parameter("value", |p| {
                p.typ("i64").description("The value of the I64.")
            })
            .return_("i64", |r| {
                r.typ("I64").description("The I64 value.")
            })
        })
        .field_mut("bitu64", |f| {
            f.typ("bitu64").description("The bitu64 library.")
        })
}

pub struct LuaStream<T: Stream<Item: IntoLua + Send> + Send + 'static> {
    pub inner: Pin<Box<T>>, // Box the stream to ensure its pinned,
}

impl<ST: Stream<Item: IntoLua + Send> + Send + 'static> LuaStream<ST> {
    pub fn new(stream: ST) -> Self {
        Self {
            inner: Box::pin(stream), // Pin the stream
        }
    }
}

impl<T: Stream<Item: IntoLua + Send> + Send + 'static> LuaUserData for LuaStream<T> {
    fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
        // Go to the next item in the stream
        methods.add_async_method_mut("next", |lua, mut this, _: ()| async move {
            match this.inner.next().await {
                Some(item) => Ok(item.into_lua(&lua)?), // Convert the item to LuaValue
                None => Ok(LuaValue::Nil),              // Return nil if the stream is exhausted
            }
        }); // Implement the method

        // Executes a callback for every entry in the stream
        methods.add_async_method_mut(
            "for_each",
            |lua, mut this, callback: LuaFunction| async move {
                while let Some(item) = this.inner.next().await {
                    let item_value = item.into_lua(&lua)?; // Convert the item to LuaValue
                    callback
                        .call_async::<()>((
                            item_value, // Convert the item to LuaValue
                        ))
                        .await?; // Call the Lua callback
                }
                Ok(())
            },
        );
    }
}

pub struct MultiOption<T: for<'a> serde::Deserialize<'a> + serde::Serialize> {
    pub inner: Option<Option<T>>,
}

impl<T: for<'a> serde::Deserialize<'a> + serde::Serialize> Default for MultiOption<T> {
    fn default() -> Self {
        Self::new(None)
    }
}

impl<T: for<'a> serde::Deserialize<'a> + serde::Serialize> MultiOption<T> {
    pub fn new(value: Option<T>) -> Self {
        Self {
            inner: value.map(Some),
        }
    }
}

// Deserialize
//
// If value is nil, we set it to None, if value is an empty object, we set it to Some(None), otherwise we set it to Some(Some(value))
impl<'de, T: for<'a> serde::Deserialize<'a> + serde::Serialize> serde::Deserialize<'de>
    for MultiOption<T>
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value: Option<serde_json::Value> = Option::deserialize(deserializer)?;
        let inner = match value {
            None => None,
            Some(v) if v.is_object() && v.as_object().unwrap().is_empty() => Some(None),
            Some(v) => Some(Some(
                serde_json::from_value(v).map_err(serde::de::Error::custom)?,
            )),
        };
        Ok(Self { inner })
    }
}

// Serialize impl
impl<T: for<'a> serde::Deserialize<'a> + serde::Serialize> serde::Serialize for MultiOption<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match &self.inner {
            None => serializer.serialize_none(),
            Some(None) => serializer.serialize_map(Some(0))?.end(),
            Some(Some(value)) => value.serialize(serializer),
        }
    }
}

// U64 type
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct U64(u64);

impl FromLua for U64 {
    fn from_lua(value: LuaValue, _lua: &Lua) -> LuaResult<Self> {
        match value {
            LuaValue::Integer(i) if i >= 0 => Ok(U64(i as u64)),
            LuaValue::String(s) => {
                let str_value = s.to_str()?;
                str_value
                    .parse::<u64>()
                    .map(U64)
                    .map_err(|_| LuaError::FromLuaConversionError {
                        from: "string",
                        to: "U64".to_string(),
                        message: Some("Value must be a non-negative integer".to_string()),
                    })
            }
            LuaValue::UserData(u) => {
                let u64 = u
                    .borrow::<U64>()
                    .map_err(|_| LuaError::FromLuaConversionError {
                        from: "UserData",
                        to: "U64".to_string(),
                        message: Some("UserData must be a U64".to_string()),
                    })?;

                Ok(U64(u64.0))
            }
            _ => Err(LuaError::FromLuaConversionError {
                from: "{integer | string}",
                to: "U64".to_string(),
                message: Some("Value must be a non-negative integer or a string".to_string()),
            }),
        }
    }
}

impl LuaUserData for U64 {
    fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
        // Metamethods
        methods.add_meta_method(LuaMetaMethod::Add, |_, this, value: U64| {
            let v = this.0.wrapping_add(value.0);
            Ok(U64(v))
        });

        methods.add_meta_method(LuaMetaMethod::Sub, |_, this, value: U64| {
            let v = this.0.wrapping_add(value.0);
            Ok(U64(v))
        });

        methods.add_meta_method(LuaMetaMethod::Mul, |_, this, value: U64| {
            let v = this.0.wrapping_mul(value.0);
            Ok(U64(v))
        });

        methods.add_meta_method(LuaMetaMethod::Div, |_, this, value: U64| {
            let v = this.0.wrapping_div(value.0);
            Ok(U64(v))
        });

        methods.add_meta_method(LuaMetaMethod::Mod, |_, this, value: U64| {
            let v = this.0.wrapping_rem(value.0);
            Ok(U64(v))
        });

        methods.add_meta_method(LuaMetaMethod::Pow, |_, this, value: u32| {
            let v = this.0.wrapping_pow(value);
            Ok(U64(v))
        });

        methods.add_meta_method(LuaMetaMethod::IDiv, |_, this, value: U64| {
            // Same as Div
            let v = this.0.wrapping_div(value.0);
            Ok(U64(v))
        });

        // Comparison
        methods.add_meta_method(LuaMetaMethod::Eq, |_, this, value: U64| {
            Ok(this.0 == value.0)
        });

        methods.add_meta_method(
            LuaMetaMethod::Lt,
            |_, this, value: U64| Ok(this.0 < value.0),
        );

        methods.add_meta_method(LuaMetaMethod::Le, |_, this, value: U64| {
            Ok(this.0 <= value.0)
        });

        // Returns the string representation of the U64 value
        methods.add_meta_method(LuaMetaMethod::ToString, |_, this, _: ()| {
            Ok(this.0.to_string())
        });

        // Type
        methods.add_meta_method(LuaMetaMethod::Type, |_, _, _: ()| Ok("U64".to_string()));

        // Byte related methods
        methods.add_method("to_ne_bytes", |_, this, _: ()| {
            let bytes = this.0.to_ne_bytes();
            Ok(bytes.to_vec())
        });

        methods.add_function("from_ne_bytes", |_, bytes: Vec<u8>| {
            if bytes.len() != 8 {
                return Err(LuaError::external("Byte array must be of length 8"));
            }
            let array: [u8; 8] = bytes
                .try_into()
                .map_err(|_| LuaError::external("Failed to convert Vec<u8> to [u8; 8]"))?;
            let value = u64::from_ne_bytes(array);
            Ok(U64(value))
        });

        methods.add_method("to_le_bytes", |_, this, _: ()| {
            let bytes = this.0.to_le_bytes();
            Ok(bytes.to_vec())
        });

        methods.add_function("from_le_bytes", |_, bytes: Vec<u8>| {
            if bytes.len() != 8 {
                return Err(LuaError::external("Byte array must be of length 8"));
            }
            let array: [u8; 8] = bytes
                .try_into()
                .map_err(|_| LuaError::external("Failed to convert Vec<u8> to [u8; 8]"))?;
            let value = u64::from_le_bytes(array);
            Ok(U64(value))
        });

        methods.add_method("to_be_bytes", |_, this, _: ()| {
            let bytes = this.0.to_be_bytes();
            Ok(bytes.to_vec())
        });

        methods.add_function("from_be_bytes", |_, bytes: Vec<u8>| {
            if bytes.len() != 8 {
                return Err(LuaError::external("Byte array must be of length 8"));
            }
            let array: [u8; 8] = bytes
                .try_into()
                .map_err(|_| LuaError::external("Failed to convert Vec<u8> to [u8; 8]"))?;
            let value = u64::from_be_bytes(array);
            Ok(U64(value))
        });

        // Conversion methods
        methods.add_method("to_i64", |_, this, _: ()| {
            if this.0 > i64::MAX as u64 {
                return Err(LuaError::external("Value is too large to convert to i64"));
            }

            Ok(I64(this.0 as i64))
        });
    }
}

// U64 type
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct I64(i64);

impl FromLua for I64 {
    fn from_lua(value: LuaValue, _lua: &Lua) -> LuaResult<Self> {
        match value {
            LuaValue::Integer(i) => Ok(I64(i as i64)),
            LuaValue::String(s) => {
                let str_value = s.to_str()?;
                str_value
                    .parse::<i64>()
                    .map(I64)
                    .map_err(|_| LuaError::FromLuaConversionError {
                        from: "string",
                        to: "I64".to_string(),
                        message: Some("Value must be an integer".to_string()),
                    })
            }
            LuaValue::UserData(u) => {
                let i64 = u
                    .borrow::<I64>()
                    .map_err(|_| LuaError::FromLuaConversionError {
                        from: "UserData",
                        to: "U64".to_string(),
                        message: Some("UserData must be a I64".to_string()),
                    })?;

                Ok(I64(i64.0))
            }
            _ => Err(LuaError::FromLuaConversionError {
                from: "{integer | string}",
                to: "I64".to_string(),
                message: Some("Value must be an integer or a string".to_string()),
            }),
        }
    }
}

impl LuaUserData for I64 {
    fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
        // Metamethods
        methods.add_meta_method(LuaMetaMethod::Add, |_, this, value: I64| {
            let v = this.0.wrapping_add(value.0);
            Ok(I64(v))
        });

        methods.add_meta_method(LuaMetaMethod::Sub, |_, this, value: I64| {
            let v = this.0.wrapping_sub(value.0);
            Ok(I64(v))
        });

        methods.add_meta_method(LuaMetaMethod::Mul, |_, this, value: I64| {
            let v = this.0.wrapping_mul(value.0);
            Ok(I64(v))
        });

        methods.add_meta_method(LuaMetaMethod::Div, |_, this, value: I64| {
            let v = this.0.wrapping_div(value.0);
            Ok(I64(v))
        });

        methods.add_meta_method(LuaMetaMethod::Mod, |_, this, value: I64| {
            let v = this.0.wrapping_rem(value.0);
            Ok(I64(v))
        });

        methods.add_meta_method(LuaMetaMethod::Pow, |_, this, value: u32| {
            let v = this.0.wrapping_pow(value);
            Ok(I64(v))
        });

        methods.add_meta_method(LuaMetaMethod::IDiv, |_, this, value: I64| {
            // Same as Div
            let v = this.0.wrapping_div(value.0);
            Ok(I64(v))
        });

        // Comparison
        methods.add_meta_method(LuaMetaMethod::Eq, |_, this, value: I64| {
            Ok(this.0 == value.0)
        });

        methods.add_meta_method(
            LuaMetaMethod::Lt,
            |_, this, value: I64| Ok(this.0 < value.0),
        );

        methods.add_meta_method(LuaMetaMethod::Le, |_, this, value: I64| {
            Ok(this.0 <= value.0)
        });

        // Returns the string representation of the U64 value
        methods.add_meta_method(LuaMetaMethod::ToString, |_, this, _: ()| {
            Ok(this.0.to_string())
        });

        // Type
        methods.add_meta_method(LuaMetaMethod::Type, |_, _, _: ()| Ok("I64".to_string()));

        // Byte related methods
        methods.add_method("to_ne_bytes", |_, this, _: ()| {
            let bytes = this.0.to_ne_bytes();
            Ok(bytes.to_vec())
        });

        methods.add_function("from_ne_bytes", |_, bytes: Vec<u8>| {
            if bytes.len() != 8 {
                return Err(LuaError::external("Byte array must be of length 8"));
            }
            let array: [u8; 8] = bytes
                .try_into()
                .map_err(|_| LuaError::external("Failed to convert Vec<u8> to [u8; 8]"))?;
            let value = u64::from_ne_bytes(array);
            Ok(U64(value))
        });

        methods.add_method("to_le_bytes", |_, this, _: ()| {
            let bytes = this.0.to_le_bytes();
            Ok(bytes.to_vec())
        });

        methods.add_function("from_le_bytes", |_, bytes: Vec<u8>| {
            if bytes.len() != 8 {
                return Err(LuaError::external("Byte array must be of length 8"));
            }
            let array: [u8; 8] = bytes
                .try_into()
                .map_err(|_| LuaError::external("Failed to convert Vec<u8> to [u8; 8]"))?;
            let value = u64::from_le_bytes(array);
            Ok(U64(value))
        });

        methods.add_method("to_be_bytes", |_, this, _: ()| {
            let bytes = this.0.to_be_bytes();
            Ok(bytes.to_vec())
        });

        methods.add_function("from_be_bytes", |_, bytes: Vec<u8>| {
            if bytes.len() != 8 {
                return Err(LuaError::external("Byte array must be of length 8"));
            }
            let array: [u8; 8] = bytes
                .try_into()
                .map_err(|_| LuaError::external("Failed to convert Vec<u8> to [u8; 8]"))?;
            let value = u64::from_be_bytes(array);
            Ok(U64(value))
        });

        // Conversion methods
        methods.add_method("to_u64", |_, this, _: ()| {
            if this.0 < 0 {
                return Err(LuaError::external(
                    "Value is negative, cannot convert to u64",
                ));
            }

            Ok(U64(this.0 as u64))
        });
    }
}

/// Aim is to implement as many functions from https://luau.org/library#bit32-library as possible but for 64 bit unsigned integers
///
/// arshift, band, bnot, bor, bxor, btest, extract, lrotate, lshift, replace, rrotate, rshift, countlz
/// countrz, byteswap
///
/// Implemented:
/// - band
/// - bnot
/// - bor
/// - bxor
/// - btest
/// - extract
/// - lrotate
/// - lshift
/// - replace
/// - rrotate
/// - rshift
/// - countlz
/// - countrz
/// - byteswap
///
/// Not yet implemented due to spec difficulties:
/// - arshift
fn bitu64(lua: &Lua) -> LuaResult<LuaTable> {
    let submodule = lua.create_table()?;

    submodule.set(
        "band",
        lua.create_function(|lua, values: LuaMultiValue| {
            if values.is_empty() {
                // Return all 1s
                return Ok(U64(u64::MAX));
            }

            let mut result = u64::MAX;

            for value in values {
                let u64_value = U64::from_lua(value, lua)?;
                result &= u64_value.0;
            }

            Ok(U64(result))
        })?,
    )?;

    submodule.set(
        "bnor",
        lua.create_function(|_lua, n: U64| {
            let result = !n.0;
            Ok(U64(result))
        })?,
    )?;

    submodule.set(
        "bor",
        lua.create_function(|lua, values: LuaMultiValue| {
            if values.is_empty() {
                // Return 0
                return Ok(U64(0));
            }

            let mut result = u64::MAX;

            for value in values {
                let u64_value = U64::from_lua(value, lua)?;
                result |= u64_value.0;
            }

            Ok(U64(result))
        })?,
    )?;

    submodule.set(
        "bxor",
        lua.create_function(|lua, values: LuaMultiValue| {
            if values.is_empty() {
                // Return 0
                return Ok(U64(0));
            }

            let mut result = u64::MAX;

            for value in values {
                let u64_value = U64::from_lua(value, lua)?;
                result ^= u64_value.0;
            }

            Ok(U64(result))
        })?,
    )?;

    submodule.set(
        "btest",
        lua.create_function(|lua, values: LuaMultiValue| {
            if values.is_empty() {
                // band != 0
                return Ok(true);
            }

            let mut result = u64::MAX;

            for value in values {
                let u64_value = U64::from_lua(value, lua)?;
                result &= u64_value.0;
            }

            Ok(result != 0)
        })?,
    )?;

    submodule.set(
        "extract",
        lua.create_function(|_lua, (n, f, w): (U64, u32, Option<u32>)| {
            let w = w.unwrap_or(1);

            if f > 63 {
                return Err(LuaError::external("F must be less than 64"));
            }

            let m = f
                .checked_add(w)
                .ok_or(LuaError::external("F + W - 1 must be less than 64"))?
                .checked_sub(1)
                .ok_or(LuaError::external("F + W - 1 must be less than 64"))?;

            if m > 63 {
                return Err(LuaError::external("F + W must be less than 64"));
            }

            // Extract W bits at from N at position F
            // UNTESTED
            let mask = (1u64 << w) - 1;
            let result = (n.0 >> f) & mask;

            Ok(U64(result))
        })?,
    )?;

    submodule.set(
        "lrotate",
        lua.create_function(|_lua, (n, i): (U64, i64)| {
            if i < 0 {
                // Right rotate
                let result = n.0.rotate_right(
                    (-i).try_into()
                        .map_err(|_| LuaError::external("Invalid rotation value"))?,
                );
                Ok(U64(result))
            } else {
                let result = n.0.rotate_left(
                    i.try_into()
                        .map_err(|_| LuaError::external("Invalid rotation value"))?,
                );
                Ok(U64(result))
            }
        })?,
    )?;

    submodule.set(
        "lshift",
        lua.create_function(|_lua, (n, i): (U64, i64)| {
            if i < 0 {
                // Right shift
                let result = n.0.wrapping_shr(
                    (-i).try_into()
                        .map_err(|_| LuaError::external("Invalid shift value"))?,
                );
                Ok(U64(result))
            } else {
                // Left shift
                let result = n.0.wrapping_shl(
                    i.try_into()
                        .map_err(|_| LuaError::external("Invalid shift value"))?,
                );
                Ok(U64(result))
            }
        })?,
    )?;

    submodule.set(
        "replace",
        lua.create_function(|_lua, (n, r, f, w): (U64, u8, u32, Option<u32>)| {
            if r != 0 && r != 1 {
                return Err(LuaError::external("R must be 0 or 1"));
            }

            let w = w.unwrap_or(1);

            if f > 63 {
                return Err(LuaError::external("F must be less than 64"));
            }

            let m = f
                .checked_add(w)
                .ok_or(LuaError::external("F + W - 1 must be less than 64"))?
                .checked_sub(1)
                .ok_or(LuaError::external("F + W - 1 must be less than 64"))?;

            if m > 63 {
                return Err(LuaError::external("F + W must be less than 64"));
            }

            // Replace W bits at from N at position F
            // UNTESTED
            let mask = (1u64 << w) - 1;
            let result = (n.0 & !(mask << f)) | ((r as u64 & mask) << f);
            Ok(U64(result))
        })?,
    )?;

    submodule.set(
        "rrotate",
        lua.create_function(|_lua, (n, i): (U64, i64)| {
            if i < 0 {
                // Left rotate
                let result = n.0.rotate_left(
                    (-i).try_into()
                        .map_err(|_| LuaError::external("Invalid rotation value"))?,
                );
                Ok(U64(result))
            } else {
                let result = n.0.rotate_right(
                    i.try_into()
                        .map_err(|_| LuaError::external("Invalid rotation value"))?,
                );
                Ok(U64(result))
            }
        })?,
    )?;

    submodule.set(
        "rshift",
        lua.create_function(|_lua, (n, i): (U64, i64)| {
            if i < 0 {
                // Left shift
                let result = n.0.wrapping_shl(
                    (-i).try_into()
                        .map_err(|_| LuaError::external("Invalid shift value"))?,
                );
                Ok(U64(result))
            } else {
                // Left shift
                let result = n.0.wrapping_shr(
                    i.try_into()
                        .map_err(|_| LuaError::external("Invalid shift value"))?,
                );
                Ok(U64(result))
            }
        })?,
    )?;

    submodule.set(
        "countlz",
        lua.create_function(|_lua, n: U64| Ok(n.0.leading_zeros()))?,
    )?;

    submodule.set(
        "countrz",
        lua.create_function(|_lua, n: U64| Ok(n.0.trailing_zeros()))?,
    )?;

    submodule.set(
        "byteswap",
        lua.create_function(|_lua, n: U64| Ok(U64(n.0.swap_bytes())))?,
    )?;

    submodule.set_readonly(true); // Block any attempt to modify this table

    Ok(submodule)
}

pub fn init_plugin(lua: &Lua) -> LuaResult<LuaTable> {
    let module = lua.create_table()?;

    module.set(
        "U64",
        lua.create_function(|lua, initial_value: LuaValue| {
            match initial_value {
                LuaValue::Nil => Ok(U64(0)), // Default value
                _ => {
                    let u64_value = U64::from_lua(initial_value, lua)?;
                    Ok(u64_value)
                }
            }
        })?,
    )?;

    module.set(
        "I64",
        lua.create_function(|lua, initial_value: LuaValue| {
            match initial_value {
                LuaValue::Nil => Ok(I64(0)), // Default value
                _ => {
                    let i64_value = I64::from_lua(initial_value, lua)?;
                    Ok(i64_value)
                }
            }
        })?,
    )?;

    module.set("bitu64", bitu64(lua)?)?;

    module.set_readonly(true); // Block any attempt to modify this table

    Ok(module)
}
