use crate::exercise::ExerciseId;
use crate::measure::MeasureKind;
use crate::session::{EntryId, SessionId};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DbError {
    SessionNotFound(SessionId),
    EntryNotFound(EntryId),
    SetNotFound {
        entry: EntryId,
        index: usize,
    },
    InvalidPosition(usize),
    ExerciseNotFound(ExerciseId),
    ExerciseArchived(ExerciseId),
    MeasureLockedByUse,
    MeasureMismatch {
        expected: MeasureKind,
        found: MeasureKind,
    },
    AnotherSessionInProgress,
    NotPlanned,
    NotInProgress,
    NotDone,
    SetsNotAllowedInPlannedSession,
    FinishedBeforeStarted,
    FinishWithoutSets,
    CancelWithSets,
    IdOverflow,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IntegrityError {
    DuplicateCustomExerciseId(u32),
    DuplicateSessionId(SessionId),
    DuplicateEntryId { session: SessionId, entry: EntryId },
    SeveralSessionsInProgress,
    UnknownExercise(ExerciseId),
    SetsInPlannedSession(SessionId),
    MeasureMismatch { session: SessionId, entry: EntryId },
    FinishedBeforeStarted(SessionId),
}
