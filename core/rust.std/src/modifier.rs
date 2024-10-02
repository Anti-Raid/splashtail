use indexmap::IndexMap;

/// A modifier matcher is used to match modifiers using a simple syntax
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct ModifierMatcher {
    pub user_ids: Vec<serenity::all::UserId>,
    pub channel_ids: Vec<serenity::all::ChannelId>,
    pub role_ids: Vec<serenity::all::RoleId>,
    pub variables: IndexMap<String, String>,
}

impl ModifierMatcher {
    pub fn add_user_id(&mut self, user_id: serenity::all::UserId) {
        self.user_ids.push(user_id);
    }

    pub fn add_channel_id(&mut self, channel_id: serenity::all::ChannelId) {
        self.channel_ids.push(channel_id);
    }

    pub fn add_role_id(&mut self, role_id: serenity::all::RoleId) {
        self.role_ids.push(role_id);
    }

    pub fn add_variable(&mut self, key: String, value: String) {
        self.variables.insert(key, value);
    }

    pub fn add_variables(&mut self, variables: IndexMap<String, String>) {
        self.variables.extend(variables);
    }

    pub fn add_member(&mut self, member: &serenity::all::Member) {
        self.add_user_id(member.user.id);
        
        for role in member.roles.iter() {
            self.add_role_id(*role);
        }
    }

    /// Matches a single modifier
    pub fn match_modifier(&self, modifier: &Modifier) -> bool {
        match modifier {
            Modifier::User(id) => self.user_ids.contains(id),
            Modifier::Channel(id) => self.channel_ids.contains(id),
            Modifier::Role(id) => self.role_ids.contains(id),
            Modifier::Custom((key, value, _)) => {
                if let Some(v) = self.variables.get(key) {
                    v == value
                } else {
                    false
                }
            }
            Modifier::Global => true,
        }
    }

    /// Matches a list of modifiers returning the modifiers that match
    pub fn match_modifiers(&self, modifiers: Vec<Modifier>) -> Vec<Modifier> {
        let mut matches = Vec::new();
        for modifier in modifiers {
            if self.match_modifier(&modifier) {
                matches.push(modifier);
            }
        }

        matches
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum Modifier {
    User(serenity::all::UserId),
    Channel(serenity::all::ChannelId),
    Role(serenity::all::RoleId),
    Custom((String, String, i32)),
    Global,
}

impl Modifier {
    pub fn from_repr(repr: &str) -> Result<Self, crate::Error> {
        let mut parts = repr.splitn(3, '/');

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
            Some("custom") => {
                let key = match parts.next() {
                    Some(key) => key.to_string(),
                    None => return Err(format!("Invalid modifier target: {}", repr).into()),
                };

                let value = match parts.next() {
                    Some(value) => value.to_string(),
                    None => return Err(format!("Invalid modifier target: {}", repr).into()),
                };

                let specificity = match parts.next() {
                    Some(specificity) => specificity.parse::<i32>()?,
                    None => return Err(format!("Invalid modifier target: {}", repr).into()),
                };

                Modifier::Custom((key, value, specificity))
            }
            Some("global") => Modifier::Global,
            _ => return Err(format!("Invalid modifier target: {}", repr).into()),
        };

        Ok(target)
    }

    /// Returns the specificity of a modifier which is used to resolve conflicts
    pub fn specificity(&self) -> i32 {
        match self {
            Modifier::Custom((_, _, specificity)) => *specificity,
            Modifier::User(_) => 3, // Most specific
            Modifier::Channel(_) => 2,
            Modifier::Role(_) => 1,
            Modifier::Global => 0, // Least specific
        }
    }
}

/// Implement partial ordering for modifiers based on specificity
impl PartialOrd for Modifier {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

/// Implement ordering for modifiers based on specificity
impl Ord for Modifier {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.specificity().cmp(&other.specificity())
    }
}
