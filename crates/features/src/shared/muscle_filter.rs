use domain::MuscleGroup;

use crate::shared::labels::muscle_group_label;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MuscleChip {
    pub group: MuscleGroup,
    pub label: &'static str,
}

pub fn muscle_chips() -> Vec<MuscleChip> {
    MuscleGroup::ALL
        .into_iter()
        .map(|group| MuscleChip {
            group,
            label: muscle_group_label(group),
        })
        .collect()
}

// A single muscle at a time: tapping the selected one clears the filter, another one replaces it.
pub fn toggle_muscle(selected: Option<MuscleGroup>, tapped: MuscleGroup) -> Option<MuscleGroup> {
    (selected != Some(tapped)).then_some(tapped)
}

pub fn matches_muscle(filter: Option<MuscleGroup>, group: MuscleGroup) -> bool {
    filter.is_none_or(|wanted| wanted == group)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn offers_every_muscle_group_in_order_with_french_labels() {
        let chips = muscle_chips();
        assert_eq!(
            chips.iter().map(|chip| chip.group).collect::<Vec<_>>(),
            MuscleGroup::ALL
        );
        assert_eq!(
            chips[0],
            MuscleChip {
                group: MuscleGroup::Chest,
                label: "Pectoraux"
            }
        );
    }

    #[test]
    fn tapping_selects_replaces_or_clears() {
        use MuscleGroup::{Back, Chest};
        assert_eq!(toggle_muscle(None, Chest), Some(Chest));
        assert_eq!(toggle_muscle(Some(Chest), Back), Some(Back));
        assert_eq!(toggle_muscle(Some(Chest), Chest), None);
    }

    #[test]
    fn no_filter_matches_every_group() {
        assert!(matches_muscle(None, MuscleGroup::Calves));
        assert!(matches_muscle(
            Some(MuscleGroup::Calves),
            MuscleGroup::Calves
        ));
        assert!(!matches_muscle(
            Some(MuscleGroup::Chest),
            MuscleGroup::Calves
        ));
    }
}
