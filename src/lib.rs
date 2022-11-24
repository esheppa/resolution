use std::{any, fmt, num, str};

mod range;
pub use range::{Cache, CacheResponse, TimeRange, TimeRangeComparison, TimeRangeIter};

mod minutes;
pub use minutes::Minutes;

pub type Minute = Minutes<1>;
pub type FiveMinute = Minutes<5>;
pub type HalfHour = Minutes<30>;
pub type Hour = Minutes<60>;

mod day;
pub use day::Day;

mod week;
pub use week::{StartDay, Week};

mod month;
pub use month::Month;
mod quarter;
pub use quarter::Quarter;
mod year;
pub use year::Year;

mod zoned;
pub use zoned::Zoned;

/// This function is useful for formatting types implementing `Monotonic` when they are stored
/// in their `i64` form instead of their `TimeResolution` form. Provided you have the `TypeId` handy
/// you can find out what they were intended to be. This function handeles all the cases implemented
/// in this library and users can handle others via the function in the `handle_unknown` parameter.
pub fn format_erased_resolution(
    handle_unknown: fn(any::TypeId, i64) -> String,
    tid: any::TypeId,
    val: i64,
) -> String {
    if tid == any::TypeId::of::<Minute>() {
        format!("Minute:{}", Minute::from_monotonic(val))
    } else if tid == any::TypeId::of::<FiveMinute>() {
        format!("FiveMinute:{}", FiveMinute::from_monotonic(val))
    } else if tid == any::TypeId::of::<HalfHour>() {
        format!("HalfHour:{}", HalfHour::from_monotonic(val))
    } else if tid == any::TypeId::of::<Hour>() {
        format!("Hour:{}", Hour::from_monotonic(val))
    } else if tid == any::TypeId::of::<Day>() {
        format!("Day:{}", Day::from_monotonic(val))
    } else if tid == any::TypeId::of::<Week<week::Monday>>() {
        format!("Week:{}", Week::<week::Monday>::from_monotonic(val))
    } else if tid == any::TypeId::of::<Week<week::Tuesday>>() {
        format!("Week:{}", Week::<week::Tuesday>::from_monotonic(val))
    } else if tid == any::TypeId::of::<Week<week::Wednesday>>() {
        format!("Week:{}", Week::<week::Wednesday>::from_monotonic(val))
    } else if tid == any::TypeId::of::<Week<week::Thursday>>() {
        format!("Week:{}", Week::<week::Thursday>::from_monotonic(val))
    } else if tid == any::TypeId::of::<Week<week::Friday>>() {
        format!("Week:{}", Week::<week::Friday>::from_monotonic(val))
    } else if tid == any::TypeId::of::<Week<week::Saturday>>() {
        format!("Week:{}", Week::<week::Saturday>::from_monotonic(val))
    } else if tid == any::TypeId::of::<Week<week::Sunday>>() {
        format!("Week:{}", Week::<week::Sunday>::from_monotonic(val))
    } else if tid == any::TypeId::of::<Month>() {
        format!("Month:{}", Month::from_monotonic(val))
    } else if tid == any::TypeId::of::<Quarter>() {
        format!("Quarter:{}", Quarter::from_monotonic(val))
    } else if tid == any::TypeId::of::<Year>() {
        format!("Year:{}", Year::from_monotonic(val))
    } else {
        handle_unknown(tid, val)
    }
}

#[derive(Debug)]
pub enum Error {
    GotNonMatchingNewData {
        point: String,
        old: String,
        new: String,
    },
    ParseInt(num::ParseIntError),
    ParseDate(chrono::ParseError),
    ParseCustom {
        ty_name: &'static str,
        input: String,
    },
    EmptyRange,
    UnexpectedStartDate {
        date: chrono::NaiveDate,
        required: chrono::Weekday,
        actual: chrono::Weekday,
    },
    UnexpectedInputLength {
        required: usize,
        actual: usize,
        format: &'static str,
    },
}

impl From<num::ParseIntError> for Error {
    fn from(e: num::ParseIntError) -> Error {
        Error::ParseInt(e)
    }
}
impl From<chrono::ParseError> for Error {
    fn from(e: chrono::ParseError) -> Error {
        Error::ParseDate(e)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Error::*;
        match self {
            GotNonMatchingNewData { point, old, new } => write!(
                f,
                "Got new data for {point}: {new} different from data already in the cache {old}"
            ),
            ParseInt(e) => write!(f, "Error parsing int: {e}"),
            ParseDate(e) => write!(f, "Error parsing date/time: {e}"),
            ParseCustom { ty_name, input } => {
                write!(f, "Error parsing {ty_name} from input: {input}")
            }
            EmptyRange => write!(
                f,
                "Time range cannot be created from an empty set of periods"
            ),
            UnexpectedStartDate {
                date,
                required,
                actual,
            } => write!(
                f,
                "Unexpected input length for date {date}, got {actual} but needed {required}"
            ),
            UnexpectedInputLength {
                required,
                actual,
                format,
            } => write!(
                f,
                "Unexpected input length for format {format}, got {actual} but needed {required}"
            ),
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;

/// `TimeResolution` should be used for contigious series of periods in time
///
/// This makes sense for the time part of a discrete timeseries, with observations
/// occurring at regular times. Some examples are:
/// * A cash-flow report aggregated to days or months
/// * Dispatch periods in the Australian Electricity Market (and similar concepts in other energy markets)
pub trait TimeResolution:
    Send + Sync + Copy + Eq + Ord + From<chrono::NaiveDateTime> + Monotonic
{
    fn succ(&self) -> Self {
        self.succ_n(1)
    }

    fn pred(&self) -> Self {
        self.pred_n(1)
    }

    // the default impls are probably inefficient
    // makes sense to require just the n
    // and give the 1 for free
    fn succ_n(&self, n: u32) -> Self;

    fn pred_n(&self, n: u32) -> Self;

    fn start_datetime(&self) -> chrono::NaiveDateTime;

    fn name(&self) -> String;
}

/// `Monotonic` is used to enable multiple different resolutions to be stored together
///
/// It is named monotonic as it is intended to provide a monotonic (order preserving) function
/// from a given implementor of `TimeResolution`, to allow converting backwards and forwards
/// between the values of the `TimeResolution` implementor and `i64`s
pub trait Monotonic {
    // we choose i64 rather than u64
    // as the behaviour on subtraction is nicer!
    fn to_monotonic(&self) -> i64;
    fn from_monotonic(idx: i64) -> Self;
    fn between(&self, other: Self) -> i64;
}

/// `SubDateResolution` should only be implemented for periods of strictly less than one day in length
pub trait SubDateResolution: TimeResolution {
    fn occurs_on_date(&self) -> chrono::NaiveDate;

    // the first of the resolutions units that occurs on the day
    fn first_on_day(day: chrono::NaiveDate) -> Self;

    fn last_on_day(day: chrono::NaiveDate) -> Self {
        Self::first_on_day(day + chrono::Duration::days(1)).pred()
    }
}

/// `DateResolution` should only be implemented for periods of one or more days in length
pub trait DateResolution: TimeResolution + From<chrono::NaiveDate> {
    fn start(&self) -> chrono::NaiveDate;
}

/// `DateResolutionExt` implements some convenience methods for types that implement `DateResolution`
// This is an extra trait to avoid the methods being overriden
pub trait DateResolutionExt: DateResolution {
    fn format<'a>(
        &self,
        fmt: &'a str,
    ) -> chrono::format::DelayedFormat<chrono::format::StrftimeItems<'a>> {
        self.start().format(fmt)
    }

    fn end(&self) -> chrono::NaiveDate {
        self.succ().start() - chrono::Duration::days(1)
    }

    fn num_days(&self) -> i64 {
        (self.end() - self.start()).num_days() + 1
    }

    fn to_sub_date_resolution<R: SubDateResolution>(&self) -> range::TimeRange<R> {
        range::TimeRange::from_start_end(R::first_on_day(self.start()), R::last_on_day(self.end()))
            .expect("Will always have at least one within the day")
    }

    fn rescale<R: DateResolution>(&self) -> range::TimeRange<R> {
        range::TimeRange::from_start_end(self.start().into(), self.end().into())
            .expect("Will always have at least one day")
    }

    // fn days(&self) -> collections::BTreeSet<chrono::NaiveDate> {
    //     (0..)
    //         .map(|n| self.start() + chrono::Duration::days(n))
    //         .filter(|d| d <= &self.end())
    //         .collect()
    // }
    // fn business_days(
    //     &self,
    //     weekend: collections::HashSet<chrono::Weekday>,
    //     holidays: collections::BTreeSet<chrono::NaiveDate>,
    // ) -> collections::BTreeSet<chrono::NaiveDate> {
    //     let base_days = (0..)
    //         .map(|n| self.start() + chrono::Duration::days(n))
    //         .filter(|d| d <= &self.end())
    //         .filter(|d| !weekend.contains(&d.weekday()))
    //         .collect::<collections::BTreeSet<_>>();
    //     base_days.difference(&holidays).copied().collect()
    // }
}

impl<T> DateResolutionExt for T where T: DateResolution {}
