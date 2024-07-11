use crate::field::{CategorizedField, Field};
use templating::engine::{Filter, Result as EngineResult, Value};

/// Field formatter
pub struct FieldFormatter {
    /// Whether or not the template defaults to a CategorizedField versus a simple Field
    pub is_categorized_default: bool,
}

impl Filter for FieldFormatter {
    fn filter(
        &self,
        val: &Value,
        args: &std::collections::HashMap<String, Value>,
    ) -> EngineResult<Value> {
        let is_categorized = args
            .get("is_categorized")
            .map_or(self.is_categorized_default, |x| {
                x.as_bool().unwrap_or(self.is_categorized_default)
            });

        if is_categorized {
            let field: CategorizedField = serde_json::from_value(val.clone())
                .map_err(|e| format!("Failed to parse categorized field: {:?}", e))?;

            let formatted = field
                .field
                .template_format()
                .map_err(|e| format!("Failed to format categorized field: {:?}", e))?;

            Ok(Value::String(formatted))
        } else {
            let field: Field = serde_json::from_value(val.clone())
                .map_err(|e| format!("Failed to parse field: {:?}", e))?;

            let formatted = field
                .template_format()
                .map_err(|e| format!("Failed to format field: {:?}", e))?;

            Ok(Value::String(formatted))
        }
    }
}
