use leptos::ev;
use leptos::prelude::*;

// Roughly the height of a page heading: past it, the heading has scrolled out of view.
const SCROLLED_PAST_HEADING_PX: f64 = 48.0;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum TitleVisibility {
    Always,
    // The page shows the same title as a large heading: repeat it only once scrolled away.
    OnceScrolled,
}

// Stays at the top of non-tab screens while scrolling.
#[component]
pub fn TopBar(
    on_back: Callback<()>,
    #[prop(optional, into)] title: MaybeProp<String>,
    #[prop(default = TitleVisibility::Always)] title_visibility: TitleVisibility,
    #[prop(optional)] action: Option<(&'static str, Callback<()>)>,
) -> impl IntoView {
    let scroll_y = || window().scroll_y().unwrap_or_default();
    let scrolled = RwSignal::new(scroll_y() > SCROLLED_PAST_HEADING_PX);
    let listener = window_event_listener(ev::scroll, move |_| {
        scrolled.set(scroll_y() > SCROLLED_PAST_HEADING_PX);
    });
    on_cleanup(move || listener.remove());
    let reveals_title = title_visibility == TitleVisibility::OnceScrolled;

    view! {
        <header class="topbar" class:topbar--scrolled=move || scrolled.get()>
            <button type="button" class="icon-btn" aria-label="Retour" on:click=move |_| on_back.run(())>
                "‹"
            </button>
            <p
                class="topbar__title"
                class:topbar__title--hidden=move || reveals_title && !scrolled.get()
            >
                {move || title.get()}
            </p>
            {match action {
                Some((label, on_action)) => view! {
                    <button type="button" class="topbar__action" on:click=move |_| on_action.run(())>
                        {label}
                    </button>
                }
                    .into_any(),
                None => view! { <span class="icon-btn icon-btn--placeholder"></span> }.into_any(),
            }}
        </header>
    }
}
