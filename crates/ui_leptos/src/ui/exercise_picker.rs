use domain::ExerciseId;
use features::shared::exercise_choice::{ExerciseChoice, ExerciseChoices};
use leptos::prelude::*;

#[component]
pub fn ExercisePicker(
    choices: ExerciseChoices,
    on_pick: Callback<ExerciseId>,
    on_close: Callback<()>,
) -> impl IntoView {
    let choices = StoredValue::new(choices);
    let query = RwSignal::new(String::new());
    let choice_button = move |choice: ExerciseChoice| {
        let id = choice.id;
        view! {
            <button type="button" class="picker__item" on:click=move |_| on_pick.run(id.clone())>
                <span class="picker__name">{choice.name}</span>
                <span class="picker__detail">{choice.detail}</span>
            </button>
        }
    };
    let section = move |title: &'static str, list: Vec<ExerciseChoice>| {
        (!list.is_empty()).then(|| {
            view! {
                <p class="eyebrow picker__section">{title}</p>
                <div class="list">{list.into_iter().map(choice_button).collect_view()}</div>
            }
        })
    };
    let results = move || {
        let query = query.get();
        if query.trim().is_empty() {
            let (recent, all) =
                choices.with_value(|choices| (choices.recent.clone(), choices.all.clone()));
            return view! {
                {section("Récents", recent)}
                {section("Tous les exercices", all)}
            }
            .into_any();
        }
        let found: Vec<ExerciseChoice> = choices.with_value(|choices| {
            choices
                .all
                .iter()
                .filter(|choice| choice.matches(&query))
                .cloned()
                .collect()
        });
        if found.is_empty() {
            view! { <p class="muted picker__empty">"Aucun exercice trouvé."</p> }.into_any()
        } else {
            section("Résultats", found).into_any()
        }
    };

    view! {
        <div class="sheet-backdrop" on:click=move |_| on_close.run(())></div>
        <div class="sheet sheet--tall picker" role="dialog" aria-label="Ajouter un exercice">
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
            <div class="picker__results">{results}</div>
        </div>
    }
}
