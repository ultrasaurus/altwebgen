use chrono::format::{Fixed, Item, Numeric, StrftimeItems};
use chrono::{DateTime, FixedOffset, NaiveDate, NaiveDateTime};
use serde_json::Value;
use std::fmt::Write;

// how much of a date is known, e.g. a publication may only have a year,
// while a blog post has a full timestamp. ordered least to most precise.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Precision {
    Year,
    Month,
    Day,
    DateTime,
}

// a frontmatter date, as precise as it was written.
// field order matters: derived Ord sorts by start of period, then less
// precise before more precise (1984 < 1984-10 < 1984-10-01), then offset
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct FmDate {
    // start of the period the date names, as clock time where it was written
    start: NaiveDateTime,
    precision: Precision,
    // UTC offset in seconds, if one was given
    offset_secs: Option<i32>,
}

fn all_digits(s: &str, len: usize) -> bool {
    s.len() == len && s.bytes().all(|b| b.is_ascii_digit())
}

impl FmDate {
    // a date from a frontmatter value: a year as a number (1984), or a string
    pub fn from_value(value: &Value) -> Option<FmDate> {
        match value {
            Value::Number(n) => FmDate::parse(&n.as_u64()?.to_string()),
            Value::String(s) => FmDate::parse(s.trim()),
            _ => None,
        }
    }

    // parses "1984", "1984-10", "2025-11-08", or a date and time with an
    // optional UTC offset, e.g. "2003-03-03T15:17:38-08:00"
    pub fn parse(s: &str) -> Option<FmDate> {
        let parts: Vec<&str> = s.split('-').collect();
        let (y, m, d, precision) = match parts.as_slice() {
            [y] if all_digits(y, 4) => (*y, "1", "1", Precision::Year),
            [y, m] if all_digits(y, 4) && all_digits(m, 2) => (*y, *m, "1", Precision::Month),
            [y, m, d] if all_digits(y, 4) && all_digits(m, 2) && all_digits(d, 2) => {
                (*y, *m, *d, Precision::Day)
            }
            _ => return FmDate::parse_datetime(s),
        };
        let date = NaiveDate::from_ymd_opt(y.parse().ok()?, m.parse().ok()?, d.parse().ok()?)?;
        Some(FmDate {
            start: date.and_hms_opt(0, 0, 0)?,
            precision,
            offset_secs: None,
        })
    }

    fn parse_datetime(s: &str) -> Option<FmDate> {
        let with_offset = DateTime::parse_from_rfc3339(s)
            .or_else(|_| DateTime::parse_from_str(s, "%Y-%m-%dT%H:%M%:z"))
            .or_else(|_| DateTime::parse_from_str(s, "%Y-%m-%d %H:%M%:z"));
        if let Ok(dt) = with_offset {
            return Some(FmDate {
                start: dt.naive_local(),
                precision: Precision::DateTime,
                offset_secs: Some(dt.offset().local_minus_utc()),
            });
        }
        [
            "%Y-%m-%dT%H:%M:%S%.f",
            "%Y-%m-%dT%H:%M",
            "%Y-%m-%d %H:%M:%S%.f",
            "%Y-%m-%d %H:%M",
        ]
        .iter()
        .find_map(|fmt| NaiveDateTime::parse_from_str(s, fmt).ok())
        .map(|start| FmDate {
            start,
            precision: Precision::DateTime,
            offset_secs: None,
        })
    }

    // display used when no format is given, or when the given format needs
    // more precision than this date has
    pub fn default_format(&self) -> &'static str {
        match self.precision {
            Precision::Year => "%Y",
            Precision::Month => "%B %Y",
            Precision::Day | Precision::DateTime => "%B %-d, %Y",
        }
    }

    // formats with a strftime string (or the default for this precision).
    // errors only if the format string itself is invalid.
    pub fn format(&self, fmt: Option<&str>) -> anyhow::Result<String> {
        let mut items: Vec<Item> = match fmt {
            Some(fmt) => StrftimeItems::new(fmt).collect(),
            None => StrftimeItems::new(self.default_format()).collect(),
        };
        if items.iter().any(|item| matches!(item, Item::Error)) {
            anyhow::bail!("invalid date format: '{}'", fmt.unwrap_or_default());
        }
        let (needs, needs_offset) = required_precision(&items);
        if needs > self.precision || (needs_offset && self.offset_secs.is_none()) {
            items = StrftimeItems::new(self.default_format()).collect();
        }

        let mut out = String::new();
        let offset = self.offset_secs.and_then(FixedOffset::east_opt);
        let written = match offset.and_then(|o| self.start.and_local_timezone(o).single()) {
            Some(dt) => write!(out, "{}", dt.format_with_items(items.iter())),
            None => write!(out, "{}", self.start.format_with_items(items.iter())),
        };
        written
            .map_err(|_| anyhow::anyhow!("could not format date: '{}'", fmt.unwrap_or_default()))?;
        Ok(out)
    }
}

// the least precise date that a format can display, and whether it shows
// a UTC offset or timezone
fn required_precision(items: &[Item]) -> (Precision, bool) {
    let mut needs = Precision::Year;
    let mut needs_offset = false;
    for item in items {
        let item_needs = match item {
            Item::Numeric(numeric, _) => match numeric {
                Numeric::Year | Numeric::YearDiv100 | Numeric::YearMod100 => Precision::Year,
                Numeric::Month => Precision::Month,
                Numeric::Day
                | Numeric::Ordinal
                | Numeric::IsoYear
                | Numeric::IsoYearDiv100
                | Numeric::IsoYearMod100
                | Numeric::IsoWeek
                | Numeric::WeekFromSun
                | Numeric::WeekFromMon
                | Numeric::NumDaysFromSun
                | Numeric::WeekdayFromMon => Precision::Day,
                _ => Precision::DateTime,
            },
            Item::Fixed(fixed) => match fixed {
                Fixed::ShortMonthName | Fixed::LongMonthName => Precision::Month,
                Fixed::ShortWeekdayName | Fixed::LongWeekdayName => Precision::Day,
                Fixed::TimezoneName
                | Fixed::TimezoneOffset
                | Fixed::TimezoneOffsetColon
                | Fixed::TimezoneOffsetDoubleColon
                | Fixed::TimezoneOffsetTripleColon
                | Fixed::TimezoneOffsetColonZ
                | Fixed::TimezoneOffsetZ
                | Fixed::RFC2822
                | Fixed::RFC3339 => {
                    needs_offset = true;
                    Precision::DateTime
                }
                _ => Precision::DateTime,
            },
            _ => Precision::Year,
        };
        needs = needs.max(item_needs);
    }
    (needs, needs_offset)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn date(s: &str) -> FmDate {
        FmDate::parse(s).unwrap()
    }

    #[test]
    fn parse_each_precision() {
        assert_eq!(date("1984").precision, Precision::Year);
        assert_eq!(date("1984-10").precision, Precision::Month);
        assert_eq!(date("2025-11-08").precision, Precision::Day);
        assert_eq!(
            date("2003-03-03T15:17:38-08:00").precision,
            Precision::DateTime
        );
        assert_eq!(date("2003-03-03T15:17").precision, Precision::DateTime);
        assert_eq!(date("2003-03-03 15:17:38").precision, Precision::DateTime);
    }

    #[test]
    fn parse_rejects_invalid() {
        assert!(FmDate::parse("Nov. 8, 2025").is_none());
        assert!(FmDate::parse("84").is_none());
        assert!(FmDate::parse("1984-13").is_none());
        assert!(FmDate::parse("2025-02-30").is_none());
        assert!(FmDate::parse("").is_none());
    }

    #[test]
    fn from_value_year_number_and_string() {
        assert_eq!(FmDate::from_value(&json!(1984)), Some(date("1984")));
        assert_eq!(
            FmDate::from_value(&json!(" 1984-10 ")),
            Some(date("1984-10"))
        );
        assert!(FmDate::from_value(&json!(["1984"])).is_none());
        assert!(FmDate::from_value(&json!(null)).is_none());
    }

    #[test]
    fn default_display_per_precision() {
        assert_eq!(date("1984").format(None).unwrap(), "1984");
        assert_eq!(date("1984-10").format(None).unwrap(), "October 1984");
        assert_eq!(date("2025-11-08").format(None).unwrap(), "November 8, 2025");
        assert_eq!(
            date("2003-03-03T15:17:38-08:00").format(None).unwrap(),
            "March 3, 2003"
        );
    }

    #[test]
    fn custom_format() {
        assert_eq!(
            date("2025-11-08").format(Some("%b %-d, %Y")).unwrap(),
            "Nov 8, 2025"
        );
        assert_eq!(
            date("2003-03-03T15:17:38-08:00")
                .format(Some("%Y-%m-%d %H:%M %z"))
                .unwrap(),
            "2003-03-03 15:17 -0800"
        );
    }

    #[test]
    fn custom_format_falls_back_when_too_precise() {
        // a day for a month-only date
        assert_eq!(
            date("1984-10").format(Some("%-d %B %Y")).unwrap(),
            "October 1984"
        );
        // a month for a year-only date
        assert_eq!(date("1984").format(Some("%b %Y")).unwrap(), "1984");
        // a time for a day-only date
        assert_eq!(
            date("2025-11-08").format(Some("%B %-d, %Y %H:%M")).unwrap(),
            "November 8, 2025"
        );
        // an offset for a time written without one
        assert_eq!(
            date("2003-03-03T15:17").format(Some("%H:%M %z")).unwrap(),
            "March 3, 2003"
        );
        // year-only formats work for any precision
        assert_eq!(date("2025-11-08").format(Some("%Y")).unwrap(), "2025");
    }

    #[test]
    fn invalid_format_is_an_error() {
        assert!(date("1984").format(Some("%Q")).is_err());
    }

    #[test]
    fn sort_less_precise_first_within_period() {
        let mut dates = [
            date("1984-10-01"),
            date("1985"),
            date("1984-10"),
            date("1984-01-01"),
            date("1984"),
            date("1984-01"),
        ];
        dates.sort();
        let written: Vec<String> = dates
            .iter()
            .map(|d| d.format(Some("%Y-%m-%d")).unwrap())
            .collect();
        assert_eq!(
            written,
            vec![
                "1984",
                "January 1984",
                "1984-01-01",
                "October 1984",
                "1984-10-01",
                "1985"
            ]
        );
    }

    #[test]
    fn sort_same_day_by_time() {
        let morning = date("2009-04-17T08:00:00-07:00");
        let evening = date("2009-04-17T20:00:00-07:00");
        let day = date("2009-04-17");
        assert!(day < morning);
        assert!(morning < evening);
    }
}
