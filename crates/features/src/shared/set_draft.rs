use std::num::NonZeroU32;

use domain::{
    CardioMeasure, Kcal, Load, MeasureKind, Meters, Permille, Seconds, SetMeasure, Weight,
};

use crate::shared::decimal::{format_scaled, parse_scaled};
use crate::shared::format::format_clock;

// Values in base units: grams, reps, seconds, meters, kcal, level, per mille.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldKind {
    Weight,
    Reps,
    Load,
    Duration,
    CardioDuration,
    Distance,
    Calories,
    Level,
    Incline,
}

impl FieldKind {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Weight => "Poids",
            Self::Reps => "Reps",
            Self::Load => "Lest",
            Self::Duration | Self::CardioDuration => "Durée",
            Self::Distance => "Distance",
            Self::Calories => "Calories",
            Self::Level => "Niveau",
            Self::Incline => "Inclinaison",
        }
    }

    pub const fn caption(self) -> &'static str {
        match self {
            Self::Weight => "Poids (kg)",
            Self::Reps => "Reps",
            Self::Load => "Lest (kg)",
            Self::Duration => "Durée (s)",
            Self::CardioDuration => "Durée (min)",
            Self::Distance => "Distance (km)",
            Self::Calories => "Calories (kcal)",
            Self::Level => "Niveau",
            Self::Incline => "Inclinaison (%)",
        }
    }

    pub const fn unit(self) -> &'static str {
        match self {
            Self::Weight | Self::Load => "kg",
            Self::Reps => "reps",
            Self::Duration => "s",
            Self::CardioDuration => "min",
            Self::Distance => "km",
            Self::Calories => "kcal",
            Self::Level => "niv.",
            Self::Incline => "%",
        }
    }

    // Step of the − / + buttons; other fields are typed on the keypad.
    pub const fn step(self) -> Option<u32> {
        match self {
            Self::Weight | Self::Load => Some(500),
            Self::Reps => Some(1),
            Self::Duration => Some(5),
            Self::CardioDuration => Some(60),
            Self::Distance | Self::Calories | Self::Level | Self::Incline => None,
        }
    }

    pub const fn is_required(self) -> bool {
        matches!(
            self,
            Self::Weight | Self::Reps | Self::Duration | Self::CardioDuration
        )
    }

    pub const fn keypad_decimals(self) -> u32 {
        match self {
            Self::Weight | Self::Load | Self::Distance => 3,
            Self::Incline => 1,
            Self::Reps | Self::Duration | Self::CardioDuration | Self::Calories | Self::Level => 0,
        }
    }

    // Weight shortcuts of the custom keypad, as (label, delta in grams).
    pub const fn keypad_shortcuts(self) -> &'static [(&'static str, i64)] {
        match self {
            Self::Weight | Self::Load => &[
                ("−2,5", -2_500),
                ("−0,5", -500),
                ("+0,5", 500),
                ("+2,5", 2_500),
            ],
            Self::Reps
            | Self::Duration
            | Self::CardioDuration
            | Self::Distance
            | Self::Calories
            | Self::Level
            | Self::Incline => &[],
        }
    }

    pub fn format_value(self, value: u32) -> String {
        match self {
            Self::CardioDuration if value.is_multiple_of(60) => (value / 60).to_string(),
            Self::CardioDuration => format_clock(i64::from(value)),
            _ => format_scaled(value, self.keypad_decimals()),
        }
    }

    pub fn keypad_text(self, value: u32) -> String {
        match self {
            Self::CardioDuration => (value / 60).to_string(),
            _ => self.format_value(value),
        }
    }

    pub fn parse_keypad_text(self, text: &str) -> Option<u32> {
        let parsed = parse_scaled(text, self.keypad_decimals())?;
        match self {
            Self::CardioDuration => parsed.checked_mul(60),
            _ => Some(parsed),
        }
    }

    const fn for_measure(measure: MeasureKind) -> &'static [Self] {
        match measure {
            MeasureKind::WeightReps => &[Self::Weight, Self::Reps],
            MeasureKind::Reps => &[Self::Reps, Self::Load],
            MeasureKind::Duration => &[Self::Duration, Self::Load],
            MeasureKind::Cardio => &[
                Self::CardioDuration,
                Self::Distance,
                Self::Calories,
                Self::Level,
                Self::Incline,
            ],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DraftField {
    pub kind: FieldKind,
    pub value: Option<u32>,
}

impl DraftField {
    pub fn display_value(&self) -> String {
        self.value
            .map(|value| self.kind.format_value(value))
            .unwrap_or_else(|| "–".to_owned())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DraftError {
    Missing(FieldKind),
    Zero(FieldKind),
}

pub fn draft_error_message(error: DraftError) -> String {
    match error {
        DraftError::Missing(kind) => format!("Valeur manquante : {}.", kind.label()),
        DraftError::Zero(kind) => format!("{} doit être supérieur à zéro.", kind.label()),
    }
}

// Input of the current set, never persisted.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SetDraft {
    measure: MeasureKind,
    fields: Vec<DraftField>,
}

impl SetDraft {
    pub fn empty(measure: MeasureKind) -> Self {
        Self {
            measure,
            fields: FieldKind::for_measure(measure)
                .iter()
                .map(|&kind| DraftField { kind, value: None })
                .collect(),
        }
    }

    pub fn prefilled(measure: MeasureKind, source: Option<&SetMeasure>) -> Self {
        let mut draft = Self::empty(measure);
        if let Some(source) = source.filter(|source| source.kind() == measure) {
            for field in &mut draft.fields {
                field.value = value_in(source, field.kind);
            }
        }
        draft
    }

    pub fn fields(&self) -> &[DraftField] {
        &self.fields
    }

    pub fn increment(&mut self, index: usize) {
        if let Some(field) = self.fields.get_mut(index)
            && let Some(step) = field.kind.step()
        {
            field.value = Some(field.value.map_or(step, |value| value.saturating_add(step)));
        }
    }

    pub fn decrement(&mut self, index: usize) {
        if let Some(field) = self.fields.get_mut(index)
            && let Some(step) = field.kind.step()
        {
            let decremented = field.value.map(|value| value.saturating_sub(step));
            field.value = decremented.filter(|value| *value > 0 || field.kind.is_required());
        }
    }

    // After the weight, the keypad moves on to the reps.
    pub fn field_to_type_after(&self, index: usize) -> Option<usize> {
        let typed = self.fields.get(index)?;
        let next_index = index + 1;
        (typed.kind == FieldKind::Weight && next_index < self.fields.len()).then_some(next_index)
    }

    // The set timer fills the duration field of timed exercises.
    pub fn timed_field_index(&self) -> Option<usize> {
        self.fields
            .iter()
            .position(|field| matches!(field.kind, FieldKind::Duration | FieldKind::CardioDuration))
    }

    pub fn set_value(&mut self, index: usize, value: Option<u32>) {
        if let Some(field) = self.fields.get_mut(index) {
            field.value = value;
        }
    }

    pub fn to_measure(&self) -> Result<SetMeasure, DraftError> {
        Ok(match self.measure {
            MeasureKind::WeightReps => SetMeasure::WeightReps {
                weight: Weight::from_grams(self.required(FieldKind::Weight)?),
                reps: self.non_zero(FieldKind::Reps)?,
            },
            MeasureKind::Reps => SetMeasure::Reps {
                reps: self.non_zero(FieldKind::Reps)?,
                load: self.load(),
            },
            MeasureKind::Duration => SetMeasure::Duration {
                duration: self.seconds(FieldKind::Duration)?,
                load: self.load(),
            },
            MeasureKind::Cardio => SetMeasure::Cardio(CardioMeasure::new(
                self.seconds(FieldKind::CardioDuration)?,
                self.optional(FieldKind::Distance)
                    .and_then(|meters| Meters::new(meters).ok()),
                self.optional(FieldKind::Calories).map(Kcal::new),
                self.optional(FieldKind::Level),
                self.optional(FieldKind::Incline).map(Permille::new),
            )),
        })
    }

    fn optional(&self, kind: FieldKind) -> Option<u32> {
        self.fields
            .iter()
            .find(|field| field.kind == kind)
            .and_then(|field| field.value)
    }

    fn required(&self, kind: FieldKind) -> Result<u32, DraftError> {
        self.optional(kind).ok_or(DraftError::Missing(kind))
    }

    fn non_zero(&self, kind: FieldKind) -> Result<NonZeroU32, DraftError> {
        NonZeroU32::new(self.required(kind)?).ok_or(DraftError::Zero(kind))
    }

    fn seconds(&self, kind: FieldKind) -> Result<Seconds, DraftError> {
        Seconds::new(self.required(kind)?).map_err(|_| DraftError::Zero(kind))
    }

    fn load(&self) -> Option<Load> {
        self.optional(FieldKind::Load)
            .and_then(|grams| Load::from_grams(grams).ok())
    }
}

fn value_in(measure: &SetMeasure, kind: FieldKind) -> Option<u32> {
    match (measure, kind) {
        (SetMeasure::WeightReps { weight, .. }, FieldKind::Weight) => Some(weight.grams()),
        (SetMeasure::WeightReps { reps, .. } | SetMeasure::Reps { reps, .. }, FieldKind::Reps) => {
            Some(reps.get())
        }
        (SetMeasure::Reps { load, .. } | SetMeasure::Duration { load, .. }, FieldKind::Load) => {
            load.map(Load::grams)
        }
        (SetMeasure::Duration { duration, .. }, FieldKind::Duration) => Some(duration.get()),
        (SetMeasure::Cardio(cardio), FieldKind::CardioDuration) => Some(cardio.duration().get()),
        (SetMeasure::Cardio(cardio), FieldKind::Distance) => cardio.distance().map(Meters::get),
        (SetMeasure::Cardio(cardio), FieldKind::Calories) => cardio.calories().map(Kcal::get),
        (SetMeasure::Cardio(cardio), FieldKind::Level) => cardio.level(),
        (SetMeasure::Cardio(cardio), FieldKind::Incline) => cardio.incline().map(Permille::get),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::{reps, weight_reps};

    fn values(draft: &SetDraft) -> Vec<Option<u32>> {
        draft.fields().iter().map(|field| field.value).collect()
    }

    #[test]
    fn fields_follow_the_measure_kind() {
        let kinds = |measure| -> Vec<FieldKind> {
            SetDraft::empty(measure)
                .fields()
                .iter()
                .map(|f| f.kind)
                .collect()
        };
        assert_eq!(
            kinds(MeasureKind::WeightReps),
            [FieldKind::Weight, FieldKind::Reps]
        );
        assert_eq!(
            kinds(MeasureKind::Duration),
            [FieldKind::Duration, FieldKind::Load]
        );
        assert_eq!(kinds(MeasureKind::Cardio).len(), 5);
    }

    #[test]
    fn prefills_from_a_set_of_the_same_kind_only() {
        let draft = SetDraft::prefilled(MeasureKind::WeightReps, Some(&weight_reps(80, 8)));
        assert_eq!(values(&draft), [Some(80_000), Some(8)]);
        let mismatch = SetDraft::prefilled(MeasureKind::WeightReps, Some(&reps(10)));
        assert_eq!(values(&mismatch), [None, None]);
    }

    #[test]
    fn steps_follow_the_specification() {
        let mut draft = SetDraft::prefilled(MeasureKind::WeightReps, Some(&weight_reps(80, 8)));
        draft.increment(0);
        draft.decrement(1);
        assert_eq!(values(&draft), [Some(80_500), Some(7)]);
    }

    #[test]
    fn optional_load_disappears_at_zero() {
        let mut draft = SetDraft::empty(MeasureKind::Reps);
        draft.increment(1);
        assert_eq!(values(&draft), [None, Some(500)]);
        draft.decrement(1);
        assert_eq!(values(&draft), [None, None]);
    }

    #[test]
    fn builds_the_measure() {
        let draft = SetDraft::prefilled(MeasureKind::WeightReps, Some(&weight_reps(80, 8)));
        assert_eq!(draft.to_measure(), Ok(weight_reps(80, 8)));

        let mut cardio = SetDraft::empty(MeasureKind::Cardio);
        cardio.set_value(0, Some(1_200));
        cardio.set_value(1, Some(5_000));
        assert_eq!(
            cardio.to_measure(),
            Ok(SetMeasure::Cardio(CardioMeasure::new(
                Seconds::new(1_200).unwrap(),
                Meters::new(5_000).ok(),
                None,
                None,
                None,
            )))
        );
    }

    #[test]
    fn reports_missing_or_zero_values() {
        assert_eq!(
            SetDraft::empty(MeasureKind::WeightReps).to_measure(),
            Err(DraftError::Missing(FieldKind::Weight))
        );
        let mut draft = SetDraft::empty(MeasureKind::Reps);
        draft.set_value(0, Some(0));
        assert_eq!(draft.to_measure(), Err(DraftError::Zero(FieldKind::Reps)));
        assert_eq!(
            draft_error_message(DraftError::Missing(FieldKind::Weight)),
            "Valeur manquante : Poids."
        );
    }

    #[test]
    fn timed_exercises_expose_their_duration_field() {
        assert_eq!(
            SetDraft::empty(MeasureKind::Duration).timed_field_index(),
            Some(0)
        );
        assert_eq!(
            SetDraft::empty(MeasureKind::Cardio).timed_field_index(),
            Some(0)
        );
        assert_eq!(
            SetDraft::empty(MeasureKind::WeightReps).timed_field_index(),
            None
        );
        assert_eq!(FieldKind::Weight.keypad_shortcuts().len(), 4);
        assert!(FieldKind::Reps.keypad_shortcuts().is_empty());
    }

    #[test]
    fn keypad_chains_weight_then_reps_only() {
        let weight_reps_draft = SetDraft::empty(MeasureKind::WeightReps);
        assert_eq!(weight_reps_draft.field_to_type_after(0), Some(1));
        assert_eq!(weight_reps_draft.field_to_type_after(1), None);
        assert_eq!(
            SetDraft::empty(MeasureKind::Reps).field_to_type_after(0),
            None
        );
    }

    #[test]
    fn keypad_text_round_trips_in_usual_units() {
        assert_eq!(FieldKind::Weight.keypad_text(82_500), "82,5");
        assert_eq!(FieldKind::Weight.parse_keypad_text("82,5"), Some(82_500));
        assert_eq!(FieldKind::CardioDuration.keypad_text(1_200), "20");
        assert_eq!(
            FieldKind::CardioDuration.parse_keypad_text("20"),
            Some(1_200)
        );
        assert_eq!(FieldKind::CardioDuration.format_value(1_230), "20:30");
        assert_eq!(FieldKind::Incline.parse_keypad_text("2,5"), Some(25));
        assert_eq!(FieldKind::Distance.format_value(5_200), "5,2");
        assert_eq!(FieldKind::Weight.caption(), "Poids (kg)");
    }
}
