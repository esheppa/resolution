#![no_std]
extern crate alloc;
#[cfg(feature = "std")]
extern crate std;

use core::{
    any, fmt,
    num::{self, ParseIntError},
    str,
};

mod range;
use alloc::{format, string::String};
use chrono::{DateTime, NaiveDate, Utc};
pub use range::{Cache, CacheResponse, TimeRange, TimeRangeComparison, TimeRangeIter};

mod minutes;
pub use minutes::{DaySubdivison, Minutes};

pub type Minute = Minutes<1>;
pub type FiveMinute = Minutes<5>;
pub type HalfHour = Minutes<30>;
pub type Hour = Minutes<60>;

mod day;
pub use day::Day;

mod week;
pub use week::{Friday, Monday, Saturday, StartDay, Sunday, Thursday, Tuesday, Wednesday, Week};

mod month;
pub use month::Month;
mod quarter;
pub use quarter::Quarter;
mod year;
pub use year::Year;

mod zoned;
pub use zoned::{FixedTimeZone, Zoned};

pub trait LongerThan<T>: LongerThanOrEqual<T> {}

pub trait LongerThanOrEqual<T> {}

pub trait ShorterThan<T>: ShorterThanOrEqual<T> {}

impl<Long, Short> ShorterThan<Long> for Short where
    Long: LongerThanOrEqual<Short> + LongerThan<Short>
{
}

pub trait ShorterThanOrEqual<T> {}

impl<Long, Short> ShorterThanOrEqual<Long> for Short where Long: LongerThan<Short> {}

// TODO: use macro for this

impl LongerThanOrEqual<Minute> for Minute {}
impl LongerThanOrEqual<Minute> for FiveMinute {}
impl LongerThanOrEqual<Minute> for HalfHour {}
impl LongerThanOrEqual<Minute> for Hour {}
impl LongerThanOrEqual<Minute> for Day {}
impl<D> LongerThanOrEqual<Minute> for Week<D> where D: StartDay {}
impl LongerThanOrEqual<Minute> for Month {}
impl LongerThanOrEqual<Minute> for Quarter {}
impl LongerThanOrEqual<Minute> for Year {}

impl LongerThan<Minute> for FiveMinute {}
impl LongerThan<Minute> for HalfHour {}
impl LongerThan<Minute> for Hour {}
impl LongerThan<Minute> for Day {}
impl<D> LongerThan<Minute> for Week<D> where D: StartDay {}
impl LongerThan<Minute> for Month {}
impl LongerThan<Minute> for Quarter {}
impl LongerThan<Minute> for Year {}

impl LongerThanOrEqual<FiveMinute> for FiveMinute {}
impl LongerThanOrEqual<FiveMinute> for HalfHour {}
impl LongerThanOrEqual<FiveMinute> for Hour {}
impl LongerThanOrEqual<FiveMinute> for Day {}
impl<D> LongerThanOrEqual<FiveMinute> for Week<D> where D: StartDay {}
impl LongerThanOrEqual<FiveMinute> for Month {}
impl LongerThanOrEqual<FiveMinute> for Quarter {}
impl LongerThanOrEqual<FiveMinute> for Year {}

impl LongerThan<FiveMinute> for HalfHour {}
impl LongerThan<FiveMinute> for Hour {}
impl LongerThan<FiveMinute> for Day {}
impl<D> LongerThan<FiveMinute> for Week<D> where D: StartDay {}
impl LongerThan<FiveMinute> for Month {}
impl LongerThan<FiveMinute> for Quarter {}
impl LongerThan<FiveMinute> for Year {}

impl LongerThanOrEqual<HalfHour> for HalfHour {}
impl LongerThanOrEqual<HalfHour> for Hour {}
impl LongerThanOrEqual<HalfHour> for Day {}
impl<D> LongerThanOrEqual<HalfHour> for Week<D> where D: StartDay {}
impl LongerThanOrEqual<HalfHour> for Month {}
impl LongerThanOrEqual<HalfHour> for Quarter {}
impl LongerThanOrEqual<HalfHour> for Year {}

impl LongerThan<HalfHour> for Hour {}
impl LongerThan<HalfHour> for Day {}
impl<D> LongerThan<HalfHour> for Week<D> where D: StartDay {}
impl LongerThan<HalfHour> for Month {}
impl LongerThan<HalfHour> for Quarter {}
impl LongerThan<HalfHour> for Year {}

impl LongerThanOrEqual<Hour> for Hour {}
impl LongerThanOrEqual<Hour> for Day {}
impl<D> LongerThanOrEqual<Hour> for Week<D> where D: StartDay {}
impl LongerThanOrEqual<Hour> for Month {}
impl LongerThanOrEqual<Hour> for Quarter {}
impl LongerThanOrEqual<Hour> for Year {}

impl LongerThan<Hour> for Day {}
impl<D> LongerThan<Hour> for Week<D> where D: StartDay {}
impl LongerThan<Hour> for Month {}
impl LongerThan<Hour> for Quarter {}
impl LongerThan<Hour> for Year {}

impl LongerThanOrEqual<Day> for Day {}
impl<D> LongerThanOrEqual<Day> for Week<D> where D: StartDay {}
impl LongerThanOrEqual<Day> for Month {}
impl LongerThanOrEqual<Day> for Quarter {}
impl LongerThanOrEqual<Day> for Year {}

impl<D> LongerThan<Day> for Week<D> where D: StartDay {}
impl LongerThan<Day> for Month {}
impl LongerThan<Day> for Quarter {}
impl LongerThan<Day> for Year {}

impl<D0, D> LongerThanOrEqual<Week<D0>> for Week<D>
where
    D: StartDay,
    D0: StartDay,
{
}
impl<D0> LongerThanOrEqual<Week<D0>> for Quarter where D0: StartDay {}
impl<D0> LongerThanOrEqual<Week<D0>> for Month where D0: StartDay {}
impl<D0> LongerThanOrEqual<Week<D0>> for Year where D0: StartDay {}

impl<D0> LongerThan<Week<D0>> for Month where D0: StartDay {}
impl<D0> LongerThan<Week<D0>> for Quarter where D0: StartDay {}
impl<D0> LongerThan<Week<D0>> for Year where D0: StartDay {}

impl LongerThanOrEqual<Month> for Month {}
impl LongerThanOrEqual<Month> for Quarter {}
impl LongerThanOrEqual<Month> for Year {}

impl LongerThan<Month> for Quarter {}
impl LongerThan<Month> for Year {}

impl LongerThanOrEqual<Quarter> for Quarter {}
impl LongerThanOrEqual<Quarter> for Year {}

impl LongerThan<Quarter> for Year {}

impl LongerThanOrEqual<Year> for Year {}

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
    ParseIntDetailed(ParseIntError, String),
    ParseDateInternal {
        message: String,
        input: String,
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
            ParseIntDetailed(e, detail) => {
                write!(f, "Error parsing {detail} as integer: {e}")
            }
            ParseDateInternal {
                message,
                input,
                format,
            } => {
                write!(
                    f,
                    "Error parsing {input} as date due to {message} using format {format}"
                )
            }
        }
    }
}

#[cfg(feature = "std")]
pub type Result<T> = std::result::Result<T, Error>;

/// `TimeResolution` should be used for contigious series of periods in time
///
/// This makes sense for the time part of a discrete timeseries, with observations
/// occurring at regular times. Some examples are:
/// * A cash-flow report aggregated to days or months
/// * Dispatch periods in the Australian Electricity Market (and similar concepts in other energy markets)
pub trait TimeResolution: Copy + Eq + Ord + Monotonic {
    fn succ(&self) -> Self {
        self.succ_n(1)
    }

    fn pred(&self) -> Self {
        self.pred_n(1)
    }

    // the default impls are probably inefficient
    // makes sense to require just the n
    // and give the 1 for free
    fn succ_n(&self, n: u64) -> Self;

    fn pred_n(&self, n: u64) -> Self;

    fn start_datetime(&self) -> DateTime<Utc>;

    fn name(&self) -> String;

    fn convert<Out>(&self) -> Out
    where
        Out: TimeResolution + From<DateTime<Utc>>,
    {
        Out::from(self.start_datetime())
    }

    // handy functions.... to avoid turbofishing when it's a pain
    fn five_minute(&self) -> FiveMinute {
        self.convert()
    }
    fn half_hour(&self) -> HalfHour {
        self.convert()
    }
    fn hour(&self) -> Hour {
        self.convert()
    }
    fn day(&self) -> Day {
        self.convert()
    }
    fn month(&self) -> Month {
        self.convert()
    }
    fn year(&self) -> Year {
        self.convert()
    }
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
    fn between(&self, other: Self) -> i64;
}

pub trait FromMonotonic: Monotonic {
    fn from_monotonic(idx: i64) -> Self;
}

/// `SubDateResolution` should only be implemented for periods of strictly less than one day in length
pub trait SubDateResolution: TimeResolution {
    type Params: Copy;

    fn params(&self) -> Self::Params;

    fn occurs_on_date(&self) -> chrono::NaiveDate;

    fn from_utc_datetime(datetime: DateTime<Utc>, params: Self::Params) -> Self;

    // the first of the resolutions units that occurs on the day
    fn first_on_day(day: chrono::NaiveDate, params: Self::Params) -> Self;

    fn last_on_day(day: chrono::NaiveDate, params: Self::Params) -> Self {
        Self::first_on_day(day + chrono::Duration::days(1), params).pred()
    }
}

/// `DateResolution` should only be implemented for periods of one or more days in length
pub trait DateResolution: TimeResolution {
    type Params;

    fn params(&self) -> Self::Params;

    fn from_date(date: NaiveDate, params: Self::Params) -> Self;

    fn start(&self) -> chrono::NaiveDate;
}

/// `DateResolutionExt` implements some convenience methods for types that implement `DateResolution`
// This is an extra trait to avoid the methods being overriden
pub trait DateResolutionExt: DateResolution {
    #[cfg(feature = "std")]
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

    fn to_sub_date_resolution<R>(&self) -> range::TimeRange<R>
    where
        R: SubDateResolution<Params = Self::Params>,
    {
        range::TimeRange::from_bounds(
            R::first_on_day(self.start(), self.params()),
            R::last_on_day(self.end(), self.params()),
        )
    }

    fn rescale<Out>(&self) -> range::TimeRange<Out>
    where
        Out: DateResolution<Params = Self::Params>,
        Self: LongerThan<Out>,
    {
        range::TimeRange::from_bounds(
            Out::from_date(self.start(), self.params()),
            Out::from_date(self.end(), self.params()),
        )
    }
}

impl<T> DateResolutionExt for T where T: DateResolution {}
