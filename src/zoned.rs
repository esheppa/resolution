use crate::DateResolution;
use crate::Monotonic;
use crate::SubDateResolution;
use crate::TimeResolution;
use alloc::format;
use alloc::string::String;
use chrono::FixedOffset;
use chrono::NaiveTime;
use chrono::Offset;
use core::fmt;

pub trait TimeZone:
    chrono::TimeZone + Copy + Clone + Send + Sync + fmt::Display + fmt::Debug
{
}

impl TimeZone for chrono::Utc {}
impl TimeZone for chrono::FixedOffset {}
impl TimeZone for chrono_tz::Tz {}

/// `Zoned` stores a `TimeResolution` representing the local time in the zone, plus the relevant
/// offset and zone itself. This is intended to allow assertion that a given resolution is in a certain
/// timezone and thus allow finding the start and end times of that resolution with their correct UTC offsets.
///
/// warning: this should not be used for `SubDateResolution`s larger than `Minutes<60>` or equivalent. (Ideally
/// this restriction will be removed later)
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Zoned<R, Z>
where
    R: TimeResolution,
    Z: TimeZone,
{
    resolution: R,
    offset: FixedOffset,
    zone: Z,
}

impl<R> Monotonic for Zoned<R, chrono::Utc>
where
    R: TimeResolution,
{
    fn to_monotonic(&self) -> i64 {
        self.resolution.to_monotonic()
    }

    fn from_monotonic(idx: i64) -> Self {
        let resolution = R::from_monotonic(idx);
        Zoned {
            resolution,
            offset: FixedOffset::east_opt(0).unwrap(),
            zone: chrono::Utc,
        }
    }

    fn between(&self, other: Self) -> i64 {
        other.to_monotonic() - self.to_monotonic()
    }
}

impl<R> From<chrono::NaiveDateTime> for Zoned<R, chrono::Utc>
where
    R: TimeResolution,
{
    fn from(value: chrono::NaiveDateTime) -> Self {
        Zoned {
            resolution: R::from(value),
            offset: FixedOffset::east_opt(0).unwrap(),
            zone: chrono::Utc,
        }
    }
}

impl<R> Copy for Zoned<R, chrono::Utc> where R: TimeResolution {}

impl<R> Ord for Zoned<R, chrono::Utc>
where
    R: TimeResolution,
{
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.resolution.cmp(&other.resolution)
    }
}

impl<R> PartialOrd for Zoned<R, chrono::Utc>
where
    R: TimeResolution,
{
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

// impl<R> SubDateResolution for Zoned<R, chrono::Utc>
// where
//     R: TimeResolution,
// {
//     fn occurs_on_date(&self) -> chrono::NaiveDate {
//         todo!()
//     }

//     fn first_on_day(day: chrono::NaiveDate) -> Self {
//         todo!()
//     }
// }

impl<R> TimeResolution for Zoned<R, chrono::Utc>
where
    R: TimeResolution,
{
    fn succ_n(&self, n: u32) -> Self {
        Self {
            resolution: self.resolution.succ_n(n),
            ..*self
        }
    }
    fn pred_n(&self, n: u32) -> Self {
        Self {
            resolution: self.resolution.pred_n(n),
            ..*self
        }
    }
    fn start_datetime(&self) -> chrono::NaiveDateTime {
        self.resolution.start_datetime()
    }
    fn name(&self) -> String {
        format!("Zoned[{},Utc]", self.resolution.name())
    }
}

impl<R, Z> Zoned<R, Z>
where
    R: TimeResolution,
    Z: TimeZone,
{
    pub fn from_local(local: chrono::DateTime<FixedOffset>, zone: Z) -> Self {
        Zoned {
            resolution: R::from(local.naive_local()),
            zone,
            offset: *local.offset(),
        }
    }
}

impl<R, Z> Zoned<R, Z>
where
    R: DateResolution,
    Z: TimeZone,
{
    pub fn from_date_resolution(resolution: R, zone: Z) -> Self {
        Zoned {
            resolution,
            zone,
            // if we have a DateResolution then
            // we don't care about the offset anyway.
            offset: FixedOffset::east_opt(0).unwrap(),
        }
    }
}

impl<R, Z> Zoned<R, Z>
where
    R: TimeResolution,
    Z: TimeZone,
{
    pub fn resolution(&self) -> R {
        self.resolution
    }
    pub fn zone(&self) -> Z {
        self.zone
    }
}

impl<R, Z> Zoned<R, Z>
where
    R: SubDateResolution,
    Z: TimeZone,
{
    pub fn start(&self) -> chrono::DateTime<Z> {
        chrono::TimeZone::from_utc_datetime(
            &self.zone,
            &(self.resolution.start_datetime() - self.offset),
        )
    }

    pub fn end_exclusive(&self) -> chrono::DateTime<Z> {
        chrono::TimeZone::from_utc_datetime(&self.zone, &self.resolution.succ().start_datetime())
    }
}

impl<R, Z> Zoned<R, Z>
where
    R: DateResolution,
    Z: TimeZone,
{
    pub fn earliest(&self) -> chrono::DateTime<Z> {
        // impl as per `chrono::Day<Tz>`.
        let base = NaiveTime::from_hms_opt(0, 0, 0).unwrap();
        for after_midnight in 0..=(24 * 60) {
            let start_time = base + chrono::Duration::minutes(after_midnight);
            match self
                .zone()
                .from_local_datetime(&self.resolution.start().and_time(start_time))
            {
                chrono::LocalResult::None => continue,
                chrono::LocalResult::Single(dt) => return dt,
                // in the ambiguous case we pick the one which has an
                // earlier UTC timestamp
                // (this could be done without calling `naive_utc`, but
                // this potentially better expresses the intent)
                chrono::LocalResult::Ambiguous(dt1, dt2) => {
                    if dt1.naive_utc() < dt2.naive_utc() {
                        return dt1;
                    } else {
                        return dt2;
                    }
                }
            }
        }
        panic!("Unable to calculate start time");
    }

    pub fn latest_exclusive(&self) -> chrono::DateTime<Z> {
        Self::from_date_resolution(self.resolution.succ(), self.zone()).earliest()
    }
}

impl<R, Z> From<chrono::DateTime<Z>> for Zoned<R, Z>
where
    R: SubDateResolution,
    Z: TimeZone,
{
    fn from(time: chrono::DateTime<Z>) -> Self {
        Zoned {
            resolution: time.naive_local().into(),
            zone: time.timezone(),
            offset: time.offset().fix(),
        }
    }
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
                    Zoned::<Minutes<N>, _>::from_local(
                        period.with_timezone(&period.offset().fix()),
                        tz
                    )
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
        fn date<R: DateResolution>(tz: chrono_tz::Tz) {
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
                let zoned =
                    Zoned::<R, _>::from_local(period.with_timezone(&period.offset().fix()), tz);
                assert_eq!(period.date_naive(), zoned.earliest().date_naive(),);
                assert_eq!(period.date_naive(), zoned.resolution().start());

                let zoned2 = Zoned::<R, _>::from_date_resolution(R::from(period.naive_local()), tz);
                assert_eq!(period.date_naive(), zoned2.earliest().date_naive(),);
                assert_eq!(period.date_naive(), zoned2.resolution().start());
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
