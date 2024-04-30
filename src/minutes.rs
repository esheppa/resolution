use core::fmt::Debug;
use core::num::NonZeroU64;

use crate::{Error, FromMonotonic, Monotonic, SubDateResolution, TimeResolution};
use alloc::{
    fmt, format, str,
    string::{String, ToString},
};
use chrono::{DateTime, Datelike, Duration, NaiveDate, NaiveTime, Timelike, Utc};

const NUM_SECS: i64 = 60;

/// Note that for sensible behaviour, the N chosen should be a number that either:
/// 1. divides into an hour with no remainder (1, 2, 3, 4, 5, 6, 10, 12, 15, 20, 30, 60)
/// 2. is exactly a whole number of hours that divides into a day with no remainder (60, 120, 180, 240, 360, 480, 1800)
/// Any other choice will result in unexpected / unuseful behaviour (eg the `Minutes` not cleanly fitting into parts of a day)
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "serde", serde(try_from = "Minutes_", into = "Minutes_"))]
pub struct Minutes<const N: u32> {
    index: i64,
}

// #[cfg(not(serde))]
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

#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub(crate) struct Minutes_ {
    index: i64,
    pub(crate) length: u32,
}

impl<const N: u32> From<DateTime<Utc>> for Minutes<N> {
    fn from(d: DateTime<Utc>) -> Self {
        Minutes {
            index: d.timestamp().div_euclid(60 * i64::from(N)),
        }
    }
}

impl<const N: u32> str::FromStr for Minutes<N> {
    type Err = crate::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if N == 1 {
            let time = parse_datetime(s)?;
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

            let start = parse_datetime(start)?;

            if (start.hour() * 60 + start.minute()).rem_euclid(N) != 0 {
                return Err(crate::Error::ParseCustom {
                    ty_name: "Minutes",
                    input: format!("Invalid start for Minutes[Length:{}]: {}", N, start,),
                });
            }
            let end = parse_datetime(end)?;

            if start + Duration::minutes(i64::from(N)) != end {
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

// TODO: make this more efficient
fn format_datetime(n: DateTime<Utc>, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(
        f,
        "{}-{:02}-{:02} {:02}:{:02}",
        n.year(),
        n.month(),
        n.day(),
        n.hour(),
        n.minute()
    )
}

fn parse_datetime(input: &str) -> Result<DateTime<Utc>, Error> {
    let year = input[0..=3]
        .parse()
        .map_err(|e| Error::ParseIntDetailed(e, input[0..=3].to_string()))?;
    let month = input[5..=6]
        .parse()
        .map_err(|e| Error::ParseIntDetailed(e, input[5..=6].to_string()))?;
    let day = input[8..=9]
        .parse()
        .map_err(|e| Error::ParseIntDetailed(e, input[8..=9].to_string()))?;
    let hour = input[11..=12]
        .parse()
        .map_err(|e| Error::ParseIntDetailed(e, input[10..=12].to_string()))?;
    let minute = input[14..=15]
        .parse()
        .map_err(|e| Error::ParseIntDetailed(e, input[14..=15].to_string()))?;

    let date =
        NaiveDate::from_ymd_opt(year, month, day).ok_or_else(|| Error::ParseDateInternal {
            message: alloc::format!("Invalid values for ymd: {year}-{month}-{day}"),
            input: input.to_string(),
            format: "%Y/%m/%d %H:%M",
        })?;

    let time =
        NaiveTime::from_hms_opt(hour, minute, 0).ok_or_else(|| Error::ParseDateInternal {
            message: alloc::format!("Invalid values for hm: {hour}:{minute}"),
            input: input.to_string(),
            format: "%Y/%m/%d %H:%M",
        })?;

    Ok(date.and_time(time).and_utc())
}

impl<const N: u32> fmt::Display for Minutes<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if N == 1 {
            format_datetime(self.start_datetime(), f)
        } else {
            format_datetime(self.start_datetime(), f)?;
            f.write_str(" => ")?;
            format_datetime(self.succ().start_datetime(), f)?;
            Ok(())
        }
    }
}

impl<const N: u32> crate::TimeResolution for Minutes<N> {
    fn succ_n(&self, n: u64) -> Minutes<N> {
        Minutes {
            index: self.index + i64::try_from(n).unwrap(),
        }
    }
    fn pred_n(&self, n: u64) -> Minutes<N> {
        Minutes {
            index: self.index - i64::try_from(n).unwrap(),
        }
    }
    fn start_datetime(&self) -> DateTime<Utc> {
        DateTime::<Utc>::from_timestamp(self.index * NUM_SECS * i64::from(N), 0)
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
    fn between(&self, other: Self) -> i64 {
        other.index - self.index
    }
}

impl<const N: u32> FromMonotonic for Minutes<N> {
    fn from_monotonic(index: i64) -> Self {
        Minutes { index }
    }
}

impl<const N: u32> Minutes<N> {}

impl<const N: u32> SubDateResolution for Minutes<N> {
    fn occurs_on_date(&self) -> chrono::NaiveDate {
        self.start_datetime().date_naive()
    }
    fn first_on_day(day: chrono::NaiveDate, _params: Self::Params) -> Self {
        Self::from_monotonic(
            day.and_hms_opt(0, 0, 0)
                .expect("valid time")
                .and_utc()
                .timestamp()
                / (i64::from(N) * NUM_SECS),
        )
    }

    type Params = ();

    fn params(&self) -> Self::Params {}

    fn from_utc_datetime(datetime: DateTime<Utc>, _params: Self::Params) -> Self {
        datetime.into()
    }
}

macro_rules! minutes_impl {
    ($i:literal) => {
        impl Minutes<$i> {
            pub fn relative(&self) -> DaySubdivison<$i> {
                DaySubdivison {
                    index: Minutes::<$i>::first_on_day(self.occurs_on_date(), ()).between(*self),
                }
            }
        }
    };
}

macro_rules! day_subdivision_impl {
    ($i:literal) => {
        // 1
        impl Debug for DaySubdivison<$i> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.debug_struct("DaySubdivison")
                    .field("index", &self.index())
                    .field("length_minutes", &$i)
                    .field("periods", &Self::PERIODS)
                    .finish()
            }
        }

        // 2
        impl DaySubdivison<$i> {
            pub const PERIODS: u32 = 1440 / $i;
            pub fn on_date(&self, date: NaiveDate) -> Minutes<$i> {
                Minutes::<$i>::from_monotonic(
                    self.index + Minutes::<$i>::first_on_day(date, ()).to_monotonic(),
                )
            }
            pub fn new(period_no: NonZeroU64) -> Option<DaySubdivison<$i>> {
                if i64::try_from(period_no.get()).ok()? > i64::from(Self::PERIODS) {
                    return None;
                }

                Some(DaySubdivison {
                    index: i64::try_from(period_no.get()).ok()? - 1,
                })
            }
            pub fn index(&self) -> NonZeroU64 {
                NonZeroU64::new(u64::try_from(self.index).unwrap() + 1).unwrap()
            }
        }
    };
}

minutes_impl!(1);
minutes_impl!(2);
minutes_impl!(3);
minutes_impl!(4);
minutes_impl!(5);
minutes_impl!(6);
minutes_impl!(10);
minutes_impl!(15);
minutes_impl!(20);
minutes_impl!(30);
minutes_impl!(60);
minutes_impl!(120);
minutes_impl!(180);
minutes_impl!(240);
minutes_impl!(360);
minutes_impl!(720);

day_subdivision_impl!(1);
day_subdivision_impl!(2);
day_subdivision_impl!(3);
day_subdivision_impl!(4);
day_subdivision_impl!(5);
day_subdivision_impl!(6);
day_subdivision_impl!(10);
day_subdivision_impl!(15);
day_subdivision_impl!(20);
day_subdivision_impl!(30);
day_subdivision_impl!(60);
day_subdivision_impl!(120);
day_subdivision_impl!(180);
day_subdivision_impl!(240);
day_subdivision_impl!(360);
day_subdivision_impl!(720);

#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct DaySubdivison<const N: u32> {
    index: i64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::TimeResolution;

    #[test]
    fn test_relative() {
        let base = "2021-01-01 00:00".parse::<Minutes<1>>().unwrap();

        for i in 0..1440 {
            assert_eq!(
                base.succ_n(i).relative(),
                DaySubdivison::<1>::new(NonZeroU64::new(i + 1).unwrap()).unwrap()
            );
            assert_eq!(base.succ_n(i * 1440).relative().index().get(), 1);
            assert_eq!(base.succ_n(i).relative().index().get(), i + 1,);
        }

        let base = "2021-01-01 00:00 => 2021-01-01 00:02"
            .parse::<Minutes<2>>()
            .unwrap();
        for i in 0..720 {
            assert_eq!(
                base.succ_n(i).relative(),
                DaySubdivison::<2>::new(NonZeroU64::new(i + 1).unwrap()).unwrap()
            );
            assert_eq!(base.succ_n(i * 720).relative().index().get(), 1);
            assert_eq!(base.succ_n(i).relative().index().get(), i + 1,);
        }

        let base = "2021-01-01 00:00 => 2021-01-01 00:05"
            .parse::<Minutes<5>>()
            .unwrap();
        for i in 0..288 {
            assert_eq!(
                base.succ_n(i).relative(),
                DaySubdivison::<5>::new(NonZeroU64::new(i + 1).unwrap()).unwrap()
            );
            assert_eq!(base.succ_n(i * 288).relative().index().get(), 1);
            assert_eq!(base.succ_n(i).relative().index().get(), i + 1,);
        }

        let base = "2021-01-01 00:00 => 2021-01-01 00:30"
            .parse::<Minutes<30>>()
            .unwrap();
        for i in 0..48 {
            assert_eq!(
                base.succ_n(i).relative(),
                DaySubdivison::<30>::new(NonZeroU64::new(i + 1).unwrap()).unwrap()
            );
            assert_eq!(base.succ_n(i * 48).relative().index().get(), 1);
            assert_eq!(base.succ_n(i).relative().index().get(), i + 1,);
        }

        let base = "2021-01-01 00:00 => 2021-01-01 01:00"
            .parse::<Minutes<60>>()
            .unwrap();
        for i in 0..24 {
            assert_eq!(
                base.succ_n(i).relative(),
                DaySubdivison::<60>::new(NonZeroU64::new(i + 1).unwrap()).unwrap()
            );
            assert_eq!(base.succ_n(i * 24).relative().index().get(), 1);
            assert_eq!(base.succ_n(i).relative().index().get(), i + 1,);
        }

        let base = "2021-01-01 00:00 => 2021-01-01 02:00"
            .parse::<Minutes<120>>()
            .unwrap();
        for i in 0..12 {
            assert_eq!(
                base.succ_n(i).relative(),
                DaySubdivison::<120>::new(NonZeroU64::new(i + 1).unwrap()).unwrap()
            );
            assert_eq!(base.succ_n(i * 12).relative().index().get(), 1);
            assert_eq!(base.succ_n(i).relative().index().get(), i + 1,);
        }
    }

    #[cfg(feature = "serde")]
    #[test]
    fn test_roundtrip() {
        use crate::SubDateResolution;

        let dt = chrono::NaiveDate::from_ymd_opt(2021, 12, 6).unwrap();
        let tm = dt.and_time(NaiveTime::MIN).and_utc();

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
                    .and_utc()
            ),
            Minutes::<2>::from(
                chrono::NaiveDate::from_ymd_opt(2021, 1, 1)
                    .unwrap()
                    .and_hms_opt(10, 3, 59)
                    .unwrap()
                    .and_utc()
            ),
        );
    }

    #[test]
    fn test_parse() {
        assert!("2021-01-01 10:05".parse::<Minutes<2>>().is_err());
        assert!("2021-01-01 10:05 => 2021-01-01 10:06"
            .parse::<Minutes<2>>()
            .is_err());
        assert!("2021-01-01 10:02 => 2021-01-01 10:04"
            .parse::<Minutes<2>>()
            .is_ok());

        assert_eq!(
            "2021-01-01 10:05".parse::<Minutes<1>>().unwrap(),
            chrono::NaiveDate::from_ymd_opt(2021, 1, 1)
                .unwrap()
                .and_hms_opt(10, 5, 0)
                .unwrap()
                .and_utc()
                .into(),
        );
        assert_eq!(
            "2021-01-01 10:05".parse::<Minutes<1>>().unwrap().succ(),
            chrono::NaiveDate::from_ymd_opt(2021, 1, 1)
                .unwrap()
                .and_hms_opt(10, 6, 0)
                .unwrap()
                .and_utc()
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
                .and_utc()
                .into(),
        );

        assert_eq!(
            "2021-01-01 10:02 => 2021-01-01 10:04"
                .parse::<Minutes<2>>()
                .unwrap(),
            chrono::NaiveDate::from_ymd_opt(2021, 1, 1)
                .unwrap()
                .and_hms_opt(10, 2, 0)
                .unwrap()
                .and_utc()
                .into(),
        );

        assert_eq!(
            "2021-01-01 10:00 => 2021-01-01 10:05"
                .parse::<Minutes<5>>()
                .unwrap(),
            chrono::NaiveDate::from_ymd_opt(2021, 1, 1)
                .unwrap()
                .and_hms_opt(10, 0, 0)
                .unwrap()
                .and_utc()
                .into(),
        );
    }
}
