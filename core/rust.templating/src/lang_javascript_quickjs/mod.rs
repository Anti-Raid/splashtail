pub const MAX_TEMPLATE_MEMORY_USAGE: usize = 256 * 1024; // 256KB maximum memory

#[cfg(test)]
mod test {
    use quickjs_runtime::{
        builder::QuickJsRuntimeBuilder, jsutils::Script, quickjsrealmadapter::QuickJsRealmAdapter,
        quickjsruntimeadapter::QuickJsRuntimeAdapter,
    };

    #[tokio::test]
    async fn quickjs_test() {
        // Grow the stack if we are within the "red zone" of 32K, and if we allocate
        // a new stack allocate 1MB of stack space.
        //
        // If we're already in bounds, just run the provided closure on current stack.
        stacker::maybe_grow(32 * 1024, 1024 * 1024, || {
            // guaranteed to have at least 32K of stack
            println!("Increasing the stack size by 1MB");
        });

        // Increase vm.max_map_count
        // sudo sysctl -w vm.max_map_count=262144

        //let mut rts = Vec::new();

        // TODO: Make this not share state across the test if possible
        let rt = QuickJsRuntimeBuilder::new()
        .memory_limit(super::MAX_TEMPLATE_MEMORY_USAGE.try_into().unwrap())
        .max_stack_size(/* 1MB */ 1024 * 1024)
        .build();

        for i in 0..100000 {
            println!("{}", i);

            // with the first Option you may specify which realm to use, None indicates the default or main realm
            rt.eval(None, Script::new("test.js", r#"function a() {}"#)).await.unwrap();
            if i % 100 == 0 {
                rt.gc().await;
            }

            //rts.push(std::rc::Rc::new(rt));
        }
    }
}
