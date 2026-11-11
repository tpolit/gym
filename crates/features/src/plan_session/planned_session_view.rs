use domain::{Db, EntryId, SessionId, SessionStatus};

use super::date_input::date_input_value;
use crate::shared::exercise_name::exercise_name;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlannedSessionView {
    pub id: SessionId,
    pub name: String,
    pub date_input: String,
    pub entries: Vec<PlannedEntryView>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlannedEntryView {
    pub entry_id: EntryId,
    pub exercise_name: String,
    pub index: usize,
    pub is_first: bool,
    pub is_last: bool,
}

pub fn planned_session_view(db: &Db, session_id: SessionId) -> Option<PlannedSessionView> {
    let session = db
        .session(session_id)
        .filter(|session| matches!(session.status(), SessionStatus::Planned))?;
    let entry_count = session.entries().len();
    Some(PlannedSessionView {
        id: session_id,
        name: session.name().to_string(),
        date_input: date_input_value(session.planned_for()),
        entries: session
            .entries()
            .iter()
            .enumerate()
            .map(|(index, entry)| PlannedEntryView {
                entry_id: entry.entry_id(),
                exercise_name: exercise_name(db, entry.exercise_id()),
                index,
                is_first: index == 0,
                is_last: index + 1 == entry_count,
            })
            .collect(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::{bench, date, db_with_session_in_progress, pull_up, text};

    #[test]
    fn describes_a_planned_session() {
        let mut db = Db::default();
        let id = db
            .plan_session(text("Push A"), Some(date(31)), &[bench(), pull_up()])
            .unwrap();
        let view = planned_session_view(&db, id).unwrap();
        assert_eq!(view.name, "Push A");
        assert_eq!(view.date_input, "2026-10-31");
        assert_eq!(view.entries[0].exercise_name, "Développé couché barre");
        assert!(view.entries[0].is_first && !view.entries[0].is_last);
        assert!(view.entries[1].is_last);
    }

    #[test]
    fn ignores_sessions_that_are_not_planned() {
        let (db, in_progress) = db_with_session_in_progress(&[]);
        assert_eq!(planned_session_view(&db, in_progress), None);
    }
}
