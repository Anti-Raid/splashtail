pub mod message;
pub mod permissions;

use moka::future::Cache;
use once_cell::sync::Lazy;

/// Maximum number of AST nodes in a template
pub const MAX_TEMPLATE_NODES: usize = 1024;

/// Timeout for template execution
pub const TEMPLATE_EXECUTION_TIMEOUT: std::time::Duration = std::time::Duration::from_millis(600);

#[allow(dead_code)]
pub const MAX_TEMPLATE_MEMORY_USAGE: usize = 1024 * 1024; // 1 MB maximum memory

/// Stores a cache of templates with the template content as key
static TEMPLATE_CACHE: Lazy<Cache<String, tera::Tera>> = Lazy::new(|| {
    Cache::builder()
        .time_to_live(std::time::Duration::from_secs(60 * 60))
        .build()
});

pub async fn compile_template(
    template: &str,
    opts: crate::CompileTemplateOptions,
) -> Result<tera::Tera, base_data::Error> {
    if !opts.ignore_cache {
        // Check if in template
        if let Some(ref tera) = TEMPLATE_CACHE.get(template).await {
            return Ok(tera.clone());
        }
    }

    log::info!("Compiling Tera template: {:?}", template);

    // Compile a new template
    let mut tera = tera::Tera::default();

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

    if opts.cache_result {
        // Store the template in the cache
        TEMPLATE_CACHE
            .insert(template.to_string(), tera.clone())
            .await;
    }

    Ok(tera)
}

/// Executes a template with the given context returning the resultant string
pub async fn execute_template(
    tera: &mut tera::Tera,
    context: &tera::Context,
) -> Result<String, base_data::Error> {
    // Render the template
    Ok(tokio::time::timeout(
        TEMPLATE_EXECUTION_TIMEOUT,
        tera.render_async("main", context),
    )
    .await
    .map_err(|_| "Template execution timed out")??)
}
