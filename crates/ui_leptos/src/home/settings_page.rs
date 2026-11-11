use features::home::body_weight_view;
use leptos::prelude::*;

use super::body_weight::BodyWeightForm;
use crate::navigation::{Navigation, Screen};
use crate::store::Store;
use crate::ui::{TitleVisibility, TopBar};

#[component]
pub fn SettingsPage(store: Store, navigation: Navigation) -> impl IntoView {
    let view_model = Memo::new(move |_| store.with(body_weight_view));
    let initial = view_model.with_untracked(|view_model| view_model.input.clone());

    view! {
        <main class="screen">
            <TopBar
                on_back=Callback::new(move |()| navigation.back())
                title="Réglages"
                title_visibility=TitleVisibility::OnceScrolled
            />
            <h1 class="title">"Réglages"</h1>
            <section class="card">
                <BodyWeightForm
                    store
                    label="Poids de corps"
                    hint="Un nouveau poids s'applique aux séances à partir d'aujourd'hui ; les séances passées gardent le poids de leur date."
                    initial
                />
            </section>
            {move || {
                let history = view_model.with(|view_model| view_model.history.clone());
                (!history.is_empty()).then(|| view! {
                    <p class="eyebrow">"Historique du poids"</p>
                    <div class="list">
                        {history
                            .into_iter()
                            .map(|row| view! {
                                <div class="settings__weight">
                                    <span class="muted">{row.date_text}</span>
                                    <span>{row.weight_text}</span>
                                </div>
                            })
                            .collect_view()}
                    </div>
                })
            }}
            <button type="button" class="btn btn--secondary" on:click=move |_| navigation.go_to(Screen::Backup)>
                "Sauvegarde des données"
            </button>
            <p class="muted settings__credits">
                "Exercices : free-exercise-db (domaine public) · "
                <a href="https://repdb.co" target="_blank" rel="noopener">"Exercise data by RepDB (repdb.co)"</a>
            </p>
        </main>
    }
}
