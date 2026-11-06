use features::shared::keypad::KeypadText;
use leptos::prelude::*;

const DIGITS_ONE_TO_NINE: [char; 9] = ['1', '2', '3', '4', '5', '6', '7', '8', '9'];

#[component]
pub fn Keypad(
    title: &'static str,
    unit: &'static str,
    initial: String,
    decimals: u32,
    shortcuts: &'static [(&'static str, i64)],
    on_done: Callback<String>,
    on_cancel: Callback<()>,
) -> impl IntoView {
    let keypad = RwSignal::new(KeypadText::new(initial, decimals));
    let allows_decimal = keypad.with_untracked(KeypadText::allows_decimal_separator);
    let digit_key = move |digit: char| {
        view! {
            <button
                type="button"
                class="keypad__key"
                on:click=move |_| keypad.update(|keypad| keypad.press_digit(digit))
            >
                {digit.to_string()}
            </button>
        }
    };
    let typed_text = move || keypad.with(|keypad| keypad.text().to_owned());

    view! {
        <div class="sheet-backdrop" on:click=move |_| on_cancel.run(())></div>
        <div class="sheet keypad" role="dialog" aria-label=title>
            <p class="eyebrow">{title}</p>
            <p class="keypad__display">
                <span class="keypad__value">{typed_text}</span>
                <span class="keypad__unit">{unit}</span>
            </p>
            {(!shortcuts.is_empty()).then(|| view! {
                <div class="keypad__shortcuts">
                    {shortcuts
                        .iter()
                        .map(|&(label, delta)| view! {
                            <button
                                type="button"
                                class="keypad__shortcut"
                                on:click=move |_| keypad.update(|keypad| keypad.add(delta))
                            >
                                {label}
                            </button>
                        })
                        .collect_view()}
                </div>
            })}
            <div class="keypad__grid">
                {DIGITS_ONE_TO_NINE.into_iter().map(digit_key).collect_view()}
                <button
                    type="button"
                    class="keypad__key keypad__key--secondary"
                    disabled=!allows_decimal
                    on:click=move |_| keypad.update(KeypadText::press_decimal_separator)
                >
                    ","
                </button>
                {digit_key('0')}
                <button
                    type="button"
                    class="keypad__key keypad__key--secondary"
                    aria-label="Effacer"
                    on:click=move |_| keypad.update(KeypadText::press_delete)
                >
                    "⌫"
                </button>
            </div>
            <div class="sheet__actions">
                <button type="button" class="btn btn--secondary" on:click=move |_| on_cancel.run(())>
                    "Annuler"
                </button>
                <button
                    type="button"
                    class="btn btn--primary"
                    on:click=move |_| on_done.run(keypad.with_untracked(|keypad| keypad.text().to_owned()))
                >
                    "OK"
                </button>
            </div>
        </div>
    }
}
