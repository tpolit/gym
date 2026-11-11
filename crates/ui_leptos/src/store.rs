use domain::Db;
use features::AppCore;
use leptos::prelude::*;
use persistence::LoadError;

#[derive(Clone, Copy)]
pub struct Store {
    core: RwSignal<AppCore>,
}

impl Store {
    pub fn new(core: AppCore) -> Self {
        Self {
            core: RwSignal::new(core),
        }
    }

    pub fn with<R>(self, read: impl FnOnce(&Db) -> R) -> R {
        self.core.with(|core| read(core.db()))
    }

    pub fn with_untracked<R>(self, read: impl FnOnce(&Db) -> R) -> R {
        self.core.with_untracked(|core| read(core.db()))
    }

    pub fn load_error(self) -> Option<LoadError> {
        self.core.with_untracked(|core| core.load_error().cloned())
    }

    pub fn snapshot_to_save(self) -> Option<Db> {
        self.core
            .with(|core| core.can_save().then(|| core.db().clone()))
    }

    pub fn execute<T, E>(self, command: impl FnOnce(&mut Db) -> Result<T, E>) -> Result<T, E> {
        let result = self.core.write_untracked().execute(command);
        self.notify_if_changed(result.is_ok());
        result
    }

    // A refused command leaves the Db unchanged: no need to re-render or save.
    fn notify_if_changed(self, changed: bool) {
        if changed {
            self.core.notify();
        }
    }
}
