use features::run_session::{
    RunSessionError, default_session_name, repeat_session, run_session_error_message,
    session_preview, start_new_session, start_planned_session,
};
use leptos::prelude::*;

use crate::navigation::{Navigation, Screen, StartSource};
use crate::store::Store;
use crate::ui::{TitleVisibility, TopBar};

#[component]
pub fn StartSessionPage(
    store: Store,
    navigation: Navigation,
    source: StartSource,
) -> impl IntoView {
    let error = RwSignal::new(None::<&'static str>);
    let open_session = move |result: Result<domain::SessionId, RunSessionError>| match result {
        Ok(session_id) => navigation.replace(Screen::RunSession(session_id)),
        Err(failure) => error.set(Some(run_session_error_message(&failure))),
    };
    let error_view = move || {
        error
            .get()
            .map(|message| view! { <p class="error" role="alert">{message}</p> })
    };

    let (title, content) = match source {
        StartSource::New => {
            let name = RwSignal::new(default_session_name(web_platform::today()));
            let form = view! {
                <h1 class="title">"Nouvelle séance"</h1>
                <form
                    class="form"
                    on:submit=move |event| {
                        event.prevent_default();
                        let typed_name = name.get_untracked();
                        open_session(store.execute(|db| start_new_session(db, &typed_name, web_platform::now())));
                    }
                >
                    <label class="field">
                        <span class="eyebrow">"Nom de la séance"</span>
                        <input
                            class="input"
                            type="text"
                            enterkeyhint="go"
                            prop:value=move || name.get()
                            on:input=move |event| name.set(event_target_value(&event))
                        />
                    </label>
                    {error_view}
                    <button type="submit" class="btn btn--primary">"Démarrer"</button>
                </form>
            }
                .into_any();
            (Some("Nouvelle séance".to_owned()), form)
        }
        StartSource::Planned(session_id) | StartSource::Repeat(session_id) => {
            let preview = store.with_untracked(|db| session_preview(db, session_id));
            let start = move |_| {
                let result = match source {
                    StartSource::Repeat(_) => {
                        store.execute(|db| repeat_session(db, session_id, web_platform::now()))
                    }
                    StartSource::New | StartSource::Planned(_) => store
                        .execute(|db| start_planned_session(db, session_id, web_platform::now()))
                        .map(|()| session_id),
                };
                open_session(result);
            };
            let eyebrow = match source {
                StartSource::Repeat(_) => "Refaire la séance",
                StartSource::New | StartSource::Planned(_) => "Lancer la séance",
            };
            let title = preview.as_ref().map(|preview| preview.name.clone());
            let preview_view = match preview {
                Some(preview) => view! {
                    <p class="eyebrow">{eyebrow}</p>
                    <h1 class="title">{preview.name}</h1>
                    <ol class="list">
                        {preview
                            .exercise_names
                            .into_iter()
                            .map(|name| view! { <li class="list__item">{name}</li> })
                            .collect_view()}
                    </ol>
                    {error_view}
                    <button type="button" class="btn btn--primary" on:click=start>"C'est parti"</button>
                }
                    .into_any(),
                None => view! { <p class="muted">"Cette séance n'est plus disponible."</p> }.into_any(),
            };
            (title, preview_view)
        }
    };

    view! {
        <main class="screen">
            <TopBar
                on_back=Callback::new(move |()| navigation.back())
                title
                title_visibility=TitleVisibility::OnceScrolled
            />
            {content}
        </main>
    }
}
