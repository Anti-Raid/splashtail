use super::types::DehoistOptions;
/// Dehoist engine
use std::{collections::HashSet, sync::OnceLock};

fn disallowed_characters() -> &'static HashSet<char> {
    static DISALLOWED: OnceLock<HashSet<char>> = OnceLock::new();
    // These characters are sorted
    DISALLOWED.get_or_init(|| {
        HashSet::from([
            '!', '"', '#', '$', '%', '&', '\'', '(', ')', '*', '+', ',', '-', '.', '/',
        ])
    })
}

fn strip_similar_leading(mut s: &str) -> &str {
    let mut leading = count_similar_leading(s);
    loop {
        if leading <= s.len() {
            s = &s[leading..];
        }
        leading = count_similar_leading(s);
        if leading == 0 {
            break;
        }
    }
    s
}

fn count_similar_leading(s: &str) -> usize {
    let mut chars = s.chars();
    // This is what we're comparing to
    let first = chars.next();
    let mut matches = 1;
    // Don't continue if string is empty
    if let Some(first) = first {
        for c in chars {
            if c == first {
                matches += 1;
            } else {
                // Break once we don't have a match
                break;
            }
        }
    }
    // If a match is less than two, it's not really a match
    if matches >= 2 {
        matches
    } else {
        0
    }
}

pub fn dehoist_user(nickname: &str, intensity: DehoistOptions) -> String {
    let mut filtered = String::new();

    for (i, c) in nickname.chars().enumerate() {
        // Are naturally sorted highly
        let mut allowed = true;

        // Special character checks are higher priority

        // StartsWith check should only occur on the first character
        if i == 0 && intensity.contains(DehoistOptions::STRIP_SPECIAL_CHARS_STARTSWITH) {
            allowed = disallowed_characters().contains(&c);
        }

        // Contains check should occur on all characters
        if intensity.contains(DehoistOptions::STRIP_SPECIAL_CHARS_CONTAINS) {
            allowed = disallowed_characters().contains(&c);
        }

        // Non-ASCII characters are higher priority
        if intensity.contains(DehoistOptions::STRIP_NON_ASCII) && allowed {
            allowed = c.is_ascii_alphanumeric();
        }

        if allowed {
            filtered.push(c);
        }
    }

    if intensity.contains(DehoistOptions::STRIP_SIMILAR_REPEATING) {
        filtered = strip_similar_leading(&filtered).to_string();
    }

    if filtered.is_empty() {
        filtered = "ar_dehoisted".to_string();
    }

    filtered
}
