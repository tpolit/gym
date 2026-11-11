use leptos::prelude::*;

#[component]
pub fn TextSheet(
    title: &'static str,
    placeholder: &'static str,
    initial: String,
    on_save: Callback<String>,
    on_cancel: Callback<()>,
) -> impl IntoView {
    let text = RwSignal::new(initial);
    view! {
        <div class="sheet-backdrop" on:click=move |_| on_cancel.run(())></div>
        <div class="sheet" role="dialog" aria-label=title>
            <p class="eyebrow">{title}</p>
            <textarea
                class="input input--multiline"
                rows="3"
                placeholder=placeholder
                prop:value=move || text.get()
                on:input=move |event| text.set(event_target_value(&event))
            ></textarea>
            <div class="sheet__actions">
                <button type="button" class="btn btn--secondary" on:click=move |_| on_cancel.run(())>
                    "Annuler"
                </button>
                <button
                    type="button"
                    class="btn btn--primary"
                    on:click=move |_| on_save.run(text.get_untracked())
                >
                    "Enregistrer"
                </button>
            </div>
        </div>
    }
}
