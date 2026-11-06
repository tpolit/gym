use chrono::NaiveDate;
use domain::{Db, SessionId, SessionStatus};

use crate::shared::exercise_name::exercise_name;
use crate::shared::format::format_short_date;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlannedSessionItem {
    pub id: SessionId,
    pub name: String,
    pub date_text: String,
    pub is_overdue: bool,
    pub exercises_text: String,
}

// Sorted by date, undated sessions last; overdue sessions are flagged, never removed.
pub fn planned_sessions(db: &Db, today: NaiveDate) -> Vec<PlannedSessionItem> {
    let mut planned: Vec<_> = db
        .sessions()
        .iter()
        .filter(|session| matches!(session.status(), SessionStatus::Planned))
        .collect();
    planned.sort_by_key(|session| {
        (
            session.planned_for().is_none(),
            session.planned_for(),
            session.id(),
        )
    });
    planned
        .into_iter()
        .map(|session| {
            let names: Vec<String> = session
                .entries()
                .iter()
                .map(|entry| exercise_name(db, entry.exercise_id()))
                .collect();
            PlannedSessionItem {
                id: session.id(),
                name: session.name().to_string(),
                date_text: planned_date_text(session.planned_for(), today),
                is_overdue: session.planned_for().is_some_and(|date| date < today),
                exercises_text: if names.is_empty() {
                    "Aucun exercice".to_owned()
                } else {
                    names.join(" · ")
                },
            }
        })
        .collect()
}

fn planned_date_text(planned_for: Option<NaiveDate>, today: NaiveDate) -> String {
    match planned_for {
        None => "Sans date".to_owned(),
        Some(date) if date < today => format!("{} · en retard", format_short_date(date)),
        Some(date) if date == today => "Aujourd'hui".to_owned(),
        Some(date) if date.pred_opt() == Some(today) => "Demain".to_owned(),
        Some(date) => format_short_date(date),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::{bench, date, pull_up, text};

    #[test]
    fn sorts_by_date_with_undated_last_and_flags_overdue() {
        let mut db = Db::default();
        db.plan_session(text("Sans date"), None, &[]).unwrap();
        db.plan_session(text("Dos"), Some(date(31)), &[bench(), pull_up()])
            .unwrap();
        db.plan_session(text("Bras"), Some(date(28)), &[]).unwrap();
        db.plan_session(text("Jambes"), Some(date(30)), &[])
            .unwrap();

        let items = planned_sessions(&db, date(30));
        let names: Vec<&str> = items.iter().map(|item| item.name.as_str()).collect();
        assert_eq!(names, ["Bras", "Jambes", "Dos", "Sans date"]);
        assert_eq!(items[0].date_text, "mer. 28 oct. · en retard");
        assert!(items[0].is_overdue);
        assert_eq!(items[1].date_text, "Aujourd'hui");
        assert_eq!(items[2].date_text, "Demain");
        assert_eq!(
            items[2].exercises_text,
            "Développé couché barre · Tractions"
        );
        assert_eq!(items[3].date_text, "Sans date");
        assert_eq!(items[3].exercises_text, "Aucun exercice");
    }
}
