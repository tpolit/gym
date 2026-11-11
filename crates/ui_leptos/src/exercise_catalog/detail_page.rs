use domain::{ExerciseId, Removed};
use features::exercise_catalog::{
    CustomState, catalog_error_message, exercise_detail, remove_custom_exercise,
    set_exercise_settings, unarchive_custom_exercise,
};
use features::shared::confirmation::Confirmation;
use leptos::prelude::*;

use crate::navigation::{Navigation, Screen};
use crate::store::Store;
use crate::ui::{ConfirmPrompt, ConfirmSheet, TextSheet, TitleVisibility, TopBar};

#[component]
pub fn ExerciseDetailPage(
    store: Store,
    navigation: Navigation,
    exercise_id: ExerciseId,
) -> impl IntoView {
    let id_for_detail = exercise_id.clone();
    let detail = Memo::new(move |_| store.with(|db| exercise_detail(db, &id_for_detail)));
    let editing_settings = RwSignal::new(false);
    let message = RwSignal::new(None::<&'static str>);
    let exercise_id = StoredValue::new(exercise_id);
    let confirm = ConfirmPrompt::new();

    let settings_sheet = move || {
        editing_settings.get().then(|| {
            let save = Callback::new(move |text: String| {
                let result = store.execute(|db| set_exercise_settings(db, exercise_id.get_value(), &text));
                message.set(result.err().map(|failure| catalog_error_message(&failure)));
                editing_settings.set(false);
            });
            view! {
                <TextSheet
                    title="Réglages de l'exercice"
                    placeholder="Siège cran 4, poulie haute…"
                    initial=detail.with_untracked(|detail| detail.as_ref().map(|detail| detail.settings.clone()).unwrap_or_default())
                    on_save=save
                    on_cancel=Callback::new(move |()| editing_settings.set(false))
                />
            }
        })
    };

    let custom_actions = move |custom: CustomState, name: String| {
        let removal = if custom.is_used {
            Confirmation::ArchiveCustomExercise { name }
        } else {
            Confirmation::DeleteCustomExercise { name }
        };
        let remove = move |_| {
            confirm.ask(removal.clone(), move || {
                match store.execute(|db| remove_custom_exercise(db, custom.id)) {
                    Ok(Removed::Deleted) => navigation.back(),
                    Ok(Removed::Archived) => message.set(Some(
                        "Exercice archivé : il reste visible dans l'historique.",
                    )),
                    Err(failure) => message.set(Some(catalog_error_message(&failure))),
                }
            });
        };
        let unarchive = move |_| {
            let result = store.execute(|db| unarchive_custom_exercise(db, custom.id));
            message.set(result.err().map(|failure| catalog_error_message(&failure)));
        };
        view! {
            <div class="catalog__actions">
                <button
                    type="button"
                    class="btn btn--secondary"
                    on:click=move |_| navigation.go_to(Screen::CustomExerciseForm(Some(custom.id)))
                >
                    "Modifier"
                </button>
                {if custom.is_archived {
                    view! { <button type="button" class="btn btn--secondary" on:click=unarchive>"Désarchiver"</button> }.into_any()
                } else {
                    let label = if custom.is_used { "Archiver" } else { "Supprimer" };
                    view! { <button type="button" class="btn btn--danger-ghost" on:click=remove>{label}</button> }.into_any()
                }}
            </div>
        }
    };

    let content = move || {
        let Some(detail) = detail.get() else {
            return view! { <p class="muted">"Cet exercice n'existe plus."</p> }.into_any();
        };
        let has_settings = !detail.settings.is_empty();
        let actions = detail
            .custom
            .map(|custom| custom_actions(custom, detail.name.clone()));
        let usage = match (detail.done_session_count, detail.last_practiced) {
            (0, _) | (_, None) => "Jamais pratiqué".to_owned(),
            (1, Some(last)) => format!("1 séance · dernière le {last}"),
            (count, Some(last)) => format!("{count} séances · dernière le {last}"),
        };
        view! {
            <p class="eyebrow">{detail.category}</p>
            <h1 class="title">{detail.name}</h1>
            <p class="muted">{detail.muscle_group} " · " {detail.equipment}</p>
            <p class="muted">"Mesure : " {detail.measure}</p>
            <p>{usage}</p>
            <button type="button" class="card catalog__settings" on:click=move |_| editing_settings.set(true)>
                <span class="eyebrow">"Réglages"</span>
                <span class:muted=!has_settings>
                    {if has_settings { detail.settings } else { "+ Ajouter des réglages".to_owned() }}
                </span>
            </button>
            {(detail.done_session_count > 0).then(|| {
                let filtered_history = Screen::History(Some(detail.id.clone()));
                view! {
                    <button
                        type="button"
                        class="btn btn--secondary"
                        on:click=move |_| navigation.go_to(filtered_history.clone())
                    >
                        "Voir les séances avec cet exercice"
                    </button>
                }
            })}
            {actions}
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
            {move || message.get().map(|text| view! { <p class="muted" role="status">{text}</p> })}
            {settings_sheet}
            <ConfirmSheet prompt=confirm />
        </main>
    }
}
