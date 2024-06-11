use crate::silverpelt::value::Value;

pub struct State {
    pub state: indexmap::IndexMap<String, Value>,
}

impl State {
    /// Given a template string, where state variables are surrounded by curly braces, return the
    /// template value (if a single variable) or a string if not
    pub fn template_to_string(&self, template: &str) -> Value {
        let mut result = template.to_string();

        if result.starts_with("{") && result.ends_with("}") {
            return self
                .state
                .get(&template[1..template.len() - 1])
                .cloned()
                .unwrap_or(Value::String(result));
        }

        for (key, value) in &self.state {
            result = result.replace(&format!("{{{}}}", key), &value.to_string());
        }

        Value::String(result)
    }

    // Creates a new state
    pub fn new() -> Self {
        State {
            state: indexmap::IndexMap::new(),
        }
    }
}
