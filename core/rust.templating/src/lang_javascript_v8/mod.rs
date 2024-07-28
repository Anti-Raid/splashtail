// Total number of threads to run, each thread will have 1 isolate
#[allow(dead_code)]
const NUM_THREADS: usize = 10;

#[allow(dead_code)]
pub fn new_isolate() -> v8::OwnedIsolate {
    extern "C" fn oom_handler(_: *const std::os::raw::c_char, _: &v8::OomDetails) {
        println!("OOM!");
        panic!("OOM! I should never happen")
    }

    extern "C" fn heap_limit_callback(
        data: *mut core::ffi::c_void,
        current_heap_limit: usize,
        _initial_heap_limit: usize,
    ) -> usize {
        println!("heap limit callback! {}", current_heap_limit);
        let isolate = unsafe { &mut *(data as *mut v8::Isolate) };
        // murder the isolate
        let terminated = isolate.terminate_execution();
        println!("near limit! {:?}", terminated);

        current_heap_limit * 2 // give us some space to kill it
    }

    const MB: usize = 1024 * 1024;

    let mut isolate = v8::Isolate::new(v8::CreateParams::default().heap_limits(0, MB));

    isolate.set_oom_error_handler(oom_handler);

    // Cast the isolate pointer to *mut c_void
    let isolate_ptr: &mut v8::Isolate = &mut isolate;
    let data: *mut core::ffi::c_void = isolate_ptr as *mut v8::Isolate as *mut core::ffi::c_void;

    isolate.add_near_heap_limit_callback(heap_limit_callback, data);

    isolate
}

#[cfg(test)]
mod test {
    #[tokio::test]
    async fn test_v8() {
        tokio::task::spawn_blocking(move || {
            let platform = v8::new_default_platform(0, false).make_shared();
            v8::V8::initialize_platform(platform);
            v8::V8::initialize();

            let mut isolate = super::new_isolate();

            for _ in 0..10000 {
                {
                    let scope = &mut v8::HandleScope::new(&mut isolate);
                    let context = v8::Context::new(scope);
                    let scope = &mut v8::ContextScope::new(scope, context);

                    let code =
                        v8::String::new(scope, r#"Array(100000000).fill('a').join('')"#).unwrap();

                    let script = v8::Script::compile(scope, code, None).unwrap();
                    script.run(scope).unwrap();
                }
            }

            Ok::<(), base_data::Error>(())
        })
        .await
        .unwrap()
        .unwrap();
    }
}
