use wasm_bindgen::{
    prelude::{wasm_bindgen, Closure},
    JsCast,
};
use web_sys::{js_sys::global, DedicatedWorkerGlobalScope};

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    pub fn log(s: &str);
}

pub fn set_panic_hook() {
    // When the `console_error_panic_hook` feature is enabled, we can call the
    // `set_panic_hook` function at least once during initialization, and then
    // we will get better error messages if our code ever panics.
    //
    // For more details see
    // https://github.com/rustwasm/console_error_panic_hook#readme
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

pub fn worker() -> DedicatedWorkerGlobalScope {
    global().dyn_into().expect("global scope is a worker")
}

pub fn request_animation_frame(f: &Closure<dyn FnMut()>) -> i32 {
    worker()
        .request_animation_frame(f.as_ref().unchecked_ref())
        .expect("should register `requestAnimationFrame` OK")
}

pub fn cancel_animation_frame(h: i32) {
    worker()
        .cancel_animation_frame(h)
        .expect("should clear `requestAnimationFrame` OK")
}
