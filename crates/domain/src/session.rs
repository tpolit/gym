// Mutations are pub(crate): they go through Db, the only place able to check cross-cutting
// invariants (existing exercise, measure kind, at most one session in progress).

use std::collections::BTreeSet;

use chrono::{DateTime, FixedOffset, NaiveDate};

use crate::error::{DbError, IntegrityError};
use crate::exercise::ExerciseId;
use crate::measure::SetMeasure;
use crate::set::WorkoutSet;
use crate::text::NonEmptyText;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SessionId(u32);

impl SessionId {
    pub const fn new(value: u32) -> Self {
        Self(value)
    }

    pub const fn get(self) -> u32 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EntryId(u32);

impl EntryId {
    pub const fn new(value: u32) -> Self {
        Self(value)
    }

    pub const fn get(self) -> u32 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionOrigin {
    Planned,
    Adhoc,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionStatus {
    Planned,
    InProgress {
        started_at: DateTime<FixedOffset>,
    },
    Done {
        started_at: DateTime<FixedOffset>,
        finished_at: DateTime<FixedOffset>,
    },
}

impl SessionStatus {
    pub const fn started_at(&self) -> Option<DateTime<FixedOffset>> {
        match self {
            Self::Planned => None,
            Self::InProgress { started_at } | Self::Done { started_at, .. } => Some(*started_at),
        }
    }

    pub const fn finished_at(&self) -> Option<DateTime<FixedOffset>> {
        match self {
            Self::Done { finished_at, .. } => Some(*finished_at),
            Self::Planned | Self::InProgress { .. } => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionExercise {
    entry_id: EntryId,
    exercise_id: ExerciseId,
    sets: Vec<WorkoutSet>,
}

impl SessionExercise {
    pub const fn restore(
        entry_id: EntryId,
        exercise_id: ExerciseId,
        sets: Vec<WorkoutSet>,
    ) -> Self {
        Self {
            entry_id,
            exercise_id,
            sets,
        }
    }

    pub const fn entry_id(&self) -> EntryId {
        self.entry_id
    }

    pub const fn exercise_id(&self) -> &ExerciseId {
        &self.exercise_id
    }

    pub fn sets(&self) -> &[WorkoutSet] {
        &self.sets
    }

    fn set_mut(&mut self, index: usize) -> Result<&mut WorkoutSet, DbError> {
        let entry = self.entry_id;
        self.sets
            .get_mut(index)
            .ok_or(DbError::SetNotFound { entry, index })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Session {
    id: SessionId,
    name: NonEmptyText,
    origin: SessionOrigin,
    planned_for: Option<NaiveDate>,
    status: SessionStatus,
    notes: Option<NonEmptyText>,
    entries: Vec<SessionExercise>,
}

impl Session {
    pub fn restore(
        id: SessionId,
        name: NonEmptyText,
        origin: SessionOrigin,
        planned_for: Option<NaiveDate>,
        status: SessionStatus,
        notes: Option<NonEmptyText>,
        entries: Vec<SessionExercise>,
    ) -> Result<Self, IntegrityError> {
        if let SessionStatus::Done {
            started_at,
            finished_at,
        } = status
            && finished_at < started_at
        {
            return Err(IntegrityError::FinishedBeforeStarted(id));
        }
        let session = Self {
            id,
            name,
            origin,
            planned_for,
            status,
            notes,
            entries,
        };
        if session.status == SessionStatus::Planned && session.has_sets() {
            return Err(IntegrityError::SetsInPlannedSession(id));
        }
        let mut entry_ids = BTreeSet::new();
        for entry in &session.entries {
            if !entry_ids.insert(entry.entry_id) {
                return Err(IntegrityError::DuplicateEntryId {
                    session: id,
                    entry: entry.entry_id,
                });
            }
        }
        Ok(session)
    }

    pub(crate) fn new_planned(
        id: SessionId,
        name: NonEmptyText,
        planned_for: Option<NaiveDate>,
        exercises: Vec<ExerciseId>,
    ) -> Self {
        Self {
            id,
            name,
            origin: SessionOrigin::Planned,
            planned_for,
            status: SessionStatus::Planned,
            notes: None,
            entries: numbered_entries(exercises),
        }
    }

    pub(crate) fn new_started(
        id: SessionId,
        name: NonEmptyText,
        exercises: Vec<ExerciseId>,
        now: DateTime<FixedOffset>,
    ) -> Self {
        Self {
            id,
            name,
            origin: SessionOrigin::Adhoc,
            planned_for: None,
            status: SessionStatus::InProgress { started_at: now },
            notes: None,
            entries: numbered_entries(exercises),
        }
    }

    pub const fn id(&self) -> SessionId {
        self.id
    }

    pub const fn name(&self) -> &NonEmptyText {
        &self.name
    }

    pub const fn origin(&self) -> SessionOrigin {
        self.origin
    }

    pub const fn planned_for(&self) -> Option<NaiveDate> {
        self.planned_for
    }

    pub const fn status(&self) -> &SessionStatus {
        &self.status
    }

    pub const fn notes(&self) -> Option<&NonEmptyText> {
        self.notes.as_ref()
    }

    pub fn entries(&self) -> &[SessionExercise] {
        &self.entries
    }

    pub fn entry(&self, entry_id: EntryId) -> Option<&SessionExercise> {
        self.entries.iter().find(|entry| entry.entry_id == entry_id)
    }

    pub fn has_sets(&self) -> bool {
        self.entries.iter().any(|entry| !entry.sets.is_empty())
    }

    pub(crate) fn start(&mut self, now: DateTime<FixedOffset>) -> Result<(), DbError> {
        match self.status {
            SessionStatus::Planned => {
                self.status = SessionStatus::InProgress { started_at: now };
                Ok(())
            }
            SessionStatus::InProgress { .. } | SessionStatus::Done { .. } => {
                Err(DbError::NotPlanned)
            }
        }
    }

    // Entries without sets are dropped.
    pub(crate) fn finish(&mut self, now: DateTime<FixedOffset>) -> Result<(), DbError> {
        let SessionStatus::InProgress { started_at } = self.status else {
            return Err(DbError::NotInProgress);
        };
        if now < started_at {
            return Err(DbError::FinishedBeforeStarted);
        }
        if !self.has_sets() {
            return Err(DbError::FinishWithoutSets);
        }
        self.entries.retain(|entry| !entry.sets.is_empty());
        self.status = SessionStatus::Done {
            started_at,
            finished_at: now,
        };
        Ok(())
    }

    pub(crate) fn ensure_cancellable(&self) -> Result<(), DbError> {
        if !matches!(self.status, SessionStatus::InProgress { .. }) {
            return Err(DbError::NotInProgress);
        }
        if self.has_sets() {
            return Err(DbError::CancelWithSets);
        }
        Ok(())
    }

    pub(crate) fn back_to_planned(&mut self) -> Result<(), DbError> {
        self.ensure_cancellable()?;
        self.status = SessionStatus::Planned;
        Ok(())
    }

    pub(crate) fn rename(&mut self, name: NonEmptyText) {
        self.name = name;
    }

    pub(crate) fn set_planned_for(&mut self, planned_for: Option<NaiveDate>) {
        self.planned_for = planned_for;
    }

    pub(crate) fn set_notes(&mut self, notes: Option<NonEmptyText>) {
        self.notes = notes;
    }

    pub(crate) fn update_times(
        &mut self,
        started_at: DateTime<FixedOffset>,
        finished_at: DateTime<FixedOffset>,
    ) -> Result<(), DbError> {
        if !matches!(self.status, SessionStatus::Done { .. }) {
            return Err(DbError::NotDone);
        }
        if finished_at < started_at {
            return Err(DbError::FinishedBeforeStarted);
        }
        self.status = SessionStatus::Done {
            started_at,
            finished_at,
        };
        Ok(())
    }

    pub(crate) fn add_entry(&mut self, exercise_id: ExerciseId) -> Result<EntryId, DbError> {
        let entry_id = EntryId(next_id(self.entries.iter().map(|e| e.entry_id.0))?);
        self.entries.push(SessionExercise {
            entry_id,
            exercise_id,
            sets: Vec::new(),
        });
        Ok(entry_id)
    }

    pub(crate) fn remove_entry(&mut self, entry_id: EntryId) -> Result<(), DbError> {
        let index = self.entry_index(entry_id)?;
        self.entries.remove(index);
        Ok(())
    }

    pub(crate) fn move_entry(
        &mut self,
        entry_id: EntryId,
        new_index: usize,
    ) -> Result<(), DbError> {
        let index = self.entry_index(entry_id)?;
        if new_index >= self.entries.len() {
            return Err(DbError::InvalidPosition(new_index));
        }
        let entry = self.entries.remove(index);
        self.entries.insert(new_index, entry);
        Ok(())
    }

    pub(crate) fn add_set(&mut self, entry_id: EntryId, set: WorkoutSet) -> Result<(), DbError> {
        if self.status == SessionStatus::Planned {
            return Err(DbError::SetsNotAllowedInPlannedSession);
        }
        self.entry_mut(entry_id)?.sets.push(set);
        Ok(())
    }

    pub(crate) fn update_set(
        &mut self,
        entry_id: EntryId,
        index: usize,
        measure: SetMeasure,
        warmup: bool,
    ) -> Result<(), DbError> {
        self.entry_mut(entry_id)?
            .set_mut(index)?
            .update(measure, warmup);
        Ok(())
    }

    pub(crate) fn remove_set(&mut self, entry_id: EntryId, index: usize) -> Result<(), DbError> {
        let entry = self.entry_mut(entry_id)?;
        if index >= entry.sets.len() {
            return Err(DbError::SetNotFound {
                entry: entry_id,
                index,
            });
        }
        entry.sets.remove(index);
        Ok(())
    }

    fn entry_index(&self, entry_id: EntryId) -> Result<usize, DbError> {
        self.entries
            .iter()
            .position(|entry| entry.entry_id == entry_id)
            .ok_or(DbError::EntryNotFound(entry_id))
    }

    fn entry_mut(&mut self, entry_id: EntryId) -> Result<&mut SessionExercise, DbError> {
        self.entries
            .iter_mut()
            .find(|entry| entry.entry_id == entry_id)
            .ok_or(DbError::EntryNotFound(entry_id))
    }
}

fn numbered_entries(exercises: Vec<ExerciseId>) -> Vec<SessionExercise> {
    exercises
        .into_iter()
        .zip(1..)
        .map(|(exercise_id, id)| SessionExercise {
            entry_id: EntryId(id),
            exercise_id,
            sets: Vec::new(),
        })
        .collect()
}

pub(crate) fn next_id(ids: impl Iterator<Item = u32>) -> Result<u32, DbError> {
    ids.max()
        .map_or(Some(1), |max| max.checked_add(1))
        .ok_or(DbError::IdOverflow)
}

#[cfg(test)]
mod tests {
    use std::num::NonZeroU32;

    use chrono::TimeZone;

    use super::*;
    use crate::measure::Weight;

    fn at(hour: u32, minute: u32) -> DateTime<FixedOffset> {
        FixedOffset::east_opt(3_600)
            .unwrap()
            .with_ymd_and_hms(2026, 10, 28, hour, minute, 0)
            .unwrap()
    }

    fn name(text: &str) -> NonEmptyText {
        NonEmptyText::new(text).unwrap()
    }

    fn bench() -> ExerciseId {
        ExerciseId::Builtin("barbell_bench_press".to_owned())
    }

    fn set() -> WorkoutSet {
        WorkoutSet::new(
            SetMeasure::WeightReps {
                weight: Weight::from_grams(80_000),
                reps: NonZeroU32::new(8).unwrap(),
            },
            false,
            None,
        )
    }

    fn entry(id: u32, sets: Vec<WorkoutSet>) -> SessionExercise {
        SessionExercise::restore(EntryId::new(id), bench(), sets)
    }

    fn restore(
        status: SessionStatus,
        entries: Vec<SessionExercise>,
    ) -> Result<Session, IntegrityError> {
        Session::restore(
            SessionId::new(1),
            name("Push A"),
            SessionOrigin::Planned,
            None,
            status,
            None,
            entries,
        )
    }

    #[test]
    fn status_exposes_its_times() {
        let done = SessionStatus::Done {
            started_at: at(18, 0),
            finished_at: at(19, 0),
        };
        assert_eq!(done.started_at(), Some(at(18, 0)));
        assert_eq!(done.finished_at(), Some(at(19, 0)));
        assert_eq!(SessionStatus::Planned.started_at(), None);
        assert_eq!(
            SessionStatus::InProgress {
                started_at: at(18, 0)
            }
            .finished_at(),
            None
        );
    }

    #[test]
    fn restore_accepts_consistent_session() {
        let status = SessionStatus::Done {
            started_at: at(18, 0),
            finished_at: at(19, 0),
        };
        let session = restore(status, vec![entry(1, vec![set()]), entry(2, vec![])]).unwrap();
        assert_eq!(session.entries().len(), 2);
        assert!(session.has_sets());
    }

    #[test]
    fn restore_rejects_finish_before_start() {
        let status = SessionStatus::Done {
            started_at: at(19, 0),
            finished_at: at(18, 0),
        };
        assert_eq!(
            restore(status, vec![]),
            Err(IntegrityError::FinishedBeforeStarted(SessionId::new(1)))
        );
    }

    #[test]
    fn restore_rejects_sets_in_planned_session() {
        assert_eq!(
            restore(SessionStatus::Planned, vec![entry(1, vec![set()])]),
            Err(IntegrityError::SetsInPlannedSession(SessionId::new(1)))
        );
    }

    #[test]
    fn restore_rejects_duplicate_entry_ids() {
        assert_eq!(
            restore(
                SessionStatus::Planned,
                vec![entry(1, vec![]), entry(1, vec![])]
            ),
            Err(IntegrityError::DuplicateEntryId {
                session: SessionId::new(1),
                entry: EntryId::new(1),
            })
        );
    }

    #[test]
    fn new_sessions_number_entries_from_one() {
        let session = Session::new_planned(
            SessionId::new(1),
            name("Push A"),
            None,
            vec![bench(), bench()],
        );
        let ids: Vec<_> = session
            .entries()
            .iter()
            .map(|e| e.entry_id().get())
            .collect();
        assert_eq!(ids, [1, 2]);
    }

    #[test]
    fn next_id_is_max_plus_one() {
        assert_eq!(next_id([].into_iter()), Ok(1));
        assert_eq!(next_id([3, 1, 2].into_iter()), Ok(4));
        assert_eq!(next_id([u32::MAX].into_iter()), Err(DbError::IdOverflow));
    }

    #[test]
    fn move_entry_reorders_entries() {
        let mut session = Session::new_planned(
            SessionId::new(1),
            name("Push A"),
            None,
            vec![bench(), bench(), bench()],
        );
        session.move_entry(EntryId::new(3), 0).unwrap();
        let ids: Vec<_> = session
            .entries()
            .iter()
            .map(|e| e.entry_id().get())
            .collect();
        assert_eq!(ids, [3, 1, 2]);
        assert_eq!(
            session.move_entry(EntryId::new(1), 3),
            Err(DbError::InvalidPosition(3))
        );
    }
}
