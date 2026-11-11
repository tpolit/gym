use leptos::ev::PointerEvent;
use leptos::prelude::*;

const SWIPE_THRESHOLD_PX: i32 = 80;
const TAP_TOLERANCE_PX: i32 = 8;
const MAX_OFFSET_PX: i32 = 120;

// Swipe left / right on a row to act on it, tap to open it.
// Vertical scrolling stays native thanks to `touch-action: pan-y`.
#[component]
pub fn SwipeRow(
    left_action: Option<(&'static str, Callback<()>)>,
    right_action: Option<(&'static str, Callback<()>)>,
    on_tap: Callback<()>,
    #[prop(optional)] class: &'static str,
    children: Children,
) -> impl IntoView {
    let start_x = StoredValue::new(None::<i32>);
    let offset = RwSignal::new(0_i32);
    let allows_left = left_action.is_some();
    let allows_right = right_action.is_some();

    let on_down = move |event: PointerEvent| start_x.set_value(Some(event.client_x()));
    let on_move = move |event: PointerEvent| {
        if let Some(start) = start_x.get_value() {
            let delta = event.client_x().saturating_sub(start);
            let allowed = (delta < 0 && allows_left) || (delta > 0 && allows_right);
            offset.set(if allowed {
                delta.clamp(-MAX_OFFSET_PX, MAX_OFFSET_PX)
            } else {
                0
            });
        }
    };
    let on_up = move |_: PointerEvent| {
        let delta = offset.get_untracked();
        let was_pressed = start_x.get_value().is_some();
        start_x.set_value(None);
        offset.set(0);
        if !was_pressed {
            return;
        }
        if delta <= -SWIPE_THRESHOLD_PX {
            if let Some((_, action)) = left_action {
                action.run(());
            }
        } else if delta >= SWIPE_THRESHOLD_PX {
            if let Some((_, action)) = right_action {
                action.run(());
            }
        } else if delta.abs() < TAP_TOLERANCE_PX {
            on_tap.run(());
        }
    };
    let on_cancel = move |_: PointerEvent| {
        start_x.set_value(None);
        offset.set(0);
    };

    view! {
        <div class=format!("swipe {class}")>
            <div class="swipe__hints" aria-hidden="true">
                <span class="swipe__hint swipe__hint--right">{right_action.map(|(label, _)| label)}</span>
                <span class="swipe__hint swipe__hint--left">{left_action.map(|(label, _)| label)}</span>
            </div>
            <div
                class="swipe__content"
                class:swipe__content--moving=move || offset.get() != 0
                style:transform=move || format!("translateX({}px)", offset.get())
                on:pointerdown=on_down
                on:pointermove=on_move
                on:pointerup=on_up
                on:pointercancel=on_cancel
                on:pointerleave=on_cancel
            >
                {children()}
            </div>
        </div>
    }
}
