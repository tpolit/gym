use domain::records::{BrokenRecord, RecordKind};
use domain::{Db, ExerciseId};

use crate::shared::decimal::format_scaled;
use crate::shared::exercise_name::exercise_name;
use crate::shared::format::{format_clock, format_short_duration};
use crate::shared::session_volume::format_volume;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordBadge {
    pub exercise_id: ExerciseId,
    pub kind: RecordKind,
    pub value: u64,
    pub exercise_name: String,
    pub text: String,
}

pub const fn record_label(kind: RecordKind) -> &'static str {
    match kind {
        RecordKind::HeaviestWeight => "Poids max",
        RecordKind::BestOneRepMax => "1RM estimé",
        RecordKind::BestSessionVolume => "Meilleur volume",
        RecordKind::MostReps => "Reps max",
        RecordKind::HeaviestLoad => "Lest max",
        RecordKind::LongestDuration => "Durée max",
        RecordKind::LongestDistance => "Distance max",
        RecordKind::BestPace => "Meilleure allure",
    }
}

pub fn format_record_value(kind: RecordKind, value: u64) -> String {
    let as_u32 = |value: u64| u32::try_from(value).unwrap_or(u32::MAX);
    match kind {
        RecordKind::HeaviestWeight | RecordKind::HeaviestLoad => {
            format!("{} kg", format_scaled(as_u32(value), 3))
        }
        RecordKind::BestOneRepMax => format!("{} kg", format_scaled(as_u32(value / 100 * 100), 3)),
        RecordKind::BestSessionVolume => format_volume(value),
        RecordKind::MostReps => format!("{value} reps"),
        RecordKind::LongestDuration => format_short_duration(as_u32(value)),
        RecordKind::LongestDistance if value < 1_000 => format!("{value} m"),
        RecordKind::LongestDistance => format!("{} km", format_scaled(as_u32(value), 3)),
        RecordKind::BestPace => format!(
            "{} /km",
            format_clock(i64::try_from(value).unwrap_or(i64::MAX))
        ),
    }
}

pub fn record_badges(db: &Db, broken: &[BrokenRecord]) -> Vec<RecordBadge> {
    broken
        .iter()
        .map(|record| RecordBadge {
            exercise_id: record.exercise_id.clone(),
            kind: record.kind,
            value: record.value,
            exercise_name: exercise_name(db, &record.exercise_id),
            text: format!(
                "{} {} (avant {})",
                record_label(record.kind),
                format_record_value(record.kind, record.value),
                format_record_value(record.kind, record.previous)
            ),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::bench;

    #[test]
    fn formats_each_kind_of_record() {
        let cases = [
            (RecordKind::HeaviestWeight, 82_500, "82,5 kg"),
            (RecordKind::BestOneRepMax, 101_333, "101,3 kg"),
            (RecordKind::BestSessionVolume, 4_820_400, "4\u{202f}820 kg"),
            (RecordKind::MostReps, 12, "12 reps"),
            (RecordKind::LongestDuration, 90, "90 s"),
            (RecordKind::LongestDistance, 5_200, "5,2 km"),
            (RecordKind::LongestDistance, 800, "800 m"),
            (RecordKind::BestPace, 245, "4:05 /km"),
        ];
        for (kind, value, expected) in cases {
            assert_eq!(format_record_value(kind, value), expected, "{kind:?}");
        }
    }

    #[test]
    fn badge_mentions_the_previous_value() {
        let broken = [BrokenRecord {
            exercise_id: bench(),
            kind: RecordKind::HeaviestWeight,
            value: 85_000,
            previous: 82_500,
        }];
        let badges = record_badges(&Db::default(), &broken);
        assert_eq!(badges[0].exercise_name, "Développé couché barre");
        assert_eq!(badges[0].text, "Poids max 85 kg (avant 82,5 kg)");
    }
}
