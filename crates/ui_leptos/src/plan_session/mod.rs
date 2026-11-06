mod create_page;
mod edit_page;

use leptos::prelude::*;

use crate::navigation::{Navigation, PlanSource};
use crate::store::Store;
use create_page::CreatePlanPage;
use edit_page::EditPlanPage;

#[component]
pub(crate) fn PlanSessionPage(
    store: Store,
    navigation: Navigation,
    source: PlanSource,
) -> impl IntoView {
    match source {
        PlanSource::New => view! { <CreatePlanPage store navigation template=None /> }.into_any(),
        PlanSource::From(template) => {
            view! { <CreatePlanPage store navigation template=Some(template) /> }.into_any()
        }
        PlanSource::Edit(session_id) => {
            view! { <EditPlanPage store navigation session_id /> }.into_any()
        }
    }
}
