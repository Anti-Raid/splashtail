pub fn get_char_limit(total_chars: usize, limit: usize, max_chars: usize) -> usize {
    if max_chars <= total_chars {
        return 0;
    }

    // If limit is 6000 and max_chars - total_chars is 1000, return 1000 etc.
    std::cmp::min(limit, max_chars - total_chars)
}

pub fn slice_chars(s: &str, total_chars: &mut usize, limit: usize, max_chars: usize) -> String {
    let char_limit = get_char_limit(*total_chars, limit, max_chars);

    if char_limit == 0 {
        return String::new();
    }

    if s.len() > char_limit {
        *total_chars += char_limit;
        s.chars().take(char_limit).collect()
    } else {
        *total_chars += s.len();
        s.to_string()
    }
}

#[derive(Default, serde::Serialize)]
/// A DiscordReply is guaranteed to map 1-1 to discords API
pub struct DiscordReply<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    pub embeds: Vec<serenity::all::CreateEmbed<'a>>,
}

/// A MessageTemplateContext is a context for message templates
/// that can be accessed in message templates
#[derive(Clone, serde::Serialize)]
pub struct MessageTemplateContext {
    pub event_titlename: String,
    pub event_name: String,
    pub fields: indexmap::IndexMap<String, gwevent::field::CategorizedField>,
}

/// A PermissionTemplateContext is a context for permission templates
/// that can be accessed in permission templates
#[derive(Clone, serde::Serialize)]
pub struct PermissionTemplateContext {
    pub member_native_permissions: serenity::all::Permissions,
    pub member_kittycat_permissions: Vec<kittycat::perms::Permission>,
    pub user_id: serenity::all::UserId,
    pub guild_id: serenity::all::GuildId,
    pub guild_owner_id: serenity::all::UserId,
    pub channel_id: Option<serenity::all::ChannelId>,
}
