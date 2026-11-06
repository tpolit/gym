use domain::{ExerciseId, SessionId};
use leptos::prelude::*;

// Existing screens only: no placeholder screen for a feature not built yet.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Screen {
    Home,
    History(Option<ExerciseId>),
    Stats,
    Catalog,
    ExerciseStats(ExerciseId),
    Settings,
    Backup,
    StartSession(StartSource),
    RunSession(SessionId),
    SessionRecap(SessionId),
    PlanSession(PlanSource),
    SessionDetail(SessionId),
    ExerciseDetail(ExerciseId),
    CustomExerciseForm(Option<u32>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StartSource {
    New,
    Planned(SessionId),
    Repeat(SessionId),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlanSource {
    New,
    Edit(SessionId),
    From(SessionId),
}

impl Screen {
    pub const fn is_tab(&self) -> bool {
        matches!(
            self,
            Self::Home | Self::History(_) | Self::Stats | Self::Catalog
        )
    }

    // A filtered history is still the history tab.
    pub fn belongs_to_same_tab(&self, other: &Self) -> bool {
        std::mem::discriminant(self) == std::mem::discriminant(other)
    }
}

#[derive(Clone, Copy)]
pub struct Navigation {
    current: RwSignal<Screen>,
    history: StoredValue<Vec<Screen>>,
}

impl Navigation {
    pub fn starting_at(screen: Screen) -> Self {
        Self {
            current: RwSignal::new(screen),
            history: StoredValue::new(Vec::new()),
        }
    }

    pub fn current(self) -> Screen {
        self.current.get()
    }

    pub fn go_to(self, screen: Screen) {
        let previous = self.current.get_untracked();
        self.history.update_value(|history| history.push(previous));
        self.current.set(screen);
    }

    pub fn back(self) {
        let previous = self
            .history
            .try_update_value(Vec::pop)
            .flatten()
            .unwrap_or(Screen::Home);
        self.current.set(previous);
    }

    pub fn replace(self, screen: Screen) {
        self.current.set(screen);
    }

    pub fn reset_to(self, screen: Screen) {
        self.history.update_value(Vec::clear);
        self.current.set(screen);
    }
}
