use domain::{ExerciseId, MuscleGroup};
use features::shared::exercise_choice::{ChoiceSection, ExerciseChoice, ExerciseChoices};
use leptos::prelude::*;

use crate::ui::MuscleChips;

#[component]
pub fn ExercisePicker(
    choices: ExerciseChoices,
    on_pick: Callback<ExerciseId>,
    on_close: Callback<()>,
) -> impl IntoView {
    let choices = StoredValue::new(choices);
    let query = RwSignal::new(String::new());
    let muscle = RwSignal::new(None::<MuscleGroup>);
    let choice_button = move |choice: ExerciseChoice| {
        let id = choice.id;
        view! {
            <button type="button" class="picker__item" on:click=move |_| on_pick.run(id.clone())>
                <span class="picker__name">{choice.name}</span>
                <span class="picker__detail">{choice.detail}</span>
            </button>
        }
    };
    let section = move |section: ChoiceSection| {
        view! {
            <p class="eyebrow picker__section">{section.title}</p>
            <div class="list">{section.choices.into_iter().map(choice_button).collect_view()}</div>
        }
    };
    let results = move || {
        let sections =
            query.with(|query| choices.with_value(|choices| choices.sections(query, muscle.get())));
        if sections.is_empty() {
            view! { <p class="muted picker__empty">"Aucun exercice trouvé."</p> }.into_any()
        } else {
            sections.into_iter().map(section).collect_view().into_any()
        }
    };

    view! {
        <div class="sheet-backdrop" on:click=move |_| on_close.run(())></div>
        <div class="sheet sheet--tall picker" role="dialog" aria-label="Ajouter un exercice">
            <div class="picker__top">
                <div class="picker__header">
                    <input
                        class="input"
                        type="search"
                        placeholder="Rechercher un exercice"
                        prop:value=move || query.get()
                        on:input=move |event| query.set(event_target_value(&event))
                    />
                    <button type="button" class="btn btn--ghost" on:click=move |_| on_close.run(())>
                        "Fermer"
                    </button>
                </div>
                <MuscleChips
                    selected=muscle.into()
                    on_select=Callback::new(move |group| muscle.set(group))
                />
            </div>
            <div class="picker__results">{results}</div>
        </div>
    }
}
