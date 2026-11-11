use domain::{EntryId, ExerciseId, SessionId};
use features::history::{
    DetailExercise, DetailSet, HistoryError, add_done_entry, add_done_set, delete_session,
    history_error_message, move_done_entry, remove_done_entry, remove_done_set, rename_session,
    session_detail, update_done_set, update_session_notes, update_session_times,
};
use features::shared::confirmation::Confirmation;
use features::shared::exercise_choices::exercise_choices;
use features::shared::set_draft::{SetDraft, draft_error_message};
use leptos::prelude::*;

use crate::navigation::{Navigation, PlanSource, Screen, StartSource};
use crate::store::Store;
use crate::ui::{
    ConfirmPrompt, ConfirmSheet, ExercisePicker, ReorderButtons, SetDraftEditor, SwipeRow,
    TextSheet, TitleVisibility, TopBar,
};

#[derive(Clone, PartialEq, Eq)]
enum OpenSheet {
    Name,
    Notes,
    Times,
    EditSet { entry_id: EntryId, set: DetailSet },
    AddSet { entry_id: EntryId, draft: SetDraft },
    Picker,
}

#[component]
pub fn SessionDetailPage(
    store: Store,
    navigation: Navigation,
    session_id: SessionId,
) -> impl IntoView {
    let detail = Memo::new(move |_| store.with(|db| session_detail(db, session_id)));
    let open_sheet = RwSignal::new(None::<OpenSheet>);
    let error = RwSignal::new(None::<String>);
    let confirm = ConfirmPrompt::new();

    let report = move |result: Result<(), HistoryError>| match result {
        Ok(()) => {
            error.set(None);
            open_sheet.set(None);
        }
        Err(failure) => error.set(Some(history_error_message(&failure).to_owned())),
    };
    let delete = move |_| {
        confirm.ask(Confirmation::DeleteSession, move || {
            match store.execute(|db| delete_session(db, session_id)) {
                Ok(()) => navigation.back(),
                Err(failure) => error.set(Some(history_error_message(&failure).to_owned())),
            }
        });
    };

    let text_sheet = move |opened: &OpenSheet| {
        let (title, placeholder, current) = detail.with_untracked(|detail| {
            let detail = detail.as_ref()?;
            match opened {
                OpenSheet::Name => Some(("Nom de la séance", "Push A", detail.name.clone())),
                OpenSheet::Notes => Some(("Notes", "Ressenti, conditions…", detail.notes.clone())),
                _ => None,
            }
        })?;
        let is_name = *opened == OpenSheet::Name;
        let save = Callback::new(move |text: String| {
            report(if is_name {
                store.execute(|db| rename_session(db, session_id, &text))
            } else {
                store.execute(|db| update_session_notes(db, session_id, &text))
            });
        });
        Some(view! {
            <TextSheet title placeholder initial=current on_save=save on_cancel=Callback::new(move |()| open_sheet.set(None)) />
        }
        .into_any())
    };

    let times_sheet = move || {
        let (started, finished) = detail.with_untracked(|detail| {
            detail
                .as_ref()
                .map(|detail| (detail.started_input.clone(), detail.finished_input.clone()))
        })?;
        let started = RwSignal::new(started);
        let finished = RwSignal::new(finished);
        let save = move |_| {
            let (started, finished) = (started.get_untracked(), finished.get_untracked());
            report(store.execute(|db| update_session_times(db, session_id, &started, &finished)));
        };
        Some(view! {
            <div class="sheet-backdrop" on:click=move |_| open_sheet.set(None)></div>
            <div class="sheet" role="dialog" aria-label="Horaires">
                <p class="eyebrow">"Horaires"</p>
                <label class="field">
                    <span class="eyebrow">"Début"</span>
                    <input class="input" type="datetime-local" prop:value=move || started.get() on:input=move |event| started.set(event_target_value(&event)) />
                </label>
                <label class="field">
                    <span class="eyebrow">"Fin"</span>
                    <input class="input" type="datetime-local" prop:value=move || finished.get() on:input=move |event| finished.set(event_target_value(&event)) />
                </label>
                {move || error.get().map(|text| view! { <p class="error" role="alert">{text}</p> })}
                <div class="sheet__actions">
                    <button type="button" class="btn btn--secondary" on:click=move |_| open_sheet.set(None)>"Annuler"</button>
                    <button type="button" class="btn btn--primary" on:click=save>"Enregistrer"</button>
                </div>
            </div>
        }
        .into_any())
    };

    let set_sheet = move |entry_id: EntryId, existing: Option<DetailSet>, initial: SetDraft| {
        let draft = RwSignal::new(initial);
        let index = existing.as_ref().map(|set| set.index);
        let is_warmup = existing.as_ref().is_some_and(|set| set.is_warmup);
        let set_text = existing.map(|set| set.text).unwrap_or_default();
        let save = move |_| match draft.with_untracked(SetDraft::to_measure) {
            Ok(measure) => report(match index {
                Some(index) => store.execute(|db| {
                    update_done_set(db, session_id, entry_id, index, measure, is_warmup)
                }),
                None => store.execute(|db| add_done_set(db, session_id, entry_id, measure)),
            }),
            Err(failure) => error.set(Some(draft_error_message(failure))),
        };
        let delete = move |_| {
            if let Some(index) = index {
                confirm.ask(
                    Confirmation::RemoveSet {
                        set_text: set_text.clone(),
                    },
                    move || {
                        let _ =
                            store.execute(|db| remove_done_set(db, session_id, entry_id, index));
                        open_sheet.set(None);
                    },
                );
            }
        };
        view! {
            <div class="sheet-backdrop" on:click=move |_| open_sheet.set(None)></div>
            <div class="sheet" role="dialog" aria-label="Série">
                <p class="eyebrow">{if index.is_some() { "Modifier la série" } else { "Ajouter une série" }}</p>
                <SetDraftEditor draft />
                {move || error.get().map(|text| view! { <p class="error" role="alert">{text}</p> })}
                <div class="sheet__actions">
                    {if index.is_some() {
                        view! { <button type="button" class="btn btn--danger" on:click=delete>"Supprimer"</button> }.into_any()
                    } else {
                        view! { <button type="button" class="btn btn--secondary" on:click=move |_| open_sheet.set(None)>"Annuler"</button> }.into_any()
                    }}
                    <button type="button" class="btn btn--primary" on:click=save>"Enregistrer"</button>
                </div>
            </div>
        }
        .into_any()
    };

    let sheet = move || {
        let opened = open_sheet.get()?;
        match opened {
            OpenSheet::Name | OpenSheet::Notes => text_sheet(&opened),
            OpenSheet::Times => times_sheet(),
            OpenSheet::EditSet { entry_id, set } => {
                let draft = SetDraft::prefilled(set.measure.kind(), Some(&set.measure));
                Some(set_sheet(entry_id, Some(set), draft))
            }
            OpenSheet::AddSet { entry_id, draft } => Some(set_sheet(entry_id, None, draft)),
            OpenSheet::Picker => Some(view! {
                <ExercisePicker
                    choices=store.with_untracked(exercise_choices)
                    on_pick=Callback::new(move |exercise_id: ExerciseId| {
                        report(store.execute(|db| add_done_entry(db, session_id, exercise_id)).map(|_| ()));
                    })
                    on_close=Callback::new(move |()| open_sheet.set(None))
                />
            }
            .into_any()),
        }
    };

    let content = move || {
        let Some(detail) = detail.get() else {
            return view! { <p class="muted">"Cette séance n'est plus dans l'historique."</p> }
                .into_any();
        };
        let has_notes = !detail.notes.is_empty();
        view! {
            <button type="button" class="history__heading" on:click=move |_| open_sheet.set(Some(OpenSheet::Name))>
                <span class="eyebrow">{detail.date_text}</span>
                <span class="title">{detail.name}</span>
            </button>
            <button type="button" class="history__times" on:click=move |_| open_sheet.set(Some(OpenSheet::Times))>
                {detail.time_range} " · " {detail.duration}
                {detail.volume.map(|volume| view! { " · " {volume} })}
                <span class="history__edit">"Modifier"</span>
            </button>
            <button type="button" class="card history__notes" on:click=move |_| open_sheet.set(Some(OpenSheet::Notes))>
                <span class="eyebrow">"Notes"</span>
                <span class:muted=!has_notes>
                    {if has_notes { detail.notes } else { "+ Ajouter une note".to_owned() }}
                </span>
            </button>
            {detail
                .exercises
                .into_iter()
                .map(|exercise| view! { <ExerciseSets store session_id exercise open_sheet confirm /> })
                .collect_view()}
            <button type="button" class="btn btn--secondary" on:click=move |_| open_sheet.set(Some(OpenSheet::Picker))>
                "+ Ajouter un exercice"
            </button>
        }
        .into_any()
    };

    view! {
        <main class="screen">
            <TopBar
                on_back=Callback::new(move |()| navigation.back())
                title=Signal::derive(move || detail.with(|detail| detail.as_ref().map(|detail| detail.name.clone())))
                title_visibility=TitleVisibility::OnceScrolled
            />
            {content}
            {move || error.get().filter(|_| open_sheet.with(Option::is_none)).map(|text| view! { <p class="error" role="alert">{text}</p> })}
            <div class="history__actions">
                <button
                    type="button"
                    class="btn btn--primary"
                    on:click=move |_| navigation.go_to(Screen::StartSession(StartSource::Repeat(session_id)))
                >
                    "Refaire cette séance"
                </button>
                <button
                    type="button"
                    class="btn btn--secondary"
                    on:click=move |_| navigation.go_to(Screen::PlanSession(PlanSource::From(session_id)))
                >
                    "Planifier à partir de cette séance"
                </button>
                <button type="button" class="btn btn--danger-ghost" on:click=delete>
                    "Supprimer la séance"
                </button>
            </div>
            {sheet}
            <ConfirmSheet prompt=confirm />
        </main>
    }
}

#[component]
fn ExerciseSets(
    store: Store,
    session_id: SessionId,
    exercise: DetailExercise,
    open_sheet: RwSignal<Option<OpenSheet>>,
    confirm: ConfirmPrompt,
) -> impl IntoView {
    let entry_id = exercise.entry_id;
    let index = exercise.index;
    let is_first = index == 0;
    let move_to = move |new_index: usize| {
        let _ = store.execute(|db| move_done_entry(db, session_id, entry_id, new_index));
    };
    let removal = Confirmation::RemoveEntry {
        exercise_name: exercise.name.clone(),
        set_count: exercise.sets.len(),
    };
    let remove = move |_| {
        confirm.ask(removal.clone(), move || {
            let _ = store.execute(|db| remove_done_entry(db, session_id, entry_id));
        });
    };
    let new_set_draft = exercise
        .sets
        .last()
        .map(|set| SetDraft::prefilled(set.measure.kind(), Some(&set.measure)))
        .or_else(|| exercise.measure_kind.map(SetDraft::empty));
    let set_row = move |set: DetailSet| {
        let set_index = set.index;
        let set_text = set.text.clone();
        let delete = Callback::new(move |()| {
            confirm.ask(
                Confirmation::RemoveSet {
                    set_text: set_text.clone(),
                },
                move || {
                    let _ =
                        store.execute(|db| remove_done_set(db, session_id, entry_id, set_index));
                },
            );
        });
        let (measure, warmup) = (set.measure, set.is_warmup);
        let toggle_warmup = Callback::new(move |()| {
            let _ = store.execute(|db| {
                update_done_set(db, session_id, entry_id, set_index, measure, !warmup)
            });
        });
        let edited = set.clone();
        view! {
            <SwipeRow
                left_action=Some(("Supprimer", delete))
                right_action=Some((if set.is_warmup { "Série normale" } else { "Échauffement" }, toggle_warmup))
                on_tap=Callback::new(move |()| open_sheet.set(Some(OpenSheet::EditSet { entry_id, set: edited.clone() })))
            >
                <div class="history__set">
                    <span class="history__set-number">
                        {if set.is_warmup { "É".to_owned() } else { set.number.to_string() }}
                    </span>
                    <span class="history__set-value">{set.text}</span>
                    <span class="history__rest">{set.rest_before.map(|rest| format!("repos {rest}"))}</span>
                </div>
            </SwipeRow>
        }
    };

    view! {
        <section class="history__exercise">
            <div class="history__exercise-header">
                <h2 class="history__exercise-name">{exercise.name}</h2>
                <ReorderButtons
                    can_move_up=!is_first
                    can_move_down=!exercise.is_last
                    on_move_up=Callback::new(move |()| move_to(index.saturating_sub(1)))
                    on_move_down=Callback::new(move |()| move_to(index + 1))
                />
                <button type="button" class="btn btn--ghost" on:click=remove>"Retirer"</button>
            </div>
            <div class="list">{exercise.sets.into_iter().map(set_row).collect_view()}</div>
            {new_set_draft.map(|draft| view! {
                <button
                    type="button"
                    class="btn btn--ghost history__add-set"
                    on:click=move |_| open_sheet.set(Some(OpenSheet::AddSet { entry_id, draft: draft.clone() }))
                >
                    "+ Ajouter une série"
                </button>
            })}
        </section>
    }
}
