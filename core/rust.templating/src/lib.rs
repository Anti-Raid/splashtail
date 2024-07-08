use moka::future::Cache;
use once_cell::sync::Lazy;
use std::sync::{Arc, RwLock};
use std::time::Duration;
use tera::Tera;

/// Maximum number of AST nodes in a template
pub const MAX_TEMPLATE_NODES: usize = 512;

/// Timeout for template execution
pub const TEMPLATE_EXECUTION_TIMEOUT: Duration = Duration::from_millis(300);

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

    tera.autoescape_on(vec![]);

    // Add main template
    tera.add_raw_template("main", template)?;

    let mut total_nodes = 0;
    for (_, t) in tera.templates.iter() {
        total_nodes += t.ast.len();

        if total_nodes > MAX_TEMPLATE_NODES {
            return Err("Template has too many nodes".into());
        }
    }

    // TODO: Enable this if required
    // Every 3 nodes, insert a check_time Node
    // This is to prevent long-running templates

    /*for (_, t) in tera.templates.iter_mut() {
        for j in 0..t.ast.len() {
            if j % 3 == 0 {
                t.ast.insert(
                    j,
                    tera::ast::Node::VariableBlock(
                        tera::ast::WS {
                            left: true,
                            right: true,
                        },
                        tera::ast::Expr::new(tera::ast::ExprVal::String("check_time".to_string())),
                    ),
                );
            }
        }
    }*/

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
pub async fn execute_template(
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
