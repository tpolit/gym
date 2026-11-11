use std::cell::RefCell;
use std::rc::Rc;

use js_sys::{Function, Promise, Reflect};
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::Closure;
use wasm_bindgen_futures::{JsFuture, spawn_local};

const VISIBILITY_EVENT: &str = "visibilitychange";

// Keeps the screen on while alive. iOS releases the lock when the app goes to the
// background, so it is requested again whenever the page becomes visible. Browsers without
// the Screen Wake Lock API are silently ignored: the app works normally.
pub struct ScreenWakeLock {
    sentinel: Rc<RefCell<Option<JsValue>>>,
    on_visibility_change: Option<Closure<dyn FnMut()>>,
}

impl ScreenWakeLock {
    pub fn keep_screen_on() -> Self {
        let sentinel = Rc::new(RefCell::new(None));
        request_lock(Rc::clone(&sentinel));
        let on_visibility_change = web_sys::window().and_then(|window| {
            let document = window.document()?;
            let sentinel_for_event = Rc::clone(&sentinel);
            let visible_document = document.clone();
            let closure = Closure::<dyn FnMut()>::new(move || {
                if visible_document.visibility_state() == web_sys::VisibilityState::Visible {
                    request_lock(Rc::clone(&sentinel_for_event));
                }
            });
            document
                .add_event_listener_with_callback(
                    VISIBILITY_EVENT,
                    closure.as_ref().unchecked_ref(),
                )
                .ok()?;
            Some(closure)
        });
        Self {
            sentinel,
            on_visibility_change,
        }
    }
}

impl Drop for ScreenWakeLock {
    fn drop(&mut self) {
        if let Some(closure) = self.on_visibility_change.take()
            && let Some(document) = web_sys::window().and_then(|window| window.document())
        {
            let _ = document.remove_event_listener_with_callback(
                VISIBILITY_EVENT,
                closure.as_ref().unchecked_ref(),
            );
        }
        if let Some(sentinel) = self.sentinel.borrow_mut().take() {
            let _ = call_method(&sentinel, "release", &JsValue::UNDEFINED);
        }
    }
}

fn request_lock(sentinel: Rc<RefCell<Option<JsValue>>>) {
    let Some(wake_lock) = web_sys::window()
        .and_then(|window| Reflect::get(&window.navigator(), &JsValue::from_str("wakeLock")).ok())
        .filter(|wake_lock| !wake_lock.is_undefined())
    else {
        return;
    };
    let Some(promise) = call_method(&wake_lock, "request", &JsValue::from_str("screen"))
        .and_then(|result| result.dyn_into::<Promise>().ok())
    else {
        return;
    };
    spawn_local(async move {
        if let Ok(acquired) = JsFuture::from(promise).await {
            sentinel.replace(Some(acquired));
        }
    });
}

fn call_method(target: &JsValue, name: &str, argument: &JsValue) -> Option<JsValue> {
    let method: Function = Reflect::get(target, &JsValue::from_str(name))
        .ok()?
        .dyn_into()
        .ok()?;
    method.call1(target, argument).ok()
}
