use features::stats::{
    PracticedExercise, days_since_muscle_group, practiced_exercises, session_averages,
    sets_per_muscle_group, weekly_activity,
};
use leptos::prelude::*;

use crate::navigation::{Navigation, Screen};
use crate::store::Store;

const WEEKS_SHOWN: usize = 8;

#[component]
pub fn StatsPage(store: Store, navigation: Navigation) -> impl IntoView {
    let today = web_platform::today();
    let activity = Memo::new(move |_| store.with(|db| weekly_activity(db, today, WEEKS_SHOWN)));
    let muscle_sets = Memo::new(move |_| store.with(|db| sets_per_muscle_group(db, today)));
    let recency = Memo::new(move |_| store.with(|db| days_since_muscle_group(db, today)));
    let averages = Memo::new(move |_| store.with(session_averages));
    let exercises = Memo::new(move |_| store.with(practiced_exercises));

    let week_bars = move || {
        let weeks = activity.with(|activity| activity.weeks.clone());
        let highest = weeks
            .iter()
            .map(|week| week.session_count)
            .max()
            .unwrap_or(0)
            .max(1);
        weeks
            .into_iter()
            .map(|week| {
                let height = week.session_count * 100 / highest;
                view! {
                    <div class="bars__column" class:bars__column--current=week.is_current>
                        <span class="bars__value">{week.session_count}</span>
                        <span class="bars__bar" style:height=format!("{height}%")></span>
                        <span class="bars__label">{week.label}</span>
                    </div>
                }
            })
            .collect_view()
    };

    let muscle_rows = move || {
        let groups = muscle_sets.get();
        let highest = groups
            .iter()
            .map(|group| group.set_count)
            .max()
            .unwrap_or(1)
            .max(1);
        if groups.is_empty() {
            return view! { <p class="muted">"Aucune série de musculation cette semaine."</p> }
                .into_any();
        }
        groups
            .into_iter()
            .map(|group| {
                let width = group.set_count * 100 / highest;
                view! {
                    <div class="hbar">
                        <span class="hbar__label">{group.label}</span>
                        <span class="hbar__track"><span class="hbar__fill" style:width=format!("{width}%")></span></span>
                        <span class="hbar__value">{group.set_count}</span>
                    </div>
                }
            })
            .collect_view()
            .into_any()
    };

    let exercise_row = move |exercise: PracticedExercise| {
        let exercise_id = exercise.id;
        view! {
            <button
                type="button"
                class="picker__item"
                on:click=move |_| navigation.go_to(Screen::ExerciseStats(exercise_id.clone()))
            >
                <span class="picker__name">{exercise.name}</span>
                <span class="picker__detail">{exercise.detail}</span>
            </button>
        }
    };

    view! {
        <main class="screen stats">
            <h1 class="title stats__title">"Stats"</h1>
            {move || match averages.get() {
                None => view! {
                    <p class="muted">"Les statistiques apparaîtront après votre première séance terminée."</p>
                }
                    .into_any(),
                Some(averages) => view! {
                    <section class="stat-grid">
                        <div class="stat-tile">
                            <span class="stat-tile__value">{averages.session_count}</span>
                            <span class="eyebrow">"Séances"</span>
                        </div>
                        <div class="stat-tile">
                            <span class="stat-tile__value">{averages.duration}</span>
                            <span class="eyebrow">"Durée moyenne"</span>
                        </div>
                        <div class="stat-tile">
                            <span class="stat-tile__value">{averages.sets}</span>
                            <span class="eyebrow">"Séries par séance"</span>
                        </div>
                        <div class="stat-tile">
                            <span class="stat-tile__value">{averages.rest.unwrap_or_else(|| "–".to_owned())}</span>
                            <span class="eyebrow">"Repos moyen"</span>
                        </div>
                    </section>
                }
                    .into_any(),
            }}
            <section class="card stats__card">
                <p class="eyebrow">"Séances par semaine"</p>
                <div class="bars">{week_bars}</div>
                <p class="muted">
                    "Volume cette semaine : " {move || activity.with(|activity| activity.current_volume_text.clone())}
                </p>
                {move || activity.with(|activity| activity.current_cardio_text.clone()).map(|text| view! {
                    <p class="muted">"Cardio cette semaine : " {text}</p>
                })}
            </section>
            <section class="card stats__card">
                <p class="eyebrow">"Séries par groupe musculaire · semaine"</p>
                {muscle_rows}
            </section>
            {move || {
                let groups = recency.get();
                (!groups.is_empty()).then(|| view! {
                    <section class="card stats__card">
                        <p class="eyebrow">"Dernier entraînement"</p>
                        {groups
                            .into_iter()
                            .map(|group| view! {
                                <div class="stats__line">
                                    <span>{group.label}</span>
                                    <span class="muted">{group.text}</span>
                                </div>
                            })
                            .collect_view()}
                    </section>
                })
            }}
            {move || {
                let list = exercises.get();
                (!list.is_empty()).then(|| view! {
                    <p class="eyebrow">"Progression par exercice"</p>
                    <div class="list">{list.into_iter().map(exercise_row).collect_view()}</div>
                })
            }}
        </main>
    }
}
