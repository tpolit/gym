use std::time::Duration;

use chrono::{DateTime, FixedOffset};
use domain::{EntryId, SessionId};
use features::run_session::{
    DoneSetRow, add_set, current_session, last_performance, last_performance_text, new_records,
    planned_set_rows, remove_set, run_session_error_message, session_records,
    set_exercise_settings, update_set,
};
use features::shared::confirmation::Confirmation;
use features::shared::format::format_clock;
use features::shared::records::RecordBadge;
use features::shared::set_draft::{SetDraft, draft_error_message};
use leptos::prelude::*;

use crate::store::Store;
use crate::ui::{ConfirmPrompt, ConfirmSheet, SetDraftEditor, SwipeRow, TextSheet, ticking_now};

const RECORD_BANNER_VISIBLE_FOR: Duration = Duration::from_secs(5);

#[component]
pub fn ExerciseView(
    store: Store,
    session_id: SessionId,
    entry_id: EntryId,
    set_timer_running: RwSignal<bool>,
) -> impl IntoView {
    // Frozen "last time" reminder: read once, without subscribing.
    let last = store.with_untracked(|db| {
        let exercise_id = db
            .session(session_id)?
            .entry(entry_id)?
            .exercise_id()
            .clone();
        last_performance(db, &exercise_id, session_id)
    });
    let last_text = last.as_ref().map(last_performance_text);
    let rows = Memo::new(move |_| {
        store.with(|db| planned_set_rows(db, session_id, entry_id, last.as_ref()))
    });
    let entry = Memo::new(move |_| {
        store.with(current_session).and_then(|session| {
            session
                .entries
                .into_iter()
                .find(|entry| entry.entry_id == entry_id)
        })
    });
    let Some(initial_rows) = rows.get_untracked() else {
        return ().into_any();
    };
    let draft = RwSignal::new(initial_rows.next_draft);
    let message = RwSignal::new(None::<String>);
    let editing_settings = RwSignal::new(false);
    let editing_set = RwSignal::new(None::<DoneSetRow>);
    let celebrated = RwSignal::new(Vec::<RecordBadge>::new());
    let confirm = ConfirmPrompt::new();
    let ask_remove_set = move |index: usize, set_text: String| {
        confirm.ask(Confirmation::RemoveSet { set_text }, move || {
            let _ = store.execute(|db| remove_set(db, session_id, entry_id, index));
            editing_set.set(None);
        });
    };

    let validate_set = move |_| {
        let measure = match draft.with_untracked(SetDraft::to_measure) {
            Ok(measure) => measure,
            Err(error) => return message.set(Some(draft_error_message(error))),
        };
        let records_before = store.with_untracked(|db| session_records(db, session_id));
        match store.execute(|db| add_set(db, session_id, entry_id, measure, web_platform::now())) {
            Ok(()) => {
                message.set(None);
                draft.set(SetDraft::prefilled(measure.kind(), Some(&measure)));
                let records_after = store.with_untracked(|db| session_records(db, session_id));
                let gained = new_records(&records_before, &records_after);
                if !gained.is_empty() {
                    celebrated.set(gained);
                    set_timeout(
                        move || celebrated.set(Vec::new()),
                        RECORD_BANNER_VISIBLE_FOR,
                    );
                }
            }
            Err(error) => message.set(Some(run_session_error_message(&error).to_owned())),
        }
    };

    let done_row = move |row: DoneSetRow| {
        let index = row.index;
        let set_text = row.text.clone();
        let delete = Callback::new(move |()| ask_remove_set(index, set_text.clone()));
        let toggle_warmup = Callback::new(move |()| {
            let _ = store.execute(|db| {
                update_set(db, session_id, entry_id, index, row.measure, !row.warmup)
            });
        });
        let edited_row = row.clone();
        view! {
            <SwipeRow
                left_action=Some(("Supprimer", delete))
                right_action=Some((if row.warmup { "Série normale" } else { "Échauffement" }, toggle_warmup))
                on_tap=Callback::new(move |()| editing_set.set(Some(edited_row.clone())))
            >
                <div class=if row.warmup { "set set--done set--warmup" } else { "set set--done" }>
                    <span class="set__number">{if row.warmup { "É".to_owned() } else { row.number.to_string() }}</span>
                    <span class="set__previous">{row.previous.unwrap_or_default()}</span>
                    <span class="set__value">{row.text}</span>
                    <span class="set__check" aria-label="Validée">"✓"</span>
                </div>
            </SwipeRow>
        }
    };

    let set_sheet = move || {
        editing_set.get().map(|row| {
            let index = row.index;
            let sheet_draft = RwSignal::new(SetDraft::prefilled(row.measure.kind(), Some(&row.measure)));
            let save = move |_| match sheet_draft.with_untracked(SetDraft::to_measure) {
                Ok(measure) => {
                    let _ = store.execute(|db| update_set(db, session_id, entry_id, index, measure, row.warmup));
                    editing_set.set(None);
                }
                Err(error) => message.set(Some(draft_error_message(error))),
            };
            let set_text = row.text.clone();
            let delete = move |_| ask_remove_set(index, set_text.clone());
            view! {
                <div class="sheet-backdrop" on:click=move |_| editing_set.set(None)></div>
                <div class="sheet" role="dialog" aria-label="Modifier la série">
                    <p class="eyebrow">"Série " {row.number}</p>
                    <SetDraftEditor draft=sheet_draft />
                    <div class="sheet__actions">
                        <button type="button" class="btn btn--danger" on:click=delete>"Supprimer"</button>
                        <button type="button" class="btn btn--primary" on:click=save>"Enregistrer"</button>
                    </div>
                </div>
            }
        })
    };

    let settings_text =
        move || entry.with(|entry| entry.as_ref().and_then(|entry| entry.settings.clone()));
    let settings_sheet = move || {
        editing_settings.get().then(|| {
            let save = Callback::new(move |text: String| {
                let exercise_id = entry
                    .with_untracked(|entry| entry.as_ref().map(|entry| entry.exercise_id.clone()));
                if let Some(exercise_id) = exercise_id {
                    let _ = store.execute(|db| set_exercise_settings(db, exercise_id, &text));
                }
                editing_settings.set(false);
            });
            view! {
                <TextSheet
                    title="Réglages de l'exercice"
                    placeholder="Siège cran 4, poulie haute…"
                    initial=settings_text().unwrap_or_default()
                    on_save=save
                    on_cancel=Callback::new(move |()| editing_settings.set(false))
                />
            }
        })
    };

    let record_banner = move || {
        let badges = celebrated.get();
        (!badges.is_empty()).then(|| view! {
            <div class="record-banner" role="status">
                <span class="medal" aria-hidden="true">"★"</span>
                <span class="record-banner__text">
                    <span class="record-banner__title">"Record !"</span>
                    {badges.into_iter().map(|badge| view! { <span>{badge.text}</span> }).collect_view()}
                </span>
            </div>
        })
    };

    view! {
        <section class="exercise">
            <header class="exercise__header">
                <p class="eyebrow">{move || entry.with(|entry| entry.as_ref().map(|entry| entry.position_label.clone()))}</p>
                <h1 class="exercise__name">{move || entry.with(|entry| entry.as_ref().map(|entry| entry.exercise_name.clone()))}</h1>
                <button type="button" class="exercise__settings" on:click=move |_| editing_settings.set(true)>
                    {move || settings_text().unwrap_or_else(|| "+ Ajouter des réglages".to_owned())}
                </button>
                {last_text.map(|text| view! {
                    <p class="exercise__last">
                        <span class="eyebrow">"Dernière fois"</span>
                        <span>{text}</span>
                    </p>
                })}
            </header>
            {record_banner}
            {move || {
                rows.get().map(|rows| {
                    view! {
                        <div class="sets">
                            <div class="sets__head">
                                <span>"Série"</span>
                                <span>"Dernière fois"</span>
                                <span>"Aujourd'hui"</span>
                                <span></span>
                            </div>
                            {rows.done.into_iter().map(done_row).collect_view()}
                            <div class="set-current">
                                <div class="set set--current">
                                    <span class="set__number">{rows.current_number}</span>
                                    <span class="set__previous">{rows.current_previous.unwrap_or_default()}</span>
                                </div>
                                <SetTimer draft set_timer_running />
                                <SetDraftEditor draft />
                                {move || message.get().map(|text| view! { <p class="error" role="alert">{text}</p> })}
                                <button type="button" class="btn btn--primary" on:click=validate_set>
                                    "Valider la série " {rows.current_number}
                                </button>
                            </div>
                            {rows
                                .upcoming
                                .into_iter()
                                .map(|row| view! {
                                    <div class="set set--upcoming">
                                        <span class="set__number">{row.number}</span>
                                        <span class="set__previous">{row.previous}</span>
                                    </div>
                                })
                                .collect_view()}
                            <p class="sets__hint">"Glisser une série : à gauche pour la supprimer, à droite pour l'échauffement."</p>
                        </div>
                    }
                })
            }}
            {set_sheet}
            {settings_sheet}
            <ConfirmSheet prompt=confirm />
        </section>
    }
    .into_any()
}

// Set timer, computed from its start time like the rest timer, so it survives sleep.
#[component]
fn SetTimer(draft: RwSignal<SetDraft>, set_timer_running: RwSignal<bool>) -> impl IntoView {
    let started_at = RwSignal::new(None::<DateTime<FixedOffset>>);
    let now = ticking_now();
    let timed_field = move || draft.with(SetDraft::timed_field_index);
    let elapsed_seconds =
        move |since: DateTime<FixedOffset>| (now.get() - since).num_seconds().max(0);
    on_cleanup(move || set_timer_running.set(false));

    let start = move |_| {
        started_at.set(Some(web_platform::now()));
        set_timer_running.set(true);
    };
    let stop = move |_| {
        if let (Some(since), Some(index)) = (started_at.get_untracked(), timed_field()) {
            let seconds = u32::try_from((web_platform::now() - since).num_seconds().max(1))
                .unwrap_or(u32::MAX);
            draft.update(|draft| draft.set_value(index, Some(seconds)));
        }
        started_at.set(None);
        set_timer_running.set(false);
    };

    move || {
        timed_field().map(|_| match started_at.get() {
            Some(since) => view! {
                <div class="set-timer set-timer--running">
                    <span class="set-timer__digits">{move || format_clock(elapsed_seconds(since))}</span>
                    <button
                        type="button"
                        class="btn btn--ghost"
                        on:click=move |_| {
                            started_at.set(None);
                            set_timer_running.set(false);
                        }
                    >
                        "Remettre à zéro"
                    </button>
                    <button type="button" class="btn btn--secondary" on:click=stop>"Arrêter"</button>
                </div>
            }
                .into_any(),
            None => view! {
                <div class="set-timer">
                    <span class="eyebrow">"Chronomètre"</span>
                    <button type="button" class="btn btn--secondary" on:click=start>"Démarrer"</button>
                </div>
            }
                .into_any(),
        })
    }
}
