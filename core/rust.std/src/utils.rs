use sqlx::postgres::types::PgInterval;
use std::collections::HashMap;
use std::sync::LazyLock;

pub fn create_special_allocation_from_str(
    special_allocations: &str,
) -> Result<HashMap<String, u32>, crate::Error> {
    let split = special_allocations.split(',').collect::<Vec<&str>>();

    if !split.is_empty() {
        let mut map = HashMap::new();

        for v in split {
            if v.is_empty() {
                continue;
            }

            let split = v.split('=').collect::<Vec<&str>>();

            if split.len() != 2 {
                return Err("Invalid special allocation format".into());
            }

            let channel_id = split[0].to_string();
            let number = split[1].parse::<u32>()?;

            map.insert(channel_id, number);
        }

        Ok(map)
    } else {
        Ok(HashMap::new())
    }
}

pub fn pg_interval_to_secs(i: PgInterval) -> i64 {
    i.microseconds / 1000000 + ((i.days * 86400) as i64) + ((i.months * 2628000) as i64)
}

pub fn secs_to_pg_interval(secs: i64) -> PgInterval {
    PgInterval {
        microseconds: secs * 1000000,
        days: (secs / 86400) as i32,
        months: (secs / 2628000) as i32,
    }
}

pub fn secs_to_pg_interval_u64(secs: u64) -> PgInterval {
    // Check if the value is too large to fit in an i64
    if secs > i64::MAX as u64 {
        // If it is, return the maximum value
        return PgInterval {
            microseconds: i64::MAX,
            days: i32::MAX,
            months: i32::MAX,
        };
    }

    secs_to_pg_interval(secs as i64)
}

pub fn parse_pg_interval(i: PgInterval) -> String {
    let seconds = pg_interval_to_secs(i);

    let dur = std::time::Duration::from_secs(seconds.try_into().unwrap_or_default());

    format!("{:?}", dur)
}

#[derive(poise::ChoiceParameter, PartialEq, Debug)]
pub enum Unit {
    #[name = "Seconds"]
    Seconds,
    #[name = "Minutes"]
    Minutes,
    #[name = "Hours"]
    Hours,
    #[name = "Days"]
    Days,
    #[name = "Weeks"]
    Weeks,
}

impl Unit {
    /// Convert the unit to seconds
    pub fn to_seconds(&self) -> u64 {
        match self {
            Unit::Seconds => 1,
            Unit::Minutes => 60,
            Unit::Hours => 3600,
            Unit::Days => 86400,
            Unit::Weeks => 604800,
        }
    }

    /// Same as to_seconds but returns an i64 instead of a u64 for easier use with sqlx
    pub fn to_seconds_i64(&self) -> i64 {
        match self {
            Unit::Seconds => 1,
            Unit::Minutes => 60,
            Unit::Hours => 3600,
            Unit::Days => 86400,
            Unit::Weeks => 604800,
        }
    }
}

impl TryFrom<&str> for Unit {
    type Error = crate::Error;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s {
            "seconds" => Ok(Unit::Seconds),
            "second" => Ok(Unit::Seconds), // Allow "second" as a shorthand for "seconds"
            "secs" => Ok(Unit::Seconds),   // Allow "secs" as a shorthand for "seconds"
            "sec" => Ok(Unit::Seconds),    // Allow "sec" as a shorthand for "seconds"
            "s" => Ok(Unit::Seconds),      // Allow "s" as a shorthand for "seconds"
            "minutes" => Ok(Unit::Minutes),
            "minute" => Ok(Unit::Minutes), // Allow "minute" as a shorthand for "minutes"
            "mins" => Ok(Unit::Minutes),   // Allow "mins" as a shorthand for "minutes"
            "min" => Ok(Unit::Minutes),    // Allow "min" as a shorthand for "minutes"
            "m" => Ok(Unit::Minutes),      // Allow "m" as a shorthand for "minutes"
            "hours" => Ok(Unit::Hours),
            "hour" => Ok(Unit::Hours), // Allow "hour" as a shorthand for "hours"
            "hrs" => Ok(Unit::Hours),  // Allow "hrs" as a shorthand for "hours"
            "hr" => Ok(Unit::Hours),   // Allow "hr" as a shorthand for "hours"
            "h" => Ok(Unit::Hours),    // Allow "h" as a shorthand for "hours"
            "days" => Ok(Unit::Days),
            "day" => Ok(Unit::Days), // Allow "day" as a shorthand for "days"
            "d" => Ok(Unit::Days),   // Allow "d" as a shorthand for "days"
            "weeks" => Ok(Unit::Weeks),
            "week" => Ok(Unit::Weeks), // Allow "week" as a shorthand for "weeks"
            "w" => Ok(Unit::Weeks),    // Allow "w" as a shorthand for "weeks"
            _ => Err("Invalid unit".into()),
        }
    }
}

/// Given a string of the format <number> days/hours/minutes/seconds, parse it into a u64 of seconds
///
/// This function should handle both spaced and non-spaced formats
pub fn parse_duration_string(s: &str) -> Result<(u64, Unit), crate::Error> {
    let mut number: u64 = 0;
    let mut unit = String::new();

    // Keep looping adding up each number until we hit a non-number which gets added to unit
    for c in s.chars() {
        if c.is_numeric() {
            number = number * 10 + c.to_digit(10).ok_or("Cannot convert to integer")? as u64;
        } else {
            if c == ' ' {
                continue;
            }

            unit.push(c);
        }
    }

    let unit = Unit::try_from(unit.as_str())?;

    Ok((number, unit))
}

/// Given a string of the format <number> days/hours/minutes/seconds, parse it into a chrono::Duration
///
/// This is a wrapper around parse_duration_string that converts the result into a chrono::Duration
pub fn parse_duration_string_to_chrono_duration(s: &str) -> Result<chrono::Duration, crate::Error> {
    let (number, unit) = parse_duration_string(s)?;

    Ok(chrono::Duration::from_std(std::time::Duration::from_secs(
        number * unit.to_seconds(),
    ))?)
}

pub static REPLACE_CHANNEL: LazyLock<Vec<(&'static str, &'static str)>> =
    LazyLock::new(|| vec![("<#", ""), (">", "")]);

pub static REPLACE_USER: LazyLock<Vec<(&'static str, &'static str)>> =
    LazyLock::new(|| vec![("<@", ""), ("!", ""), (">", "")]);

pub static REPLACE_ROLE: LazyLock<Vec<(&'static str, &'static str)>> =
    LazyLock::new(|| vec![("<@", ""), ("&", ""), (">", "")]);

/// Parse a numeric list from a string without knowing its separator
pub fn parse_numeric_list<T: std::str::FromStr + Send + Sync>(
    s: &str,
    replace: &[(&'static str, &'static str)],
) -> Result<Vec<T>, T::Err> {
    let mut list = Vec::new();
    let mut number = String::new();

    for c in s.chars() {
        if c.is_numeric() {
            number.push(c);
        } else if !number.is_empty() {
            for (from, to) in replace {
                number = number.replace(from, to);
            }
            list.push(number.parse::<T>()?);
            number.clear();
        }
    }

    if !number.is_empty() {
        list.push(number.parse::<T>()?);
    }

    Ok(list)
}

/// Parse a numeric list from a string without knowing its separator, returning a string instead of a number
#[allow(dead_code)]
pub fn parse_numeric_list_to_str<T: std::fmt::Display + std::str::FromStr + Send + Sync>(
    s: &str,
    replace: &[(&'static str, &'static str)],
) -> Result<Vec<String>, T::Err> {
    let mut list = Vec::new();
    let mut number = String::new();

    for c in s.chars() {
        if c.is_numeric() {
            number.push(c);
        } else if !number.is_empty() {
            for (from, to) in replace {
                number = number.replace(from, to);
            }
            list.push(number.parse::<T>()?.to_string());
            number.clear();
        }
    }

    if !number.is_empty() {
        list.push(number.parse::<T>()?.to_string());
    }

    Ok(list)
}

pub fn split_input_to_string(s: &str, separator: &str) -> Vec<String> {
    s.split(separator)
        .filter_map(|s| {
            let s = s.trim();
            if s.is_empty() {
                None
            } else {
                Some(s.to_string())
            }
        })
        .collect()
}

pub mod sql_utils {
    pub const SPECIAL_VARS: [&str; 2] = ["__limit", "__offset"];

    use std::collections::HashSet;

    /// Helper method to create a WHERE clause from a set of filters
    ///
    /// E.g. a = $1 AND b IS NULL AND c = $2 etc.
    ///
    /// This does NOT check against column set and is hence potentially vulnerable to SQL injection if not used correctly
    pub fn create_where_clause_unchecked(
        filters: &indexmap::IndexMap<String, crate::value::Value>,
        offset: usize,
    ) -> String {
        let mut filters_str = String::new();

        let mut i = 0;

        let mut spec_limit = false;
        let mut spec_offset = false;

        let mut needs_sep = false;
        for (key, v) in filters.iter() {
            if key == "__limit" {
                if let crate::value::Value::Integer(_) = v {
                    spec_limit = true;
                }
                continue;
            } else if key == "__offset" {
                if let crate::value::Value::Integer(_) = v {
                    spec_offset = true;
                }
                continue;
            }

            if needs_sep {
                filters_str.push_str(" AND ")
            }

            if matches!(v, crate::value::Value::None) {
                filters_str.push_str(format!(" \"{}\" IS NULL", key).as_str());
            } else {
                filters_str.push_str(format!(" \"{}\" = ${}", key, (i + 1) + offset).as_str());
                i += 1; // Only update i if we actually add a filter that binds a value
            }

            needs_sep = true;
        }

        if filters_str.is_empty() {
            // HACK: Use 1 = 1
            filters_str.push_str("1 = 1");
        }

        // Add the limit and offset *LAST* if they exist
        if spec_limit {
            filters_str.push_str(format!(" LIMIT ${}", (i + 1) + offset).as_str());
        }

        if spec_offset {
            filters_str.push_str(format!(" OFFSET ${}", (i + 2) + offset).as_str());
        }

        filters_str
    }

    /// Helper method to create a WHERE clause from a set of filters
    ///
    /// E.g. a = $1 AND b IS NULL AND c = $2 etc.
    pub fn create_where_clause(
        valid_columns: &HashSet<String>,
        filters: &indexmap::IndexMap<String, crate::value::Value>,
        offset: usize,
    ) -> Result<String, crate::Error> {
        for (key, _) in filters.iter() {
            // The __limit, __offset etc key is special and is used for pagination
            if SPECIAL_VARS.contains(&key.as_str()) {
                continue;
            }

            // Validate the column to avoid SQL injection
            let parts = key.split("__").collect::<Vec<&str>>();

            if !valid_columns.contains(&parts[0].to_string()) {
                return Err(format!("Invalid column [part 0 not valid column]: {}", key).into());
            }

            // Ensure all other parts are alphanumeric and/or contains an _
            for part in parts.iter().skip(1) {
                if !part.chars().all(|c| c.is_alphanumeric() || c == '_') {
                    return Err(format!("Invalid column [rest not valid]: {}", key).into());
                }
            }
        }

        Ok(create_where_clause_unchecked(filters, offset))
    }

    /// Helper method to create a SET clause from a set of entries
    /// E.g. "a" = $1, "b" = $2, "c" = $3 etc.
    ///
    /// This does NOT check against column set and is hence potentially vulnerable to SQL injection if not used correctly
    pub fn create_update_set_clause_unchecked(
        entry: &indexmap::IndexMap<String, crate::value::Value>,
        offset: usize,
    ) -> String {
        let mut col_params = "".to_string();

        let mut i = 0;

        #[allow(clippy::explicit_counter_loop)]
        for (col, _) in entry.iter() {
            // $1 is first col param
            col_params.push_str(&format!("\"{}\" = ${},", col, (i + 1) + offset));
            i += 1;
        }

        // Remove the trailing comma
        col_params.pop();

        col_params
    }

    /// Helper method to create a SET clause from a set of entries
    /// E.g. "a" = $1, "b" = $2, "c" = $3 etc.
    pub fn create_update_set_clause(
        valid_columns: &HashSet<String>,
        entry: &indexmap::IndexMap<String, crate::value::Value>,
        offset: usize,
    ) -> Result<String, crate::Error> {
        for (col, _) in entry.iter() {
            if SPECIAL_VARS.contains(&col.as_str()) {
                continue;
            }

            // Validate the column to avoid SQL injection, here we don't really need to care about parts etc.
            if !valid_columns.contains(col) {
                return Err(format!("Invalid column [part 0 not valid column]: {}", col).into());
            }
        }

        Ok(create_update_set_clause_unchecked(entry, offset))
    }

    /// Helper method to create the col_params ("col1", "col2", "col3" etc.) and the n_params ($1, $2, $3 etc.)
    /// for a query
    pub fn create_col_and_n_params(
        valid_columns: &HashSet<String>,
        entry: &indexmap::IndexMap<String, crate::value::Value>,
        offset: usize,
    ) -> Result<(String, String), crate::Error> {
        let mut n_params = "".to_string();
        let mut col_params = "".to_string();
        for (i, (col, _)) in entry.iter().enumerate() {
            // Validate the column to avoid SQL injection, here we don't really need to care about parts etc.
            if !valid_columns.contains(col) {
                return Err(format!("Invalid column [part 0 not valid column]: {}", col).into());
            }

            n_params.push_str(&format!("${},", (i + 1) + offset));
            col_params.push_str(&format!("\"{}\",", col));
        }

        // Remove the trailing comma
        n_params.pop();
        col_params.pop();

        Ok((col_params, n_params))
    }
}

#[cfg(test)]
mod test {
    pub use super::*;

    #[test]
    fn test_parse_numeric_list() {
        assert_eq!(
            parse_numeric_list::<i32>("1,2,3,4,5", &[]).unwrap(),
            vec![1, 2, 3, 4, 5]
        );
        assert_eq!(
            parse_numeric_list::<i32>("1,2,3,4,5,", &[]).unwrap(),
            vec![1, 2, 3, 4, 5]
        );
        assert_eq!(
            parse_numeric_list_to_str::<serenity::all::ChannelId>("1,2", &[(",", "")]).unwrap(),
            vec!["1", "2"]
        );
    }

    #[test]
    fn test_parse_duration_string() {
        assert_eq!(parse_duration_string("1d").unwrap(), (1, Unit::Days));
        assert_eq!(parse_duration_string("1 day").unwrap(), (1, Unit::Days));
        assert_eq!(parse_duration_string("1 days").unwrap(), (1, Unit::Days));
    }
}
