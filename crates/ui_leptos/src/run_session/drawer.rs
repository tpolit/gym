use domain::{EntryId, SessionId};
use features::run_session::{
    EntryView, SessionView, cancel_session, delete_session, finish_session, move_entry,
    remove_entry, run_session_error_message,
};
use features::shared::confirmation::Confirmation;
use leptos::prelude::*;

use crate::navigation::{Navigation, Screen};
use crate::store::Store;
use crate::ui::{ConfirmPrompt, ConfirmSheet, ReorderButtons, SwipeRow};

// Drawer listing the session exercises, with add, finish and cancel.
#[component]
pub fn SessionDrawer(
    store: Store,
    navigation: Navigation,
    session_id: SessionId,
    session: Memo<Option<SessionView>>,
    current_entry: RwSignal<Option<EntryId>>,
    on_close: Callback<()>,
    on_add: Callback<()>,
) -> impl IntoView {
    let message = RwSignal::new(None::<&'static str>);
    let confirm = ConfirmPrompt::new();
    let has_sets =
        move || session.with(|session| session.as_ref().is_some_and(|session| session.has_sets));

    let finish =
        move |_| match store.execute(|db| finish_session(db, session_id, web_platform::now())) {
            Ok(()) => navigation.replace(Screen::SessionRecap(session_id)),
            Err(error) => message.set(Some(run_session_error_message(&error))),
        };
    let cancel = move |_| {
        confirm.ask(Confirmation::AbandonSession, move || {
            match store.execute(|db| cancel_session(db, session_id)) {
                Ok(_) => navigation.reset_to(Screen::Home),
                Err(error) => message.set(Some(run_session_error_message(&error))),
            }
        });
    };
    let delete = move |_| {
        confirm.ask(Confirmation::DeleteSession, move || {
            match store.execute(|db| delete_session(db, session_id)) {
                Ok(()) => navigation.reset_to(Screen::Home),
                Err(error) => message.set(Some(run_session_error_message(&error))),
            }
        });
    };

    let entry_row = move |entry: EntryView| {
        let entry_id = entry.entry_id;
        let index = entry.index;
        let is_first = index == 0;
        let move_to = move |new_index: usize| {
            let _ = store.execute(|db| move_entry(db, session_id, entry_id, new_index));
        };
        let is_current = move || current_entry.get() == Some(entry_id);
        let status_class = move || match (is_current(), entry.set_count > 0) {
            (true, _) => "status status--current",
            (false, true) => "status status--done",
            (false, false) => "status status--todo",
        };
        let select = Callback::new(move |()| {
            current_entry.set(Some(entry_id));
            on_close.run(());
        });
        let removal = Confirmation::RemoveEntry {
            exercise_name: entry.exercise_name.clone(),
            set_count: entry.set_count,
        };
        let remove = Callback::new(move |()| {
            confirm.ask(removal.clone(), move || {
                let removed = store.execute(|db| remove_entry(db, session_id, entry_id));
                if removed.is_ok() && current_entry.get_untracked() == Some(entry_id) {
                    let first_remaining = session.with_untracked(|session| {
                        session
                            .as_ref()
                            .and_then(|session| session.entries.first().map(|entry| entry.entry_id))
                    });
                    current_entry.set(first_remaining);
                }
            });
        });
        view! {
            <div class="drawer__entry">
                <SwipeRow left_action=Some(("Retirer", remove)) right_action=None on_tap=select class="drawer__swipe">
                    <div class="drawer__select">
                        <span class=status_class>{if entry.set_count > 0 { "✓" } else { "" }}</span>
                        <span class="drawer__entry-text">
                            <span class="drawer__entry-name">{entry.exercise_name}</span>
                            <span class="muted">{entry.set_count_text}</span>
                        </span>
                    </div>
                </SwipeRow>
                <ReorderButtons
                    can_move_up=!is_first
                    can_move_down=!entry.is_last
                    on_move_up=Callback::new(move |()| move_to(index.saturating_sub(1)))
                    on_move_down=Callback::new(move |()| move_to(index + 1))
                />
            </div>
        }
    };

    view! {
        <div class="sheet-backdrop" on:click=move |_| on_close.run(())></div>
        <div class="sheet sheet--tall drawer" role="dialog" aria-label="Exercices de la séance">
            <span class="grabber"></span>
            <p class="eyebrow">"Exercices de la séance"</p>
            <div class="list drawer__list">
                {move || {
                    session
                        .get()
                        .map(|session| session.entries.into_iter().map(entry_row).collect_view())
                }}
            </div>
            <button type="button" class="btn btn--secondary" on:click=move |_| on_add.run(())>
                "+ Ajouter un exercice"
            </button>
            {move || message.get().map(|text| view! { <p class="error" role="alert">{text}</p> })}
            <div class="drawer__actions">
                {move || {
                    if has_sets() {
                        view! {
                            <button type="button" class="btn btn--primary" on:click=finish>
                                "Terminer la séance"
                            </button>
                            <button type="button" class="btn btn--danger-ghost" on:click=delete>
                                "Supprimer la séance"
                            </button>
                        }
                            .into_any()
                    } else {
                        view! {
                            <button type="button" class="btn btn--secondary" on:click=cancel>
                                "Abandonner la séance"
                            </button>
                        }
                            .into_any()
                    }
                }}
            </div>
        </div>
        <ConfirmSheet prompt=confirm />
    }
}
