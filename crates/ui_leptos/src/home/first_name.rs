use features::home::set_first_name;
use leptos::prelude::*;

use crate::store::Store;

// Asked on the home screen until given.
#[component]
pub fn FirstNamePrompt(store: Store) -> impl IntoView {
    let save = Callback::new(move |typed: String| {
        let _ = store.execute(|db| set_first_name(db, &typed));
    });
    view! {
        <div class="card">
            <FirstNameForm label="Comment vous appelez-vous ?" initial=String::new() on_save=save />
        </div>
    }
}

#[component]
pub fn FirstNameSheet(store: Store, initial: String, on_close: Callback<()>) -> impl IntoView {
    let save = Callback::new(move |typed: String| {
        let _ = store.execute(|db| set_first_name(db, &typed));
        on_close.run(());
    });
    view! {
        <div class="sheet-backdrop" on:click=move |_| on_close.run(())></div>
        <div class="sheet" role="dialog" aria-label="Prénom">
            <FirstNameForm label="Votre prénom" initial on_save=save on_cancel=on_close />
        </div>
    }
}

#[component]
fn FirstNameForm(
    label: &'static str,
    initial: String,
    on_save: Callback<String>,
    #[prop(optional)] on_cancel: Option<Callback<()>>,
) -> impl IntoView {
    let typed = RwSignal::new(initial);
    let save_button =
        view! { <button type="submit" class="btn btn--primary">"Enregistrer"</button> };
    let actions = match on_cancel {
        Some(cancel) => view! {
            <div class="sheet__actions">
                <button type="button" class="btn btn--secondary" on:click=move |_| cancel.run(())>
                    "Annuler"
                </button>
                {save_button}
            </div>
        }
        .into_any(),
        None => save_button.into_any(),
    };
    view! {
        <form
            class="form"
            on:submit=move |event| {
                event.prevent_default();
                on_save.run(typed.get_untracked());
            }
        >
            <label class="field">
                <span class="eyebrow">{label}</span>
                <input
                    class="input"
                    type="text"
                    placeholder="Prénom"
                    autocomplete="given-name"
                    autocapitalize="words"
                    enterkeyhint="done"
                    prop:value=move || typed.get()
                    on:input=move |event| typed.set(event_target_value(&event))
                />
            </label>
            {actions}
        </form>
    }
}
