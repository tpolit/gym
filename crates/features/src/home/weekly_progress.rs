use chrono::NaiveDate;
use domain::weekly::{
    average_previous_week_volume, done_count_in_week, goal_streak, week_start, week_volume_grams,
};
use domain::{Db, WeeklyGoal};

use crate::shared::format::format_week_count;

const VOLUME_AVERAGE_WEEKS: usize = 8;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WeeklyProgress {
    pub done_count: usize,
    pub goal: Option<u8>,
    pub percent: u32,
    pub label: String,
    pub streak_text: Option<String>,
    pub volume_text: Option<String>,
}

// Without a goal, only the number of sessions (no ring, no streak).
pub fn weekly_progress(db: &Db, today: NaiveDate) -> WeeklyProgress {
    let monday = week_start(today);
    let done_count = done_count_in_week(db, monday);
    let goal = db.preferences().weekly_goal();
    let percent = goal.map_or(0, |goal| {
        let ratio = done_count.min(usize::from(goal.get())) * 100 / usize::from(goal.get());
        u32::try_from(ratio).unwrap_or(100)
    });
    WeeklyProgress {
        done_count,
        goal: goal.map(WeeklyGoal::get),
        percent,
        label: format_week_count(done_count, goal.map(WeeklyGoal::get)),
        streak_text: goal.and_then(|goal| streak_text(goal_streak(db, goal, today))),
        volume_text: volume_text(
            week_volume_grams(db, monday),
            average_previous_week_volume(db, today, VOLUME_AVERAGE_WEEKS),
        ),
    }
}

fn streak_text(streak: usize) -> Option<String> {
    match streak {
        0 => None,
        1 => Some("Objectif tenu cette semaine".to_owned()),
        weeks => Some(format!("{weeks} semaines d'affilée")),
    }
}

fn volume_text(this_week: u64, average: Option<u64>) -> Option<String> {
    let average = average.filter(|average| *average > 0)?;
    if this_week == 0 {
        return None;
    }
    let difference = i128::from(this_week) - i128::from(average);
    let percent = difference * 100 / i128::from(average);
    Some(format!("Volume {percent:+} % vs moyenne"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::{bench, done_session, weight_reps};

    fn date(day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, 10, day).unwrap()
    }

    #[test]
    fn counts_sessions_without_goal() {
        let mut db = Db::default();
        done_session(&mut db, 27, &[(bench(), vec![weight_reps(80, 10)])]);
        let progress = weekly_progress(&db, date(28));
        assert_eq!(progress.done_count, 1);
        assert_eq!(progress.goal, None);
        assert_eq!(progress.label, "1 séance cette semaine");
        assert_eq!(progress.streak_text, None);
    }

    #[test]
    fn tracks_goal_streak_and_volume() {
        let mut db = Db::default();
        db.set_weekly_goal(WeeklyGoal::new(2).ok());
        done_session(&mut db, 19, &[(bench(), vec![weight_reps(100, 10)])]);
        done_session(&mut db, 21, &[(bench(), vec![weight_reps(100, 10)])]);
        done_session(&mut db, 27, &[(bench(), vec![weight_reps(110, 10)])]);
        done_session(&mut db, 28, &[(bench(), vec![weight_reps(110, 10)])]);
        let progress = weekly_progress(&db, date(28));
        assert_eq!(progress.percent, 100);
        assert_eq!(progress.label, "2 / 2 séances cette semaine");
        assert_eq!(
            progress.streak_text.as_deref(),
            Some("2 semaines d'affilée")
        );
        assert_eq!(
            progress.volume_text.as_deref(),
            Some("Volume +10 % vs moyenne")
        );
    }

    #[test]
    fn partial_goal_gives_a_percentage() {
        let mut db = Db::default();
        db.set_weekly_goal(WeeklyGoal::new(4).ok());
        done_session(&mut db, 27, &[(bench(), vec![weight_reps(80, 10)])]);
        let progress = weekly_progress(&db, date(28));
        assert_eq!(progress.percent, 25);
        assert_eq!(progress.streak_text, None);
        assert_eq!(progress.volume_text, None);
    }
}
