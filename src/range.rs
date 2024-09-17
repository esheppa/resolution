use crate::{
    DateResolution, DateResolutionExt, FixedTimeZone, FromMonotonic, LongerThanOrEqual,
    SubDateResolution, TimeResolution, Zoned,
};
use alloc::{collections, fmt, vec::Vec};
use chrono::{DateTime, Utc};
use core::{mem, num};
#[cfg(feature = "serde")]
use serde::de;

/// `TimeRange` stores a contigious sequence of underlying periods of a given `TimeResolution`.
///
/// This is useful to represent the time axis of a timeseries.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct TimeRange<P: TimeResolution> {
    #[cfg_attr(
        feature = "serde",
        serde(bound(deserialize = "P: de::DeserializeOwned"))
    )]
    start: P,
    len: num::NonZeroU64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimeRangeComparison {
    Superset,
    Subset,
    Earlier,
    Later,
}

impl<P: SubDateResolution> TimeRange<P> {}

impl<P: DateResolution> TimeRange<P> {
    pub fn to_sub_date_resolution<S>(&self) -> TimeRange<S>
    where
        S: SubDateResolution<Params = P::Params>,
    {
        // get first start
        let first_start = S::first_on_day(self.start.start(), self.start.params());
        // get last end
        let last_end = S::last_on_day(self.end().end(), self.end().params());
        // do from_start_end and expect it
        TimeRange::from_bounds(first_start, last_end)
    }
}

impl<P: TimeResolution + FromMonotonic> TimeRange<P> {
    pub fn from_map(map: collections::BTreeSet<i64>) -> Vec<TimeRange<P>> {
        let mut ranges = Vec::new();
        if map.is_empty() {
            return ranges;
        }

        let mut iter = map.into_iter();

        let mut prev = match iter.next() {
            Some(n) => n,
            None => return ranges,
        };
        let mut current_range = TimeRange {
            start: P::from_monotonic(prev),
            len: num::NonZeroU64::new(1).unwrap(),
        };
        for val in iter {
            if val == prev + 1 {
                current_range.len =
                    num::NonZeroU64::new(current_range.len.get().saturating_add(1)).unwrap();
            } else {
                let mut old_range = TimeRange {
                    start: P::from_monotonic(val),
                    len: num::NonZeroU64::new(1).unwrap(),
                };
                mem::swap(&mut current_range, &mut old_range);
                if !ranges.contains(&old_range) {
                    ranges.push(old_range);
                }
            }

            prev = val;
        }

        ranges
    }
}

impl<P: TimeResolution> TimeRange<P> {
    pub fn to_indexes(&self) -> collections::BTreeSet<i64> {
        self.iter().map(|p| p.to_monotonic()).collect()
    }

    pub fn from_set(set: &collections::BTreeSet<P>) -> Option<TimeRange<P>> {
        if u32::try_from(set.len()).is_err() {
            return None;
        }
        if set.is_empty() {
            return None;
        }
        Some(TimeRange {
            start: set.iter().next().copied()?,
            len: num::NonZeroU64::new(u64::try_from(set.len()).ok()?)?,
        })
    }

    pub fn maybe_new(start: P, len: u64) -> Option<TimeRange<P>> {
        Some(TimeRange {
            start,
            len: num::NonZeroU64::new(len)?,
        })
    }
    pub fn new(start: P, len: num::NonZeroU64) -> TimeRange<P> {
        TimeRange { start, len }
    }
    pub fn index_of(&self, point: P) -> Option<usize> {
        if point < self.start || point > self.end() {
            None
        } else {
            Some(
                usize::try_from(self.start.between(point))
                    .expect("Point is earlier than end so this is always ok"),
            )
        }
    }
    pub fn from_bounds(a: P, b: P) -> TimeRange<P> {
        if a <= b {
            TimeRange {
                start: a,
                len: num::NonZeroU64::new(1 + u64::try_from(a.between(b)).unwrap()).unwrap(),
            }
        } else {
            TimeRange {
                start: a,
                len: num::NonZeroU64::new(1 + u64::try_from(b.between(a)).unwrap()).unwrap(),
            }
        }
    }

    pub fn len(&self) -> num::NonZeroU64 {
        self.len
    }

    pub fn intersection(&self, other: &TimeRange<P>) -> Option<TimeRange<P>> {
        let max_start = self.start().max(other.start());
        let min_end = self.end().min(other.end());

        if max_start <= min_end {
            Some(TimeRange::from_bounds(max_start, min_end))
        } else {
            None
        }
    }
    pub fn union(&self, other: &TimeRange<P>) -> Option<TimeRange<P>> {
        if self.intersection(other).is_some() {
            let min_start = self.start().min(other.start());
            let max_end = self.end().max(other.end());
            Some(TimeRange::from_bounds(min_start, max_end))
        } else {
            None
        }
    }

    // pub fn subtract(&self, other: &TimeRange<P>) -> (Option<TimeRange<P>>, Option<TimeRange<P>>) {
    //     (
    //         {

    //             Some(TimeRange::from_bounds(self.start(), other.start().pred().min(self.end())))
    //         },
    //         {
    //             Some(TimeRange::from_bounds(other.end().succ().max(self.start()), self.end()))
    //         },
    //     )
    // }

    // pub fn compare(&self, other: &TimeRange<P>) -> TimeRangeComparison {
    //     match self.subtract(other) {
    //         (Some(_), Some(_)) => TimeRangeComparison::Superset,
    //         (Some(_), None) => TimeRangeComparison::Earlier,
    //         (None, Some(_)) => TimeRangeComparison::Later,
    //         (None, None) => TimeRangeComparison::Subset,
    //     }
    // }

    pub fn start(&self) -> P {
        self.start
    }
    pub fn end(&self) -> P {
        self.start.succ_n(self.len.get() - 1)
    }
    pub fn contains<O>(&self, rhs: O) -> bool
    where
        O: TimeResolution,
        P: LongerThanOrEqual<O>,
    {
        extern crate std;
        use std::dbg;

        let range_start = self.start.start_datetime();
        let range_end = self.end().succ().start_datetime();

        let comparison_start = rhs.start_datetime();
        let comparison_end = rhs.succ().start_datetime();

        dbg!(range_start, range_end, comparison_start, comparison_end);

        (range_start..range_end).contains(&comparison_start)
            && (range_start..range_end).contains(&comparison_end)
    }
    pub fn set(&self) -> collections::BTreeSet<P> {
        self.iter().collect()
    }
    pub fn iter(&self) -> TimeRangeIter<P> {
        TimeRangeIter {
            current: self.start(),
            end: self.end(),
        }
    }

    pub fn rescale<Out>(&self) -> TimeRange<Out>
    where
        Out: TimeResolution + From<DateTime<Utc>>,
    {
        // get the exact start
        let start = Out::from(self.start().start_datetime());

        // for the end, we can't use something like 23:59:59
        // so we instead get the next period then look back.
        let end = Out::from(self.end().succ().start_datetime()).pred();

        TimeRange::from_bounds(start, end)
    }
}

pub struct TimeRangeIter<P: TimeResolution> {
    current: P,
    end: P,
}

impl<P: TimeResolution> Iterator for TimeRangeIter<P> {
    type Item = P;
    fn next(&mut self) -> Option<Self::Item> {
        if self.current <= self.end {
            let ret = self.current;
            self.current = self.current.succ();
            Some(ret)
        } else {
            None
        }
    }
}

impl<P: TimeResolution, Z: FixedTimeZone> TimeRange<Zoned<P, Z>> {
    pub fn local(&self) -> TimeRange<P> {
        TimeRange::new(self.start().local_resolution(), self.len)
    }
}

pub struct Cache<K: Ord + fmt::Debug + Copy, T: Send + fmt::Debug + Eq + Copy> {
    // The actual data in the cache
    data: collections::BTreeMap<K, T>,
    // The requests for data which has been cached
    requests: collections::BTreeSet<K>,
}

// merge a request into a set of requests, grouping contigious on the way
fn missing_pieces<K: Ord + fmt::Debug + Copy>(
    request: collections::BTreeSet<K>,
    requests: &collections::BTreeSet<K>,
) -> Vec<collections::BTreeSet<K>> {
    let mut to_request = Vec::new();
    let mut current_request = collections::BTreeSet::new();

    // there is a fundamental assumption that `request` is contigious
    // as long as `request` is contigious, each of the returned requests
    // will also be contigious
    // there is no need to worry about filling gaps to reduce the total number
    // of requests - the consumer will handle this
    for requested in request {
        if !requests.contains(&requested) {
            current_request.insert(requested);
        } else if !current_request.is_empty() {
            to_request.push(mem::take(&mut current_request));
        }
    }

    if !current_request.is_empty() {
        to_request.push(current_request);
    }

    to_request
}

// No concept of partial, becuse we will simply request the missing data, then ask the cache again.
pub enum CacheResponse<K: Ord + fmt::Debug + Copy, T: Send + fmt::Debug + Eq + Copy> {
    Hit(collections::BTreeMap<K, T>), // means the whole request as able to be replied, doesn't necessarily mean the whole range of data is filled
    Miss(Vec<collections::BTreeSet<K>>), // will be a minimal reasonable set of time ranges to request from the provider
}

impl<K: Ord + fmt::Debug + Copy, T: Send + fmt::Debug + Eq + Copy> Cache<K, T> {
    pub fn get(&self, request: collections::BTreeSet<K>) -> CacheResponse<K, T> {
        if request.is_empty() {
            CacheResponse::Hit(collections::BTreeMap::new())
        } else if self.requests.is_superset(&request) {
            CacheResponse::Hit(
                self.data
                    .iter()
                    // mustn't be empty othewise we would have returned out of the first arm of the `if`
                    .filter(|(k, _)| request.iter().next().unwrap() <= *k)
                    .filter(|(k, _)| request.iter().next_back().unwrap() >= *k)
                    .map(|(k, v)| (*k, *v))
                    .collect(),
            )
        } else {
            CacheResponse::Miss(missing_pieces(request, &self.requests))
        }
    }
    pub fn empty() -> Cache<K, T> {
        Cache {
            data: collections::BTreeMap::new(),
            requests: collections::BTreeSet::new(),
        }
    }
    // could also store versioned data, with a DateTIme<Utc> associated with each T at each P?
    // or allow overwriting, etc
    // but this default seems better for now
    pub fn add(
        &mut self,
        mut request_range: collections::BTreeSet<K>,
        data: collections::BTreeMap<K, T>,
    ) {
        self.requests.append(&mut request_range);
        for (point, datum) in data {
            // should we check if the data point already exists?
            // if it does exist, what should we do?
            // for now, ignoring, as otherwise
            // this function would need to be fallible
            self.data.insert(point, datum);
        }
    }
}
#[cfg(test)]
mod tests {
    use alloc::string::ToString;
    use chrono::{NaiveDate, NaiveDateTime, NaiveTime};

    use crate::{Day, FiveMinute, Hour, Minutes, Month, Year};

    use super::*;

    #[test]
    fn test_missing_pieces() {
        let pieces = missing_pieces(
            collections::BTreeSet::from([1, 2, 3, 4, 5, 6, 7, 8, 9, 10]),
            &collections::BTreeSet::from([2, 3, 7, 8]),
        );
        assert_eq!(
            pieces,
            Vec::from([
                collections::BTreeSet::from([1]),
                collections::BTreeSet::from([4, 5, 6]),
                collections::BTreeSet::from([9, 10]),
            ])
        )
    }
    #[test]
    fn test_contains() {
        extern crate std;
        use std::dbg;

        let mth = Month::from_parts(2024, chrono::Month::January).unwrap();

        let day_range = mth.rescale::<Day>();

        dbg!(
            mth.to_string(),
            day_range.start.start(),
            day_range.end().start()
        );

        assert!(day_range.contains(Minutes::<5>::from_utc_datetime(
            NaiveDateTime::new(
                NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                NaiveTime::from_hms_opt(15, 15, 0).unwrap(),
            )
            .and_utc(),
            ()
        )));

        let year = Year::new(2024);

        let month_range = year.rescale::<Month>();

        assert!(month_range.contains(mth))
    }

    #[test]
    fn test_rescale() {
        let start = Year::new(2024);
        let year = TimeRange::from_bounds(start, start);

        let fiveminute = year.rescale::<FiveMinute>();
        assert_eq!(fiveminute.len().get(), 366 * 288);
        assert_eq!(fiveminute.rescale::<Year>(), year);

        let hours = year.rescale::<Hour>();
        assert_eq!(hours.len().get(), 366 * 24);
        assert_eq!(hours.rescale::<Year>(), year);
        assert_eq!(fiveminute.rescale::<Hour>(), hours);

        let days = year.rescale::<Day>();
        assert_eq!(days.len().get(), 366);
        assert_eq!(days.rescale::<Year>(), year);
        assert_eq!(fiveminute.rescale::<Day>(), days);
        assert_eq!(hours.rescale::<Day>(), days);

        let months = year.rescale::<Month>();
        assert_eq!(months.len().get(), 12);
        assert_eq!(months.rescale::<Year>(), year);
        assert_eq!(fiveminute.rescale::<Month>(), months);
        assert_eq!(hours.rescale::<Month>(), months);
        assert_eq!(days.rescale::<Month>(), months);
    }
}
