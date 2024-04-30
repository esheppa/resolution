use crate::{month, DateResolution, DateResolutionExt};
use alloc::string::{String, ToString};
use chrono::{DateTime, Datelike, NaiveDate, NaiveTime, Utc};
use core::{convert::TryFrom, fmt, str};

#[derive(Clone, Copy, Debug, Eq, PartialOrd, PartialEq, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "serde", serde(transparent))]
pub struct Year(i64);

impl crate::DateResolution for Year {
    fn start(&self) -> chrono::NaiveDate {
        chrono::NaiveDate::from_ymd_opt(self.year_num(), 1, 1).expect("valid time")
    }
    type Params = ();

    fn params(&self) -> Self::Params {}

    fn from_date(d: NaiveDate, _params: Self::Params) -> Self {
        Year(i64::from(d.year()))
    }
}

impl From<NaiveDate> for Year {
    fn from(value: NaiveDate) -> Year {
        Year::from_date(value, ())
    }
}

impl crate::TimeResolution for Year {
    fn succ_n(&self, n: u64) -> Year {
        Year(self.0 + i64::try_from(n).unwrap())
    }
    fn pred_n(&self, n: u64) -> Year {
        Year(self.0 - i64::try_from(n).unwrap())
    }
    fn start_datetime(&self) -> DateTime<Utc> {
        self.start().and_time(NaiveTime::MIN).and_utc()
    }

    fn name(&self) -> String {
        "Year".to_string()
    }
}

impl crate::Monotonic for Year {
    fn to_monotonic(&self) -> i64 {
        self.0
    }
    fn between(&self, other: Self) -> i64 {
        other.0 - self.0
    }
}

impl crate::FromMonotonic for Year {
    fn from_monotonic(idx: i64) -> Self {
        Year(idx)
    }
}

impl From<DateTime<Utc>> for Year {
    fn from(d: DateTime<Utc>) -> Self {
        d.date_naive().into()
    }
}

impl Year {
    pub fn first_month(&self) -> month::Month {
        self.start().into()
    }
    pub fn first_quarter(&self) -> month::Month {
        self.start().into()
    }
    pub fn last_month(&self) -> month::Month {
        self.end().into()
    }
    pub fn last_quarter(&self) -> month::Month {
        self.end().into()
    }
    pub fn year_num(&self) -> i32 {
        i32::try_from(self.0).expect("Not pre/post historic")
    }
    pub fn new(year: i32) -> Self {
        Year(i64::from(year))
    }
}

impl fmt::Display for Year {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl str::FromStr for Year {
    type Err = crate::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Year(s.parse()?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{DateResolution, TimeResolution};

    #[test]
    #[cfg(feature = "serde")]
    fn test_roundtrip() {
        let dt = chrono::NaiveDate::from_ymd_opt(2021, 12, 6).unwrap();

        let yr = Year::from(dt);
        assert!(yr.start() <= dt && yr.end() >= dt);

        assert_eq!(
            yr,
            serde_json::from_str(&serde_json::to_string(&yr).unwrap()).unwrap()
        )
    }

    #[test]
    fn test_parse() {
        assert_eq!(
            "2021".parse::<Year>().unwrap().start(),
            chrono::NaiveDate::from_ymd_opt(2021, 1, 1).unwrap(),
        );
        assert_eq!(
            "2021".parse::<Year>().unwrap().succ().start(),
            chrono::NaiveDate::from_ymd_opt(2022, 1, 1).unwrap(),
        );
        assert_eq!(
            "2021".parse::<Year>().unwrap().succ().pred().start(),
            chrono::NaiveDate::from_ymd_opt(2021, 1, 1).unwrap(),
        );

        assert!("a2021".parse::<Year>().is_err(),);
    }
}
