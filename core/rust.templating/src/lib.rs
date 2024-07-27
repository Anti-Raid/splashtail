use moka::future::Cache;
use once_cell::sync::Lazy;
use std::time::Duration;
use tera::Tera;

// Re-export Tera as Engine
pub mod engine {
    pub use tera::Context;
    pub use tera::Filter;
    pub use tera::Function;
    pub use tera::Result;
    pub use tera::Tera as Engine;
    pub use tera::Value;
}

pub mod lang_javascript_quickjs;
pub mod lang_javascript_v8__notworking;
pub mod lang_rhai;
pub mod message;
pub mod permissions;

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

/// Maximum number of AST nodes in a template
pub const MAX_TEMPLATE_NODES: usize = 1024;

/// Timeout for template execution
pub const TEMPLATE_EXECUTION_TIMEOUT: Duration = Duration::from_millis(600);
pub const MAX_TEMPLATE_MEMORY_USAGE: usize = 1024 * 1024; // 1 MB maximum memory

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

pub fn make_templating_context() -> tera::Context {
    tera::Context::new()
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

    if opts.cache_result {
        // Store the template in the cache
        TEMPLATE_CACHE
            .insert(template.to_string(), tera.clone())
            .await;
    }

    Ok(tera)
}

/// Executes a template with the given context returning the resultant string
///
/// Note that for message templates, the `execute_template_for_message` function should be used instead
pub async fn execute_template(
    tera: &mut Tera,
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
