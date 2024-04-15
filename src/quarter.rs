use crate::{month, year, DateResolution, DateResolutionExt};
use alloc::{
    fmt, str,
    string::{String, ToString},
    vec::Vec,
};
use chrono::Datelike;
use core::convert::TryFrom;
#[cfg(feature = "serde")]
use serde::de;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Quarter(i64);

impl crate::TimeResolution for Quarter {
    fn succ_n(&self, n: u32) -> Self {
        Quarter(self.0 + i64::from(n))
    }
    fn pred_n(&self, n: u32) -> Self {
        Quarter(self.0 - i64::from(n))
    }
    fn start_datetime(&self) -> chrono::NaiveDateTime {
        self.start().and_hms_opt(0, 0, 0).expect("valid time")
    }

    fn name(&self) -> String {
        "Quarter".to_string()
    }
}

impl crate::Monotonic for Quarter {
    fn from_monotonic(idx: i64) -> Self {
        Quarter(idx)
    }
    fn to_monotonic(&self) -> i64 {
        self.0
    }
    fn between(&self, other: Self) -> i64 {
        other.0 - self.0
    }
}

impl crate::DateResolution for Quarter {
    fn start(&self) -> chrono::NaiveDate {
        let years = i32::try_from(self.0.div_euclid(4)).expect("Not pre/post historic");
        let qtr = self.quarter_num();
        chrono::NaiveDate::from_ymd_opt(years, qtr * 3 - 2, 1).expect("valid time")
    }
}

fn quarter_num(d: chrono::NaiveDate) -> i64 {
    match d.month() {
        1..=3 => 1,
        4..=6 => 2,
        7..=9 => 3,
        10..=12 => 4,
        mn => panic!("Unexpected month number {}", mn),
    }
}

impl From<chrono::NaiveDate> for Quarter {
    fn from(d: chrono::NaiveDate) -> Self {
        Quarter(quarter_num(d) - 1 + i64::from(d.year()) * 4)
    }
}

impl From<chrono::NaiveDateTime> for Quarter {
    fn from(d: chrono::NaiveDateTime) -> Self {
        d.date().into()
    }
}

impl Quarter {
    pub fn first_month(&self) -> month::Month {
        self.start().into()
    }
    pub fn last_month(&self) -> month::Month {
        self.end().into()
    }
    pub fn year(&self) -> year::Year {
        super::Year::new(self.year_num())
    }
    pub fn year_num(&self) -> i32 {
        self.start().year()
    }
    pub fn quarter_num(&self) -> u32 {
        u32::try_from(1 + self.0.rem_euclid(4)).expect("Range of 1-4")
    }
}

impl fmt::Display for Quarter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Q{}-{}", self.quarter_num(), self.year_num())
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

        let wk = Quarter::from(dt);
        assert!(wk.start() <= dt && wk.end() >= dt);

        assert_eq!(
            wk,
            serde_json::from_str(&serde_json::to_string(&wk).unwrap()).unwrap()
        )
    }
    #[test]
    fn test_parse_quarter_syntax() {
        assert_eq!(
            "Q1-2021".parse::<Quarter>().unwrap().start(),
            chrono::NaiveDate::from_ymd_opt(2021, 1, 1).unwrap(),
        );
        assert_eq!(
            "Q1-2021".parse::<Quarter>().unwrap().succ().start(),
            chrono::NaiveDate::from_ymd_opt(2021, 4, 1).unwrap(),
        );
        assert_eq!(
            "Q1-2021".parse::<Quarter>().unwrap().succ().pred().start(),
            chrono::NaiveDate::from_ymd_opt(2021, 1, 1).unwrap(),
        );
    }

    #[test]
    fn test_parse_date_syntax() {
        assert_eq!(
            "2021-01-01".parse::<Quarter>().unwrap().start(),
            chrono::NaiveDate::from_ymd_opt(2021, 1, 1).unwrap(),
        );
        assert_eq!(
            "2021-01-01".parse::<Quarter>().unwrap().succ().start(),
            chrono::NaiveDate::from_ymd_opt(2021, 4, 1).unwrap(),
        );
        assert_eq!(
            "2021-01-01"
                .parse::<Quarter>()
                .unwrap()
                .succ()
                .pred()
                .start(),
            chrono::NaiveDate::from_ymd_opt(2021, 1, 1).unwrap(),
        );
    }

    #[test]
    fn test_start() {
        assert_eq!(
            Quarter(2).start(),
            chrono::NaiveDate::from_ymd_opt(0, 7, 1).unwrap()
        );
        assert_eq!(
            Quarter(1).start(),
            chrono::NaiveDate::from_ymd_opt(0, 4, 1).unwrap()
        );
        assert_eq!(
            Quarter(0).start(),
            chrono::NaiveDate::from_ymd_opt(0, 1, 1).unwrap()
        );
        assert_eq!(
            Quarter(-1).start(),
            chrono::NaiveDate::from_ymd_opt(-1, 10, 1).unwrap()
        );
        assert_eq!(
            Quarter(-2).start(),
            chrono::NaiveDate::from_ymd_opt(-1, 7, 1).unwrap()
        );
    }
}

#[cfg(feature = "serde")]
impl<'de> de::Deserialize<'de> for Quarter {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Quarter, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let date = s.parse::<Quarter>().map_err(serde::de::Error::custom)?;
        Ok(date)
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for Quarter {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let s = self.to_string();
        serializer.serialize_str(&s)
    }
}

impl str::FromStr for Quarter {
    type Err = crate::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(parsed) = chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d") {
            Ok(parsed.into())
        } else {
            let split = s
                .split('-')
                .map(ToString::to_string)
                .collect::<Vec<String>>();
            if split.len() == 2 {
                let qtr = split[0]
                    .chars()
                    .nth(1)
                    .unwrap()
                    .to_string()
                    .parse::<u32>()?;
                let year = split[1].parse()?;
                let date =
                    chrono::NaiveDate::from_ymd_opt(year, qtr * 3 - 2, 1).expect("valid date");
                Ok(date.into())
            } else {
                Err(crate::Error::ParseCustom {
                    ty_name: "Quarter",
                    input: s.to_string(),
                })
            }
        }
    }
}
