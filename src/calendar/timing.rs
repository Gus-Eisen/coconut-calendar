use chk::State;
use chrono::offset::LocalResult;
use chrono::{DateTime, NaiveDate, NaiveDateTime, NaiveTime, TimeDelta, TimeZone, Timelike, Utc};
use chrono_tz::Tz;
use ramp::prism::air::{Id, Name};
use serde::{Deserialize, Serialize};

use std::cmp::Ordering;
use std::collections::BTreeSet;

use super::Event;

/// Chosen by the creating device, so it is known before AIR confirms the event.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct EventId(Id);

impl EventId {
    pub fn random() -> Self { EventId(Id::random()) }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Timing {
    AllDay {
        first_day: NaiveDate,
        last_day: NaiveDate,
    },
    /// `start` and `end` are clock times in `time_zone`.
    Timed {
        start: NaiveDateTime,
        end: NaiveDateTime,
        time_zone: Tz,
    },
}

impl Timing {
    /// The device's time zone, or UTC if the device reports one that isn't recognised.
    pub fn device_zone() -> Tz {
        iana_time_zone::get_timezone().ok().and_then(|zone| zone.parse().ok()).unwrap_or(Tz::UTC)
    }

    /// Today's date in the device's time zone.
    pub fn today() -> NaiveDate {
        Utc::now().with_timezone(&Self::device_zone()).date_naive()
    }

    pub fn format_time(time: NaiveTime) -> String { time.format("%-I:%M %p").to_string() }

    /// A list row's time, without ":00" on the hour: "11 PM", "9:30 PM".
    pub fn format_short_time(time: NaiveTime) -> String {
        time.format(if time.minute() == 0 { "%-I %p" } else { "%-I:%M %p" }).to_string()
    }

    /// The regular form's timing: it ends on `date`, or the next day if `end` is earlier than
    /// `start`. `None` if `end` equals `start`.
    pub fn from_times(date: NaiveDate, start: NaiveTime, end: NaiveTime, time_zone: Tz) -> Option<Timing> {
        let end_date = match end.cmp(&start) {
            Ordering::Greater => date,
            Ordering::Less => date.succ_opt()?,
            Ordering::Equal => return None,
        };
        Some(Timing::Timed { start: date.and_time(start), end: end_date.and_time(end), time_zone })
    }

    /// Reads a form's timing steps: First Day and Last Day, Date, Start Time and End Time (the
    /// overnight rule), or Change time's Date, Start Time, End Date and End Time. The validators
    /// have checked every input, so the unwraps can't fail.
    pub fn from_states(states: &[State], time_zone: Tz) -> Timing {
        let [State::Number(day), rest @ ..] = states else { unreachable!() };
        Self::from_start(Event::date(day).unwrap(), rest, time_zone)
    }

    /// Reads the timing steps after Date or First Day, starting on `day`. The Day view's Create
    /// Event already knows the date (year included), so its forms skip that step.
    pub fn from_start(day: NaiveDate, states: &[State], time_zone: Tz) -> Timing {
        let time = |input: &str, after, later_day| Event::time(input, after, later_day).unwrap();
        match states {
            [State::Number(last_day)] => {
                let last_day = Event::date_from(last_day, day).unwrap();
                Timing::AllDay { first_day: day, last_day }
            }
            [State::Text(start), State::Text(end)] => {
                let start = time(start, None, false);
                Timing::from_times(day, start, time(end, Some(start), false), time_zone).unwrap()
            }
            [State::Text(start), State::Number(end_day), State::Text(end)] => {
                let start = time(start, None, false);
                let end_day = Event::date_from(end_day, day).unwrap();
                let end = end_day.and_time(time(end, Some(start), end_day > day));
                Timing::Timed { start: day.and_time(start), end, time_zone }
            }
            _ => unreachable!(),
        }
    }

    pub fn is_valid(&self) -> bool {
        match *self {
            Timing::AllDay { first_day, last_day } => last_day >= first_day,
            Timing::Timed { start, end, time_zone } => {
                Self::instant(start, time_zone).zip(Self::instant(end, time_zone))
                    .is_some_and(|(start, end)| end > start)
            }
        }
    }

    /// The first day, or the start's date in the event's own time zone.
    pub fn start_date(&self) -> NaiveDate {
        match *self {
            Timing::AllDay { first_day, .. } => first_day,
            Timing::Timed { start, .. } => start.date(),
        }
    }

    /// How many calendar days after the start date it ends, in its own zone (0 for one day).
    pub fn days(&self) -> u64 {
        let (first, last) = match *self {
            Timing::AllDay { first_day, last_day } => (first_day, last_day),
            Timing::Timed { start, end, .. } => (start.date(), end.date()),
        };
        (last - first).num_days().max(0) as u64
    }

    /// The same timing moved to start on `date`: the same clock times, or the same number of
    /// days. The clock times are resolved again on the new date, so 9:00 stays 9:00 across
    /// daylight saving. `None` if the moved times can't be worked out.
    pub fn moved_to(&self, date: NaiveDate) -> Option<Timing> {
        let timing = match *self {
            Timing::AllDay { first_day, last_day } => {
                let last_day = date.checked_add_signed(last_day - first_day)?;
                Timing::AllDay { first_day: date, last_day }
            }
            Timing::Timed { start, end, time_zone } => {
                let moved = date.and_time(start.time());
                let end = moved.checked_add_signed(end - start)?;
                Timing::Timed { start: moved, end, time_zone }
            }
        };
        timing.is_valid().then_some(timing)
    }

    /// The start and end as clock times in `zone`; `None` for an all-day event.
    pub fn in_zone(&self, zone: Tz) -> Option<(NaiveDateTime, NaiveDateTime)> {
        let (start, end) = self.instants()?;
        Some((start.with_timezone(&zone).naive_local(), end.with_timezone(&zone).naive_local()))
    }

    /// Whether any part of the event falls on `date` in `zone`. All-day events ignore `zone`.
    pub fn covers(&self, date: NaiveDate, zone: Tz) -> bool {
        match self {
            Timing::AllDay { first_day, last_day } => (*first_day..=*last_day).contains(&date),
            Timing::Timed { .. } => self.instants().is_some_and(|(start, end)| {
                let midnight = |date: NaiveDate| {
                    Self::instant(date.and_time(NaiveTime::MIN), zone).unwrap()
                };
                start < midnight(date.succ_opt().unwrap()) && end > midnight(date)
            }),
        }
    }

    /// Panics on a time that can't be resolved. `is_valid` rejects those, so `apply` never
    /// stores one.
    pub fn instants(&self) -> Option<(DateTime<Utc>, DateTime<Utc>)> {
        match self {
            Timing::AllDay { .. } => None,
            Timing::Timed { start, end, time_zone } => Some((
                Self::instant(*start, *time_zone).unwrap(),
                Self::instant(*end, *time_zone).unwrap(),
            )),
        }
    }

    /// A clock time that doesn't exist (clocks spring forward) moves forward one hour; one that
    /// happens twice (clocks fall back) uses the first. `None` past the ends of chrono's range.
    fn instant(local: NaiveDateTime, zone: Tz) -> Option<DateTime<Utc>> {
        match zone.from_local_datetime(&local) {
            LocalResult::Single(time) | LocalResult::Ambiguous(time, _) => Some(time.to_utc()),
            LocalResult::None => Self::instant(local.checked_add_signed(TimeDelta::hours(1))?, zone),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TimeProposal {
    proposer: Name,
    occurrence: Option<NaiveDate>,
    timing: Timing,
    approvals: BTreeSet<Name>,
}
