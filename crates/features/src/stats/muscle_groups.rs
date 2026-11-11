use std::collections::BTreeMap;

use chrono::NaiveDate;
use domain::records::done_sessions_chronologically;
use domain::weekly::week_start;
use domain::{Category, Db, MuscleGroup};

use crate::shared::labels::muscle_group_label;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MuscleGroupSets {
    pub label: &'static str,
    pub set_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MuscleGroupRecency {
    pub label: &'static str,
    pub days_since: i64,
    pub text: String,
}

// Sets per muscle group in the week of `today`, strength exercises only, warm-ups excluded.
pub fn sets_per_muscle_group(db: &Db, today: NaiveDate) -> Vec<MuscleGroupSets> {
    let monday = week_start(today);
    let mut counts: BTreeMap<MuscleGroup, usize> = BTreeMap::new();
    for (session, started_at) in done_sessions_chronologically(db) {
        if week_start(started_at.date_naive()) != monday {
            continue;
        }
        for entry in session.entries() {
            let Some(info) = db.exercise(entry.exercise_id()) else {
                continue;
            };
            if info.category() == Category::Strength {
                *counts.entry(info.muscle_group()).or_default() +=
                    entry.sets().iter().filter(|set| !set.is_warmup()).count();
            }
        }
    }
    let mut groups: Vec<MuscleGroupSets> = counts
        .into_iter()
        .filter(|(_, count)| *count > 0)
        .map(|(group, set_count)| MuscleGroupSets {
            label: muscle_group_label(group),
            set_count,
        })
        .collect();
    groups.sort_by_key(|group| std::cmp::Reverse(group.set_count));
    groups
}

// Days since each muscle group was last trained, the most neglected first.
pub fn days_since_muscle_group(db: &Db, today: NaiveDate) -> Vec<MuscleGroupRecency> {
    let mut last_trained: BTreeMap<MuscleGroup, NaiveDate> = BTreeMap::new();
    for (session, started_at) in done_sessions_chronologically(db) {
        for entry in session
            .entries()
            .iter()
            .filter(|entry| !entry.sets().is_empty())
        {
            if let Some(info) = db.exercise(entry.exercise_id())
                && info.category() == Category::Strength
            {
                last_trained.insert(info.muscle_group(), started_at.date_naive());
            }
        }
    }
    let mut groups: Vec<MuscleGroupRecency> = last_trained
        .into_iter()
        .map(|(group, date)| {
            let days_since = (today - date).num_days().max(0);
            MuscleGroupRecency {
                label: muscle_group_label(group),
                days_since,
                text: match days_since {
                    0 => "aujourd'hui".to_owned(),
                    1 => "hier".to_owned(),
                    days => format!("il y a {days} jours"),
                },
            }
        })
        .collect();
    groups.sort_by_key(|group| std::cmp::Reverse(group.days_since));
    groups
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::{bench, date, done_session, pull_up, reps, weight_reps};

    #[test]
    fn counts_sets_of_the_week_per_group() {
        let mut db = Db::default();
        done_session(&mut db, 20, &[(bench(), vec![weight_reps(80, 8)])]);
        done_session(
            &mut db,
            27,
            &[
                (bench(), vec![weight_reps(80, 8), weight_reps(80, 8)]),
                (pull_up(), vec![reps(10)]),
            ],
        );
        assert_eq!(
            sets_per_muscle_group(&db, date(28)),
            [
                MuscleGroupSets {
                    label: "Pectoraux",
                    set_count: 2
                },
                MuscleGroupSets {
                    label: "Dos",
                    set_count: 1
                },
            ]
        );
    }

    #[test]
    fn reports_days_since_last_training() {
        let mut db = Db::default();
        done_session(&mut db, 20, &[(pull_up(), vec![reps(10)])]);
        done_session(&mut db, 27, &[(bench(), vec![weight_reps(80, 8)])]);
        let recency = days_since_muscle_group(&db, date(28));
        assert_eq!(recency[0].label, "Dos");
        assert_eq!(recency[0].text, "il y a 8 jours");
        assert_eq!(recency[1].text, "hier");
    }
}
