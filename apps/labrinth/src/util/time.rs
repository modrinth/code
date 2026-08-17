use std::{fmt, str::FromStr};

use chrono::{DateTime, Datelike, Days, Months, NaiveDate, Utc};
use serde::{Deserialize, Deserializer, Serialize, Serializer, de::Error};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct YearMonth(NaiveDate);

impl YearMonth {
    pub fn from_day1(date: NaiveDate) -> Self {
        Self(date.with_day(1).expect("every month has a first day"))
    }

    pub fn from_year_month(year: i32, month: u32) -> Option<Self> {
        NaiveDate::from_ymd_opt(year, month, 1).map(Self)
    }

    #[must_use]
    pub fn date(self) -> NaiveDate {
        self.0
    }
}

/// Calculate when a payout period becomes available under Net-60 terms.
pub fn net_60_payout_available_at(period: YearMonth) -> Option<DateTime<Utc>> {
    period
        .date()
        .checked_add_months(Months::new(1))?
        .and_hms_opt(0, 0, 0)?
        .and_utc()
        .checked_add_days(Days::new(59))
}

impl fmt::Display for YearMonth {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.0.format("%Y-%m"))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("expected a valid year and month in `YYYY-MM` format")]
pub struct ParseYearMonthError;

impl FromStr for YearMonth {
    type Err = ParseYearMonthError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let mut segments = value.split('-');
        let (Some(year), Some(month), None) =
            (segments.next(), segments.next(), segments.next())
        else {
            return Err(ParseYearMonthError);
        };

        let year = year.parse().map_err(|_| ParseYearMonthError)?;
        let month = month.parse().map_err(|_| ParseYearMonthError)?;
        Self::from_year_month(year, month).ok_or(ParseYearMonthError)
    }
}

impl Serialize for YearMonth {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.collect_str(self)
    }
}

impl<'de> Deserialize<'de> for YearMonth {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        value.parse().map_err(D::Error::custom)
    }
}

impl utoipa::PartialSchema for YearMonth {
    fn schema() -> utoipa::openapi::RefOr<utoipa::openapi::schema::Schema> {
        utoipa::openapi::ObjectBuilder::new()
            .schema_type(utoipa::openapi::schema::Type::String)
            .pattern(Some(r"^\d{4}-(0[1-9]|1[0-2])$"))
            .into()
    }
}

impl utoipa::ToSchema for YearMonth {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constructs_from_year_and_month() {
        let year_month = YearMonth::from_year_month(2026, 7).unwrap();

        assert_eq!(
            year_month.date(),
            NaiveDate::from_ymd_opt(2026, 7, 1).unwrap()
        );
        assert!(YearMonth::from_year_month(2026, 13).is_none());
    }

    #[test]
    fn constructs_from_date_using_first_day() {
        let date = NaiveDate::from_ymd_opt(2026, 7, 20).unwrap();

        assert_eq!(
            YearMonth::from_day1(date).date(),
            NaiveDate::from_ymd_opt(2026, 7, 1).unwrap()
        );
    }

    #[test]
    fn serializes_as_year_and_month() {
        let date = NaiveDate::from_ymd_opt(2026, 7, 1).unwrap();
        let year_month = YearMonth::from_day1(date);

        assert_eq!(serde_json::to_string(&year_month).unwrap(), r#""2026-07""#);
    }

    #[test]
    fn deserializes_year_and_month_to_the_first() {
        let year_month: YearMonth =
            serde_json::from_str(r#""2026-07""#).unwrap();

        assert_eq!(
            year_month.date(),
            NaiveDate::from_ymd_opt(2026, 7, 1).unwrap()
        );
    }

    #[test]
    fn parses_year_and_month_to_the_first() {
        let year_month = YearMonth::from_str("2026-7").unwrap();

        assert_eq!(
            year_month.date(),
            NaiveDate::from_ymd_opt(2026, 7, 1).unwrap()
        );
        assert_eq!(year_month.to_string(), "2026-07");
    }

    #[test]
    fn rejects_other_serialized_formats() {
        for value in [r#""2026""#, r#""2026-07-01""#, r#""2026-13""#] {
            assert!(serde_json::from_str::<YearMonth>(value).is_err());
        }
    }

    #[test]
    fn calculates_net_60_payout_availability() {
        let august = YearMonth::from_year_month(2026, 8).unwrap();
        let december = YearMonth::from_year_month(2026, 12).unwrap();

        assert_eq!(
            net_60_payout_available_at(august),
            Some(
                NaiveDate::from_ymd_opt(2026, 10, 30)
                    .unwrap()
                    .and_hms_opt(0, 0, 0)
                    .unwrap()
                    .and_utc()
            )
        );
        assert_eq!(
            net_60_payout_available_at(december),
            Some(
                NaiveDate::from_ymd_opt(2027, 3, 1)
                    .unwrap()
                    .and_hms_opt(0, 0, 0)
                    .unwrap()
                    .and_utc()
            )
        );
    }
}
