use domain::MuscleGroup;
use features::shared::muscle_filter::{muscle_chips, toggle_muscle};
use leptos::prelude::*;

#[component]
pub fn MuscleChips(
    selected: Signal<Option<MuscleGroup>>,
    on_select: Callback<Option<MuscleGroup>>,
) -> impl IntoView {
    let chip = move |group: MuscleGroup, label: &'static str| {
        let is_on = move || selected.get() == Some(group);
        view! {
            <button
                type="button"
                class="chip"
                class:chip--on=is_on
                aria-pressed=move || if is_on() { "true" } else { "false" }
                on:click=move |_| on_select.run(toggle_muscle(selected.get_untracked(), group))
            >
                {label}
            </button>
        }
    };
    view! {
        <div class="chips muscle-chips" role="group" aria-label="Filtrer par muscle">
            {muscle_chips().into_iter().map(|muscle| chip(muscle.group, muscle.label)).collect_view()}
        </div>
    }
}
