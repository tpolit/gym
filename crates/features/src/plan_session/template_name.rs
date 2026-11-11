use domain::{Db, SessionId, SessionStatus};

// Name of the done session used as a template when planning from it.
pub fn template_name(db: &Db, source: SessionId) -> Option<String> {
    db.session(source)
        .filter(|session| matches!(session.status(), SessionStatus::Done { .. }))
        .map(|session| session.name().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::{bench, done_session, start_session, weight_reps};

    #[test]
    fn names_only_done_sessions() {
        let mut db = Db::default();
        let done = done_session(&mut db, 27, &[(bench(), vec![weight_reps(80, 8)])]);
        let current = start_session(&mut db, &[]);
        assert_eq!(template_name(&db, done).as_deref(), Some("Push A"));
        assert_eq!(template_name(&db, current), None);
    }
}
