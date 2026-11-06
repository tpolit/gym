use chrono::{DateTime, FixedOffset, NaiveDate, Offset, Utc};

// Time read from the JavaScript Date: chrono is built without `clock` across the project,
// so `Utc::now()` (which would panic in wasm) cannot be called anywhere.
pub fn now() -> DateTime<FixedOffset> {
    let date = js_sys::Date::new_0();
    // getTimezoneOffset is in minutes, positive west of Greenwich.
    let offset_seconds = date.get_timezone_offset() as i32 * 60;
    let offset = FixedOffset::west_opt(offset_seconds).unwrap_or(Utc.fix());
    DateTime::from_timestamp_millis(date.get_time() as i64)
        .unwrap_or_default()
        .with_timezone(&offset)
}

pub fn today() -> NaiveDate {
    now().date_naive()
}
