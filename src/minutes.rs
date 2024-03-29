use crate::{Monotonic, TimeResolution};
use chrono::Timelike;
use std::{fmt, str};

const NUM_SECS: i64 = 60;

const PARSE_FORMAT: &str = "%Y-%m-%d %H:%M";

/// Note that for sensible behaviour, the N chosen should be a number that either:
/// 1. divides into an hour with no remainder (1, 2, 3, 4, 5, 6, 10, 12, 15, 20, 30, 60)
/// 2. is exactly a whole number of hours that divides into a day with no remainder (60, 120, 180, 240, 360, 480, 1800)
/// Any other choice will result in unexpected / unuseful behaviour (eg the `Minutes` not cleanly fitting into parts of a day)
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[cfg_attr(feature = "with_serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(
    feature = "with_serde",
    serde(try_from = "Minutes_", into = "Minutes_")
)]
pub struct Minutes<const N: u32> {
    index: i64,
}

// #[cfg(not(with_serde))]
// #[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
// pub struct Minutes<const N: u32> {
//     index: i64,
// }

impl<const N: u32> TryFrom<Minutes_> for Minutes<N> {
    type Error = String;
    fn try_from(value: Minutes_) -> Result<Self, Self::Error> {
        if value.length == N {
            Ok(Minutes { index: value.index })
        } else {
            Err(format!(
                "To create a Minutes[Length:{}], the length field should be {} but was instead {}",
                N, N, value.length
            ))
        }
    }
}

impl<const N: u32> From<Minutes<N>> for Minutes_ {
    fn from(w: Minutes<N>) -> Self {
        Minutes_ {
            index: w.index,
            length: N,
        }
    }
}

#[cfg_attr(feature = "with_serde", derive(serde::Deserialize, serde::Serialize))]
pub(crate) struct Minutes_ {
    index: i64,
    pub(crate) length: u32,
}

impl<const N: u32> From<chrono::NaiveDateTime> for Minutes<N> {
    fn from(d: chrono::NaiveDateTime) -> Self {
        Minutes {
            index: d.timestamp().div_euclid(60 * i64::from(N)),
            // index: d.timestamp() / (60 * i64::from(N)),
        }
    }
}

impl<const N: u32> str::FromStr for Minutes<N> {
    type Err = crate::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if N == 1 {
            let time = chrono::NaiveDateTime::parse_from_str(s, PARSE_FORMAT)?;
            if time.second() != 0 {
                Err(crate::Error::ParseCustom {
                    ty_name: "Minutes",
                    input: s.into(),
                })
            } else {
                Ok(time.into())
            }
        } else {
            let mut splits = s.split(" => ");

            let start = splits.next().ok_or_else(|| crate::Error::ParseCustom {
                ty_name: "Minutes",
                input: s.into(),
            })?;

            let end = splits.next().ok_or_else(|| crate::Error::ParseCustom {
                ty_name: "Minutes",
                input: s.into(),
            })?;

            let start = chrono::NaiveDateTime::parse_from_str(start, PARSE_FORMAT)?;

            if (start.hour() * 60 + start.minute()).rem_euclid(N) != 0 {
                return Err(crate::Error::ParseCustom {
                    ty_name: "Minutes",
                    input: format!("Invalid start for Minutes[Length:{}]: {}", N, start,),
                });
            }
            let end = chrono::NaiveDateTime::parse_from_str(end, PARSE_FORMAT)?;

            if (end - start).num_minutes() + 1 != i64::from(N) {
                return Err(crate::Error::ParseCustom {
                    ty_name: "Minutes",
                    input: format!(
                        "Invalid start-end combination for Minutes[Length:{}]: {}",
                        N, s
                    ),
                });
            }

            Ok(start.into())
        }
    }
}

impl<const N: u32> fmt::Display for Minutes<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if N == 1 {
            write!(f, "{}", self.start_datetime())
        } else {
            write!(
                f,
                "{} => {}",
                self.start_datetime(),
                self.succ().start_datetime()
            )
        }
    }
}

impl<const N: u32> crate::TimeResolution for Minutes<N> {
    fn succ_n(&self, n: u32) -> Minutes<N> {
        Minutes {
            index: self.index + i64::from(n),
        }
    }
    fn pred_n(&self, n: u32) -> Minutes<N> {
        Minutes {
            index: self.index - i64::from(n),
        }
    }
    fn start_datetime(&self) -> chrono::NaiveDateTime {
        chrono::NaiveDateTime::from_timestamp_opt(self.index * NUM_SECS * i64::from(N), 0)
            .expect("valid timestamp")
    }
    fn name(&self) -> String {
        format!("Minutes[Length:{}]", N)
    }
}

impl<const N: u32> Monotonic for Minutes<N> {
    fn to_monotonic(&self) -> i64 {
        self.index
    }
    fn from_monotonic(index: i64) -> Self {
        Minutes { index }
    }
    fn between(&self, other: Self) -> i64 {
        other.index - self.index
    }
}

impl<const N: u32> Minutes<N> {}

impl<const N: u32> crate::SubDateResolution for Minutes<N> {
    fn occurs_on_date(&self) -> chrono::NaiveDate {
        self.start_datetime().date()
    }
    fn first_on_day(day: chrono::NaiveDate) -> Self {
        Self::from_monotonic(
            day.and_hms_opt(0, 0, 0).expect("valid time").timestamp() / (i64::from(N) * NUM_SECS),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{SubDateResolution, TimeResolution};

    #[test]
    fn test_roundtrip() {
        let dt = chrono::NaiveDate::from_ymd_opt(2021, 12, 6).unwrap();
        let tm = dt.and_hms_opt(0, 0, 0).unwrap();

        let min = Minutes::<1>::from(tm);
        assert!(min.occurs_on_date() == dt);
        assert!(min.start_datetime() == tm);

        let min = Minutes::<2>::from(tm);
        assert!(min.occurs_on_date() == dt);
        assert!(min.start_datetime() == tm);

        let min = Minutes::<3>::from(tm);
        assert!(min.occurs_on_date() == dt);
        assert!(min.start_datetime() == tm);

        let min = Minutes::<4>::from(tm);
        assert!(min.occurs_on_date() == dt);
        assert!(min.start_datetime() == tm);

        let min = Minutes::<5>::from(tm);
        assert!(min.occurs_on_date() == dt);
        assert!(min.start_datetime() == tm);

        assert_eq!(
            min,
            serde_json::from_str(&serde_json::to_string(&min).unwrap()).unwrap()
        )
    }

    #[test]
    fn test_into() {
        assert_eq!(
            Minutes::<2>::from(
                chrono::NaiveDate::from_ymd_opt(2021, 1, 1)
                    .unwrap()
                    .and_hms_opt(10, 2, 0)
                    .unwrap()
            ),
            Minutes::<2>::from(
                chrono::NaiveDate::from_ymd_opt(2021, 1, 1)
                    .unwrap()
                    .and_hms_opt(10, 3, 59)
                    .unwrap()
            ),
        );
    }

    #[test]
    fn test_parse() {
        assert!("2021-01-01 10:05".parse::<Minutes<2>>().is_err());
        assert!("2021-01-01 10:05 => 2021-01-01 10:06"
            .parse::<Minutes<2>>()
            .is_err());
        assert!("2021-01-01 10:02 => 2021-01-01 10:03"
            .parse::<Minutes<2>>()
            .is_ok());

        assert_eq!(
            "2021-01-01 10:05".parse::<Minutes<1>>().unwrap(),
            chrono::NaiveDate::from_ymd_opt(2021, 1, 1)
                .unwrap()
                .and_hms_opt(10, 5, 0)
                .unwrap()
                .into(),
        );
        assert_eq!(
            "2021-01-01 10:05".parse::<Minutes<1>>().unwrap().succ(),
            chrono::NaiveDate::from_ymd_opt(2021, 1, 1)
                .unwrap()
                .and_hms_opt(10, 6, 0)
                .unwrap()
                .into(),
        );
        assert_eq!(
            "2021-01-01 10:05"
                .parse::<Minutes<1>>()
                .unwrap()
                .succ()
                .pred(),
            chrono::NaiveDate::from_ymd_opt(2021, 1, 1)
                .unwrap()
                .and_hms_opt(10, 5, 0)
                .unwrap()
                .into(),
        );

        assert_eq!(
            "2021-01-01 10:02 => 2021-01-01 10:03"
                .parse::<Minutes<2>>()
                .unwrap(),
            chrono::NaiveDate::from_ymd_opt(2021, 1, 1)
                .unwrap()
                .and_hms_opt(10, 2, 0)
                .unwrap()
                .into(),
        );

        assert_eq!(
            "2021-01-01 10:00 => 2021-01-01 10:04"
                .parse::<Minutes<5>>()
                .unwrap(),
            chrono::NaiveDate::from_ymd_opt(2021, 1, 1)
                .unwrap()
                .and_hms_opt(10, 0, 0)
                .unwrap()
                .into(),
        );
    }
}
