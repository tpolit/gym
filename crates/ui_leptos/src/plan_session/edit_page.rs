use domain::{ExerciseId, SessionId};
use features::plan_session::{
    PlannedEntryView, add_planned_exercise, delete_planned_session, move_planned_entry,
    parse_date_input, plan_error_message, planned_session_view, remove_planned_entry,
    update_planned_session,
};
use features::shared::confirmation::Confirmation;
use features::shared::exercise_choices::exercise_choices;
use leptos::prelude::*;

use crate::navigation::{Navigation, Screen, StartSource};
use crate::store::Store;
use crate::ui::{ConfirmPrompt, ConfirmSheet, ExercisePicker, ReorderButtons, TopBar};

#[component]
pub fn EditPlanPage(store: Store, navigation: Navigation, session_id: SessionId) -> impl IntoView {
    let plan = Memo::new(move |_| store.with(|db| planned_session_view(db, session_id)));
    let Some(initial) = plan.get_untracked() else {
        return view! {
            <main class="screen">
                <p class="muted">"Cette séance n'est plus planifiée."</p>
                <button type="button" class="btn btn--secondary" on:click=move |_| navigation.back()>
                    "Retour"
                </button>
            </main>
        }
        .into_any();
    };
    let name = RwSignal::new(initial.name);
    let date = RwSignal::new(initial.date_input);
    let error = RwSignal::new(None::<&'static str>);
    let picker_open = RwSignal::new(false);
    let confirm = ConfirmPrompt::new();

    let save_details = move || {
        let typed_name = name.get_untracked();
        let planned_for = parse_date_input(&date.get_untracked());
        let result =
            store.execute(|db| update_planned_session(db, session_id, &typed_name, planned_for));
        error.set(result.as_ref().err().map(plan_error_message));
        result.is_ok()
    };
    // The fields are also saved on `change`; saving again here covers a field still being edited.
    let validate = Callback::new(move |()| {
        if save_details() {
            navigation.back();
        }
    });
    // Leaving keeps the last valid values when the typed ones are rejected (empty name).
    let leave = Callback::new(move |()| {
        save_details();
        navigation.back();
    });
    let add_exercise = Callback::new(move |exercise_id: ExerciseId| {
        if let Err(failure) = store.execute(|db| add_planned_exercise(db, session_id, exercise_id))
        {
            error.set(Some(plan_error_message(&failure)));
        }
        picker_open.set(false);
    });
    let delete = move |_| {
        confirm.ask(Confirmation::DeletePlannedSession, move || {
            match store.execute(|db| delete_planned_session(db, session_id)) {
                Ok(()) => navigation.back(),
                Err(failure) => error.set(Some(plan_error_message(&failure))),
            }
        });
    };

    let entry_row = move |entry: PlannedEntryView| {
        let entry_id = entry.entry_id;
        let index = entry.index;
        let move_to = move |new_index: usize| {
            let _ = store.execute(|db| move_planned_entry(db, session_id, entry_id, new_index));
        };
        let removal = Confirmation::RemoveEntry {
            exercise_name: entry.exercise_name.clone(),
            set_count: 0,
        };
        let remove = move |_| {
            confirm.ask(removal.clone(), move || {
                let _ = store.execute(|db| remove_planned_entry(db, session_id, entry_id));
            });
        };
        view! {
            <div class="plan__entry">
                <span class="plan__entry-name">{entry.exercise_name}</span>
                <ReorderButtons
                    can_move_up=!entry.is_first
                    can_move_down=!entry.is_last
                    on_move_up=Callback::new(move |()| move_to(index.saturating_sub(1)))
                    on_move_down=Callback::new(move |()| move_to(index + 1))
                />
                <button type="button" class="btn btn--ghost" on:click=remove>"Retirer"</button>
            </div>
        }
    };

    view! {
        <main class="screen">
            <TopBar on_back=leave title="Séance planifiée" action=("OK", validate) />
            <label class="field">
                <span class="eyebrow">"Nom"</span>
                <input
                    class="input"
                    type="text"
                    prop:value=move || name.get()
                    on:input=move |event| name.set(event_target_value(&event))
                    on:change=move |_| {
                        save_details();
                    }
                />
            </label>
            <label class="field">
                <span class="eyebrow">"Date (facultative)"</span>
                <input
                    class="input"
                    type="date"
                    prop:value=move || date.get()
                    on:input=move |event| date.set(event_target_value(&event))
                    on:change=move |_| {
                        save_details();
                    }
                />
            </label>
            {move || error.get().map(|message| view! { <p class="error" role="alert">{message}</p> })}
            <p class="eyebrow">"Exercices"</p>
            {move || {
                let entries = plan.get().map(|plan| plan.entries).unwrap_or_default();
                if entries.is_empty() {
                    view! { <p class="muted">"Aucun exercice pour l'instant."</p> }.into_any()
                } else {
                    view! { <div class="list">{entries.into_iter().map(entry_row).collect_view()}</div> }.into_any()
                }
            }}
            <button type="button" class="btn btn--secondary" on:click=move |_| picker_open.set(true)>
                "+ Ajouter un exercice"
            </button>
            <button
                type="button"
                class="btn btn--primary"
                on:click=move |_| navigation.go_to(Screen::StartSession(StartSource::Planned(session_id)))
            >
                "Lancer la séance"
            </button>
            <button type="button" class="btn btn--danger-ghost" on:click=delete>
                "Supprimer la séance"
            </button>
            {move || {
                picker_open.get().then(|| view! {
                    <ExercisePicker
                        choices=store.with_untracked(exercise_choices)
                        on_pick=add_exercise
                        on_close=Callback::new(move |()| picker_open.set(false))
                    />
                })
            }}
            <ConfirmSheet prompt=confirm />
        </main>
    }
    .into_any()
}
