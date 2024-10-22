// https://docs.rs/thread-local-panic-hook/latest/src/thread_local_panic_hook/panic.rs.html#24-30
use std::cell::RefCell;
use std::ops::Deref;
use std::panic::PanicHookInfo as PanicInfo;
use std::sync::Once;
use std::{panic, thread};

static SET_HOOK: Once = Once::new();
static mut DEFAULT_HOOK: Option<Hook> = None;

thread_local! {
    static PANIC_HOOK: RefCell<Option<Hook>> = const { RefCell::new(None) };
}

#[derive(Copy, Clone)]
struct Hook(*mut (dyn Fn(&PanicInfo<'_>) + 'static + Sync + Send));

impl Hook {
    pub fn new(f: impl Fn(&PanicInfo<'_>) + 'static + Sync + Send) -> Self {
        Self(Box::into_raw(Box::new(f)))
    }
}

#[allow(dead_code)]
/// See [`std::panic::set_hook`]
pub fn set_hook(hook: Box<dyn Fn(&PanicInfo<'_>) + 'static + Sync + Send>) {
    common();

    PANIC_HOOK.with(|cell| {
        cell.replace(Some(Hook::new(hook)));
    });
}

#[allow(dead_code)]
/// See [`std::panic::take_hook`]
pub fn take_hook() -> Box<dyn Fn(&PanicInfo<'_>) + 'static + Sync + Send> {
    common();

    PANIC_HOOK.with(|cell| match cell.take() {
        None => unsafe { Box::from_raw(DEFAULT_HOOK.unwrap().0) },
        Some(Hook(ptr)) => unsafe { Box::from_raw(ptr) },
    })
}

#[allow(dead_code)]
/// See [`std::panic::update_hook`]
pub fn update_hook<F>(hook_fn: F)
where
    F: Fn(&(dyn Fn(&PanicInfo<'_>) + Send + Sync + 'static), &PanicInfo<'_>)
        + Sync
        + Send
        + 'static,
{
    let old = take_hook();
    set_hook(Box::new(move |e| {
        hook_fn(&old, e);
    }));
}

#[inline]
fn common() {
    if thread::panicking() {
        panic!("cannot modify the panic hook from a panicking thread");
    }

    SET_HOOK.call_once(|| {
        unsafe {
            DEFAULT_HOOK = Some(Hook::new(panic::take_hook()));
        }
        panic::set_hook(Box::new(default));
    });
}

fn default(e: &PanicInfo) {
    PANIC_HOOK.with(move |cell| {
        let borrow = cell.borrow();
        unsafe {
            let hook = borrow.deref().unwrap_or_else(|| DEFAULT_HOOK.unwrap());
            let ptr = hook.0;
            (*ptr)(e);
        }
    })
}
