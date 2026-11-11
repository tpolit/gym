#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Confirmation {
    RemoveSet {
        set_text: String,
    },
    RemoveEntry {
        exercise_name: String,
        set_count: usize,
    },
    DeleteSession,
    AbandonSession,
    DeletePlannedSession,
    DeleteCustomExercise {
        name: String,
    },
    ArchiveCustomExercise {
        name: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfirmationText {
    pub title: String,
    pub message: &'static str,
    pub action: &'static str,
}

const PERMANENT: &str = "Cette action est définitive.";

pub fn confirmation_text(confirmation: &Confirmation) -> ConfirmationText {
    let (title, message, action) = match confirmation {
        Confirmation::RemoveSet { set_text } => (
            format!("Supprimer la série « {set_text} » ?"),
            PERMANENT,
            "Supprimer",
        ),
        Confirmation::RemoveEntry {
            exercise_name,
            set_count,
        } => (
            format!("Retirer « {exercise_name} » de la séance ?"),
            match set_count {
                0 => "Vous pourrez l'ajouter à nouveau.",
                1 => "Sa série sera définitivement supprimée.",
                _ => "Ses séries seront définitivement supprimées.",
            },
            "Retirer",
        ),
        Confirmation::DeleteSession => (
            "Supprimer cette séance ?".to_owned(),
            "Ses exercices et ses séries seront définitivement effacés.",
            "Supprimer",
        ),
        Confirmation::AbandonSession => (
            "Abandonner la séance ?".to_owned(),
            "Aucune série n'a été enregistrée : la séance ne sera pas conservée dans l'historique.",
            "Abandonner",
        ),
        Confirmation::DeletePlannedSession => (
            "Supprimer cette séance planifiée ?".to_owned(),
            PERMANENT,
            "Supprimer",
        ),
        Confirmation::DeleteCustomExercise { name } => (
            format!("Supprimer « {name} » ?"),
            "Il n'a jamais été utilisé : il sera définitivement effacé.",
            "Supprimer",
        ),
        Confirmation::ArchiveCustomExercise { name } => (
            format!("Archiver « {name} » ?"),
            "Il ne sera plus proposé dans les séances, mais restera visible dans l'historique.",
            "Archiver",
        ),
    };
    ConfirmationText {
        title,
        message,
        action,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_the_removed_item() {
        let text = confirmation_text(&Confirmation::RemoveSet {
            set_text: "82,5 × 8".to_owned(),
        });
        assert_eq!(text.title, "Supprimer la série « 82,5 × 8 » ?");
        assert_eq!(text.action, "Supprimer");
        let text = confirmation_text(&Confirmation::ArchiveCustomExercise {
            name: "Rowing Technogym".to_owned(),
        });
        assert_eq!(text.title, "Archiver « Rowing Technogym » ?");
        assert_eq!(text.action, "Archiver");
    }

    #[test]
    fn warns_about_the_sets_of_a_removed_exercise() {
        let message = |set_count| {
            confirmation_text(&Confirmation::RemoveEntry {
                exercise_name: "Tractions".to_owned(),
                set_count,
            })
            .message
        };
        assert_eq!(message(0), "Vous pourrez l'ajouter à nouveau.");
        assert_eq!(message(1), "Sa série sera définitivement supprimée.");
        assert_eq!(message(3), "Ses séries seront définitivement supprimées.");
    }

    #[test]
    fn abandoning_uses_its_own_verb() {
        let text = confirmation_text(&Confirmation::AbandonSession);
        assert_eq!(text.title, "Abandonner la séance ?");
        assert_eq!(text.action, "Abandonner");
    }
}
