use domain::rest::rest_before_each_set;
use domain::{Db, EntryId, MeasureKind, SessionId, SessionStatus, SetMeasure};

use super::time_input::time_input_value;
use crate::shared::exercise_name::exercise_name;
use crate::shared::format::{
    format_clock, format_long_date, format_long_duration, format_set_measure, format_time,
};
use crate::shared::session_volume::session_volume_text;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionDetail {
    pub id: SessionId,
    pub name: String,
    pub date_text: String,
    pub volume: Option<String>,
    pub time_range: String,
    pub duration: String,
    pub started_input: String,
    pub finished_input: String,
    pub notes: String,
    pub exercises: Vec<DetailExercise>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DetailExercise {
    pub entry_id: EntryId,
    pub name: String,
    pub measure_kind: Option<MeasureKind>,
    pub index: usize,
    pub is_last: bool,
    pub sets: Vec<DetailSet>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DetailSet {
    pub index: usize,
    pub number: usize,
    pub text: String,
    pub measure: SetMeasure,
    pub is_warmup: bool,
    pub rest_before: Option<String>,
}

pub fn session_detail(db: &Db, session_id: SessionId) -> Option<SessionDetail> {
    let session = db.session(session_id)?;
    let SessionStatus::Done {
        started_at,
        finished_at,
    } = *session.status()
    else {
        return None;
    };
    let rests = rest_before_each_set(session);
    let entry_count = session.entries().len();
    Some(SessionDetail {
        id: session_id,
        name: session.name().to_string(),
        date_text: format_long_date(started_at.date_naive()),
        volume: session_volume_text(db, session),
        time_range: format!("{} → {}", format_time(started_at), format_time(finished_at)),
        duration: format_long_duration((finished_at - started_at).num_seconds()),
        started_input: time_input_value(started_at),
        finished_input: time_input_value(finished_at),
        notes: session.notes().map(ToString::to_string).unwrap_or_default(),
        exercises: session
            .entries()
            .iter()
            .zip(rests)
            .enumerate()
            .map(|(index, (entry, entry_rests))| DetailExercise {
                entry_id: entry.entry_id(),
                name: exercise_name(db, entry.exercise_id()),
                measure_kind: db.exercise(entry.exercise_id()).map(|info| info.measure()),
                index,
                is_last: index + 1 == entry_count,
                sets: entry
                    .sets()
                    .iter()
                    .zip(entry_rests)
                    .enumerate()
                    .map(|(set_index, (set, rest))| DetailSet {
                        index: set_index,
                        number: set_index + 1,
                        text: format_set_measure(set.measure()),
                        measure: *set.measure(),
                        is_warmup: set.is_warmup(),
                        rest_before: rest.map(|rest| format_clock(rest.num_seconds())),
                    })
                    .collect(),
            })
            .collect(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::{bench, done_session, pull_up, reps, start_session, weight_reps};

    #[test]
    fn details_a_done_session_with_rest_times() {
        let mut db = Db::default();
        let id = done_session(
            &mut db,
            28,
            &[
                (bench(), vec![weight_reps(60, 10), weight_reps(80, 8)]),
                (pull_up(), vec![reps(10)]),
            ],
        );
        let detail = session_detail(&db, id).unwrap();
        assert_eq!(detail.name, "Push A");
        assert_eq!(detail.date_text, "mercredi 28 octobre");
        assert_eq!(detail.time_range, "18:00 → 19:00");
        assert_eq!(detail.duration, "1 h 00");
        assert_eq!(detail.started_input, "2026-10-28T18:00");
        assert_eq!(detail.finished_input, "2026-10-28T19:00");
        assert_eq!(detail.volume.as_deref(), Some("1\u{202f}240 kg"));

        let bench_entry = &detail.exercises[0];
        assert_eq!(bench_entry.name, "Développé couché barre");
        assert_eq!(bench_entry.entry_id, EntryId::new(1));
        assert_eq!(bench_entry.measure_kind, Some(MeasureKind::WeightReps));
        assert!(!bench_entry.is_last && detail.exercises[1].is_last);
        let second_set = &bench_entry.sets[1];
        assert_eq!((second_set.index, second_set.number), (1, 2));
        assert_eq!(second_set.text, "80 × 8");
        assert_eq!(second_set.measure, weight_reps(80, 8));
        assert_eq!(second_set.rest_before.as_deref(), Some("5:00"));
        assert_eq!(bench_entry.sets[0].rest_before, None);
        assert_eq!(
            detail.exercises[1].sets[0].rest_before.as_deref(),
            Some("5:00")
        );
    }

    #[test]
    fn has_no_volume_without_weighted_sets() {
        let mut db = Db::default();
        let id = done_session(&mut db, 28, &[(pull_up(), vec![reps(10)])]);
        assert_eq!(session_detail(&db, id).unwrap().volume, None);
    }

    #[test]
    fn needs_a_done_session() {
        let mut db = Db::default();
        let current = start_session(&mut db, &[]);
        assert_eq!(session_detail(&db, current), None);
    }
}
