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
    let mut filtered = nickname
        .chars()
        .filter(|c| {
            // Are naturally sorted highly
            let mut allowed = !disallowed_characters().contains(c);

            // Non-ASCII characters are higher priority
            if intensity.contains(DehoistOptions::STRIP_NON_ASCII) && allowed {
                allowed = c.is_ascii_alphanumeric();
            }

            allowed
        })
        .collect::<String>();

    if intensity.contains(DehoistOptions::STRIP_SIMILAR_REPEATING) {
        filtered = strip_similar_leading(&filtered).to_string();
    }

    if filtered.is_empty() {
        filtered = "ar_dehoisted".to_string();
    }

    filtered
}
