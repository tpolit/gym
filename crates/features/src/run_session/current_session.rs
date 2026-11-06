use chrono::{DateTime, FixedOffset};
use domain::{Db, EntryId, ExerciseId, SessionId};

use crate::shared::exercise_name::exercise_name;
use crate::shared::format::format_set_count;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionView {
    pub id: SessionId,
    pub name: String,
    pub started_at: DateTime<FixedOffset>,
    pub entries: Vec<EntryView>,
    pub has_sets: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntryView {
    pub entry_id: EntryId,
    pub exercise_id: ExerciseId,
    pub exercise_name: String,
    pub settings: Option<String>,
    pub set_count: usize,
    pub set_count_text: String,
    pub position_label: String,
    pub index: usize,
    pub is_last: bool,
}

pub fn current_session(db: &Db) -> Option<SessionView> {
    let session = db.session_in_progress()?;
    let entry_count = session.entries().len();
    Some(SessionView {
        id: session.id(),
        name: session.name().to_string(),
        started_at: session.status().started_at()?,
        entries: session
            .entries()
            .iter()
            .enumerate()
            .map(|(index, entry)| EntryView {
                entry_id: entry.entry_id(),
                exercise_id: entry.exercise_id().clone(),
                exercise_name: exercise_name(db, entry.exercise_id()),
                settings: db
                    .exercise_settings(entry.exercise_id())
                    .map(|settings| settings.text().to_string()),
                set_count: entry.sets().len(),
                set_count_text: format_set_count(entry.sets().len()),
                position_label: format!("Exercice {} / {entry_count}", index + 1),
                index,
                is_last: index + 1 == entry_count,
            })
            .collect(),
        has_sets: session.has_sets(),
    })
}

#[cfg(test)]
mod tests {
    use domain::{ExerciseSettings, NonEmptyText};

    use super::*;
    use crate::test_support::{
        add_set_at, at, bench, db_with_session_in_progress, pull_up, weight_reps,
    };

    #[test]
    fn describes_the_session_in_progress() {
        let (mut db, id) = db_with_session_in_progress(&[bench(), pull_up()]);
        add_set_at(&mut db, id, 1, weight_reps(80, 8), 5);
        db.set_exercise_settings(
            bench(),
            Some(ExerciseSettings::new(
                NonEmptyText::new("Banc cran 2").unwrap(),
            )),
        )
        .unwrap();

        let view = current_session(&db).unwrap();
        assert_eq!(view.id, id);
        assert_eq!(view.name, "Push A");
        assert_eq!(view.started_at, at(30, 18, 0));
        assert!(view.has_sets);
        assert_eq!(view.entries[0].exercise_name, "Développé couché barre");
        assert_eq!(view.entries[0].settings.as_deref(), Some("Banc cran 2"));
        assert_eq!(view.entries[0].set_count, 1);
        assert_eq!(view.entries[0].set_count_text, "1 série");
        assert_eq!(view.entries[1].position_label, "Exercice 2 / 2");
        assert_eq!(view.entries[1].exercise_name, "Tractions");
        assert_eq!(view.entries[1].set_count, 0);
    }

    #[test]
    fn is_empty_without_session_in_progress() {
        assert_eq!(current_session(&Db::default()), None);
    }
}
