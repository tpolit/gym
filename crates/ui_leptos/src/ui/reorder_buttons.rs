use leptos::prelude::*;

#[component]
pub fn ReorderButtons(
    can_move_up: bool,
    can_move_down: bool,
    on_move_up: Callback<()>,
    on_move_down: Callback<()>,
) -> impl IntoView {
    view! {
        <span class="reorder">
            <button
                type="button"
                class="reorder__button"
                aria-label="Monter"
                disabled=!can_move_up
                on:click=move |_| on_move_up.run(())
            >
                "↑"
            </button>
            <button
                type="button"
                class="reorder__button"
                aria-label="Descendre"
                disabled=!can_move_down
                on:click=move |_| on_move_down.run(())
            >
                "↓"
            </button>
        </span>
    }
}
