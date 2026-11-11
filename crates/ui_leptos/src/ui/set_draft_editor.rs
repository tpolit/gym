use features::shared::set_draft::SetDraft;
use leptos::prelude::*;

use crate::ui::Keypad;

// − / + steppers for main values, chips for optional ones, custom keypad when a value is tapped.
#[component]
pub fn SetDraftEditor(draft: RwSignal<SetDraft>) -> impl IntoView {
    let editing_field = RwSignal::new(None::<usize>);

    let fields = move || {
        draft.with(|current_draft| {
            current_draft
                .fields()
                .iter()
                .copied()
                .enumerate()
                .map(|(index, field)| {
                    let value = field.display_value();
                    let open_keypad = move |_| editing_field.set(Some(index));
                    if field.kind.step().is_some() {
                        view! {
                            <div class="stepper">
                                <button
                                    type="button"
                                    class="stepper__button"
                                    aria-label="Diminuer"
                                    on:click=move |_| draft.update(|draft| draft.decrement(index))
                                >
                                    "−"
                                </button>
                                <button type="button" class="stepper__value" on:click=open_keypad>
                                    <b>{value}</b>
                                    <small>{field.kind.caption()}</small>
                                </button>
                                <button
                                    type="button"
                                    class="stepper__button"
                                    aria-label="Augmenter"
                                    on:click=move |_| draft.update(|draft| draft.increment(index))
                                >
                                    "+"
                                </button>
                            </div>
                        }
                        .into_any()
                    } else {
                        view! {
                            <button type="button" class="chip" on:click=open_keypad>
                                {field.kind.caption()} " " <b>{value}</b>
                            </button>
                        }
                        .into_any()
                    }
                })
                .collect_view()
        })
    };

    let keypad = move || {
        let index = editing_field.get()?;
        let field = draft.with_untracked(|draft| draft.fields().get(index).copied())?;
        let on_done = Callback::new(move |text: String| {
            draft.update(|draft| draft.set_value(index, field.kind.parse_keypad_text(&text)));
            editing_field.set(draft.with_untracked(|draft| draft.field_to_type_after(index)));
        });
        Some(view! {
            <Keypad
                title=field.kind.label()
                unit=field.kind.unit()
                initial=field.value.map(|value| field.kind.keypad_text(value)).unwrap_or_default()
                decimals=field.kind.keypad_decimals()
                shortcuts=field.kind.keypad_shortcuts()
                on_done
                on_cancel=Callback::new(move |()| editing_field.set(None))
            />
        })
    };

    view! {
        <div class="set-editor">{fields}</div>
        {keypad}
    }
}
