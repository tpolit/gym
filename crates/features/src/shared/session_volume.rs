use domain::volume::session_volume_grams;
use domain::{Db, Session};

use crate::shared::format::format_integer;

pub fn format_volume(grams: u64) -> String {
    format!("{} kg", format_integer((grams + 500) / 1_000))
}

pub fn session_volume_text(db: &Db, session: &Session) -> Option<String> {
    session_volume_grams(db, session).map(format_volume)
}

#[cfg(test)]
mod tests {
    use domain::{BodyWeight, EntryId};

    use super::*;
    use crate::test_support::{bench, date, done_session, pull_up, reps, weight_reps};

    #[test]
    fn rounds_to_the_nearest_kilogram() {
        assert_eq!(format_volume(4_820_400), "4\u{202f}820 kg");
        assert_eq!(format_volume(499), "0 kg");
        assert_eq!(format_volume(1_500), "2 kg");
    }

    #[test]
    fn sums_weighted_sets_without_warm_ups() {
        let mut db = Db::default();
        let id = done_session(
            &mut db,
            28,
            &[
                (bench(), vec![weight_reps(60, 10), weight_reps(80, 8)]),
                (pull_up(), vec![reps(10)]),
            ],
        );
        db.update_set(id, EntryId::new(1), 0, weight_reps(60, 10), true)
            .unwrap();
        let text = |db: &Db| session_volume_text(db, db.session(id).unwrap());
        assert_eq!(text(&db).as_deref(), Some("640 kg"));
        db.record_body_weight(date(1), BodyWeight::from_grams(80_000).unwrap());
        assert_eq!(text(&db).as_deref(), Some("1\u{202f}440 kg"));
    }
}
