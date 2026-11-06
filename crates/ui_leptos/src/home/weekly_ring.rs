use features::home::{set_weekly_goal, weekly_goal_choices, weekly_progress};
use leptos::prelude::*;

use crate::store::Store;

const RING_RADIUS: u32 = 54;
// 2π × 54 ≈ 339.3, rounded up so that 100 % closes the ring; integers avoid float
// formatting in the .wasm.
const RING_CIRCUMFERENCE: u32 = 340;

// Ring of the weekly goal; tapping it opens the goal choice.
#[component]
pub fn WeeklyRing(store: Store) -> impl IntoView {
    let progress = Memo::new(move |_| store.with(|db| weekly_progress(db, web_platform::today())));
    let choosing_goal = RwSignal::new(false);
    let arc_length =
        move || progress.with(|progress| progress.percent.min(100)) * RING_CIRCUMFERENCE / 100;

    let choose = move |goal: Option<u8>| {
        let _ = store.execute(|db| set_weekly_goal(db, goal));
        choosing_goal.set(false);
    };

    let goal_sheet = move || {
        choosing_goal.get().then(|| {
            let current = progress.with_untracked(|progress| progress.goal);
            view! {
                <div class="sheet-backdrop" on:click=move |_| choosing_goal.set(false)></div>
                <div class="sheet" role="dialog" aria-label="Objectif de la semaine">
                    <p class="eyebrow">"Séances par semaine"</p>
                    <div class="weekly__choices">
                        {weekly_goal_choices()
                            .into_iter()
                            .map(|count| view! {
                                <button
                                    type="button"
                                    class="weekly__choice"
                                    class:weekly__choice--current=current == Some(count)
                                    on:click=move |_| choose(Some(count))
                                >
                                    {count}
                                </button>
                            })
                            .collect_view()}
                    </div>
                    <button type="button" class="btn btn--secondary" on:click=move |_| choose(None)>
                        "Pas d'objectif"
                    </button>
                </div>
            }
        })
    };

    view! {
        <button type="button" class="card weekly" on:click=move |_| choosing_goal.set(true)>
            {move || {
                let has_goal = progress.with(|progress| progress.goal.is_some());
                has_goal.then(|| view! {
                    <svg class="weekly__ring" viewBox="0 0 132 132" aria-hidden="true">
                        <circle class="weekly__track" cx="66" cy="66" r=RING_RADIUS />
                        <circle
                            class="weekly__arc"
                            cx="66"
                            cy="66"
                            r=RING_RADIUS
                            stroke-dasharray=move || format!("{} {RING_CIRCUMFERENCE}", arc_length())
                            transform="rotate(-90 66 66)"
                        />
                        <text class="weekly__count" x="66" y="74" text-anchor="middle">
                            {move || progress.with(|progress| progress.done_count)}
                        </text>
                    </svg>
                })
            }}
            <span class="weekly__text">
                <span class="weekly__label">{move || progress.with(|progress| progress.label.clone())}</span>
                {move || progress.with(|progress| progress.streak_text.clone()).map(|text| view! { <span class="weekly__streak">{text}</span> })}
                {move || progress.with(|progress| progress.volume_text.clone()).map(|text| view! { <span class="muted">{text}</span> })}
                {move || progress.with(|progress| progress.goal.is_none()).then(|| view! {
                    <span class="weekly__set-goal">"Définir un objectif"</span>
                })}
            </span>
        </button>
        {goal_sheet}
    }
}
