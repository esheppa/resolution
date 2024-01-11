use crate::DateResolution;
use chrono::Datelike;
#[cfg(feature = "with_serde")]
use serde::de;
use std::{convert::TryFrom, fmt, str};

const DATE_FORMAT: &str = "%b-%Y";

#[cfg(feature = "with_serde")]
impl<'de> de::Deserialize<'de> for Month {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Month, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let date = s.parse::<Month>().map_err(serde::de::Error::custom)?;
        Ok(date)
    }
}

#[cfg(feature = "with_serde")]
impl serde::Serialize for Month {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let s = self.to_string();
        serializer.serialize_str(&s)
    }
}

fn month_num_from_name(name: &str) -> Result<u32, crate::Error> {
    let num = match name {
        "Jan" => 1,
        "Feb" => 2,
        "Mar" => 3,
        "Apr" => 4,
        "May" => 5,
        "Jun" => 6,
        "Jul" => 7,
        "Aug" => 8,
        "Sep" => 9,
        "Oct" => 10,
        "Nov" => 11,
        "Dec" => 12,
        n => {
            return Err(crate::Error::ParseCustom {
                ty_name: "Month",
                input: format!("Unknown month name `{}`", n),
            })
        }
    };
    Ok(num)
}

impl str::FromStr for Month {
    type Err = crate::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut split = s.split('-');
        let month =
            month_num_from_name(split.next().ok_or_else(|| crate::Error::ParseCustom {
                ty_name: "Month",
                input: s.to_string(),
            })?)?;
        let year = split
            .next()
            .ok_or_else(|| crate::Error::ParseCustom {
                ty_name: "Month",
                input: s.to_string(),
            })?
            .parse()?;
        let date = chrono::NaiveDate::from_ymd_opt(year, month, 1).expect("valid datetime");
        Ok(date.into())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Month(i64); // number of months +- since 0AD

impl crate::TimeResolution for Month {
    fn succ_n(&self, n: u32) -> Self {
        Month(self.0 + i64::from(n))
    }
    fn pred_n(&self, n: u32) -> Self {
        Month(self.0 - i64::from(n))
    }
    fn start_datetime(&self) -> chrono::NaiveDateTime {
        self.start().and_hms_opt(0, 0, 0).expect("valid datetime")
    }

    fn name(&self) -> String {
        "Month".to_string()
    }
}

impl crate::Monotonic for Month {
    fn to_monotonic(&self) -> i64 {
        self.0
    }
    fn from_monotonic(idx: i64) -> Self {
        Month(idx)
    }
    fn between(&self, other: Self) -> i64 {
        other.0 - self.0
    }
}

impl crate::DateResolution for Month {
    fn start(&self) -> chrono::NaiveDate {
        let years = i32::try_from(self.0.div_euclid(12)).expect("Not pre/post historic");
        let months = u32::try_from(1 + self.0.rem_euclid(12)).expect("valid datetime");
        chrono::NaiveDate::from_ymd_opt(years, months, 1).expect("valid datetime")
    }
}

impl From<chrono::NaiveDate> for Month {
    fn from(d: chrono::NaiveDate) -> Self {
        Month(i64::from(d.month0()) + i64::from(d.year()) * 12)
    }
}

impl From<chrono::NaiveDateTime> for Month {
    fn from(d: chrono::NaiveDateTime) -> Self {
        d.date().into()
    }
}

impl Month {
    pub fn year(&self) -> super::Year {
        self.start().into()
    }
    pub fn quarter(&self) -> super::Quarter {
        self.start().into()
    }
    pub fn year_num(&self) -> i32 {
        self.start().year()
    }
    pub fn month_num(&self) -> u32 {
        self.start().month()
    }
    pub fn month(&self) -> chrono::Month {
        match self.month_num() {
            1 => chrono::Month::January,
            2 => chrono::Month::February,
            3 => chrono::Month::March,
            4 => chrono::Month::April,
            5 => chrono::Month::May,
            6 => chrono::Month::June,
            7 => chrono::Month::July,
            8 => chrono::Month::August,
            9 => chrono::Month::September,
            10 => chrono::Month::October,
            11 => chrono::Month::November,
            12 => chrono::Month::December,
            _ => unreachable!(),
        }
    }
}

impl fmt::Display for Month {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.start().format(DATE_FORMAT))
    }
}

#[cfg(test)]
mod tests {
    use super::Month;
    use crate::{DateResolution, DateResolutionExt, TimeResolution};

    #[test]
    fn test_roundtrip() {
        let dt = chrono::NaiveDate::from_ymd_opt(2021, 12, 6).unwrap();

        let m1 = Month::from(dt);
        assert!(m1.start() <= dt && m1.end() >= dt);

        let dt = chrono::NaiveDate::from_ymd_opt(2019, 7, 1).unwrap();

        let m2 = Month::from(dt);

        assert!(m2.start() == dt);

        assert_eq!(
            m1,
            serde_json::from_str(&serde_json::to_string(&m1).unwrap()).unwrap()
        )
    }

    #[test]
    fn test_parse() {
        assert_eq!(
            "Jan-2021".parse::<Month>().unwrap().start(),
            chrono::NaiveDate::from_ymd_opt(2021, 1, 1).unwrap(),
        );
        assert_eq!(
            "Jan-2021".parse::<Month>().unwrap().succ().start(),
            chrono::NaiveDate::from_ymd_opt(2021, 2, 1).unwrap(),
        );
        assert_eq!(
            "Jan-2021".parse::<Month>().unwrap().succ().pred().start(),
            chrono::NaiveDate::from_ymd_opt(2021, 1, 1).unwrap(),
        );
    }

    #[test]
    fn test_start() {
        assert_eq!(
            Month(24240).start(),
            chrono::NaiveDate::from_ymd_opt(2020, 1, 1).unwrap()
        );
        assert_eq!(
            Month(24249).start(),
            chrono::NaiveDate::from_ymd_opt(2020, 10, 1).unwrap()
        );
        assert_eq!(
            Month(15).start(),
            chrono::NaiveDate::from_ymd_opt(1, 4, 1).unwrap()
        );
        assert_eq!(
            Month(2).start(),
            chrono::NaiveDate::from_ymd_opt(0, 3, 1).unwrap()
        );
        assert_eq!(
            Month(1).start(),
            chrono::NaiveDate::from_ymd_opt(0, 2, 1).unwrap()
        );
        assert_eq!(
            Month(0).start(),
            chrono::NaiveDate::from_ymd_opt(0, 1, 1).unwrap()
        );
        assert_eq!(
            Month(-1).start(),
            chrono::NaiveDate::from_ymd_opt(-1, 12, 1).unwrap()
        );
        assert_eq!(
            Month(-2).start(),
            chrono::NaiveDate::from_ymd_opt(-1, 11, 1).unwrap()
        );
        assert_eq!(
            Month(-15).start(),
            chrono::NaiveDate::from_ymd_opt(-2, 10, 1).unwrap()
        );
    }
}
