use crate::DateResolution;
use crate::FromMonotonic;
use crate::LongerThan;
use crate::LongerThanOrEqual;
use crate::Minutes;
use crate::Monotonic;
use crate::SubDateResolution;
use crate::TimeResolution;
use alloc::format;
use alloc::string::String;
use chrono::DateTime;
use chrono::NaiveDate;
use chrono::NaiveTime;
use chrono::Offset;
use chrono::TimeZone;
use chrono::Utc;
use core::fmt;

// marker trait for `TimeZone`s that can be constructed directly
pub trait TzNew: TimeZone + Copy {
    fn new() -> Self;
}

/// `Zoned` stores a `TimeResolution` representing the local time in the zone, plus the relevant
/// offset and zone itself. This is intended to allow assertion that a given resolution is in a certain
/// timezone and thus allow finding the start and end times of that resolution with their correct UTC offsets.
///
/// warning: this should not be used for `SubDateResolution`s larger than `Minutes<60>` or equivalent. (Ideally
/// this restriction will be removed later)
pub struct Zoned<R, Z>
where
    R: TimeResolution,
    Z: TimeZone + Copy + fmt::Debug,
{
    utc_resolution: R,
    zone: Z,
}

impl<R, Z> fmt::Debug for Zoned<R, Z>
where
    R: TimeResolution + fmt::Debug,
    Z: TimeZone + fmt::Debug + Copy,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Zoned")
            .field(
                "start_time",
                &self
                    .utc_resolution
                    .start_datetime()
                    .with_timezone(&self.zone)
                    .naive_local(),
            )
            .field("utc_resolution", &self.utc_resolution)
            .field("zone", &self.zone)
            .finish()
    }
}

impl<R, Z> Monotonic for Zoned<R, Z>
where
    Z: TimeZone + Copy + fmt::Debug,
    R: TimeResolution,
{
    fn to_monotonic(&self) -> i64 {
        self.utc_resolution.to_monotonic()
    }
    fn between(&self, other: Self) -> i64 {
        other.to_monotonic() - self.to_monotonic()
    }
}

impl<R> FromMonotonic for Zoned<R, chrono::Utc>
where
    R: TimeResolution + FromMonotonic,
{
    fn from_monotonic(idx: i64) -> Self {
        let utc_resolution = R::from_monotonic(idx);
        Zoned {
            utc_resolution,
            zone: chrono::Utc,
        }
    }
}

impl<R, Z> TimeResolution for Zoned<R, Z>
where
    R: TimeResolution,
    Z: TimeZone + Copy + fmt::Debug,
{
    fn succ_n(&self, n: u64) -> Self {
        Self {
            utc_resolution: self.utc_resolution.succ_n(n),
            ..*self
        }
    }
    fn pred_n(&self, n: u64) -> Self {
        Self {
            utc_resolution: self.utc_resolution.pred_n(n),
            ..*self
        }
    }
    fn start_datetime(&self) -> DateTime<Utc> {
        self.start().to_utc()
    }
    fn name(&self) -> String {
        format!("Zoned[{},{:?}]", self.utc_resolution.name(), self.zone)
    }
}

impl<R1, R2, Z> LongerThan<Zoned<R2, Z>> for Zoned<R1, Z>
where
    R1: TimeResolution,
    R2: TimeResolution,
    Z: TimeZone + Copy + fmt::Debug,
    R1: LongerThan<R2>,
{
}

impl<R1, R2, Z> LongerThanOrEqual<Zoned<R2, Z>> for Zoned<R1, Z>
where
    R1: TimeResolution,
    R2: TimeResolution,
    Z: TimeZone + Copy + fmt::Debug,
    R1: LongerThanOrEqual<R2>,
{
}

impl<R, Z> SubDateResolution for Zoned<R, Z>
where
    R: SubDateResolution<Params = ()>,
    Z: TimeZone + Copy + fmt::Debug,
{
    type Params = Z;
    fn params(&self) -> Self::Params {
        self.zone().clone()
    }
    fn occurs_on_date(&self) -> chrono::NaiveDate {
        todo!()
    }

    fn first_on_day(day: chrono::NaiveDate, params: Self::Params) -> Self {
        // find the start time of the day in UTC!
        // unwrap: should be ok, becuase empirically no recent TZ offset transitions at midnight
        // however, these could theoretically happen.
        let start_time_of_day = day
            .and_time(NaiveTime::MIN)
            .and_local_timezone(params)
            .single()
            .unwrap()
            .to_utc();
        Self::from_utc_datetime(start_time_of_day, params)
    }

    fn from_utc_datetime(datetime: DateTime<Utc>, params: Self::Params) -> Self {
        Zoned {
            utc_resolution: R::from_utc_datetime(datetime, ()),
            zone: params,
        }
    }
}

impl<R, Z> DateResolution for Zoned<R, Z>
where
    R: DateResolution<Params = ()>,
    Z: TimeZone + Copy + fmt::Debug,
{
    type Params = Z;
    fn params(&self) -> Self::Params {
        self.zone().clone()
    }
    fn start(&self) -> chrono::NaiveDate {
        self.utc_resolution.start()
    }

    fn from_date(date: NaiveDate, params: Self::Params) -> Self {
        Zoned {
            utc_resolution: R::from_date(date, ()),
            zone: params,
        }
    }
}

// impl<const N: u32, Z> From<chrono::DateTime<Z>> for Zoned<Minutes<N>, Z>
// where
//     Z: TimeZone + Copy + fmt::Debug,
// {
//     fn from(time: chrono::DateTime<Z>) -> Self {
//         Zoned {
//             utc_resolution: time.to_utc().into(),
//             zone: time.timezone(),
//         }
//     }
// }

impl<Z, R> From<chrono::DateTime<Z>> for Zoned<R, Z>
where
    R: SubDateResolution<Params = ()>,
    Z: TimeZone + Copy + fmt::Debug,
{
    fn from(time: chrono::DateTime<Z>) -> Self {
        Zoned {
            utc_resolution: R::from_utc_datetime(time.to_utc(), ()),
            zone: time.timezone(),
        }
    }
}

// impl<R, Z> From<DateTime<Utc>> for Zoned<R, Z>
// where
// R: TimeResolution,
// Z: TimeZone,
// {
//     fn from(value: DateTime<Utc>) -> Self {
//         Zoned {
//             utc_resolution: R::from(value),
//             zone: chrono::Utc,
//         }
//     }
// }

impl<R, Z> Copy for Zoned<R, Z>
where
    R: TimeResolution,
    Z: TimeZone + Copy + fmt::Debug,
{
}

impl<R, Z> Clone for Zoned<R, Z>
where
    R: TimeResolution,
    Z: TimeZone + Copy + fmt::Debug,
{
    fn clone(&self) -> Self {
        Self {
            utc_resolution: self.utc_resolution.clone(),
            zone: self.zone.clone(),
        }
    }
}

impl<R, Z> Eq for Zoned<R, Z>
where
    R: TimeResolution,
    Z: TimeZone + Copy + fmt::Debug,
{
}

impl<R, Z> PartialEq for Zoned<R, Z>
where
    R: TimeResolution,
    Z: TimeZone + Copy + fmt::Debug,
{
    fn eq(&self, other: &Self) -> bool {
        self.utc_resolution == other.utc_resolution
    }
}

impl<R, Z> Ord for Zoned<R, Z>
where
    R: TimeResolution,
    Z: TimeZone + Copy + fmt::Debug,
{
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.utc_resolution.cmp(&other.utc_resolution)
    }
}

impl<R, Z> PartialOrd for Zoned<R, Z>
where
    R: TimeResolution,
    Z: TimeZone + Copy + fmt::Debug,
{
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<const N: u32, Z> Zoned<Minutes<N>, Z>
where
    Z: TimeZone + Copy + fmt::Debug,
{
    pub fn resolution1(&self) -> Minutes<N> {
        self.start().naive_local().and_utc().into()
    }
}

impl<R, Z> Zoned<R, Z>
where
    R: SubDateResolution<Params = ()>,
    Z: TimeZone + Copy + fmt::Debug,
{
    pub fn sub_date_resolution(&self) -> R {
        // sketchy, but works
        R::from_utc_datetime(self.start_datetime().naive_local().and_utc(), ())
    }
}

impl<R, Z> Zoned<R, Z>
where
    R: DateResolution<Params = ()>,
    Z: TimeZone + Copy + fmt::Debug,
{
    pub fn date_resolution(&self) -> R {
        // sketchy, but works
        R::from_date(self.utc_resolution.start(), ())
    }
}

// impl<D, Z> Zoned<Week<D>, Z>
// where
//     D: StartDay,
//     Z: TimeZone,
// {
//     pub fn resolution(&self) -> Week<D> {
//         self.start().date_naive().into()
//     }
// }

// impl<Z> Zoned<Month, Z>
// where
//     Z: TimeZone,
// {
//     pub fn resolution(&self) -> Month {
//         self.start().date_naive().into()
//     }
// }

// impl<Z> Zoned<Quarter, Z>
// where
//     Z: TimeZone,
// {
//     pub fn resolution(&self) -> Quarter {
//         self.start().date_naive().into()
//     }
// }

// impl<Z> Zoned<Year, Z>
// where
//     Z: TimeZone,
// {
//     pub fn resolution(&self) -> Year {
//         self.start().date_naive().into()
//     }
// }

// impl<Z> Zoned<Day, Z>
// where
//     Z: TimeZone,
// {
//     pub fn resolution(&self) -> Day {
//         self.start().date_naive().into()
//     }
// }

impl<R, Z> Zoned<R, Z>
where
    R: TimeResolution,
    Z: TimeZone + Copy + fmt::Debug,
{
    pub fn zone(&self) -> Z {
        self.zone
    }
    pub fn start(&self) -> chrono::DateTime<Z> {
        self.utc_resolution
            .start_datetime()
            .with_timezone(&self.zone)
    }

    pub fn end_exclusive(&self) -> chrono::DateTime<Z> {
        self.succ().start()
    }

    pub fn succ_n(&self, n: u64) -> Self {
        Self {
            utc_resolution: self.utc_resolution.succ_n(n),
            zone: self.zone().clone(),
        }
    }
    pub fn pred_n(&self, n: u64) -> Self {
        Self {
            utc_resolution: self.utc_resolution.pred_n(n),
            zone: self.zone().clone(),
        }
    }
    pub fn succ(&self) -> Self {
        Self {
            utc_resolution: self.utc_resolution.succ(),
            zone: self.zone().clone(),
        }
    }
    pub fn pred(&self) -> Self {
        Self {
            utc_resolution: self.utc_resolution.pred(),
            zone: self.zone().clone(),
        }
    }
    pub fn name(&self) -> String {
        format!(
            "Zoned[{},{}]",
            self.utc_resolution.name(),
            self.zone()
                .offset_from_utc_datetime(&self.utc_resolution.start_datetime().naive_utc())
                .fix()
        )
    }
}

impl<R, Z> Zoned<R, Z>
where
    R: DateResolution,
    Z: TimeZone + Copy + fmt::Debug,
{
}

impl<R, Z> Zoned<R, Z>
where
    R: SubDateResolution,
    Z: TimeZone + Copy + fmt::Debug,
{
}

#[cfg(test)]
mod tests {
    use crate::DateResolution;
    use crate::Day;
    use crate::Minutes;
    use crate::Zoned;
    use alloc::vec::Vec;
    use chrono::Offset;

    #[test]
    fn test_subdate() {
        fn subdate<const N: u32>(tz: chrono_tz::Tz) {
            let start = chrono::NaiveDate::from_ymd_opt(2022, 1, 1)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap()
                .and_local_timezone(tz)
                .unwrap();

            let periods = (0..((24 * 60 / N) * 365))
                .map(|i| start + chrono::Duration::minutes((i * N).into()))
                .collect::<Vec<_>>();

            for period in periods {
                assert_eq!(
                    period,
                    Zoned::<Minutes<N>, _>::from(period.with_timezone(&period.offset().fix()),)
                        .start(),
                )
            }
        }
        for tz in [
            chrono_tz::Australia::Sydney,
            chrono_tz::Australia::Adelaide,
            chrono_tz::Asia::Kathmandu,
        ] {
            subdate::<1>(tz);
            subdate::<2>(tz);
            subdate::<5>(tz);
            subdate::<6>(tz);
            subdate::<10>(tz);
            subdate::<15>(tz);
            subdate::<30>(tz);
            subdate::<60>(tz);

            // this is ... problematic ... with daylight savings
            // zoned may not be possible for times larger than an hour and less than a day
            // subdate::<120>(tz);
        }
    }

    #[test]
    fn test_date() {
        fn date<R: DateResolution<Params = ()>>(tz: chrono_tz::Tz) {
            let start = chrono::NaiveDate::from_ymd_opt(2022, 1, 1)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap()
                .and_local_timezone(tz)
                .unwrap();

            let periods = (0..365)
                .map(|i| start + chrono::Days::new(i))
                .collect::<Vec<_>>();

            for period in periods {
                let zoned = Zoned::<R, _>::from_date(period.date_naive(), tz);
                assert_eq!(period.date_naive(), zoned.start().date_naive(),);
                assert_eq!(period.date_naive(), zoned.date_resolution().start());

                let zoned2 = Zoned::<R, _>::from_date(period.date_naive(), tz);
                assert_eq!(period.date_naive(), zoned2.start().date_naive(),);
                assert_eq!(period.date_naive(), zoned2.date_resolution().start());
            }
        }
        for tz in [
            chrono_tz::Australia::Sydney,
            chrono_tz::Australia::Adelaide,
            chrono_tz::Asia::Kathmandu,
        ] {
            date::<Day>(tz);
        }
    }
}
