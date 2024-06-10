use crate::silverpelt::value::Value;

pub struct State {
    pub state: indexmap::IndexMap<String, Value>,
}

impl State {
    /// Given a template string, where state variables are surrounded by curly braces, return the
    /// string with the state variables replaced with their values
    pub fn template_to_string(&self, template: &str) -> String {
        let mut result = template.to_string();
        for (key, value) in &self.state {
            result = result.replace(&format!("{{{}}}", key), &value.to_string());
        }
        result
    }

    // Creates a new state
    pub fn new() -> Self {
        State {
            state: indexmap::IndexMap::new(),
        }
    }
}
