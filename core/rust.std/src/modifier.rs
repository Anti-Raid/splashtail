#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum Modifier {
    User(serenity::all::UserId),
    Channel(serenity::all::ChannelId),
    Role(serenity::all::RoleId),
    Global,
}

impl Modifier {
    pub fn from_repr(repr: &str) -> Result<Self, crate::Error> {
        let mut parts = repr.splitn(2, '/');

        let target = match parts.next() {
            Some("user") => {
                let id = match parts.next() {
                    Some(id) => id.parse::<serenity::all::UserId>()?,
                    None => return Err(format!("Invalid modifier target: {}", repr).into()),
                };

                Modifier::User(id)
            }
            Some("channel") => {
                let id = match parts.next() {
                    Some(id) => id.parse::<serenity::all::ChannelId>()?,
                    None => return Err(format!("Invalid modifier target: {}", repr).into()),
                };

                Modifier::Channel(id)
            }
            Some("role") => {
                let id = match parts.next() {
                    Some(id) => id.parse::<serenity::all::RoleId>()?,
                    None => return Err(format!("Invalid modifier target: {}", repr).into()),
                };

                Modifier::Role(id)
            }
            Some("global") => Modifier::Global,
            _ => return Err(format!("Invalid modifier target: {}", repr).into()),
        };

        Ok(target)
    }

    pub fn specificity(&self) -> i32 {
        match self {
            Modifier::User(_) => 3, // Most specific
            Modifier::Channel(_) => 2,
            Modifier::Role(_) => 1,
            Modifier::Global => 0, // Least specific
        }
    }

    pub fn is_user(&self, user_id: serenity::all::UserId) -> bool {
        match self {
            Modifier::User(id) => *id == user_id,
            _ => false,
        }
    }

    pub fn is_channel(&self, channel_id: serenity::all::ChannelId) -> bool {
        match self {
            Modifier::Channel(id) => *id == channel_id,
            _ => false,
        }
    }

    pub fn is_role(&self, role_id: serenity::all::RoleId) -> bool {
        match self {
            Modifier::Role(id) => *id == role_id,
            _ => false,
        }
    }

    pub fn is_global(&self) -> bool {
        match self {
            Modifier::Global => true,
            _ => false,
        }
    }

    /// Helper method to check if a modifier contains a role modifier
    ///
    /// Note: As all role modifiers have the same specificity, this just returns a bool to save on computation
    pub fn contains_role_modifier(modifiers: &[Self]) -> bool {
        for modifier in modifiers {
            if matches!(modifier, Modifier::Role(_)) {
                return true;
            }
        }

        false
    }

    /// Check if a member matches this modifier
    pub fn matches_member(
        &self,
        member: &serenity::all::Member,
        channel_id: Option<serenity::all::ChannelId>,
    ) -> bool {
        if self.is_global() {
            return true;
        }

        if self.is_user(member.user.id) {
            return true;
        }

        for role in member.roles.iter() {
            if self.is_role(*role) {
                return true;
            }
        }

        if let Some(channel_id) = channel_id {
            if self.is_channel(channel_id) {
                return true;
            }
        }

        false
    }

    /// Check if a user id matches this modifier
    pub fn matches_user_id(
        &self,
        user_id: serenity::all::UserId,
        channel_id: Option<serenity::all::ChannelId>,
    ) -> bool {
        if self.is_global() {
            return true;
        }

        if self.is_user(user_id) {
            return true;
        }

        if let Some(channel_id) = channel_id {
            if self.is_channel(channel_id) {
                return true;
            }
        }

        false
    }

    /// Helper method to check if a member matches a list of modifiers
    pub fn set_matches_member(
        modifiers: &[Self],
        member: &serenity::all::Member,
        channel_id: Option<serenity::all::ChannelId>,
    ) -> Vec<Modifier> {
        let mut matches = Vec::new();
        for modifier in modifiers {
            if modifier.matches_member(member, channel_id) {
                matches.push(*modifier);
            }
        }

        matches
    }

    /// Helper method to check if a user id matches a list of modifiers
    ///
    /// Note that unlike `set_matches_member`, this method does not check for role as user objects do not contain role information
    pub fn set_matches_user_id(
        modifiers: &[Self],
        user_id: serenity::all::UserId,
        channel_id: Option<serenity::all::ChannelId>,
    ) -> Vec<Modifier> {
        let mut matches = Vec::new();
        for modifier in modifiers {
            if modifier.matches_user_id(user_id, channel_id) {
                matches.push(*modifier);
            }
        }
        matches
    }
}
