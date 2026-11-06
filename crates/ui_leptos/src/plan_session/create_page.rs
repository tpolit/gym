use domain::SessionId;
use features::plan_session::{
    parse_date_input, plan_error_message, plan_from_session, plan_session, template_name,
};
use leptos::prelude::*;

use crate::navigation::{Navigation, PlanSource, Screen};
use crate::store::Store;
use crate::ui::{TitleVisibility, TopBar};

#[component]
pub fn CreatePlanPage(
    store: Store,
    navigation: Navigation,
    template: Option<SessionId>,
) -> impl IntoView {
    let name = RwSignal::new(String::new());
    let date = RwSignal::new(String::new());
    let error = RwSignal::new(None::<&'static str>);
    let template_title =
        template.and_then(|source| store.with_untracked(|db| template_name(db, source)));

    let create = move || {
        let planned_for = parse_date_input(&date.get_untracked());
        let result = match template {
            Some(source) => store.execute(|db| plan_from_session(db, source, planned_for)),
            None => {
                let typed_name = name.get_untracked();
                store.execute(|db| plan_session(db, &typed_name, planned_for, &[]))
            }
        };
        match result {
            Ok(session_id) => navigation.replace(Screen::PlanSession(PlanSource::Edit(session_id))),
            Err(failure) => error.set(Some(plan_error_message(&failure))),
        }
    };

    view! {
        <main class="screen">
            <TopBar
                on_back=Callback::new(move |()| navigation.back())
                title="Planifier"
                title_visibility=TitleVisibility::OnceScrolled
            />
            <h1 class="title">"Planifier"</h1>
            {template_title.clone().map(|title| view! {
                <p class="muted">"Mêmes exercices que « " {title} " »."</p>
            })}
            <form
                class="form"
                on:submit=move |event| {
                    event.prevent_default();
                    create();
                }
            >
                {template_title.is_none().then(|| view! {
                    <label class="field">
                        <span class="eyebrow">"Nom de la séance"</span>
                        <input
                            class="input"
                            type="text"
                            placeholder="Push A, Jambes…"
                            prop:value=move || name.get()
                            on:input=move |event| name.set(event_target_value(&event))
                        />
                    </label>
                })}
                <label class="field">
                    <span class="eyebrow">"Date (facultative)"</span>
                    <input
                        class="input"
                        type="date"
                        prop:value=move || date.get()
                        on:input=move |event| date.set(event_target_value(&event))
                    />
                </label>
                {move || error.get().map(|message| view! { <p class="error" role="alert">{message}</p> })}
                <button type="submit" class="btn btn--primary">"Planifier"</button>
            </form>
        </main>
    }
}
