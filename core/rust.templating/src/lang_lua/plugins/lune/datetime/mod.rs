/// Adapted from https://github.com/lune-org/lune/blob/main/crates/lune-std-datetime/src/lib.rs
///
/// SPDX-License-Identifier: MPL-2.0
pub mod date_time;
pub mod result;
pub mod values;

use crate::lang_lua::plugins::lune::{datetime::date_time::DateTime, utils::TableBuilder};
use mlua::prelude::*;

/**
    Creates the `datetime` standard library module.

    # Errors

    Errors when out of memory.
*/
pub fn init_plugin(lua: &Lua) -> LuaResult<LuaTable> {
    TableBuilder::new(lua)?
        .with_function("fromIsoDate", |_, iso_date: String| {
            Ok(DateTime::from_iso_date(iso_date)?)
        })?
        .with_function("fromLocalTime", |_, values| {
            Ok(DateTime::from_local_time(&values)?)
        })?
        .with_function("fromUniversalTime", |_, values| {
            Ok(DateTime::from_universal_time(&values)?)
        })?
        .with_function("fromUnixTimestamp", |_, timestamp| {
            Ok(DateTime::from_unix_timestamp_float(timestamp)?)
        })?
        .with_function("now", |_, ()| Ok(DateTime::now()))?
        .build_readonly()
}
