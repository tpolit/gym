use domain::SessionId;
use features::run_session::{SessionSummary, session_summary};
use leptos::prelude::*;

use crate::navigation::{Navigation, Screen};
use crate::store::Store;

#[component]
pub fn SessionRecapPage(
    store: Store,
    navigation: Navigation,
    session_id: SessionId,
) -> impl IntoView {
    let summary = store.with_untracked(|db| session_summary(db, session_id));
    view! {
        <main class="screen recap">
            <p class="eyebrow recap__eyebrow">"Séance terminée"</p>
            {summary.map(|summary| view! { <RecapContent summary /> })}
            <button type="button" class="btn btn--primary recap__done" on:click=move |_| navigation.reset_to(Screen::Home)>
                "Terminé"
            </button>
        </main>
    }
}

#[component]
fn RecapContent(summary: SessionSummary) -> impl IntoView {
    let stat = |label: &'static str, value: String| {
        view! {
            <div class="stat-tile">
                <span class="stat-tile__value">{value}</span>
                <span class="eyebrow">{label}</span>
            </div>
        }
    };
    view! {
        <h1 class="title">{summary.name}</h1>
        <section class="stat-grid">
            {stat("Durée", summary.duration)}
            {stat("Séries", summary.set_count.to_string())}
            {summary.volume.map(|volume| stat("Volume", volume))}
            {summary.average_rest.map(|rest| stat("Repos moyen", rest))}
        </section>
        <p class="recap__week">{summary.week_text}</p>
        {(!summary.records.is_empty()).then(|| view! {
            <section class="list recap__records">
                <p class="eyebrow recap__records-title">"Records battus"</p>
                {summary
                    .records
                    .into_iter()
                    .map(|badge| view! {
                        <div class="recap__record">
                            <span class="medal" aria-hidden="true">"★"</span>
                            <span class="recap__record-text">
                                <span class="recap__exercise-name">{badge.exercise_name}</span>
                                <span class="muted">{badge.text}</span>
                            </span>
                        </div>
                    })
                    .collect_view()}
            </section>
        })}
        <section class="list">
            {summary
                .exercises
                .into_iter()
                .map(|exercise| {
                    view! {
                        <div class="list__item recap__exercise">
                            <span class="recap__exercise-name">{exercise.name}</span>
                            <span class="muted">{exercise.sets.join(" · ")}</span>
                        </div>
                    }
                })
                .collect_view()}
        </section>
    }
}
