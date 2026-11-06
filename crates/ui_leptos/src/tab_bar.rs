use leptos::prelude::*;

use crate::navigation::{Navigation, Screen};

const TABS: [(Screen, &str, &str); 4] = [
    (Screen::Home, "Accueil", "⌂"),
    (Screen::History(None), "Historique", "☰"),
    (Screen::Stats, "Stats", "◔"),
    (Screen::Catalog, "Exercices", "◎"),
];

#[component]
pub fn TabBar(navigation: Navigation) -> impl IntoView {
    view! {
        <nav class="tab-bar" aria-label="Navigation principale">
            {TABS
                .into_iter()
                .map(|(screen, label, icon)| {
                    let target = screen.clone();
                    let is_current = Signal::derive(move || navigation.current().belongs_to_same_tab(&screen));
                    view! {
                        <button
                            type="button"
                            class="tab-bar__tab"
                            class:tab-bar__tab--current=is_current
                            aria-current=move || is_current.get().then_some("page")
                            on:click=move |_| navigation.reset_to(target.clone())
                        >
                            <span class="tab-bar__icon" aria-hidden="true">{icon}</span>
                            <span>{label}</span>
                        </button>
                    }
                })
                .collect_view()}
        </nav>
    }
}
