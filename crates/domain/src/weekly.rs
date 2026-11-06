// ISO weeks (Monday to Sunday), computed in the time zone recorded with each session.
// A session belongs to the week of its start date.

use std::collections::BTreeMap;

use chrono::{Datelike, Days, NaiveDate};

use crate::db::Db;
use crate::preferences::WeeklyGoal;
use crate::records::done_sessions_chronologically;
use crate::session::Session;
use crate::volume::session_volume_grams;

const DAYS_PER_WEEK: u64 = 7;

pub fn week_start(date: NaiveDate) -> NaiveDate {
    date - Days::new(u64::from(date.weekday().num_days_from_monday()))
}

pub fn previous_week_start(monday: NaiveDate) -> NaiveDate {
    monday - Days::new(DAYS_PER_WEEK)
}

pub fn done_sessions_by_week(db: &Db) -> BTreeMap<NaiveDate, Vec<&Session>> {
    let mut weeks: BTreeMap<NaiveDate, Vec<&Session>> = BTreeMap::new();
    for (session, started_at) in done_sessions_chronologically(db) {
        weeks
            .entry(week_start(started_at.date_naive()))
            .or_default()
            .push(session);
    }
    weeks
}

pub fn done_count_in_week(db: &Db, monday: NaiveDate) -> usize {
    done_sessions_by_week(db).get(&monday).map_or(0, Vec::len)
}

// Consecutive weeks reaching the goal. The current week only adds to the streak once reached:
// a week still in progress never breaks it (kind tone).
pub fn goal_streak(db: &Db, goal: WeeklyGoal, today: NaiveDate) -> usize {
    let weeks = done_sessions_by_week(db);
    let reached = |monday: &NaiveDate| {
        weeks
            .get(monday)
            .is_some_and(|sessions| sessions.len() >= usize::from(goal.get()))
    };
    let current = week_start(today);
    let mut streak = usize::from(reached(&current));
    let mut monday = previous_week_start(current);
    while reached(&monday) {
        streak += 1;
        monday = previous_week_start(monday);
    }
    streak
}

pub fn week_volume_grams(db: &Db, monday: NaiveDate) -> u64 {
    done_sessions_by_week(db)
        .get(&monday)
        .map_or(0, |sessions| sessions_volume_grams(db, sessions))
}

pub fn sessions_volume_grams(db: &Db, sessions: &[&Session]) -> u64 {
    sessions
        .iter()
        .filter_map(|session| session_volume_grams(db, session))
        .sum()
}

// Average weekly volume over the previous active weeks (weeks without sessions are ignored).
pub fn average_previous_week_volume(db: &Db, today: NaiveDate, week_count: usize) -> Option<u64> {
    let current = week_start(today);
    let volumes: Vec<u64> = done_sessions_by_week(db)
        .into_iter()
        .rev()
        .filter(|(monday, _)| *monday < current)
        .take(week_count)
        .map(|(_, sessions)| sessions_volume_grams(db, &sessions))
        .filter(|volume| *volume > 0)
        .collect();
    let count = u64::try_from(volumes.len())
        .ok()
        .filter(|count| *count > 0)?;
    Some(volumes.iter().sum::<u64>() / count)
}

#[cfg(test)]
mod tests {
    use std::num::NonZeroU32;

    use chrono::{DateTime, FixedOffset, TimeZone};

    use super::*;
    use crate::exercise::ExerciseId;
    use crate::measure::{SetMeasure, Weight};
    use crate::session::EntryId;
    use crate::set::WorkoutSet;
    use crate::text::NonEmptyText;

    fn at(month: u32, day: u32) -> DateTime<FixedOffset> {
        FixedOffset::east_opt(3_600)
            .unwrap()
            .with_ymd_and_hms(2026, month, day, 18, 0, 0)
            .unwrap()
    }

    fn date(month: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, month, day).unwrap()
    }

    fn done(db: &mut Db, month: u32, day: u32, kg: u32) {
        let bench = ExerciseId::Builtin("barbell_bench_press".to_owned());
        let id = db
            .start_new_session(NonEmptyText::new("Push").unwrap(), &[bench], at(month, day))
            .unwrap();
        let measure = SetMeasure::WeightReps {
            weight: Weight::from_grams(kg * 1_000),
            reps: NonZeroU32::new(10).unwrap(),
        };
        db.add_set(id, EntryId::new(1), WorkoutSet::new(measure, false, None))
            .unwrap();
        db.finish_session(id, at(month, day) + chrono::TimeDelta::hours(1))
            .unwrap();
    }

    #[test]
    fn weeks_start_on_monday() {
        assert_eq!(week_start(date(11, 2)), date(11, 2));
        assert_eq!(week_start(date(11, 8)), date(11, 2));
        assert_eq!(previous_week_start(date(11, 2)), date(10, 26));
    }

    #[test]
    fn streak_counts_consecutive_weeks_reaching_the_goal() {
        let mut db = Db::default();
        let goal = WeeklyGoal::new(2).unwrap();
        for (month, day) in [(10, 12), (10, 14), (10, 19), (10, 21), (10, 27)] {
            done(&mut db, month, day, 50);
        }
        // Week of Oct 26 has a single session: the current week (Nov 2) does not break anything.
        assert_eq!(goal_streak(&db, goal, date(10, 28)), 2);
        assert_eq!(goal_streak(&db, goal, date(11, 4)), 0);
        done(&mut db, 10, 29, 50);
        assert_eq!(goal_streak(&db, goal, date(10, 30)), 3);
        assert_eq!(done_count_in_week(&db, date(10, 26)), 2);
    }

    #[test]
    fn volumes_per_week() {
        let mut db = Db::default();
        done(&mut db, 10, 19, 50);
        done(&mut db, 10, 27, 100);
        done(&mut db, 11, 3, 80);
        assert_eq!(week_volume_grams(&db, date(11, 2)), 800_000);
        assert_eq!(
            average_previous_week_volume(&db, date(11, 4), 4),
            Some((500_000 + 1_000_000) / 2)
        );
        assert_eq!(average_previous_week_volume(&db, date(10, 20), 4), None);
    }
}
