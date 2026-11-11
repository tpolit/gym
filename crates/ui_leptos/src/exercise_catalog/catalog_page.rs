use features::exercise_catalog::{CatalogFilter, CatalogItem, Family, search_exercises};
use leptos::prelude::*;

use crate::navigation::{Navigation, Screen};
use crate::store::Store;
use crate::ui::MuscleChips;

#[component]
pub fn CatalogPage(store: Store, navigation: Navigation) -> impl IntoView {
    let filter = RwSignal::new(CatalogFilter::default());
    let items =
        Memo::new(move |_| filter.with(|filter| store.with(|db| search_exercises(db, filter))));

    let family_chip = move |family: Option<Family>| {
        let label = family.map_or("Tous", Family::label);
        view! {
            <button
                type="button"
                class="chip"
                class:chip--on=move || filter.with(|filter| filter.family == family)
                on:click=move |_| filter.update(|filter| filter.family = family)
            >
                {label}
            </button>
        }
    };
    let row = move |item: CatalogItem| {
        let exercise_id = item.id;
        view! {
            <button
                type="button"
                class="picker__item"
                on:click=move |_| navigation.go_to(Screen::ExerciseDetail(exercise_id.clone()))
            >
                <span class="picker__name">
                    {item.name}
                    {item.is_custom.then_some(" · perso")}
                    {item.is_archived.then_some(" · archivé")}
                </span>
                <span class="picker__detail">{item.detail}</span>
            </button>
        }
    };

    view! {
        <main class="screen">
            <h1 class="title catalog__title">"Exercices"</h1>
            <input
                class="input"
                type="search"
                placeholder="Rechercher un exercice"
                prop:value=move || filter.with(|filter| filter.query.clone())
                on:input=move |event| filter.update(|filter| filter.query = event_target_value(&event))
            />
            <div class="chips">
                {family_chip(None)}
                {Family::ALL.into_iter().map(|family| family_chip(Some(family))).collect_view()}
            </div>
            <MuscleChips
                selected=Signal::derive(move || filter.with(|filter| filter.muscle_group))
                on_select=Callback::new(move |group| filter.update(|filter| filter.muscle_group = group))
            />
            <label class="toggle">
                <input
                    type="checkbox"
                    prop:checked=move || filter.with(|filter| filter.show_archived)
                    on:change=move |event| filter.update(|filter| filter.show_archived = event_target_checked(&event))
                />
                <span>"Afficher les exercices archivés"</span>
            </label>
            <button
                type="button"
                class="btn btn--secondary"
                on:click=move |_| navigation.go_to(Screen::CustomExerciseForm(None))
            >
                "+ Créer un exercice"
            </button>
            {move || {
                let items = items.get();
                if items.is_empty() {
                    view! { <p class="muted">"Aucun exercice trouvé."</p> }.into_any()
                } else {
                    view! { <div class="list">{items.into_iter().map(row).collect_view()}</div> }.into_any()
                }
            }}
        </main>
    }
}
