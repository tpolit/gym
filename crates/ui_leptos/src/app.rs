use leptos::prelude::*;

use crate::backup::BackupPage;
use crate::exercise_catalog::{CatalogPage, CustomExerciseFormPage, ExerciseDetailPage};
use crate::history::{HistoryPage, SessionDetailPage};
use crate::home::{HomePage, SettingsPage};
use crate::navigation::{Navigation, Screen};
use crate::plan_session::PlanSessionPage;
use crate::run_session::{RunSessionPage, SessionMiniPlayer, SessionRecapPage, StartSessionPage};
use crate::stats::{ExerciseStatsPage, StatsPage};
use crate::store::Store;
use crate::tab_bar::TabBar;

#[component]
pub fn App(store: Store) -> impl IntoView {
    let navigation = Navigation::starting_at(Screen::Home);
    let current_screen = move || match navigation.current() {
        Screen::Home => view! { <HomePage store navigation /> }.into_any(),
        Screen::History(filter) => view! { <HistoryPage store navigation filter /> }.into_any(),
        Screen::Stats => view! { <StatsPage store navigation /> }.into_any(),
        Screen::Catalog => view! { <CatalogPage store navigation /> }.into_any(),
        Screen::ExerciseStats(exercise_id) => {
            view! { <ExerciseStatsPage store navigation exercise_id /> }.into_any()
        }
        Screen::Settings => view! { <SettingsPage store navigation /> }.into_any(),
        Screen::Backup => view! { <BackupPage store navigation /> }.into_any(),
        Screen::StartSession(source) => {
            view! { <StartSessionPage store navigation source /> }.into_any()
        }
        Screen::RunSession(session_id) => {
            view! { <RunSessionPage store navigation session_id /> }.into_any()
        }
        Screen::SessionRecap(session_id) => {
            view! { <SessionRecapPage store navigation session_id /> }.into_any()
        }
        Screen::PlanSession(source) => {
            view! { <PlanSessionPage store navigation source /> }.into_any()
        }
        Screen::SessionDetail(session_id) => {
            view! { <SessionDetailPage store navigation session_id /> }.into_any()
        }
        Screen::ExerciseDetail(exercise_id) => {
            view! { <ExerciseDetailPage store navigation exercise_id /> }.into_any()
        }
        Screen::CustomExerciseForm(custom_id) => {
            view! { <CustomExerciseFormPage store navigation custom_id /> }.into_any()
        }
    };
    let shows_tab_bar = Memo::new(move |_| navigation.current().is_tab());

    view! {
        <div class=move || if shows_tab_bar.get() { "shell shell--tabs" } else { "shell" }>
            {current_screen}
        </div>
        <Show when=move || shows_tab_bar.get()>
            <footer class="shell__bottom">
                <SessionMiniPlayer store navigation />
                <TabBar navigation />
            </footer>
        </Show>
    }
}
