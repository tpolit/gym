use std::collections::{BTreeMap, BTreeSet};

use chrono::{DateTime, FixedOffset, NaiveDate};

use crate::body_weight::BodyWeight;
use crate::catalog::{self, BuiltinExercise};
use crate::error::{DbError, IntegrityError};
use crate::exercise::{CustomExercise, ExerciseId, ExerciseInfo, ExerciseSettings};
use crate::measure::SetMeasure;
use crate::preferences::{Preferences, WeeklyGoal};
use crate::session::{EntryId, Session, SessionId, SessionOrigin, SessionStatus, next_id};
use crate::set::WorkoutSet;
use crate::text::NonEmptyText;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Db {
    custom_exercises: Vec<CustomExercise>,
    exercise_settings: BTreeMap<ExerciseId, ExerciseSettings>,
    sessions: Vec<Session>,
    preferences: Preferences,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Removed {
    Deleted,
    Archived,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cancelled {
    BackToPlanned,
    Deleted,
}

impl Db {
    pub fn restore(
        custom_exercises: Vec<CustomExercise>,
        exercise_settings: BTreeMap<ExerciseId, ExerciseSettings>,
        sessions: Vec<Session>,
        preferences: Preferences,
    ) -> Result<Self, IntegrityError> {
        let db = Self {
            custom_exercises,
            exercise_settings,
            sessions,
            preferences,
        };
        db.check_integrity()?;
        Ok(db)
    }

    fn check_integrity(&self) -> Result<(), IntegrityError> {
        let mut custom_ids = BTreeSet::new();
        for exercise in &self.custom_exercises {
            if !custom_ids.insert(exercise.id()) {
                return Err(IntegrityError::DuplicateCustomExerciseId(exercise.id()));
            }
        }
        for id in self.exercise_settings.keys() {
            if self.exercise(id).is_none() {
                return Err(IntegrityError::UnknownExercise(id.clone()));
            }
        }
        let mut session_ids = BTreeSet::new();
        let mut in_progress = 0_usize;
        for session in &self.sessions {
            if !session_ids.insert(session.id()) {
                return Err(IntegrityError::DuplicateSessionId(session.id()));
            }
            if matches!(session.status(), SessionStatus::InProgress { .. }) {
                in_progress += 1;
            }
            for entry in session.entries() {
                let info = self
                    .exercise(entry.exercise_id())
                    .ok_or_else(|| IntegrityError::UnknownExercise(entry.exercise_id().clone()))?;
                if entry
                    .sets()
                    .iter()
                    .any(|set| set.measure().kind() != info.measure())
                {
                    return Err(IntegrityError::MeasureMismatch {
                        session: session.id(),
                        entry: entry.entry_id(),
                    });
                }
            }
        }
        if in_progress > 1 {
            return Err(IntegrityError::SeveralSessionsInProgress);
        }
        Ok(())
    }

    pub const fn preferences(&self) -> &Preferences {
        &self.preferences
    }

    pub fn custom_exercises(&self) -> &[CustomExercise] {
        &self.custom_exercises
    }

    pub fn custom_exercise(&self, id: u32) -> Option<&CustomExercise> {
        self.custom_exercises
            .iter()
            .find(|exercise| exercise.id() == id)
    }

    pub fn exercise(&self, id: &ExerciseId) -> Option<&ExerciseInfo> {
        match id {
            ExerciseId::Builtin(id) => catalog::builtin_exercise(id).map(BuiltinExercise::info),
            ExerciseId::Custom(id) => self.custom_exercise(*id).map(CustomExercise::info),
        }
    }

    pub fn is_exercise_used(&self, id: &ExerciseId) -> bool {
        self.sessions
            .iter()
            .flat_map(Session::entries)
            .any(|entry| entry.exercise_id() == id)
    }

    pub fn exercise_settings(&self, id: &ExerciseId) -> Option<&ExerciseSettings> {
        self.exercise_settings.get(id)
    }

    pub const fn all_exercise_settings(&self) -> &BTreeMap<ExerciseId, ExerciseSettings> {
        &self.exercise_settings
    }

    pub fn sessions(&self) -> &[Session] {
        &self.sessions
    }

    pub fn session(&self, id: SessionId) -> Option<&Session> {
        self.sessions.iter().find(|session| session.id() == id)
    }

    pub fn session_in_progress(&self) -> Option<&Session> {
        self.sessions
            .iter()
            .find(|session| matches!(session.status(), SessionStatus::InProgress { .. }))
    }

    pub fn set_weekly_goal(&mut self, goal: Option<WeeklyGoal>) {
        self.preferences.set_weekly_goal(goal);
    }

    pub fn set_first_name(&mut self, first_name: Option<NonEmptyText>) {
        self.preferences.set_first_name(first_name);
    }

    pub fn record_body_weight(&mut self, date: NaiveDate, weight: BodyWeight) {
        self.preferences.record_body_weight(date, weight);
    }

    pub fn add_custom_exercise(&mut self, info: ExerciseInfo) -> Result<ExerciseId, DbError> {
        let id = next_id(self.custom_exercises.iter().map(CustomExercise::id))?;
        self.custom_exercises
            .push(CustomExercise::restore(id, info, false));
        Ok(ExerciseId::Custom(id))
    }

    pub fn update_custom_exercise(&mut self, id: u32, info: ExerciseInfo) -> Result<(), DbError> {
        let current = self
            .custom_exercise(id)
            .ok_or(DbError::ExerciseNotFound(ExerciseId::Custom(id)))?;
        if current.info().measure() != info.measure()
            && self.is_exercise_used(&ExerciseId::Custom(id))
        {
            return Err(DbError::MeasureLockedByUse);
        }
        self.custom_exercise_mut(id)?.set_info(info);
        Ok(())
    }

    pub fn remove_custom_exercise(&mut self, id: u32) -> Result<Removed, DbError> {
        let exercise_id = ExerciseId::Custom(id);
        if self.is_exercise_used(&exercise_id) {
            self.custom_exercise_mut(id)?.set_archived(true);
            return Ok(Removed::Archived);
        }
        let index = self
            .custom_exercises
            .iter()
            .position(|exercise| exercise.id() == id)
            .ok_or(DbError::ExerciseNotFound(exercise_id.clone()))?;
        self.custom_exercises.remove(index);
        self.exercise_settings.remove(&exercise_id);
        Ok(Removed::Deleted)
    }

    pub fn unarchive_custom_exercise(&mut self, id: u32) -> Result<(), DbError> {
        self.custom_exercise_mut(id)?.set_archived(false);
        Ok(())
    }

    pub fn set_exercise_settings(
        &mut self,
        id: ExerciseId,
        settings: Option<ExerciseSettings>,
    ) -> Result<(), DbError> {
        if self.exercise(&id).is_none() {
            return Err(DbError::ExerciseNotFound(id));
        }
        match settings {
            Some(settings) => self.exercise_settings.insert(id, settings),
            None => self.exercise_settings.remove(&id),
        };
        Ok(())
    }

    pub fn plan_session(
        &mut self,
        name: NonEmptyText,
        planned_for: Option<NaiveDate>,
        exercises: &[ExerciseId],
    ) -> Result<SessionId, DbError> {
        self.ensure_addable_all(exercises)?;
        let id = self.next_session_id()?;
        self.sessions.push(Session::new_planned(
            id,
            name,
            planned_for,
            exercises.to_vec(),
        ));
        Ok(id)
    }

    pub fn start_new_session(
        &mut self,
        name: NonEmptyText,
        exercises: &[ExerciseId],
        now: DateTime<FixedOffset>,
    ) -> Result<SessionId, DbError> {
        self.ensure_no_session_in_progress()?;
        self.ensure_addable_all(exercises)?;
        let id = self.next_session_id()?;
        self.sessions
            .push(Session::new_started(id, name, exercises.to_vec(), now));
        Ok(id)
    }

    pub fn session_template(
        &self,
        source: SessionId,
    ) -> Result<(NonEmptyText, Vec<ExerciseId>), DbError> {
        let session = self.find_session(source)?;
        if !matches!(session.status(), SessionStatus::Done { .. }) {
            return Err(DbError::NotDone);
        }
        let exercises = session
            .entries()
            .iter()
            .map(|entry| entry.exercise_id())
            .filter(|id| !self.is_archived(id))
            .cloned()
            .collect();
        Ok((session.name().clone(), exercises))
    }

    pub fn start_planned_session(
        &mut self,
        id: SessionId,
        now: DateTime<FixedOffset>,
    ) -> Result<(), DbError> {
        if self.find_session(id)?.status() != &SessionStatus::Planned {
            return Err(DbError::NotPlanned);
        }
        self.ensure_no_session_in_progress()?;
        self.session_mut(id)?.start(now)
    }

    pub fn finish_session(
        &mut self,
        id: SessionId,
        now: DateTime<FixedOffset>,
    ) -> Result<(), DbError> {
        self.session_mut(id)?.finish(now)
    }

    pub fn cancel_session(&mut self, id: SessionId) -> Result<Cancelled, DbError> {
        let session = self.find_session(id)?;
        session.ensure_cancellable()?;
        match session.origin() {
            SessionOrigin::Planned => {
                self.session_mut(id)?.back_to_planned()?;
                Ok(Cancelled::BackToPlanned)
            }
            SessionOrigin::Adhoc => {
                self.delete_session(id)?;
                Ok(Cancelled::Deleted)
            }
        }
    }

    pub fn delete_session(&mut self, id: SessionId) -> Result<(), DbError> {
        let index = self
            .sessions
            .iter()
            .position(|session| session.id() == id)
            .ok_or(DbError::SessionNotFound(id))?;
        self.sessions.remove(index);
        Ok(())
    }

    pub fn rename_session(&mut self, id: SessionId, name: NonEmptyText) -> Result<(), DbError> {
        self.session_mut(id)?.rename(name);
        Ok(())
    }

    pub fn set_session_planned_for(
        &mut self,
        id: SessionId,
        planned_for: Option<NaiveDate>,
    ) -> Result<(), DbError> {
        self.session_mut(id)?.set_planned_for(planned_for);
        Ok(())
    }

    pub fn set_session_notes(
        &mut self,
        id: SessionId,
        notes: Option<NonEmptyText>,
    ) -> Result<(), DbError> {
        self.session_mut(id)?.set_notes(notes);
        Ok(())
    }

    pub fn update_session_times(
        &mut self,
        id: SessionId,
        started_at: DateTime<FixedOffset>,
        finished_at: DateTime<FixedOffset>,
    ) -> Result<(), DbError> {
        self.session_mut(id)?.update_times(started_at, finished_at)
    }

    pub fn add_entry(
        &mut self,
        id: SessionId,
        exercise_id: ExerciseId,
    ) -> Result<EntryId, DbError> {
        self.ensure_addable(&exercise_id)?;
        self.session_mut(id)?.add_entry(exercise_id)
    }

    pub fn remove_entry(&mut self, id: SessionId, entry_id: EntryId) -> Result<(), DbError> {
        self.session_mut(id)?.remove_entry(entry_id)
    }

    pub fn move_entry(
        &mut self,
        id: SessionId,
        entry_id: EntryId,
        new_index: usize,
    ) -> Result<(), DbError> {
        self.session_mut(id)?.move_entry(entry_id, new_index)
    }

    pub fn add_set(
        &mut self,
        id: SessionId,
        entry_id: EntryId,
        set: WorkoutSet,
    ) -> Result<(), DbError> {
        self.ensure_measure(id, entry_id, set.measure())?;
        self.session_mut(id)?.add_set(entry_id, set)
    }

    pub fn update_set(
        &mut self,
        id: SessionId,
        entry_id: EntryId,
        index: usize,
        measure: SetMeasure,
        warmup: bool,
    ) -> Result<(), DbError> {
        self.ensure_measure(id, entry_id, &measure)?;
        self.session_mut(id)?
            .update_set(entry_id, index, measure, warmup)
    }

    pub fn remove_set(
        &mut self,
        id: SessionId,
        entry_id: EntryId,
        index: usize,
    ) -> Result<(), DbError> {
        self.session_mut(id)?.remove_set(entry_id, index)
    }

    fn find_session(&self, id: SessionId) -> Result<&Session, DbError> {
        self.session(id).ok_or(DbError::SessionNotFound(id))
    }

    fn session_mut(&mut self, id: SessionId) -> Result<&mut Session, DbError> {
        self.sessions
            .iter_mut()
            .find(|session| session.id() == id)
            .ok_or(DbError::SessionNotFound(id))
    }

    fn custom_exercise_mut(&mut self, id: u32) -> Result<&mut CustomExercise, DbError> {
        self.custom_exercises
            .iter_mut()
            .find(|exercise| exercise.id() == id)
            .ok_or(DbError::ExerciseNotFound(ExerciseId::Custom(id)))
    }

    fn next_session_id(&self) -> Result<SessionId, DbError> {
        next_id(self.sessions.iter().map(|session| session.id().get())).map(SessionId::new)
    }

    fn is_archived(&self, id: &ExerciseId) -> bool {
        match id {
            ExerciseId::Builtin(id) => {
                catalog::builtin_exercise(id).is_some_and(BuiltinExercise::is_retired)
            }
            ExerciseId::Custom(id) => self
                .custom_exercise(*id)
                .is_some_and(CustomExercise::is_archived),
        }
    }

    fn ensure_addable(&self, id: &ExerciseId) -> Result<(), DbError> {
        if self.exercise(id).is_none() {
            return Err(DbError::ExerciseNotFound(id.clone()));
        }
        if self.is_archived(id) {
            return Err(DbError::ExerciseArchived(id.clone()));
        }
        Ok(())
    }

    fn ensure_addable_all(&self, ids: &[ExerciseId]) -> Result<(), DbError> {
        ids.iter().try_for_each(|id| self.ensure_addable(id))
    }

    fn ensure_no_session_in_progress(&self) -> Result<(), DbError> {
        match self.session_in_progress() {
            Some(_) => Err(DbError::AnotherSessionInProgress),
            None => Ok(()),
        }
    }

    fn ensure_measure(
        &self,
        id: SessionId,
        entry_id: EntryId,
        measure: &SetMeasure,
    ) -> Result<(), DbError> {
        let entry = self
            .find_session(id)?
            .entry(entry_id)
            .ok_or(DbError::EntryNotFound(entry_id))?;
        let expected = self
            .exercise(entry.exercise_id())
            .ok_or_else(|| DbError::ExerciseNotFound(entry.exercise_id().clone()))?
            .measure();
        let found = measure.kind();
        if expected == found {
            Ok(())
        } else {
            Err(DbError::MeasureMismatch { expected, found })
        }
    }
}

#[cfg(test)]
mod tests;
