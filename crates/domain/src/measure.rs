use std::num::NonZeroU32;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MeasureKind {
    WeightReps,
    Reps,
    Duration,
    Cardio,
}

impl MeasureKind {
    pub const ALL: [Self; 4] = [Self::WeightReps, Self::Reps, Self::Duration, Self::Cardio];
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValueError {
    Zero,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Weight(u32);

impl Weight {
    pub const fn from_grams(grams: u32) -> Self {
        Self(grams)
    }

    pub const fn grams(self) -> u32 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Load(NonZeroU32);

impl Load {
    pub fn from_grams(grams: u32) -> Result<Self, ValueError> {
        NonZeroU32::new(grams).map(Self).ok_or(ValueError::Zero)
    }

    pub const fn grams(self) -> u32 {
        self.0.get()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Seconds(NonZeroU32);

impl Seconds {
    pub fn new(seconds: u32) -> Result<Self, ValueError> {
        NonZeroU32::new(seconds).map(Self).ok_or(ValueError::Zero)
    }

    pub const fn get(self) -> u32 {
        self.0.get()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Meters(NonZeroU32);

impl Meters {
    pub fn new(meters: u32) -> Result<Self, ValueError> {
        NonZeroU32::new(meters).map(Self).ok_or(ValueError::Zero)
    }

    pub const fn get(self) -> u32 {
        self.0.get()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Kcal(u32);

impl Kcal {
    pub const fn new(kcal: u32) -> Self {
        Self(kcal)
    }

    pub const fn get(self) -> u32 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Permille(u32);

impl Permille {
    pub const fn new(permille: u32) -> Self {
        Self(permille)
    }

    pub const fn get(self) -> u32 {
        self.0
    }
}

// Public variant fields are safe: every value is already validated by its type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SetMeasure {
    WeightReps {
        weight: Weight,
        reps: NonZeroU32,
    },
    Reps {
        reps: NonZeroU32,
        load: Option<Load>,
    },
    Duration {
        duration: Seconds,
        load: Option<Load>,
    },
    Cardio(CardioMeasure),
}

impl SetMeasure {
    pub const fn kind(&self) -> MeasureKind {
        match self {
            Self::WeightReps { .. } => MeasureKind::WeightReps,
            Self::Reps { .. } => MeasureKind::Reps,
            Self::Duration { .. } => MeasureKind::Duration,
            Self::Cardio(_) => MeasureKind::Cardio,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CardioMeasure {
    duration: Seconds,
    distance: Option<Meters>,
    calories: Option<Kcal>,
    level: Option<u32>,
    incline: Option<Permille>,
}

impl CardioMeasure {
    pub const fn new(
        duration: Seconds,
        distance: Option<Meters>,
        calories: Option<Kcal>,
        level: Option<u32>,
        incline: Option<Permille>,
    ) -> Self {
        Self {
            duration,
            distance,
            calories,
            level,
            incline,
        }
    }

    pub const fn duration(&self) -> Seconds {
        self.duration
    }

    pub const fn distance(&self) -> Option<Meters> {
        self.distance
    }

    pub const fn calories(&self) -> Option<Kcal> {
        self.calories
    }

    pub const fn level(&self) -> Option<u32> {
        self.level
    }

    pub const fn incline(&self) -> Option<Permille> {
        self.incline
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strictly_positive_values_reject_zero() {
        assert_eq!(Load::from_grams(0), Err(ValueError::Zero));
        assert_eq!(Seconds::new(0), Err(ValueError::Zero));
        assert_eq!(Meters::new(0), Err(ValueError::Zero));
    }

    #[test]
    fn values_keep_their_unit() {
        assert_eq!(Weight::from_grams(0).grams(), 0);
        assert_eq!(Weight::from_grams(82_500).grams(), 82_500);
        assert_eq!(Load::from_grams(10_000).unwrap().grams(), 10_000);
        assert_eq!(Seconds::new(60).unwrap().get(), 60);
        assert_eq!(Meters::new(5_000).unwrap().get(), 5_000);
        assert_eq!(Kcal::new(0).get(), 0);
        assert_eq!(Permille::new(25).get(), 25);
    }

    #[test]
    fn set_measure_reports_its_kind() {
        let reps = NonZeroU32::new(8).unwrap();
        let duration = Seconds::new(60).unwrap();
        let cases = [
            (
                SetMeasure::WeightReps {
                    weight: Weight::from_grams(80_000),
                    reps,
                },
                MeasureKind::WeightReps,
            ),
            (SetMeasure::Reps { reps, load: None }, MeasureKind::Reps),
            (
                SetMeasure::Duration {
                    duration,
                    load: None,
                },
                MeasureKind::Duration,
            ),
            (
                SetMeasure::Cardio(CardioMeasure::new(duration, None, None, None, None)),
                MeasureKind::Cardio,
            ),
        ];
        for (measure, kind) in cases {
            assert_eq!(measure.kind(), kind);
        }
    }

    #[test]
    fn cardio_measure_exposes_optional_fields() {
        let cardio = CardioMeasure::new(
            Seconds::new(1_200).unwrap(),
            Some(Meters::new(5_000).unwrap()),
            None,
            Some(6),
            Some(Permille::new(25)),
        );
        assert_eq!(cardio.duration().get(), 1_200);
        assert_eq!(cardio.distance().map(Meters::get), Some(5_000));
        assert_eq!(cardio.calories(), None);
        assert_eq!(cardio.level(), Some(6));
        assert_eq!(cardio.incline().map(Permille::get), Some(25));
    }
}
