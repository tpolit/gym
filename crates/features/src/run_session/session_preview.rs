use domain::{Db, ExerciseId, SessionId, SessionStatus};

use crate::shared::exercise_name::exercise_name;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionPreview {
    pub name: String,
    pub exercise_names: Vec<String>,
}

// What starting a planned session, or repeating a done one, will produce.
pub fn session_preview(db: &Db, session_id: SessionId) -> Option<SessionPreview> {
    let session = db.session(session_id)?;
    let (name, exercises): (String, Vec<ExerciseId>) = match session.status() {
        SessionStatus::Planned => (
            session.name().to_string(),
            session
                .entries()
                .iter()
                .map(|entry| entry.exercise_id().clone())
                .collect(),
        ),
        SessionStatus::Done { .. } => {
            let (name, exercises) = db.session_template(session_id).ok()?;
            (name.to_string(), exercises)
        }
        SessionStatus::InProgress { .. } => return None,
    };
    Some(SessionPreview {
        name,
        exercise_names: exercises.iter().map(|id| exercise_name(db, id)).collect(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::{
        bench, done_session, pull_up, reps, start_session, text, weight_reps,
    };

    #[test]
    fn previews_planned_and_done_sessions() {
        let mut db = Db::default();
        let done = done_session(
            &mut db,
            27,
            &[
                (bench(), vec![weight_reps(80, 8)]),
                (pull_up(), vec![reps(10)]),
            ],
        );
        let planned = db.plan_session(text("Jambes"), None, &[bench()]).unwrap();
        assert_eq!(
            session_preview(&db, done),
            Some(SessionPreview {
                name: "Push A".to_owned(),
                exercise_names: vec!["Développé couché barre".to_owned(), "Tractions".to_owned()],
            })
        );
        assert_eq!(session_preview(&db, planned).unwrap().name, "Jambes");
        let current = start_session(&mut db, &[]);
        assert_eq!(session_preview(&db, current), None);
    }
}
