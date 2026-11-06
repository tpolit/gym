use domain::ExerciseId;
use features::stats::{exercise_progress, personal_records};
use leptos::prelude::*;

use super::progress_chart::ProgressChart;
use crate::navigation::{Navigation, Screen};
use crate::store::Store;
use crate::ui::{TitleVisibility, TopBar};

// Headline figure with trend, curve explored by touch, dense list of records.
#[component]
pub fn ExerciseStatsPage(
    store: Store,
    navigation: Navigation,
    exercise_id: ExerciseId,
) -> impl IntoView {
    let progress = store.with_untracked(|db| exercise_progress(db, &exercise_id));
    let records = store.with_untracked(|db| personal_records(db, &exercise_id));
    let filtered_history = Screen::History(Some(exercise_id));
    let title = progress
        .as_ref()
        .map(|progress| progress.exercise_name.clone());

    view! {
        <main class="screen">
            <TopBar
                on_back=Callback::new(move |()| navigation.back())
                title
                title_visibility=TitleVisibility::OnceScrolled
            />
            {match progress {
                None => view! { <p class="muted">"Exercice introuvable."</p> }.into_any(),
                Some(progress) => view! {
                    <p class="eyebrow">{progress.metric_label}</p>
                    <h1 class="title">{progress.exercise_name}</h1>
                    <p class="stats__headline">{progress.headline.unwrap_or_else(|| "–".to_owned())}</p>
                    {progress.trend.map(|trend| view! { <p class="stats__trend">{trend}</p> })}
                    {(!progress.points.is_empty()).then(|| view! {
                        <section class="card stats__card">
                            <ProgressChart points=progress.points />
                        </section>
                    })}
                }
                    .into_any(),
            }}
            {(!records.is_empty()).then(|| view! {
                <p class="eyebrow">"Records"</p>
                <div class="list">
                    {records
                        .into_iter()
                        .map(|record| view! {
                            <div class="stats__record">
                                <span class="medal medal--small" aria-hidden="true">"★"</span>
                                <span class="stats__record-label">{record.label}</span>
                                <span class="stats__record-value">{record.value_text}</span>
                                <span class="stats__record-date">{record.date_text}</span>
                            </div>
                        })
                        .collect_view()}
                </div>
            })}
            <button
                type="button"
                class="btn btn--secondary"
                on:click=move |_| navigation.go_to(filtered_history.clone())
            >
                "Voir les séances avec cet exercice"
            </button>
        </main>
    }
}
