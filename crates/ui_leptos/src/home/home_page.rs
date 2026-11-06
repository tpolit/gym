use features::home::{RepeatChoice, SessionCard, home_summary, sessions_to_repeat};
use leptos::prelude::*;

use super::body_weight::BodyWeightPrompt;
use super::first_name::{FirstNamePrompt, FirstNameSheet};
use super::weekly_ring::WeeklyRing;
use crate::navigation::{Navigation, PlanSource, Screen, StartSource};
use crate::store::Store;

const REPEAT_LIMIT: usize = 3;

#[component]
pub fn HomePage(store: Store, navigation: Navigation) -> impl IntoView {
    let summary = Memo::new(move |_| store.with(|db| home_summary(db, web_platform::today())));
    let repeat_choices = Memo::new(move |_| store.with(|db| sessions_to_repeat(db, REPEAT_LIMIT)));
    let in_progress = move || summary.with(|summary| summary.in_progress.clone());
    let can_start = move || summary.with(|summary| summary.in_progress.is_none());
    let needs_first_name = Memo::new(move |_| summary.with(|summary| summary.first_name.is_none()));
    let needs_body_weight = Memo::new(move |_| summary.with(|summary| summary.needs_body_weight));
    let editing_first_name = RwSignal::new(false);

    let first_name_prompt = move || {
        needs_first_name
            .get()
            .then(|| view! { <FirstNamePrompt store /> })
    };
    let first_name_sheet = move || {
        editing_first_name.get().then(|| {
            let initial = summary.with_untracked(|summary| summary.first_name.clone().unwrap_or_default());
            view! {
                <FirstNameSheet store initial on_close=Callback::new(move |()| editing_first_name.set(false)) />
            }
        })
    };

    let main_action = move || match in_progress() {
        Some(card) => view! {
            <button
                type="button"
                class="btn btn--primary"
                on:click=move |_| navigation.go_to(Screen::RunSession(card.id))
            >
                "Reprendre la séance"
            </button>
        }
        .into_any(),
        None => view! {
            <button
                type="button"
                class="btn btn--primary"
                on:click=move |_| navigation.go_to(Screen::StartSession(StartSource::New))
            >
                "Démarrer une séance"
            </button>
        }
        .into_any(),
    };

    let repeat_section = move || {
        let choices = repeat_choices.get();
        (can_start() && !choices.is_empty()).then(|| {
            view! {
                <section class="home__section">
                    <p class="eyebrow">"Refaire une séance"</p>
                    <div class="home__repeat">
                        {choices.into_iter().map(|choice| view! { <RepeatButton choice navigation /> }).collect_view()}
                    </div>
                </section>
            }
        })
    };

    let upcoming_section = move || {
        let upcoming = summary.with(|summary| summary.upcoming.clone());
        view! {
            <section class="home__section">
                <p class="eyebrow">"Prochaines séances"</p>
                {(!upcoming.is_empty()).then(|| view! {
                    <div class="list">
                        {upcoming
                            .into_iter()
                            .map(|card| view! { <PlannedRow card navigation can_start=can_start() /> })
                            .collect_view()}
                    </div>
                })}
                <button
                    type="button"
                    class="btn btn--secondary"
                    on:click=move |_| navigation.go_to(Screen::PlanSession(PlanSource::New))
                >
                    "Planifier une séance"
                </button>
            </section>
        }
    };

    let welcome = move || {
        summary
            .with(|summary| !summary.has_done_sessions && summary.in_progress.is_none())
            .then(|| view! {
                <p class="muted home__welcome">
                    "Démarrez une séance : vos séries sont enregistrées sur ce téléphone, même hors ligne."
                </p>
            })
    };

    view! {
        <main class="screen home">
            <header class="home__header">
                <div class="home__heading">
                    <p class="eyebrow">{move || summary.with(|summary| summary.today.clone())}</p>
                    <h1 class="title">
                        <button type="button" class="home__greeting" on:click=move |_| editing_first_name.set(true)>
                            {move || summary.with(|summary| summary.greeting.clone())}
                        </button>
                    </h1>
                </div>
                <button
                    type="button"
                    class="icon-btn"
                    aria-label="Réglages"
                    on:click=move |_| navigation.go_to(Screen::Settings)
                >
                    "⚙"
                </button>
            </header>
            {first_name_prompt}
            {move || needs_body_weight.get().then(|| view! { <BodyWeightPrompt store /> })}
            {main_action}
            <WeeklyRing store />
            {welcome}
            {repeat_section}
            {upcoming_section}
            {first_name_sheet}
        </main>
    }
}

#[component]
fn RepeatButton(choice: RepeatChoice, navigation: Navigation) -> impl IntoView {
    let session_id = choice.id;
    view! {
        <button
            type="button"
            class="card home__repeat-card"
            on:click=move |_| navigation.go_to(Screen::StartSession(StartSource::Repeat(session_id)))
        >
            <span class="home__card-title">{choice.name}</span>
            <span class="muted">{choice.last_done}</span>
        </button>
    }
}

#[component]
fn PlannedRow(card: SessionCard, navigation: Navigation, can_start: bool) -> impl IntoView {
    let session_id = card.id;
    view! {
        <div class="home__planned">
            <button
                type="button"
                class="home__planned-text"
                on:click=move |_| navigation.go_to(Screen::PlanSession(PlanSource::Edit(session_id)))
            >
                <span class="home__planned-name">{card.name}</span>
                <span class="muted">{card.detail}</span>
            </button>
            {can_start.then(|| view! {
                <button
                    type="button"
                    class="btn btn--accent-outline"
                    on:click=move |_| navigation.go_to(Screen::StartSession(StartSource::Planned(session_id)))
                >
                    "Lancer"
                </button>
            })}
        </div>
    }
}
