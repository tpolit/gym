use chrono::{DateTime, FixedOffset};
use domain::rest::latest_completed_set;
use domain::{Db, SessionId};

use crate::shared::exercise_name::exercise_name;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RestStart {
    pub since: DateTime<FixedOffset>,
    pub label: String,
}

// The timer is computed from the completion time, so it stays right after sleep.
pub fn rest_started_at(db: &Db, session_id: SessionId) -> Option<RestStart> {
    let session = db.session(session_id)?;
    let latest = latest_completed_set(session)?;
    let entry = session.entry(latest.entry_id)?;
    Some(RestStart {
        since: latest.completed_at,
        label: format!(
            "Depuis la série {} · {}",
            latest.index + 1,
            exercise_name(db, entry.exercise_id())
        ),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::{
        add_set_at, at, bench, db_with_session_in_progress, pull_up, reps, weight_reps,
    };

    #[test]
    fn starts_at_the_last_validated_set_of_any_exercise() {
        let (mut db, id) = db_with_session_in_progress(&[bench(), pull_up()]);
        assert_eq!(rest_started_at(&db, id), None);
        add_set_at(&mut db, id, 1, weight_reps(80, 8), 5);
        add_set_at(&mut db, id, 1, weight_reps(80, 8), 8);
        add_set_at(&mut db, id, 2, reps(10), 12);
        assert_eq!(
            rest_started_at(&db, id),
            Some(RestStart {
                since: at(30, 18, 12),
                label: "Depuis la série 1 · Tractions".to_owned(),
            })
        );
    }
}
