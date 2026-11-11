mod app;
mod backup;
mod exercise_catalog;
mod history;
mod home;
mod navigation;
mod plan_session;
mod run_session;
mod stats;
mod store;
mod tab_bar;
mod ui;

use std::rc::Rc;

use features::{AppCore, load_failure_message, raw_export_file_name};
use leptos::prelude::*;
use leptos::task::{spawn_local, spawn_local_scoped};
use persistence::{DbRepository, JsonCodec, Storage};
use web_platform::LocalStorage;

use crate::app::App;
use crate::store::Store;

// Kept from the project codename: localStorage is shared by every app under tpolit.github.io.
const STORAGE_KEY: &str = "gym.db";
pub(crate) const FILE_MIME_TYPE: &str = "application/json";

// Format of both the stored data and the backup files: chosen here only.
type FileCodec = JsonCodec;

pub(crate) const fn file_codec() -> FileCodec {
    JsonCodec
}

type Repository = DbRepository<LocalStorage, FileCodec>;

fn main() {
    console_error_panic_hook::set_once();

    // In development, the offline cache would hide `trunk serve` rebuilds.
    if !cfg!(debug_assertions)
        && let Err(error) = web_platform::register_service_worker("sw.js")
    {
        leptos::logging::warn!("Service worker not registered: {error:?}");
    }

    leptos::mount::mount_to_body(Root);
}

#[component]
fn Root() -> impl IntoView {
    let repository = Rc::new(DbRepository::new(
        LocalStorage::new(STORAGE_KEY),
        file_codec(),
    ));
    let loaded_store = RwSignal::new(None::<Store>);
    let save_failed = RwSignal::new(false);

    spawn_local_scoped(async move {
        let store = Store::new(AppCore::from_load(repository.load().await));
        save_after_each_change(store, repository, save_failed);
        loaded_store.set(Some(store));
    });

    move || match loaded_store.get() {
        None => view! { <main class="screen" aria-busy="true"></main> }.into_any(),
        Some(store) => match store.load_error() {
            Some(error) => {
                view! { <LoadFailure message=load_failure_message(&error) /> }.into_any()
            }
            None => view! {
                <Show when=move || save_failed.get()>
                    <p class="save-failure" role="alert">
                        "La dernière modification n'a pas pu être enregistrée."
                    </p>
                </Show>
                <App store />
            }
            .into_any(),
        },
    }
}

fn save_after_each_change(store: Store, repository: Rc<Repository>, save_failed: RwSignal<bool>) {
    Effect::new(move || {
        if let Some(db) = store.snapshot_to_save() {
            let repository = Rc::clone(&repository);
            spawn_local(async move {
                save_failed.set(repository.save(&db).await.is_err());
            });
        }
    });
}

// The unreadable content can still be downloaded as is, to recover it.
#[component]
fn LoadFailure(message: &'static str) -> impl IntoView {
    let export_status = RwSignal::new(None::<&'static str>);
    let export_raw = move |_| {
        spawn_local(async move {
            let status = match LocalStorage::new(STORAGE_KEY).read().await {
                Ok(Some(bytes)) => {
                    let file_name = raw_export_file_name(web_platform::today());
                    match web_platform::download_file(&file_name, &bytes, FILE_MIME_TYPE) {
                        Ok(()) => "Fichier créé.",
                        Err(_) => "Le navigateur a refusé le téléchargement.",
                    }
                }
                Ok(None) => "Aucune donnée à exporter.",
                Err(_) => "Les données brutes sont illisibles.",
            };
            export_status.set(Some(status));
        });
    };
    view! {
        <main class="screen">
            <h1 class="title">"Chargement impossible"</h1>
            <p>{message}</p>
            <p class="muted">
                "Vos données n'ont pas été modifiées : l'application ne sauvegarde rien tant que ce problème n'est pas résolu."
            </p>
            <button type="button" class="btn btn--secondary" on:click=export_raw>
                "Exporter les données brutes"
            </button>
            {move || export_status.get().map(|status| view! { <p class="muted" role="status">{status}</p> })}
        </main>
    }
}
