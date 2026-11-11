use wasm_bindgen::JsValue;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServiceWorkerError {
    NoWindow,
    Unsupported,
}

pub fn register_service_worker(script_url: &str) -> Result<(), ServiceWorkerError> {
    let window = web_sys::window().ok_or(ServiceWorkerError::NoWindow)?;
    let navigator = window.navigator();
    // `navigator.serviceWorker` is missing outside a secure context: calling it would fail.
    let supported =
        js_sys::Reflect::has(&navigator, &JsValue::from_str("serviceWorker")).unwrap_or(false);
    if !supported {
        return Err(ServiceWorkerError::Unsupported);
    }
    // Promise not awaited: the browser reports installation failures in the console.
    let _installation = navigator.service_worker().register(script_url);
    Ok(())
}
