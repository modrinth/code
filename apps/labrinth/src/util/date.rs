use chrono::Utc;

// this converts timestamps to the timestamp format clickhouse requires/uses
pub fn get_current_tenths_of_ms() -> i64 {
    Utc::now()
        .timestamp_nanos_opt()
        .expect("value can not be represented in a timestamp with nanosecond precision.")
        / 100_000
}
