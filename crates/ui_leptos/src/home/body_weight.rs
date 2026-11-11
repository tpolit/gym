use features::home::{body_weight_error_message, set_body_weight};
use leptos::prelude::*;

use crate::store::Store;

// Asked on the home screen until given, then changed in the settings.
#[component]
pub fn BodyWeightPrompt(store: Store) -> impl IntoView {
    view! {
        <div class="card">
            <BodyWeightForm
                store
                label="Quel est votre poids de corps ?"
                hint="Il compte dans le volume des exercices au poids du corps."
                initial=String::new()
            />
        </div>
    }
}

#[component]
pub fn BodyWeightForm(
    store: Store,
    label: &'static str,
    hint: &'static str,
    initial: String,
) -> impl IntoView {
    let typed = RwSignal::new(initial);
    let error = RwSignal::new(None::<&'static str>);
    let save = move || {
        let result =
            store.execute(|db| set_body_weight(db, &typed.get_untracked(), web_platform::today()));
        error.set(result.err().map(body_weight_error_message));
    };
    view! {
        <form
            class="form body-weight"
            on:submit=move |event| {
                event.prevent_default();
                save();
            }
        >
            <label class="field">
                <span class="eyebrow">{label}</span>
                <span class="body-weight__row">
                    <input
                        class="input"
                        type="text"
                        inputmode="decimal"
                        placeholder="72,5"
                        enterkeyhint="done"
                        prop:value=move || typed.get()
                        on:input=move |event| typed.set(event_target_value(&event))
                    />
                    <span class="body-weight__unit">"kg"</span>
                </span>
            </label>
            <p class="muted body-weight__hint">{hint}</p>
            {move || error.get().map(|text| view! { <p class="error" role="alert">{text}</p> })}
            <button type="submit" class="btn btn--primary">"Enregistrer"</button>
        </form>
    }
}
