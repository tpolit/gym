use chrono::{DateTime, FixedOffset};

use crate::measure::SetMeasure;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkoutSet {
    measure: SetMeasure,
    warmup: bool,
    // None: set added afterwards, when correcting a session.
    completed_at: Option<DateTime<FixedOffset>>,
}

impl WorkoutSet {
    pub const fn new(
        measure: SetMeasure,
        warmup: bool,
        completed_at: Option<DateTime<FixedOffset>>,
    ) -> Self {
        Self {
            measure,
            warmup,
            completed_at,
        }
    }

    pub const fn measure(&self) -> &SetMeasure {
        &self.measure
    }

    pub const fn is_warmup(&self) -> bool {
        self.warmup
    }

    pub const fn completed_at(&self) -> Option<DateTime<FixedOffset>> {
        self.completed_at
    }

    // The completion time is kept: fixing a value must not distort rest times.
    pub(crate) fn update(&mut self, measure: SetMeasure, warmup: bool) {
        self.measure = measure;
        self.warmup = warmup;
    }
}

#[cfg(test)]
mod tests {
    use std::num::NonZeroU32;

    use chrono::TimeZone;

    use super::*;
    use crate::measure::Weight;

    #[test]
    fn update_keeps_completion_time() {
        let at = FixedOffset::east_opt(3_600)
            .unwrap()
            .with_ymd_and_hms(2026, 10, 28, 18, 5, 0)
            .unwrap();
        let measure = |kg: u32, reps: u32| SetMeasure::WeightReps {
            weight: Weight::from_grams(kg * 1_000),
            reps: NonZeroU32::new(reps).unwrap(),
        };
        let mut set = WorkoutSet::new(measure(80, 8), false, Some(at));
        set.update(measure(60, 10), true);
        assert_eq!(set.measure(), &measure(60, 10));
        assert!(set.is_warmup());
        assert_eq!(set.completed_at(), Some(at));
    }
}
