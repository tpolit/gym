use chrono::{DateTime, Datelike, FixedOffset, NaiveDate, Timelike};
use domain::{CardioMeasure, Load, SetMeasure};

use crate::shared::decimal::format_scaled;

const SHORT_WEEKDAYS: [&str; 7] = ["lun.", "mar.", "mer.", "jeu.", "ven.", "sam.", "dim."];
const LONG_WEEKDAYS: [&str; 7] = [
    "lundi", "mardi", "mercredi", "jeudi", "vendredi", "samedi", "dimanche",
];
const SHORT_MONTHS: [&str; 12] = [
    "janv.", "févr.", "mars", "avr.", "mai", "juin", "juil.", "août", "sept.", "oct.", "nov.",
    "déc.",
];
const LONG_MONTHS: [&str; 12] = [
    "janvier",
    "février",
    "mars",
    "avril",
    "mai",
    "juin",
    "juillet",
    "août",
    "septembre",
    "octobre",
    "novembre",
    "décembre",
];
const NARROW_NO_BREAK_SPACE: char = '\u{202f}';

pub fn format_kg(grams: u32) -> String {
    format_scaled(grams, 3)
}

pub fn format_integer(value: u64) -> String {
    let digits = value.to_string();
    let mut grouped = String::new();
    for (position, digit) in digits.chars().enumerate() {
        if position > 0 && (digits.len() - position).is_multiple_of(3) {
            grouped.push(NARROW_NO_BREAK_SPACE);
        }
        grouped.push(digit);
    }
    grouped
}

pub fn format_clock(seconds: i64) -> String {
    let seconds = seconds.max(0);
    let (hours, minutes, seconds) = (seconds / 3_600, seconds / 60 % 60, seconds % 60);
    if hours > 0 {
        format!("{hours}:{minutes:02}:{seconds:02}")
    } else {
        format!("{minutes}:{seconds:02}")
    }
}

pub fn format_long_duration(seconds: i64) -> String {
    let total_minutes = seconds.max(0) / 60;
    let (hours, minutes) = (total_minutes / 60, total_minutes % 60);
    if hours > 0 {
        format!("{hours} h {minutes:02}")
    } else {
        format!("{minutes} min")
    }
}

pub fn format_short_duration(seconds: u32) -> String {
    if seconds < 120 {
        return format!("{seconds} s");
    }
    match seconds % 60 {
        0 => format!("{} min", seconds / 60),
        remainder => format!("{} min {remainder:02}", seconds / 60),
    }
}

pub fn format_short_date(date: NaiveDate) -> String {
    format!(
        "{} {} {}",
        SHORT_WEEKDAYS[date.weekday().num_days_from_monday() as usize],
        date.day(),
        SHORT_MONTHS[date.month0() as usize]
    )
}

pub fn format_long_date(date: NaiveDate) -> String {
    format!(
        "{} {} {}",
        LONG_WEEKDAYS[date.weekday().num_days_from_monday() as usize],
        date.day(),
        LONG_MONTHS[date.month0() as usize]
    )
}

pub fn format_set_count(count: usize) -> String {
    match count {
        0 => "aucune série".to_owned(),
        1 => "1 série".to_owned(),
        count => format!("{count} séries"),
    }
}

pub fn format_week_count(done: usize, goal: Option<u8>) -> String {
    let unit = |count: usize| if count > 1 { "séances" } else { "séance" };
    match goal {
        Some(goal) => format!("{done} / {goal} {} cette semaine", unit(usize::from(goal))),
        None => format!("{done} {} cette semaine", unit(done)),
    }
}

pub fn format_time(time: DateTime<FixedOffset>) -> String {
    format!("{}:{:02}", time.hour(), time.minute())
}

pub fn format_set_measure(measure: &SetMeasure) -> String {
    match measure {
        SetMeasure::WeightReps { weight, reps } => {
            format!("{} × {reps}", format_kg(weight.grams()))
        }
        SetMeasure::Reps { reps, load } => {
            let unit = if reps.get() == 1 { "rep" } else { "reps" };
            format!("{reps} {unit}{}", format_load(*load))
        }
        SetMeasure::Duration { duration, load } => {
            format!(
                "{}{}",
                format_short_duration(duration.get()),
                format_load(*load)
            )
        }
        SetMeasure::Cardio(cardio) => format_cardio(cardio),
    }
}

fn format_load(load: Option<Load>) -> String {
    load.map(|load| format!(" +{} kg", format_kg(load.grams())))
        .unwrap_or_default()
}

fn format_cardio(cardio: &CardioMeasure) -> String {
    let distance = cardio.distance().map(|distance| match distance.get() {
        meters @ ..1_000 => format!("{meters} m"),
        meters => format!("{} km", format_scaled(meters, 3)),
    });
    let calories = cardio.calories().map(|kcal| format!("{} kcal", kcal.get()));
    let level = cardio.level().map(|level| format!("niv. {level}"));
    let incline = cardio
        .incline()
        .map(|incline| format!("{} %", format_scaled(incline.get(), 1)));
    std::iter::once(Some(format_short_duration(cardio.duration().get())))
        .chain([distance, calories, level, incline])
        .flatten()
        .collect::<Vec<_>>()
        .join(" · ")
}

#[cfg(test)]
mod tests {
    use std::num::NonZeroU32;

    use chrono::TimeZone;
    use domain::{Kcal, Meters, Permille, Seconds, Weight};

    use super::*;

    fn reps(value: u32) -> NonZeroU32 {
        NonZeroU32::new(value).unwrap()
    }

    #[test]
    fn formats_weights_and_integers() {
        assert_eq!(format_kg(82_500), "82,5");
        assert_eq!(format_integer(4_820), "4\u{202f}820");
        assert_eq!(format_integer(1_234_567), "1\u{202f}234\u{202f}567");
        assert_eq!(format_integer(820), "820");
    }

    #[test]
    fn formats_durations() {
        assert_eq!(format_clock(84), "1:24");
        assert_eq!(format_clock(5), "0:05");
        assert_eq!(format_clock(3_723), "1:02:03");
        assert_eq!(format_clock(-3), "0:00");
        assert_eq!(format_long_duration(4_080), "1 h 08");
        assert_eq!(format_long_duration(2_700), "45 min");
        assert_eq!(format_short_duration(60), "60 s");
        assert_eq!(format_short_duration(150), "2 min 30");
        assert_eq!(format_short_duration(1_200), "20 min");
    }

    #[test]
    fn formats_week_counts() {
        assert_eq!(format_week_count(0, None), "0 séance cette semaine");
        assert_eq!(format_week_count(2, None), "2 séances cette semaine");
        assert_eq!(format_week_count(2, Some(4)), "2 / 4 séances cette semaine");
        assert_eq!(format_week_count(1, Some(1)), "1 / 1 séance cette semaine");
    }

    #[test]
    fn formats_set_counts() {
        assert_eq!(format_set_count(0), "aucune série");
        assert_eq!(format_set_count(1), "1 série");
        assert_eq!(format_set_count(12), "12 séries");
    }

    #[test]
    fn formats_dates_in_french() {
        let date = NaiveDate::from_ymd_opt(2026, 10, 28).unwrap();
        assert_eq!(format_short_date(date), "mer. 28 oct.");
        assert_eq!(format_long_date(date), "mercredi 28 octobre");
        let time = FixedOffset::east_opt(3_600)
            .unwrap()
            .with_ymd_and_hms(2026, 10, 28, 8, 5, 0)
            .unwrap();
        assert_eq!(format_time(time), "8:05");
    }

    #[test]
    fn formats_each_kind_of_set() {
        let load = Load::from_grams(10_000).ok();
        let cases = [
            (
                SetMeasure::WeightReps {
                    weight: Weight::from_grams(82_500),
                    reps: reps(8),
                },
                "82,5 × 8",
            ),
            (
                SetMeasure::Reps {
                    reps: reps(12),
                    load,
                },
                "12 reps +10 kg",
            ),
            (
                SetMeasure::Reps {
                    reps: reps(1),
                    load: None,
                },
                "1 rep",
            ),
            (
                SetMeasure::Duration {
                    duration: Seconds::new(60).unwrap(),
                    load: None,
                },
                "60 s",
            ),
            (
                SetMeasure::Cardio(CardioMeasure::new(
                    Seconds::new(1_200).unwrap(),
                    Meters::new(5_000).ok(),
                    None,
                    None,
                    None,
                )),
                "20 min · 5 km",
            ),
            (
                SetMeasure::Cardio(CardioMeasure::new(
                    Seconds::new(900).unwrap(),
                    Meters::new(800).ok(),
                    Some(Kcal::new(150)),
                    Some(6),
                    Some(Permille::new(25)),
                )),
                "15 min · 800 m · 150 kcal · niv. 6 · 2,5 %",
            ),
        ];
        for (measure, expected) in cases {
            assert_eq!(format_set_measure(&measure), expected);
        }
    }
}
