use domain::{Db, WeeklyGoal};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WeeklyGoalError {
    OutOfRange,
}

// 1 to 14 sessions per week, or `None` for no goal.
pub fn set_weekly_goal(db: &mut Db, sessions_per_week: Option<u8>) -> Result<(), WeeklyGoalError> {
    let goal = sessions_per_week
        .map(|count| WeeklyGoal::new(count).map_err(|_| WeeklyGoalError::OutOfRange))
        .transpose()?;
    db.set_weekly_goal(goal);
    Ok(())
}

pub fn weekly_goal_choices() -> Vec<u8> {
    (WeeklyGoal::MIN..=WeeklyGoal::MAX).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sets_or_clears_the_goal() {
        let mut db = Db::default();
        set_weekly_goal(&mut db, Some(4)).unwrap();
        assert_eq!(db.preferences().weekly_goal().map(WeeklyGoal::get), Some(4));
        assert_eq!(
            set_weekly_goal(&mut db, Some(15)),
            Err(WeeklyGoalError::OutOfRange)
        );
        assert_eq!(db.preferences().weekly_goal().map(WeeklyGoal::get), Some(4));
        set_weekly_goal(&mut db, None).unwrap();
        assert_eq!(db.preferences().weekly_goal(), None);
        assert_eq!(weekly_goal_choices().len(), 14);
    }
}
