use domain::ExerciseId;
use features::history::{HistoryItem, history, history_exercise_choices, history_filter_title};
use leptos::prelude::*;

use crate::navigation::{Navigation, Screen};
use crate::store::Store;

const ALL_EXERCISES: &str = "";

// The filter lives in the screen itself, so it survives a visit to a session detail.
#[component]
pub fn HistoryPage(
    store: Store,
    navigation: Navigation,
    filter: Option<ExerciseId>,
) -> impl IntoView {
    let filter = StoredValue::new(filter);
    let items = Memo::new(move |_| {
        filter.with_value(|filter| store.with(|db| history(db, filter.as_ref())))
    });
    let choices = Memo::new(move |_| store.with(history_exercise_choices));
    let title = move || {
        filter.with_value(|filter| store.with(|db| history_filter_title(db, filter.as_ref())))
    };

    let choose = move |event: leptos::ev::Event| {
        let selected = event_target_value(&event);
        let exercise = choices.with_untracked(|choices| {
            selected
                .parse::<usize>()
                .ok()
                .and_then(|index| choices.get(index))
                .map(|choice| choice.id.clone())
        });
        navigation.replace(Screen::History(exercise));
    };

    let row = move |item: HistoryItem| {
        let session_id = item.id;
        view! {
            <button
                type="button"
                class="history__row"
                on:click=move |_| navigation.go_to(Screen::SessionDetail(session_id))
            >
                <span class="history__name">{item.name}</span>
                <span class="history__date">{item.date_text}</span>
                <span class="muted history__summary">{item.summary}</span>
            </button>
        }
    };

    view! {
        <main class="screen">
            <h1 class="title history__title">"Historique"</h1>
            {move || {
                let choices = choices.get();
                (!choices.is_empty()).then(|| view! {
                    <label class="field">
                        <span class="eyebrow">"Exercice"</span>
                        <select class="input" on:change=choose>
                            <option value=ALL_EXERCISES selected=filter.with_value(Option::is_none)>
                                "Tous les exercices"
                            </option>
                            {choices
                                .into_iter()
                                .enumerate()
                                .map(|(index, choice)| {
                                    let is_selected = filter.with_value(|filter| filter.as_ref() == Some(&choice.id));
                                    view! {
                                        <option value=index.to_string() selected=is_selected>{choice.name}</option>
                                    }
                                })
                                .collect_view()}
                        </select>
                    </label>
                })
            }}
            <p class="muted">{title}</p>
            {move || {
                let items = items.get();
                if items.is_empty() {
                    let message = if filter.with_value(Option::is_some) {
                        "Aucune séance terminée avec cet exercice."
                    } else {
                        "Les séances terminées apparaîtront ici."
                    };
                    view! { <p class="muted">{message}</p> }.into_any()
                } else {
                    view! { <div class="list">{items.into_iter().map(row).collect_view()}</div> }.into_any()
                }
            }}
        </main>
    }
}
