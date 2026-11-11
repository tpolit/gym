use std::time::Duration;

use chrono::{DateTime, FixedOffset};
use leptos::prelude::*;

pub fn ticking_now() -> ReadSignal<DateTime<FixedOffset>> {
    let (now, set_now) = signal(web_platform::now());
    if let Ok(interval) = set_interval_with_handle(
        move || set_now.set(web_platform::now()),
        Duration::from_secs(1),
    ) {
        on_cleanup(move || interval.clear());
    }
    now
}
