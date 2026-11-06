use features::run_session::{current_session, rest_started_at};
use features::shared::format::format_clock;
use leptos::prelude::*;

use crate::navigation::{Navigation, Screen};
use crate::store::Store;
use crate::ui::ticking_now;

// The session in progress stays one tap away, with the live rest timer.
#[component]
pub fn SessionMiniPlayer(store: Store, navigation: Navigation) -> impl IntoView {
    let session = Memo::new(move |_| store.with(current_session));
    move || {
        session.get().map(|session| {
            let session_id = session.id;
            let now = ticking_now();
            let rest = Memo::new(move |_| store.with(|db| rest_started_at(db, session_id)));
            let timer = move || match rest.get() {
                Some(rest) => format!(
                    "Repos {}",
                    format_clock((now.get() - rest.since).num_seconds())
                ),
                None => format!(
                    "Séance {}",
                    format_clock((now.get() - session.started_at).num_seconds())
                ),
            };
            view! {
                <button
                    type="button"
                    class="mini-player"
                    on:click=move |_| navigation.go_to(Screen::RunSession(session_id))
                >
                    <span class="mini-player__text">
                        <span class="eyebrow mini-player__live">"Séance en cours"</span>
                        <span class="mini-player__name">{session.name}</span>
                    </span>
                    <span class="mini-player__timer">{timer}</span>
                </button>
            }
        })
    }
}
