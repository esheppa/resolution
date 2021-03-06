use crate::DateResolution;
use serde::{
    de,
    ser::{self, SerializeStruct},
};
use std::{str, fmt};

const DATE_FORMAT: &str = "%Y-%m-%d";

impl<'de> de::Deserialize<'de> for Date 
{
    fn deserialize<D>(
        deserializer: D,
    ) -> std::result::Result<Date, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let date = chrono::NaiveDate::parse_from_str(&s, DATE_FORMAT)
            .map_err(serde::de::Error::custom)?;
        Ok(date.into())
    }
}

impl serde::Serialize for Date {
    fn serialize<S>(
        &self,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let s = self.to_string();
        serializer.serialize_str(&s)
    }
}




#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct Date(i64);

fn base() -> chrono::NaiveDate {
    chrono::NaiveDate::from_ymd(0, 1, 1)
}


impl str::FromStr for Date {
    type Err = crate::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let date = chrono::NaiveDate::parse_from_str(s, DATE_FORMAT)?;
        Ok(date.into())
    }
}


impl fmt::Display for Date {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.start())
    }
}

impl crate::DateResolution for Date {
    fn start(&self) -> chrono::NaiveDate {
        base() + chrono::Duration::days(self.0)
    }
}

impl std::convert::From<chrono::NaiveDate> for Date {
    fn from(d: chrono::NaiveDate) -> Date {
        Date((base() - d).num_days())
    }
}

impl crate::TimeResolution for Date {
    fn between(&self, other: Self) -> i64 {
        other.0 - self.0
    }
    fn succ_n(&self, n: u32) -> Date {
        Date(self.0 + i64::from(n))
    }
    fn pred_n(&self, n: u32) -> Date {
        Date(self.0 - i64::from(n))
    }
    fn naive_date_time(&self) -> chrono::NaiveDateTime {
        self.start().and_hms(0, 0, 0)
    }
    fn to_monotonic(&self) -> i64 {
        self.0
    }
    fn from_monotonic(idx: i64) -> Self {
        Date(idx)
    }
}

impl Date {}
