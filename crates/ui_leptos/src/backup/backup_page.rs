use features::backup::{
    ImportPreview, backup_error_message, data_summary, export_data, read_import, replace_all_data,
};
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos::web_sys::HtmlInputElement;

use crate::navigation::{Navigation, Screen};
use crate::store::Store;
use crate::ui::{TitleVisibility, TopBar};
use crate::{FILE_MIME_TYPE, file_codec};

#[component]
pub fn BackupPage(store: Store, navigation: Navigation) -> impl IntoView {
    let message = RwSignal::new(None::<&'static str>);
    let preview = RwSignal::new(None::<ImportPreview>);
    let current_summary = Memo::new(move |_| store.with(data_summary));

    let export = move || {
        let file = store.with_untracked(|db| export_data(db, &file_codec(), web_platform::today()));
        message.set(Some(match file {
            Ok(file) => {
                match web_platform::download_file(&file.file_name, &file.bytes, FILE_MIME_TYPE) {
                    Ok(()) => "Fichier de sauvegarde créé.",
                    Err(_) => "Le navigateur a refusé le téléchargement.",
                }
            }
            Err(error) => backup_error_message(&error),
        }));
    };

    let choose_file = move |event: leptos::ev::Event| {
        let input: HtmlInputElement = event_target(&event);
        let Some(file) = input.files().and_then(|files| files.get(0)) else {
            return;
        };
        input.set_value("");
        spawn_local(async move {
            match web_platform::read_file_bytes(&file).await {
                Ok(bytes) => match read_import(&bytes, &file_codec()) {
                    Ok(read) => {
                        message.set(None);
                        preview.set(Some(read));
                    }
                    Err(error) => message.set(Some(backup_error_message(&error))),
                },
                Err(_) => message.set(Some("Impossible de lire ce fichier.")),
            }
        });
    };

    let replace = move |_| {
        let Some(imported) = preview.get_untracked() else {
            return;
        };
        let _ = store.execute(|db| replace_all_data(db, imported.db));
        preview.set(None);
        navigation.reset_to(Screen::Home);
    };

    view! {
        <main class="screen">
            <TopBar
                on_back=Callback::new(move |()| navigation.back())
                title="Sauvegarde"
                title_visibility=TitleVisibility::OnceScrolled
            />
            <h1 class="title">"Sauvegarde"</h1>
            <p class="muted">
                "Vos données ne sont que sur ce téléphone. Exportez-les régulièrement (Fichiers, iCloud Drive…)."
            </p>
            <section class="card backup__card">
                <p class="eyebrow">"Données actuelles"</p>
                <p>{current_summary}</p>
                <button type="button" class="btn btn--primary" on:click=move |_| export()>
                    "Exporter mes données"
                </button>
            </section>
            <section class="card backup__card">
                <p class="eyebrow">"Importer une sauvegarde"</p>
                <p class="muted">"L'import remplace toutes les données actuelles."</p>
                <label class="btn btn--secondary backup__file">
                    "Choisir un fichier"
                    <input type="file" accept=".json,application/json" on:change=choose_file />
                </label>
            </section>
            {move || message.get().map(|text| view! { <p class="muted" role="status">{text}</p> })}
            {move || {
                preview.get().map(|imported| view! {
                    <div class="sheet-backdrop" on:click=move |_| preview.set(None)></div>
                    <div class="sheet" role="dialog" aria-label="Confirmer l'import">
                        <p class="eyebrow">"Remplacer vos données ?"</p>
                        <p>"Fichier : " {imported.summary}</p>
                        <p class="muted">"Actuellement : " {current_summary}</p>
                        <button type="button" class="btn btn--secondary" on:click=move |_| export()>
                            "Exporter d'abord mes données actuelles"
                        </button>
                        <div class="sheet__actions">
                            <button type="button" class="btn btn--secondary" on:click=move |_| preview.set(None)>
                                "Annuler"
                            </button>
                            <button type="button" class="btn btn--danger" on:click=replace>
                                "Remplacer"
                            </button>
                        </div>
                    </div>
                })
            }}
        </main>
    }
}
