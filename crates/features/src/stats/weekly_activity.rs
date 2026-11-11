use chrono::NaiveDate;
use domain::weekly::{
    done_sessions_by_week, previous_week_start, sessions_volume_grams, week_start,
};
use domain::{Db, SetMeasure};

use crate::shared::format::{format_integer, format_short_date};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WeekActivity {
    pub label: String,
    pub session_count: usize,
    pub volume_kg: u64,
    pub cardio_minutes: u64,
    pub cardio_meters: u64,
    pub is_current: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WeeklyActivity {
    pub weeks: Vec<WeekActivity>,
    pub current_volume_text: String,
    pub current_cardio_text: Option<String>,
}

// Sessions, volume and cardio of the last weeks, oldest first.
pub fn weekly_activity(db: &Db, today: NaiveDate, week_count: usize) -> WeeklyActivity {
    let by_week = done_sessions_by_week(db);
    let current = week_start(today);
    let mut mondays = vec![current];
    while mondays.len() < week_count {
        let earliest = mondays.last().copied().unwrap_or(current);
        mondays.push(previous_week_start(earliest));
    }
    mondays.reverse();
    let weeks: Vec<WeekActivity> = mondays
        .into_iter()
        .map(|monday| {
            let sessions = by_week.get(&monday).map(Vec::as_slice).unwrap_or_default();
            let sets: Vec<_> = sessions
                .iter()
                .flat_map(|session| session.entries())
                .flat_map(|entry| entry.sets())
                .filter(|set| !set.is_warmup())
                .collect();
            let (cardio_seconds, cardio_meters) =
                sets.iter().fold((0_u64, 0_u64), |(seconds, meters), set| {
                    match set.measure() {
                        SetMeasure::Cardio(cardio) => (
                            seconds + u64::from(cardio.duration().get()),
                            meters
                                + cardio
                                    .distance()
                                    .map_or(0, |distance| u64::from(distance.get())),
                        ),
                        SetMeasure::WeightReps { .. }
                        | SetMeasure::Reps { .. }
                        | SetMeasure::Duration { .. } => (seconds, meters),
                    }
                });
            WeekActivity {
                label: format_short_date(monday)
                    .split_once(' ')
                    .map_or_else(String::new, |(_, rest)| rest.to_owned()),
                session_count: sessions.len(),
                volume_kg: (sessions_volume_grams(db, sessions) + 500) / 1_000,
                cardio_minutes: cardio_seconds / 60,
                cardio_meters,
                is_current: monday == current,
            }
        })
        .collect();
    let this_week = weeks.last();
    WeeklyActivity {
        current_volume_text: format!(
            "{} kg",
            format_integer(this_week.map_or(0, |week| week.volume_kg))
        ),
        current_cardio_text: this_week
            .filter(|week| week.cardio_minutes > 0)
            .map(|week| cardio_text(week.cardio_minutes, week.cardio_meters)),
        weeks,
    }
}

fn cardio_text(minutes: u64, meters: u64) -> String {
    if meters == 0 {
        format!("{minutes} min")
    } else {
        format!(
            "{minutes} min · {},{} km",
            meters / 1_000,
            meters % 1_000 / 100
        )
    }
}

#[cfg(test)]
mod tests {
    use domain::{CardioMeasure, ExerciseId, Meters, Seconds};

    use super::*;
    use crate::test_support::{bench, date, done_session, weight_reps};

    #[test]
    fn summarizes_each_week_oldest_first() {
        let mut db = Db::default();
        done_session(&mut db, 20, &[(bench(), vec![weight_reps(80, 10)])]);
        let rower = ExerciseId::Builtin("indoor_rower".to_owned());
        let cardio = SetMeasure::Cardio(CardioMeasure::new(
            Seconds::new(1_200).unwrap(),
            Meters::new(5_200).ok(),
            None,
            None,
            None,
        ));
        done_session(
            &mut db,
            27,
            &[(bench(), vec![weight_reps(100, 10)]), (rower, vec![cardio])],
        );
        done_session(&mut db, 28, &[(bench(), vec![weight_reps(50, 10)])]);

        let activity = weekly_activity(&db, date(29), 3);
        assert_eq!(activity.weeks.len(), 3);
        let counts: Vec<usize> = activity
            .weeks
            .iter()
            .map(|week| week.session_count)
            .collect();
        assert_eq!(counts, [0, 1, 2]);
        assert_eq!(activity.weeks[2].label, "26 oct.");
        assert!(activity.weeks[2].is_current);
        assert_eq!(activity.weeks[2].volume_kg, 1_500);
        assert_eq!(activity.current_volume_text, "1\u{202f}500 kg");
        assert_eq!(
            activity.current_cardio_text.as_deref(),
            Some("20 min · 5,2 km")
        );
    }
}
