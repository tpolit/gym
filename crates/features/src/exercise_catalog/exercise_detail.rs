use domain::{Db, ExerciseId, SessionStatus};

use crate::shared::format::format_short_date;
use crate::shared::labels::{category_label, equipment_label, measure_label, muscle_group_label};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExerciseDetail {
    pub id: ExerciseId,
    pub name: String,
    pub category: &'static str,
    pub muscle_group: &'static str,
    pub equipment: &'static str,
    pub measure: &'static str,
    pub settings: String,
    pub done_session_count: usize,
    pub last_practiced: Option<String>,
    pub custom: Option<CustomState>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CustomState {
    pub id: u32,
    pub is_archived: bool,
    pub is_used: bool,
}

pub fn exercise_detail(db: &Db, id: &ExerciseId) -> Option<ExerciseDetail> {
    let info = db.exercise(id)?;
    let finish_times: Vec<_> = db
        .sessions()
        .iter()
        .filter(|session| {
            session
                .entries()
                .iter()
                .any(|entry| entry.exercise_id() == id && !entry.sets().is_empty())
        })
        .filter_map(|session| match session.status() {
            SessionStatus::Done { finished_at, .. } => Some(*finished_at),
            SessionStatus::Planned | SessionStatus::InProgress { .. } => None,
        })
        .collect();
    let custom = match id {
        ExerciseId::Builtin(_) => None,
        ExerciseId::Custom(custom_id) => {
            db.custom_exercise(*custom_id).map(|exercise| CustomState {
                id: *custom_id,
                is_archived: exercise.is_archived(),
                is_used: db.is_exercise_used(id),
            })
        }
    };
    Some(ExerciseDetail {
        id: id.clone(),
        name: info.name().to_string(),
        category: category_label(info.category()),
        muscle_group: muscle_group_label(info.muscle_group()),
        equipment: equipment_label(info.equipment()),
        measure: measure_label(info.measure()),
        settings: db
            .exercise_settings(id)
            .map(|settings| settings.text().to_string())
            .unwrap_or_default(),
        done_session_count: finish_times.len(),
        last_practiced: finish_times
            .iter()
            .max()
            .map(|finished_at| format_short_date(finished_at.date_naive())),
        custom,
    })
}

#[cfg(test)]
mod tests {
    use domain::MeasureKind;

    use super::*;
    use crate::test_support::{bench, custom_info, done_session, weight_reps};

    #[test]
    fn describes_a_practiced_builtin_exercise() {
        let mut db = Db::default();
        done_session(&mut db, 20, &[(bench(), vec![weight_reps(80, 8)])]);
        done_session(&mut db, 27, &[(bench(), vec![weight_reps(82, 8)])]);
        let detail = exercise_detail(&db, &bench()).unwrap();
        assert_eq!(detail.name, "Développé couché barre");
        assert_eq!(detail.category, "Musculation");
        assert_eq!(detail.muscle_group, "Pectoraux");
        assert_eq!(detail.equipment, "Barre");
        assert_eq!(detail.done_session_count, 2);
        assert_eq!(detail.last_practiced.as_deref(), Some("mar. 27 oct."));
        assert_eq!(detail.custom, None);
    }

    #[test]
    fn describes_a_custom_exercise() {
        let mut db = Db::default();
        let id = db
            .add_custom_exercise(custom_info("Rowing", MeasureKind::WeightReps))
            .unwrap();
        let detail = exercise_detail(&db, &id).unwrap();
        assert_eq!(
            detail.custom,
            Some(CustomState {
                id: 1,
                is_archived: false,
                is_used: false
            })
        );
        assert_eq!(detail.last_practiced, None);
        assert_eq!(exercise_detail(&db, &ExerciseId::Custom(5)), None);
    }
}
