use std::convert::Infallible;

use domain::Db;

// Second step, after confirmation: full replacement, no merge.
pub fn replace_all_data(db: &mut Db, imported: Db) -> Result<(), Infallible> {
    *db = imported;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::{bench, done_session, weight_reps};

    #[test]
    fn replaces_everything() {
        let mut current = Db::default();
        done_session(&mut current, 20, &[(bench(), vec![weight_reps(80, 8)])]);
        replace_all_data(&mut current, Db::default()).unwrap();
        assert_eq!(current, Db::default());
    }
}
