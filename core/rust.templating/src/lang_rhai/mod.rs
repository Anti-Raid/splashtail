use moka::future::Cache;
use once_cell::sync::Lazy;
use rhai::exported_module;
use rhai::module_resolvers::StaticModuleResolver;
use std::time::Duration;

pub mod plugins;

static ENGINE: Lazy<rhai::Engine> = Lazy::new(|| {
    let mut engine = rhai::Engine::new();

    let mut resolver = StaticModuleResolver::new();

    resolver.insert("message", exported_module!(plugins::message::plugin));
    resolver.insert(
        "permissions",
        exported_module!(plugins::permissions::plugin),
    );

    engine.set_module_resolver(resolver);

    engine
});

/// Stores a cache of templates with the template content as key
static TEMPLATE_CACHE: Lazy<Cache<String, rhai::AST>> = Lazy::new(|| {
    Cache::builder()
        .time_to_live(Duration::from_secs(60 * 60))
        .build()
});

pub async fn execute<T: Clone + rhai::Variant>(
    template: &str,
    args: indexmap::IndexMap<String, serde_json::Value>,
    compile_opts: crate::CompileTemplateOptions,
) -> Result<T, base_data::Error> {
    let ast = {
        if compile_opts.ignore_cache {
            ENGINE.compile(template)?
        } else {
            match TEMPLATE_CACHE.get(&template.to_string()).await {
                Some(ast) => ast.clone(),
                None => {
                    let compiled = ENGINE.compile(template)?;
                    TEMPLATE_CACHE
                        .insert(template.to_string(), compiled.clone())
                        .await;

                    compiled
                }
            }
        }
    };

    let mut scope = rhai::Scope::new();
    let dyn_val: rhai::Dynamic =
        rhai::serde::to_dynamic(&args).map_err(|e| format!("Failed to deserialize args: {}", e))?;
    scope.set_value("args", args);
    scope.set_value("args_dyn", dyn_val);

    let v: T = ENGINE.eval_ast_with_scope(&mut scope, &ast)?;

    Ok(v)
}
