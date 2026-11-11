use std::convert::Infallible;

use domain::{Db, NonEmptyText};

// A blank name clears it, so the home screen asks again.
pub fn set_first_name(db: &mut Db, typed: &str) -> Result<(), Infallible> {
    db.set_first_name(NonEmptyText::optional(typed));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn first_name(db: &Db) -> Option<&str> {
        db.preferences().first_name().map(NonEmptyText::as_str)
    }

    #[test]
    fn sets_a_trimmed_first_name() {
        let mut db = Db::default();
        set_first_name(&mut db, "  Thomas ").unwrap();
        assert_eq!(first_name(&db), Some("Thomas"));
    }

    #[test]
    fn blank_text_clears_the_first_name() {
        let mut db = Db::default();
        set_first_name(&mut db, "Thomas").unwrap();
        set_first_name(&mut db, "   ").unwrap();
        assert_eq!(first_name(&db), None);
    }
}
