use chrono::{DateTime, Datelike, NaiveDate, NaiveDateTime, NaiveTime, Utc};

pub fn get_claimable_time(
    current: DateTime<Utc>,
    future: bool,
) -> DateTime<Utc> {
    let adder = if current.month() == 1 && !future {
        (-1, 12)
    } else if current.month() == 12 && future {
        (1, 1)
    } else {
        (0, current.month())
    };

    DateTime::from_utc(
        NaiveDateTime::new(
            NaiveDate::from_ymd(current.year() + adder.0, adder.1, 16),
            NaiveTime::default(),
        ),
        Utc,
    )
}
