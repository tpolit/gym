use chrono::NaiveDate;

use crate::body_weight::{BodyWeight, BodyWeightLog};
use crate::text::NonEmptyText;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct WeeklyGoal(u8);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WeeklyGoalError {
    OutOfRange,
}

impl WeeklyGoal {
    pub const MIN: u8 = 1;
    pub const MAX: u8 = 14;

    pub const fn new(sessions_per_week: u8) -> Result<Self, WeeklyGoalError> {
        if sessions_per_week >= Self::MIN && sessions_per_week <= Self::MAX {
            Ok(Self(sessions_per_week))
        } else {
            Err(WeeklyGoalError::OutOfRange)
        }
    }

    pub const fn get(self) -> u8 {
        self.0
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Preferences {
    weekly_goal: Option<WeeklyGoal>,
    first_name: Option<NonEmptyText>,
    body_weights: BodyWeightLog,
}

impl Preferences {
    pub const fn new(
        weekly_goal: Option<WeeklyGoal>,
        first_name: Option<NonEmptyText>,
        body_weights: BodyWeightLog,
    ) -> Self {
        Self {
            weekly_goal,
            first_name,
            body_weights,
        }
    }

    pub const fn body_weights(&self) -> &BodyWeightLog {
        &self.body_weights
    }

    pub const fn weekly_goal(&self) -> Option<WeeklyGoal> {
        self.weekly_goal
    }

    pub const fn first_name(&self) -> Option<&NonEmptyText> {
        self.first_name.as_ref()
    }

    pub(crate) fn set_weekly_goal(&mut self, goal: Option<WeeklyGoal>) {
        self.weekly_goal = goal;
    }

    pub(crate) fn set_first_name(&mut self, first_name: Option<NonEmptyText>) {
        self.first_name = first_name;
    }

    pub(crate) fn record_body_weight(&mut self, date: NaiveDate, weight: BodyWeight) {
        self.body_weights.record(date, weight);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn weekly_goal_accepts_1_to_14() {
        assert_eq!(WeeklyGoal::new(1).unwrap().get(), 1);
        assert_eq!(WeeklyGoal::new(14).unwrap().get(), 14);
    }

    #[test]
    fn weekly_goal_rejects_out_of_range() {
        assert_eq!(WeeklyGoal::new(0), Err(WeeklyGoalError::OutOfRange));
        assert_eq!(WeeklyGoal::new(15), Err(WeeklyGoalError::OutOfRange));
    }

    #[test]
    fn preferences_default_to_no_goal_and_no_first_name() {
        assert_eq!(Preferences::default().weekly_goal(), None);
        assert_eq!(Preferences::default().first_name(), None);
    }
}
