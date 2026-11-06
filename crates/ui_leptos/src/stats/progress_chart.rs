use features::stats::ProgressPoint;
use leptos::prelude::*;

// Integer coordinates: one unit is about one pixel, and formatting floats would add
// several kilobytes to the .wasm.
const WIDTH: u32 = 340;
const HEIGHT: u32 = 180;
const PADDING: u32 = 20;
const RECORD_LABEL_OFFSET: u32 = 18;

// Monochrome curve: the record is shown by its shape (bigger point with a halo and a
// "Record" label), never by another color. Tapping a point shows its date and value.
#[component]
pub fn ProgressChart(points: Vec<ProgressPoint>) -> impl IntoView {
    let selected = RwSignal::new(points.len().checked_sub(1));
    let lowest = points.iter().map(|point| point.value).min().unwrap_or(0);
    let highest = points.iter().map(|point| point.value).max().unwrap_or(0);
    let span = (highest - lowest).max(1);
    let last_index = points.len().saturating_sub(1) as u64;
    let coordinates: Vec<(u32, u32)> = points
        .iter()
        .enumerate()
        .map(|(index, point)| {
            let x = if last_index > 0 {
                PADDING + scale(index as u64, last_index, WIDTH - 2 * PADDING)
            } else {
                WIDTH / 2
            };
            let y = HEIGHT - PADDING - scale(point.value - lowest, span, HEIGHT - 2 * PADDING);
            (x, y)
        })
        .collect();
    let path = coordinates
        .iter()
        .enumerate()
        .map(|(index, (x, y))| format!("{}{x},{y}", if index == 0 { "M" } else { " L" }))
        .collect::<String>();
    let points = StoredValue::new(points);

    let dots = coordinates
        .iter()
        .copied()
        .enumerate()
        .map(|(index, (x, y))| {
            let is_record = points.with_value(|points| points[index].is_record);
            let label_y = y - RECORD_LABEL_OFFSET;
            view! {
                <g class="chart__point" on:click=move |_| selected.set(Some(index))>
                    <circle class="chart__hit" cx=x cy=y r="16" />
                    {is_record.then(|| view! {
                        <circle class="chart__halo" cx=x cy=y r="12" />
                        <text class="chart__record-label" x=x y=label_y text-anchor="middle">"Record"</text>
                    })}
                    <circle
                        class="chart__dot"
                        class:chart__dot--selected=move || selected.get() == Some(index)
                        cx=x
                        cy=y
                        r=if is_record { "7" } else { "4" }
                    />
                </g>
            }
        })
        .collect_view();

    view! {
        <svg class="chart" viewBox=format!("0 0 {WIDTH} {HEIGHT}") role="img" aria-label="Courbe de progression">
            <path class="chart__line" d=path />
            {dots}
        </svg>
        <p class="chart__bubble">
            {move || {
                selected
                    .get()
                    .and_then(|index| points.with_value(|points| points.get(index).cloned()))
                    .map(|point| format!("{} · {}", point.date_text, point.value_text))
            }}
        </p>
    }
}

fn scale(value: u64, max: u64, length: u32) -> u32 {
    u32::try_from(value.saturating_mul(u64::from(length)) / max).unwrap_or(length)
}
