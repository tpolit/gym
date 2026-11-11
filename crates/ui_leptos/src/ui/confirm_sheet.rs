use features::shared::confirmation::{Confirmation, confirmation_text};
use leptos::prelude::*;

#[derive(Clone)]
struct PendingConfirmation {
    confirmation: Confirmation,
    on_confirm: Callback<()>,
}

// Every removal waits for an explicit confirmation: there is no undo.
#[derive(Clone, Copy)]
pub struct ConfirmPrompt(RwSignal<Option<PendingConfirmation>>);

impl ConfirmPrompt {
    pub fn new() -> Self {
        Self(RwSignal::new(None))
    }

    pub fn ask(self, confirmation: Confirmation, on_confirm: impl Fn() + Send + Sync + 'static) {
        self.0.set(Some(PendingConfirmation {
            confirmation,
            on_confirm: Callback::new(move |()| on_confirm()),
        }));
    }

    fn dismiss(self) {
        self.0.set(None);
    }
}

// Stacked above any other sheet, so that cancelling returns to it.
#[component]
pub fn ConfirmSheet(prompt: ConfirmPrompt) -> impl IntoView {
    move || {
        prompt.0.get().map(|pending| {
            let text = confirmation_text(&pending.confirmation);
            let label = text.title.clone();
            let confirm = move |_| {
                prompt.dismiss();
                pending.on_confirm.run(());
            };
            view! {
                <div class="sheet-backdrop sheet-backdrop--over" on:click=move |_| prompt.dismiss()></div>
                <div class="sheet sheet--over confirm" role="alertdialog" aria-label=label>
                    <p class="confirm__title">{text.title}</p>
                    <p class="muted">{text.message}</p>
                    <div class="sheet__actions">
                        <button type="button" class="btn btn--secondary" on:click=move |_| prompt.dismiss()>
                            "Annuler"
                        </button>
                        <button type="button" class="btn btn--danger confirm__action" on:click=confirm>
                            {text.action}
                        </button>
                    </div>
                </div>
            }
        })
    }
}
