use domain::{ExerciseId, SessionId};
use features::run_session::{add_entry, current_session, entry_to_resume, rest_started_at};
use features::shared::exercise_choices::exercise_choices;
use features::shared::format::format_clock;
use leptos::prelude::*;

use super::drawer::SessionDrawer;
use super::exercise_view::ExerciseView;
use crate::navigation::{Navigation, Screen};
use crate::store::Store;
use crate::ui::{ExercisePicker, TopBar, ticking_now};

#[component]
pub fn RunSessionPage(
    store: Store,
    navigation: Navigation,
    session_id: SessionId,
) -> impl IntoView {
    let session = Memo::new(move |_| {
        store
            .with(current_session)
            .filter(|session| session.id == session_id)
    });
    let is_running = Memo::new(move |_| session.with(Option::is_some));
    let current_entry = RwSignal::new(store.with_untracked(|db| entry_to_resume(db, session_id)));
    let drawer_open = RwSignal::new(false);
    let picker_open = RwSignal::new(false);
    let set_timer_running = RwSignal::new(false);
    // Dropped with this screen, which releases the wake lock.
    let _screen_on = StoredValue::new_local(web_platform::ScreenWakeLock::keep_screen_on());

    let add_exercise = Callback::new(move |exercise_id: ExerciseId| {
        if let Ok(entry_id) = store.execute(|db| add_entry(db, session_id, exercise_id)) {
            current_entry.set(Some(entry_id));
        }
        picker_open.set(false);
        drawer_open.set(false);
    });

    let running_view = move || {
        view! {
            <TopBar
                on_back=Callback::new(move |()| navigation.back())
                title=Signal::derive(move || session.with(|session| session.as_ref().map(|session| session.name.clone())))
            />
            <Show when=move || !set_timer_running.get()>
                <RestHero store session_id />
            </Show>
            {move || match current_entry.get() {
                Some(entry_id) => view! { <ExerciseView store session_id entry_id set_timer_running /> }.into_any(),
                None => {
                    view! {
                        <section class="card run-session__empty">
                            <p>"Ajoutez un premier exercice pour commencer."</p>
                            <button type="button" class="btn btn--primary" on:click=move |_| picker_open.set(true)>
                                "Ajouter un exercice"
                            </button>
                        </section>
                    }
                        .into_any()
                }
            }}
            <button type="button" class="run-session__drawer-handle" on:click=move |_| drawer_open.set(true)>
                <span class="grabber"></span>
                <span>
                    "Exercices de la séance · "
                    {move || session.with(|session| session.as_ref().map_or(0, |session| session.entries.len()))}
                </span>
            </button>
            {move || {
                drawer_open
                    .get()
                    .then(|| {
                        view! {
                            <SessionDrawer
                                store
                                navigation
                                session_id
                                session
                                current_entry
                                on_close=Callback::new(move |()| drawer_open.set(false))
                                on_add=Callback::new(move |()| picker_open.set(true))
                            />
                        }
                    })
            }}
            {move || {
                picker_open
                    .get()
                    .then(|| {
                        view! {
                            <ExercisePicker
                                choices=store.with_untracked(exercise_choices)
                                on_pick=add_exercise
                                on_close=Callback::new(move |()| picker_open.set(false))
                            />
                        }
                    })
            }}
        }
    };

    view! {
        <main class="screen run-session">
            {move || {
                if is_running.get() {
                    running_view().into_any()
                } else {
                    view! {
                        <section class="card run-session__gone">
                            <p>"Cette séance n'est plus en cours."</p>
                            <button type="button" class="btn btn--secondary" on:click=move |_| navigation.reset_to(Screen::Home)>
                                "Retour à l'accueil"
                            </button>
                        </section>
                    }
                        .into_any()
                }
            }}
        </main>
    }
}

// Giant rest timer, computed from the last completion time.
#[component]
fn RestHero(store: Store, session_id: SessionId) -> impl IntoView {
    let now = ticking_now();
    let rest = Memo::new(move |_| store.with(|db| rest_started_at(db, session_id)));
    let started_at = store
        .with_untracked(current_session)
        .map(|session| session.started_at);
    let elapsed_since = move |since: chrono::DateTime<chrono::FixedOffset>| {
        format_clock((now.get() - since).num_seconds())
    };

    move || match rest.get() {
        Some(rest) => view! {
            <section class="rest-hero" aria-live="off">
                <div>
                    <p class="eyebrow rest-hero__title">"Repos"</p>
                    <p class="rest-hero__label">{rest.label}</p>
                </div>
                <p class="rest-hero__digits">{move || elapsed_since(rest.since)}</p>
            </section>
        }
        .into_any(),
        None => view! {
            <section class="rest-hero rest-hero--session">
                <p class="eyebrow">"Séance"</p>
                <p class="rest-hero__digits">{move || started_at.map(elapsed_since)}</p>
            </section>
        }
        .into_any(),
    }
}
