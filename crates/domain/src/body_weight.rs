use std::collections::BTreeMap;

use chrono::NaiveDate;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BodyWeight(u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BodyWeightError {
    OutOfRange,
}

impl BodyWeight {
    pub const MIN_GRAMS: u32 = 20_000;
    pub const MAX_GRAMS: u32 = 300_000;

    pub const fn from_grams(grams: u32) -> Result<Self, BodyWeightError> {
        if grams >= Self::MIN_GRAMS && grams <= Self::MAX_GRAMS {
            Ok(Self(grams))
        } else {
            Err(BodyWeightError::OutOfRange)
        }
    }

    pub const fn grams(self) -> u32 {
        self.0
    }
}

// At most one weight per day, so that a correction on the same day replaces the value.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct BodyWeightLog(BTreeMap<NaiveDate, BodyWeight>);

impl BodyWeightLog {
    pub fn from_entries(entries: impl IntoIterator<Item = (NaiveDate, BodyWeight)>) -> Self {
        Self(entries.into_iter().collect())
    }

    pub fn entries(&self) -> impl DoubleEndedIterator<Item = (NaiveDate, BodyWeight)> + '_ {
        self.0.iter().map(|(date, weight)| (*date, *weight))
    }

    pub fn latest(&self) -> Option<(NaiveDate, BodyWeight)> {
        self.entries().next_back()
    }

    // Before the first recorded weight, that first weight is the closest known value.
    pub fn on(&self, date: NaiveDate) -> Option<BodyWeight> {
        self.0
            .range(..=date)
            .next_back()
            .or_else(|| self.0.iter().next())
            .map(|(_, weight)| *weight)
    }

    pub(crate) fn record(&mut self, date: NaiveDate, weight: BodyWeight) {
        self.0.insert(date, weight);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kg(value: u32) -> BodyWeight {
        BodyWeight::from_grams(value * 1_000).unwrap()
    }

    fn day(day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, 10, day).unwrap()
    }

    #[test]
    fn body_weight_accepts_20_to_300_kg() {
        assert!(BodyWeight::from_grams(20_000).is_ok());
        assert!(BodyWeight::from_grams(300_000).is_ok());
        assert_eq!(
            BodyWeight::from_grams(19_999),
            Err(BodyWeightError::OutOfRange)
        );
        assert_eq!(
            BodyWeight::from_grams(300_001),
            Err(BodyWeightError::OutOfRange)
        );
    }

    #[test]
    fn uses_the_weight_in_effect_on_a_date() {
        let log = BodyWeightLog::from_entries([(day(10), kg(80)), (day(20), kg(78))]);
        assert_eq!(log.on(day(5)), Some(kg(80)));
        assert_eq!(log.on(day(15)), Some(kg(80)));
        assert_eq!(log.on(day(20)), Some(kg(78)));
        assert_eq!(log.on(day(30)), Some(kg(78)));
        assert_eq!(BodyWeightLog::default().on(day(30)), None);
    }

    #[test]
    fn recording_on_the_same_day_replaces_the_weight() {
        let mut log = BodyWeightLog::default();
        log.record(day(10), kg(80));
        log.record(day(10), kg(81));
        log.record(day(12), kg(82));
        assert_eq!(log.entries().count(), 2);
        assert_eq!(log.on(day(11)), Some(kg(81)));
        assert_eq!(log.latest(), Some((day(12), kg(82))));
    }
}
