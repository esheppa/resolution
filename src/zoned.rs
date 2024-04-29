use crate::DateResolution;
use crate::FromMonotonic;
use crate::Monotonic;
use crate::SubDateResolution;
use crate::TimeResolution;
use alloc::format;
use alloc::string::String;
use chrono::DateTime;
use chrono::NaiveDate;
use chrono::Offset;
use chrono::TimeZone;
use chrono::Utc;
use core::fmt;

/// `Zoned` stores a `TimeResolution` representing the local time in the zone, plus the relevant
/// offset and zone itself. This is intended to allow assertion that a given resolution is in a certain
/// timezone and thus allow finding the start and end times of that resolution with their correct UTC offsets.
///
/// warning: this should not be used for `SubDateResolution`s larger than `Minutes<60>` or equivalent. (Ideally
/// this restriction will be removed later)
pub struct Zoned<R, Z>
where
    R: TimeResolution,
    Z: TimeZone,
{
    utc_resolution: R,
    zone: Z,
}

impl<R, Z> fmt::Debug for Zoned<R, Z>
where
    R: TimeResolution + fmt::Debug,
    Z: TimeZone + fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Zoned")
            .field(
                "start_time",
                &self
                    .utc_resolution
                    .start_datetime()
                    .and_utc()
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
    Z: TimeZone,
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
    Z: TimeZone + Copy,
{
    fn succ_n(&self, n: u32) -> Self {
        Self {
            utc_resolution: self.utc_resolution.succ_n(n),
            ..*self
        }
    }
    fn pred_n(&self, n: u32) -> Self {
        Self {
            utc_resolution: self.utc_resolution.pred_n(n),
            ..*self
        }
    }
    fn start_datetime(&self) -> DateTime<Utc> {
        self.start().to_utc()
    }
    fn name(&self) -> String {
        format!("Zoned[{},Utc]", self.utc_resolution.name())
    }
}

impl<R> SubDateResolution for Zoned<R, chrono::Utc>
where
    R: SubDateResolution,
{
    fn occurs_on_date(&self) -> chrono::NaiveDate {
        todo!()
    }

    fn first_on_day(day: chrono::NaiveDate) -> Self {
        todo!()
    }
}

impl<R> DateResolution for Zoned<R, chrono::Utc>
where
    R: DateResolution,
{
    fn start(&self) -> chrono::NaiveDate {
        todo!()
    }
}

impl<R, Z> From<NaiveDate> for Zoned<R, Z>
where
    R: DateResolution,
    Z: TimeZone,
{
    fn from(value: NaiveDate) -> Self {
        todo!()
    }
}

impl<R, Z> From<chrono::DateTime<Z>> for Zoned<R, Z>
where
    R: SubDateResolution,
    Z: TimeZone,
{
    fn from(time: chrono::DateTime<Z>) -> Self {
        Zoned {
            utc_resolution: time.to_utc().into(),
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
    Z: TimeZone + Copy,
{
}

impl<R, Z> Clone for Zoned<R, Z>
where
    R: TimeResolution,
    Z: TimeZone,
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
    Z: TimeZone,
{
}

impl<R, Z> PartialEq for Zoned<R, Z>
where
    R: TimeResolution,
    Z: TimeZone,
{
    fn eq(&self, other: &Self) -> bool {
        self.utc_resolution == other.utc_resolution
    }
}

impl<R, Z> Ord for Zoned<R, Z>
where
    R: TimeResolution,
    Z: TimeZone,
{
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.utc_resolution.cmp(&other.utc_resolution)
    }
}

impl<R, Z> PartialOrd for Zoned<R, Z>
where
    R: TimeResolution,
    Z: TimeZone,
{
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<R, Z> Zoned<R, Z>
where
    R: TimeResolution,
    Z: TimeZone,
{
    pub fn resolution(&self) -> R {
        R::from(self.start().naive_local())
    }
    pub fn zone(&self) -> &Z {
        &self.zone
    }
    pub fn start(&self) -> chrono::DateTime<Z> {
        self.utc_resolution
            .start_datetime()
            .and_utc()
            .with_timezone(&self.zone)
    }

    pub fn end_exclusive(&self) -> chrono::DateTime<Z> {
        self.succ().start()
    }

    pub fn succ_n(&self, n: u32) -> Self {
        Self {
            utc_resolution: self.utc_resolution.succ_n(n),
            zone: self.zone().clone(),
        }
    }
    pub fn pred_n(&self, n: u32) -> Self {
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
                .offset_from_utc_datetime(&self.utc_resolution.start_datetime())
                .fix()
        )
    }
}

impl<R, Z> Zoned<R, Z>
where
    R: DateResolution,
    Z: TimeZone,
{
}

impl<R, Z> Zoned<R, Z>
where
    R: SubDateResolution,
    Z: TimeZone,
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
