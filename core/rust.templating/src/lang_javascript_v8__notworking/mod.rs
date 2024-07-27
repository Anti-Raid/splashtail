#[cfg(test)]
mod test {
    use core::ffi::c_void;

    #[tokio::test]
    async fn test_v8() {
        tokio::task::spawn_blocking(
            move || {
                extern "C" fn oom_handler(_: *const std::os::raw::c_char, _: &v8::OomDetails) {
                    println!("OOM!");
                    panic!("OOM! I should never happen")
                }
                
                extern "C" fn heap_limit_callback(
                    data: *mut c_void,
                    current_heap_limit: usize,
                    _initial_heap_limit: usize,
                ) -> usize {
                    println!("heap limit callback! {}", current_heap_limit);
                    let isolate = unsafe {&mut *(data as *mut v8::Isolate)};
                    // murder the isolate
                    let terminated = isolate.terminate_execution();
                    println!("near limit! {:?}", terminated);
                    current_heap_limit * 2 // give us some space to kill it
                }

                let platform = v8::new_default_platform(0, false).make_shared();
                v8::V8::initialize_platform(platform);
                v8::V8::initialize();

                const KB: usize = 1024;
                const MB: usize = 1024 * 1024;

                let mut rts = Vec::new();
                for i in 0..5000 {   
                    let mut isolate = v8::Isolate::new(v8::CreateParams::default());
                    
                    isolate.low_memory_notification();

                    isolate.set_oom_error_handler(oom_handler);

                    // Cast the isolate pointer to *mut c_void
                    let isolate_ptr: &mut v8::Isolate = &mut isolate;
                    let data: *mut c_void = isolate_ptr as *mut v8::Isolate as *mut c_void;
                    
                    isolate.add_near_heap_limit_callback(heap_limit_callback, data);

                    {
                        let scope = &mut v8::HandleScope::new(&mut isolate);
                        let context = v8::Context::new(scope);
                        let scope = &mut v8::ContextScope::new(scope, context);
                        
                        let code = v8::String::new(scope, "'Hello' + ' World!'").unwrap();
                        println!("javascript code: {}", code.to_rust_string_lossy(scope));
                        
                        let script = v8::Script::compile(scope, code, None).unwrap();
                        let result = script.run(scope).unwrap();
                        let result = result.to_string(scope).unwrap();
                        println!("result: {}", result.to_rust_string_lossy(scope));
                    }

                    if i & 100 == 0 {
                        rts.push(std::rc::Rc::new(isolate));
                    }
                }

                // Drop rts in reverse order
                rts.reverse();

                for rt in rts {
                    drop(rt);
                }

                Ok::<(), base_data::Error>(())
            }
        ).await.unwrap().unwrap();
    }
}