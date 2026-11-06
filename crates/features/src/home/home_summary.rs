use chrono::NaiveDate;
use domain::{Db, NonEmptyText, Session, SessionId, SessionStatus};

use crate::shared::format::{format_long_date, format_set_count, format_short_date, format_time};

const UPCOMING_LIMIT: usize = 3;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HomeSummary {
    pub today: String,
    pub greeting: String,
    pub first_name: Option<String>,
    pub needs_body_weight: bool,
    pub in_progress: Option<SessionCard>,
    pub upcoming: Vec<SessionCard>,
    pub has_done_sessions: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionCard {
    pub id: SessionId,
    pub name: String,
    pub detail: String,
}

pub fn home_summary(db: &Db, today: NaiveDate) -> HomeSummary {
    let in_progress = db.session_in_progress().map(|session| SessionCard {
        id: session.id(),
        name: session.name().to_string(),
        detail: in_progress_detail(session),
    });
    let mut planned: Vec<&Session> = db
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
    let first_name = db.preferences().first_name().map(NonEmptyText::to_string);
    HomeSummary {
        today: format_long_date(today),
        greeting: match &first_name {
            Some(name) => format!("Bonjour {name}"),
            None => "Bonjour".to_owned(),
        },
        first_name,
        needs_body_weight: db.preferences().body_weights().latest().is_none(),
        in_progress,
        upcoming: planned
            .into_iter()
            .take(UPCOMING_LIMIT)
            .map(|session| SessionCard {
                id: session.id(),
                name: session.name().to_string(),
                detail: planned_detail(session, today),
            })
            .collect(),
        has_done_sessions: db
            .sessions()
            .iter()
            .any(|session| matches!(session.status(), SessionStatus::Done { .. })),
    }
}

fn in_progress_detail(session: &Session) -> String {
    let started = session
        .status()
        .started_at()
        .map(|started_at| format!("Commencée à {} · ", format_time(started_at)))
        .unwrap_or_default();
    format!("{started}{}", set_count_text(session))
}

fn planned_detail(session: &Session, today: NaiveDate) -> String {
    let when = match session.planned_for() {
        None => "Sans date".to_owned(),
        Some(date) if date < today => format!("{} · en retard", format_short_date(date)),
        Some(date) if date == today => "Aujourd'hui".to_owned(),
        Some(date) => format_short_date(date),
    };
    let exercise_count = session.entries().len();
    match exercise_count {
        0 => when,
        1 => format!("{when} · 1 exercice"),
        count => format!("{when} · {count} exercices"),
    }
}

fn set_count_text(session: &Session) -> String {
    format_set_count(
        session
            .entries()
            .iter()
            .map(|entry| entry.sets().len())
            .sum(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::{at, bench, date, done_session, pull_up, text, weight_reps};

    #[test]
    fn summarizes_sessions_for_the_home_screen() {
        let mut db = Db::default();
        done_session(&mut db, 27, &[(bench(), vec![weight_reps(80, 8)])]);
        db.plan_session(text("Sans date"), None, &[]).unwrap();
        db.plan_session(text("Dos"), Some(date(31)), &[bench(), pull_up()])
            .unwrap();
        db.plan_session(text("Bras"), Some(date(29)), &[bench()])
            .unwrap();
        db.plan_session(text("Jambes"), Some(date(30)), &[])
            .unwrap();
        let current = db
            .start_new_session(text("Cardio"), &[], at(30, 18, 2))
            .unwrap();

        let summary = home_summary(&db, date(30));
        assert_eq!(summary.today, "vendredi 30 octobre");
        assert_eq!(
            summary.in_progress,
            Some(SessionCard {
                id: current,
                name: "Cardio".to_owned(),
                detail: "Commencée à 18:02 · aucune série".to_owned(),
            })
        );
        let upcoming: Vec<(&str, &str)> = summary
            .upcoming
            .iter()
            .map(|card| (card.name.as_str(), card.detail.as_str()))
            .collect();
        assert_eq!(
            upcoming,
            [
                ("Bras", "jeu. 29 oct. · en retard · 1 exercice"),
                ("Jambes", "Aujourd'hui"),
                ("Dos", "sam. 31 oct. · 2 exercices"),
            ]
        );
        assert!(summary.has_done_sessions);
    }

    #[test]
    fn greets_by_first_name_once_known() {
        let mut db = Db::default();
        db.set_first_name(Some(text("Thomas")));
        let summary = home_summary(&db, date(30));
        assert_eq!(summary.greeting, "Bonjour Thomas");
        assert_eq!(summary.first_name.as_deref(), Some("Thomas"));
    }

    #[test]
    fn empty_db_has_nothing_to_show() {
        let summary = home_summary(&Db::default(), date(30));
        assert_eq!(summary.greeting, "Bonjour");
        assert_eq!(summary.first_name, None);
        assert!(summary.needs_body_weight);
        assert_eq!(summary.in_progress, None);
        assert!(summary.upcoming.is_empty());
        assert!(!summary.has_done_sessions);
    }
}
