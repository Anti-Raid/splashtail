use moka::future::Cache;
use once_cell::sync::Lazy;
use std::sync::{Arc, RwLock};
use std::time::Duration;
use tera::Tera;

/// Stores a cache of templates with the template content as key
static TEMPLATE_CACHE: Lazy<Cache<String, Tera>> = Lazy::new(|| {
    Cache::builder()
        .time_to_live(Duration::from_secs(60 * 60))
        .build()
});

pub struct CompileTemplateOptions {
    /// Cache the result of the template compilation
    pub cache_result: bool,
    /// Ignore the cache and compile the template again
    pub ignore_cache: bool,
}

pub async fn compile_template(
    template: &str,
    opts: CompileTemplateOptions,
) -> Result<Tera, base_data::Error> {
    if !opts.ignore_cache {
        // Check if in template
        if let Some(ref tera) = TEMPLATE_CACHE.get(template).await {
            return Ok(tera.clone());
        }
    }

    // Compile a new template
    let mut tera = Tera::default();

    // Add main template
    tera.add_raw_template("main", template)?;

    if opts.cache_result {
        // Store the template in the cache
        TEMPLATE_CACHE
            .insert(template.to_string(), tera.clone())
            .await;
    }

    Ok(tera)
}

#[derive(Debug, Default)]
struct InternalTemplateExecuteState {
    /// The fields that were set by the template
    fields: RwLock<indexmap::IndexMap<String, String>>,
    // TODO: Add more properties
}

struct FieldFunction {
    state: Arc<InternalTemplateExecuteState>,
}

impl tera::Function for FieldFunction {
    fn call(
        &self,
        args: &std::collections::HashMap<String, tera::Value>,
    ) -> tera::Result<tera::Value> {
        let field_name = args.get("name").ok_or("Field name not provided")?;
        let field_value = args.get("value").ok_or("Field not found")?;

        // Lock fields for writing
        let mut fields_writer = self
            .state
            .fields
            .write()
            .map_err(|_| "Failed to write to fields")?;

        // Insert the field
        fields_writer.insert(field_name.to_string(), field_value.to_string());

        // Drop the lock
        drop(fields_writer);

        Ok(tera::Value::Null)
    }
}

struct StubFunction {}

impl tera::Function for StubFunction {
    fn call(
        &self,
        _args: &std::collections::HashMap<String, tera::Value>,
    ) -> tera::Result<tera::Value> {
        Ok(tera::Value::Null)
    }
}

pub struct ExecutedTemplate {
    pub description: String,
    pub fields: indexmap::IndexMap<String, String>,
}

/// Executes a template with the given context
pub fn execute_template(
    tera: &mut Tera,
    context: &tera::Context,
) -> Result<ExecutedTemplate, base_data::Error> {
    let ites = Arc::new(InternalTemplateExecuteState::default());

    // Add field function
    tera.register_function(
        "field",
        FieldFunction {
            state: ites.clone(),
        },
    );

    // Stub out dangerous functions
    tera.register_function("get_env", StubFunction {});

    // Render the template
    let rendered = tera.render("main", context)?;

    // Read the fields now
    let fields_reader = ites.fields.read().map_err(|_| "Failed to read fields")?;

    Ok(ExecutedTemplate {
        description: rendered,
        fields: (*fields_reader).clone(),
    })
}

/// Spawns execute template with the given context using ``tokio::task::spawn_blocking``
pub async fn spawn_execute_template(
    mut tera: Tera,
    context: tera::Context,
) -> Result<ExecutedTemplate, base_data::Error> {
    tokio::task::spawn_blocking(move || execute_template(&mut tera, &context))
        .await
        .map_err(base_data::Error::from)?
}
