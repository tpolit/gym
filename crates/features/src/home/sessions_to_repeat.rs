use std::collections::BTreeSet;

use domain::{Db, SessionId, SessionStatus};

use crate::shared::format::format_short_date;
use crate::shared::search::normalize_for_search;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepeatChoice {
    pub id: SessionId,
    pub name: String,
    pub last_done: String,
}

// Most recent done sessions, one per name.
pub fn sessions_to_repeat(db: &Db, limit: usize) -> Vec<RepeatChoice> {
    let mut done: Vec<_> = db
        .sessions()
        .iter()
        .filter_map(|session| match session.status() {
            SessionStatus::Done { finished_at, .. } => Some((session, *finished_at)),
            SessionStatus::Planned | SessionStatus::InProgress { .. } => None,
        })
        .collect();
    done.sort_by_key(|(_, finished_at)| std::cmp::Reverse(*finished_at));
    let mut seen_names = BTreeSet::new();
    done.into_iter()
        .filter(|(session, _)| seen_names.insert(normalize_for_search(session.name().as_str())))
        .take(limit)
        .map(|(session, finished_at)| RepeatChoice {
            id: session.id(),
            name: session.name().to_string(),
            last_done: format_short_date(finished_at.date_naive()),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::{bench, done_session, weight_reps};

    #[test]
    fn keeps_the_latest_session_of_each_name() {
        let mut db = Db::default();
        done_session(&mut db, 20, &[(bench(), vec![weight_reps(80, 8)])]);
        let latest = done_session(&mut db, 27, &[(bench(), vec![weight_reps(82, 8)])]);
        let choices = sessions_to_repeat(&db, 3);
        assert_eq!(
            choices,
            [RepeatChoice {
                id: latest,
                name: "Push A".to_owned(),
                last_done: "mar. 27 oct.".to_owned(),
            }]
        );
        assert!(sessions_to_repeat(&db, 0).is_empty());
    }
}
