use rhai::exported_module;
use rhai::module_resolvers::StaticModuleResolver;

pub mod plugins;

/// Timeout for template execution
pub const TEMPLATE_EXECUTION_TIMEOUT: std::time::Duration = std::time::Duration::from_millis(600);

pub static TEMPLATE_CACHE: once_cell::sync::Lazy<moka::sync::Cache<String, rhai::AST>> =
    once_cell::sync::Lazy::new(|| {
        moka::sync::Cache::builder()
            .time_to_live(std::time::Duration::from_secs(60 * 60))
            .build()
    });

pub fn create_engine() -> rhai::Engine {
    let mut engine = rhai::Engine::new();

    let mut resolver = StaticModuleResolver::new();

    resolver.insert("message", exported_module!(plugins::message::plugin));
    resolver.insert(
        "permissions",
        exported_module!(plugins::permissions::plugin),
    );

    engine.set_module_resolver(resolver);

    engine
}

/// To execute, first create the scope using `create_scope` and then call `eval_ast_with_scope` on the engine:
///
/// let v: T = ENGINE.eval_ast_with_scope(&mut scope, &ast)?;
pub fn compile(
    engine: &rhai::Engine,
    template: &str,
    compile_opts: crate::CompileTemplateOptions, // We don't support this yet
) -> Result<rhai::AST, base_data::Error> {
    let ast = {
        if compile_opts.ignore_cache {
            engine.compile(template)?
        } else {
            match TEMPLATE_CACHE.get(&template.to_string()) {
                Some(ast) => ast.clone(),
                None => {
                    let compiled = engine.compile(template)?;
                    TEMPLATE_CACHE.insert(template.to_string(), compiled.clone());

                    compiled
                }
            }
        }
    };

    Ok(ast)
}

pub fn apply_sandboxing(engine: &mut rhai::Engine) {
    engine.set_max_string_size(500); // allow strings only up to 500 bytes long (in UTF-8 format)
    engine.set_max_array_size(100); // allow arrays only up to 100 items
    engine.set_max_map_size(100); // allow object maps with only up to 500 properties
    engine.set_max_operations(100); // allow only up to 100 operations for this script
    engine.set_max_variables(20); // allow only up to 20 variables in the script
    engine.set_max_functions(5); // allow only up to 5 functions in the script
    engine.set_max_modules(5); // allow only up to 5 modules in the script
    engine.set_max_call_levels(3); // allow only up to 3 levels of function calls
    engine.set_max_expr_depths(20, 5); // allow only up to 20 levels of expression nesting

    let start = std::time::Instant::now(); // get the current system time

    engine.on_progress(move |_| {
        // Check 1: Execution timeout
        let now = std::time::Instant::now();

        if now.duration_since(start) > TEMPLATE_EXECUTION_TIMEOUT {
            // Return a dummy token just to force-terminate the script
            // after running for more than 60 seconds!
            return Some(rhai::Dynamic::UNIT);
        }

        None
    });
}

#[cfg(test)]
mod test {
    use super::*;

    pub fn create_scope<'a>(
        args: indexmap::IndexMap<String, serde_json::Value>,
    ) -> Result<rhai::Scope<'a>, base_data::Error> {
        let mut scope = rhai::Scope::new();
        let dyn_val: rhai::Dynamic = rhai::serde::to_dynamic(&args)
            .map_err(|e| format!("Failed to deserialize args: {}", e))?;

        scope.set_value("args", args);
        scope.set_value("args_dyn", dyn_val);

        Ok(scope)
    }

    #[tokio::test]
    async fn test_100000_concurrent() {
        let mut rts = Vec::new();

        for i in 0..100000 {
            println!("{}", i);

            let rt = tokio::task::spawn_blocking(move || {
                let engine = create_engine();
                //apply_sandboxing(&mut engine);

                let ast = match compile(
                    &engine,
                    "return 1",
                    crate::CompileTemplateOptions {
                        ignore_cache: false,
                        cache_result: false,
                    },
                ) {
                    Ok(ast) => ast,
                    Err(e) => {
                        println!("Error: {}", e);
                        panic!("Failed to compile template");
                    }
                };

                let mut scope = create_scope(indexmap::indexmap! {
                    "name".to_string() => serde_json::Value::String("world".to_string())
                })
                .unwrap();

                let _result: i64 = engine.eval_ast_with_scope(&mut scope, &ast).unwrap();
            });

            rts.push(rt);
        }

        for rt in rts {
            rt.await.unwrap();
        }
    }
}
