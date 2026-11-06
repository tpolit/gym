use domain::records::records_broken_in;
use domain::{Db, SessionId};

use crate::shared::records::{RecordBadge, record_badges};

// Records broken during the session, compared with earlier done sessions.
pub fn session_records(db: &Db, session_id: SessionId) -> Vec<RecordBadge> {
    record_badges(db, &records_broken_in(db, session_id))
}

// Records gained by the last validated set, to celebrate right away.
pub fn new_records(before: &[RecordBadge], after: &[RecordBadge]) -> Vec<RecordBadge> {
    after
        .iter()
        .filter(|badge| {
            !before.iter().any(|earlier| {
                earlier.exercise_id == badge.exercise_id
                    && earlier.kind == badge.kind
                    && earlier.value == badge.value
            })
        })
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests {
    use domain::records::RecordKind;

    use super::*;
    use crate::test_support::{add_set_at, bench, done_session, start_session, weight_reps};

    #[test]
    fn detects_records_as_sets_are_added() {
        let mut db = Db::default();
        done_session(&mut db, 27, &[(bench(), vec![weight_reps(80, 8)])]);
        let current = start_session(&mut db, &[bench()]);
        add_set_at(&mut db, current, 1, weight_reps(70, 8), 5);
        let before = session_records(&db, current);
        assert!(before.is_empty());

        add_set_at(&mut db, current, 1, weight_reps(85, 5), 8);
        let after = session_records(&db, current);
        let gained = new_records(&before, &after);
        let kinds: Vec<RecordKind> = gained.iter().map(|badge| badge.kind).collect();
        assert_eq!(
            kinds,
            [RecordKind::HeaviestWeight, RecordKind::BestSessionVolume]
        );
        assert!(new_records(&after, &after).is_empty());
    }
}
