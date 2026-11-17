use chk::{Context, State};
use chrono::offset::LocalResult;
use chrono::{
    DateTime, Datelike, Days, Local, Months, NaiveDate, NaiveDateTime, NaiveTime, TimeDelta,
    TimeZone, Utc, Weekday,
};
use chrono_tz::Tz;
use ramp::prism::air::{Contract, Id, Instance, Metadata, Name};
use serde::{Deserialize, Serialize};

use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};

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
        let date = |input: &str| Event::date(input).unwrap();
        let time = |input: &str| Event::time(input).unwrap();
        match states {
            [State::Number(first_day), State::Number(last_day)] => {
                let first_day = date(first_day);
                let last_day = Event::date_from(last_day, first_day).unwrap();
                Timing::AllDay { first_day, last_day }
            }
            [State::Number(day), State::Number(start), State::Number(end)] => {
                Timing::from_times(date(day), time(start), time(end), time_zone).unwrap()
            }
            [
                State::Number(day), State::Number(start), State::Number(end_day),
                State::Number(end),
            ] => {
                let day = date(day);
                let end_day = Event::date_from(end_day, day).unwrap();
                Timing::Timed {
                    start: day.and_time(time(start)), end: end_day.and_time(time(end)), time_zone,
                }
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
    fn days(&self) -> u64 {
        let (first, last) = match *self {
            Timing::AllDay { first_day, last_day } => (first_day, last_day),
            Timing::Timed { start, end, .. } => (start.date(), end.date()),
        };
        (last - first).num_days().max(0) as u64
    }

    /// The same timing moved to start on `date`: the same clock times, or the same number of
    /// days. The clock times are resolved again on the new date, so 9:00 stays 9:00 across
    /// daylight saving. `None` if the moved times can't be worked out.
    fn moved_to(&self, date: NaiveDate) -> Option<Timing> {
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
    fn instants(&self) -> Option<(DateTime<Utc>, DateTime<Utc>)> {
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Category {
    Work,
    Personal,
}

impl Category {
    /// In the order the new-event form lists them; `Personal` first, as new events default to it.
    pub fn options() -> [Category; 2] { [Category::Personal, Category::Work] }

    pub fn label(self) -> &'static str {
        match self {
            Category::Work => "Work",
            Category::Personal => "Personal",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Attendee {
    name: Name,
    response: Response,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Response {
    Pending,
    Accepted,
    Declined,
    Tentative,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TimeProposal {
    proposer: Name,
    occurrence: Option<NaiveDate>,
    timing: Timing,
    approvals: BTreeSet<Name>,
}

/// Matching dates are counted from the start: every `interval` days, weeks (from the start's
/// Monday), months or years.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Recurrence {
    frequency: Frequency,
    interval: u16,
    until: Option<NaiveDate>,
    /// How many times it happens in total. Never set together with `until`.
    count: Option<u16>,
    /// Keyed by the date the occurrence would normally start on.
    exceptions: BTreeMap<NaiveDate, Exception>,
}

impl Recurrence {
    pub fn new(frequency: Frequency, interval: u16, until: Option<NaiveDate>, count: Option<u16>) -> Recurrence {
        Recurrence { frequency, interval, until, count, exceptions: BTreeMap::new() }
    }

    /// Reads the Repeats, Every, Until and Times steps; `None` for "Doesn't repeat". Until is
    /// the first such date on or after `earliest`. Blank or 0 (chk's number keypad starts at
    /// "0") means every 1, and no limit on Times. The validators have checked every input.
    pub fn from_states(states: &[State], start: NaiveDate, earliest: NaiveDate) -> Option<Recurrence> {
        let [
            State::Enumerator(repeat), State::Number(every), State::Number(until),
            State::Number(times),
        ] = states else {
            unreachable!()
        };
        let frequency = Repeat::options()[*repeat].frequency(start)?;
        let number = |input: &str| input.parse::<u16>().ok().filter(|number| *number > 0);
        let until = Event::date_from(until, earliest);
        Some(Recurrence::new(frequency, number(every).unwrap_or(1), until, number(times)))
    }

    pub fn frequency(&self) -> &Frequency { &self.frequency }
    pub fn interval(&self) -> u16 { self.interval }
    pub fn until(&self) -> Option<NaiveDate> { self.until }
    pub fn count(&self) -> Option<u16> { self.count }

    /// The rule `apply` and the forms share. Expects weekly days already `normalised`.
    pub fn is_valid(&self, timing: &Timing) -> bool {
        let start = timing.start_date();
        let frequency = match &self.frequency {
            Frequency::Daily | Frequency::Yearly => true,
            Frequency::Weekly { days } => !days.is_empty(),
            Frequency::Monthly { day_of_month } => u32::from(*day_of_month) == start.day(),
            Frequency::MonthlyWeekday { week, weekday } => {
                *week == WeekOfMonth::of(start) && *weekday == start.weekday()
            }
        };
        frequency && (1..=99).contains(&self.interval)
            && self.until.is_none_or(|until| until >= start)
            && self.count.is_none_or(|count| (1..=999).contains(&count))
            && (self.until.is_none() || self.count.is_none())
    }

    /// How often and on which days, as two short labels: ("Weekly", "Mon–Fri"),
    /// ("Every 2 weeks", "Tuesday"), ("Monthly", "Last Friday"), ("Daily", none). chk clips long
    /// table values, so the screens show them as separate Repeats and On rows. `start` is the
    /// series' start date, which a yearly rule repeats.
    pub fn describe(&self, start: NaiveDate) -> (String, Option<String>) {
        let every = |once: &str, unit: &str| match self.interval {
            1 => once.to_string(),
            interval => format!("Every {interval} {unit}"),
        };
        match &self.frequency {
            Frequency::Daily => (every("Daily", "days"), None),
            Frequency::Weekly { days } => {
                let which = match days.as_slice() {
                    [Weekday::Mon, Weekday::Tue, Weekday::Wed, Weekday::Thu, Weekday::Fri] => {
                        "Mon–Fri".to_string()
                    }
                    [Weekday::Sat, Weekday::Sun] => "Sat–Sun".to_string(),
                    [day] => Self::day_name(*day).to_string(),
                    days => days.iter().map(Weekday::to_string).collect::<Vec<_>>().join(", "),
                };
                (every("Weekly", "weeks"), Some(which))
            }
            Frequency::Monthly { day_of_month } => {
                (every("Monthly", "months"), Some(format!("The {}", Self::ordinal(*day_of_month))))
            }
            Frequency::MonthlyWeekday { week, weekday } => {
                let which = format!("{} {}", week.label(), Self::day_name(*weekday));
                (every("Monthly", "months"), Some(which))
            }
            Frequency::Yearly => {
                (every("Yearly", "years"), Some(start.format("%-d %b").to_string()))
            }
        }
    }

    /// How it ends: "Never", "On 30 Jun 2027" or "After 10 times".
    pub fn ends(&self) -> String {
        match (self.until, self.count) {
            (Some(until), _) => format!("On {}", until.format("%-d %b %Y")),
            (None, Some(1)) => "After 1 time".to_string(),
            (None, Some(count)) => format!("After {count} times"),
            (None, None) => "Never".to_string(),
        }
    }

    /// The first date on or after `start` that the rule picks. It differs from `start` only for
    /// a weekly set that doesn't include the start's weekday.
    pub fn first(&self, start: NaiveDate) -> Option<NaiveDate> {
        self.next(start.pred_opt()?, start)
    }

    /// Weekly days sorted Mon..Sun without duplicates, so equal rules compare equal.
    fn normalised(mut self) -> Recurrence {
        if let Frequency::Weekly { days } = &mut self.frequency {
            days.sort_by_key(|day| day.num_days_from_monday());
            days.dedup();
        }
        self
    }

    /// The rule once the start moves from `from` to `to`: monthly rules take the new start's
    /// day, and the "same day" weekly rule takes its weekday. Other weekly sets stay.
    fn moved(&self, from: NaiveDate, to: NaiveDate) -> Recurrence {
        let frequency = match &self.frequency {
            Frequency::Monthly { .. } => Frequency::Monthly { day_of_month: to.day() as u8 },
            Frequency::MonthlyWeekday { .. } => {
                Frequency::MonthlyWeekday { week: WeekOfMonth::of(to), weekday: to.weekday() }
            }
            Frequency::Weekly { days } if *days == [from.weekday()] => {
                Frequency::Weekly { days: vec![to.weekday()] }
            }
            frequency => frequency.clone(),
        };
        Recurrence { frequency, ..self.clone() }
    }

    /// Whether an occurrence starts on `date`, given the series' start and `last` (see `last`).
    fn occurs_on(&self, date: NaiveDate, start: NaiveDate, last: Option<NaiveDate>) -> bool {
        date >= start && last.is_none_or(|last| date <= last) && self.matches(date, start)
    }

    /// The last date it can happen on: `until`, the date of the `count`th occurrence, or `None`
    /// if it never ends. The count is walked one occurrence at a time, at most 999 steps, so
    /// `on` never has to walk from the start.
    fn last(&self, start: NaiveDate) -> Option<NaiveDate> {
        let count = match (self.until, self.count) {
            (Some(until), _) => return Some(until),
            (None, Some(count)) => count,
            (None, None) => return None,
        };
        // No first occurrence (only past chrono's range): end before the start.
        let Some(mut last) = self.first(start) else { return start.pred_opt() };
        for _ in 1..count {
            match self.next(last, start) {
                Some(next) => last = next,
                None => break,
            }
        }
        Some(last)
    }

    /// Whether the rule picks `date`, ignoring the start and end.
    fn matches(&self, date: NaiveDate, start: NaiveDate) -> bool {
        let interval = i64::from(self.interval);
        match &self.frequency {
            Frequency::Daily => (date - start).num_days().rem_euclid(interval) == 0,
            Frequency::Weekly { days } => {
                days.contains(&date.weekday())
                    && Self::weeks(start, date).is_some_and(|weeks| weeks.rem_euclid(interval) == 0)
            }
            _ => {
                Self::months(start, date).rem_euclid(self.period()) == 0
                    && self.in_month(date, start) == Some(date)
            }
        }
    }

    /// The first date after `after` that the rule picks. Jumps over whole periods that can't
    /// match, so each call takes a few steps, not one per day.
    fn next(&self, after: NaiveDate, start: NaiveDate) -> Option<NaiveDate> {
        let interval = i64::from(self.interval);
        match &self.frequency {
            Frequency::Daily => {
                let since = (after - start).num_days().rem_euclid(interval);
                after.checked_add_days(Days::new((interval - since) as u64))
            }
            Frequency::Weekly { days } => {
                let mut day = after.succ_opt()?;
                // At most a week to the end of a matching week, one jump, then a week to a day.
                for _ in 0..16 {
                    let weeks = Self::weeks(start, day)?.rem_euclid(interval);
                    if weeks == 0 {
                        if days.contains(&day.weekday()) { return Some(day); }
                        day = day.succ_opt()?;
                    } else {
                        let monday = i64::from(day.weekday().num_days_from_monday());
                        let to_next = (interval - weeks) * 7 - monday;
                        day = day.checked_add_days(Days::new(to_next as u64))?;
                    }
                }
                None
            }
            _ => {
                let period = self.period();
                let first_month = start.with_day(1)?;
                let mut months = Self::months(start, after).max(0) / period * period;
                // Months without the date are skipped; a yearly 29 February waits for a leap year.
                for _ in 0..100 {
                    let month = Months::new(u32::try_from(months).ok()?);
                    let month = first_month.checked_add_months(month)?;
                    if let Some(date) = self.in_month(month, start) && date > after {
                        return Some(date);
                    }
                    months += period;
                }
                None
            }
        }
    }

    /// The months between matches of a monthly or yearly rule.
    fn period(&self) -> i64 {
        let interval = i64::from(self.interval);
        if self.frequency == Frequency::Yearly { interval * 12 } else { interval }
    }

    /// The date a monthly or yearly rule picks in `month`'s month, if it has one.
    fn in_month(&self, month: NaiveDate, start: NaiveDate) -> Option<NaiveDate> {
        let first = month.with_day(1)?;
        match &self.frequency {
            Frequency::Monthly { day_of_month } => first.with_day(u32::from(*day_of_month)),
            Frequency::MonthlyWeekday { week, weekday } => week.date_in(first, *weekday),
            Frequency::Yearly if month.month() == start.month() => first.with_day(start.day()),
            Frequency::Yearly | Frequency::Daily | Frequency::Weekly { .. } => None,
        }
    }

    /// Whole weeks from `from`'s Monday to `to`'s Monday.
    fn weeks(from: NaiveDate, to: NaiveDate) -> Option<i64> {
        let monday = |date: NaiveDate| {
            date.checked_sub_days(Days::new(date.weekday().num_days_from_monday().into()))
        };
        Some((monday(to)? - monday(from)?).num_days() / 7)
    }

    fn months(from: NaiveDate, to: NaiveDate) -> i64 {
        let index = |date: NaiveDate| i64::from(date.year()) * 12 + i64::from(date.month0());
        index(to) - index(from)
    }

    fn day_name(day: Weekday) -> &'static str {
        match day {
            Weekday::Mon => "Monday",
            Weekday::Tue => "Tuesday",
            Weekday::Wed => "Wednesday",
            Weekday::Thu => "Thursday",
            Weekday::Fri => "Friday",
            Weekday::Sat => "Saturday",
            Weekday::Sun => "Sunday",
        }
    }

    /// "1st", "2nd", "3rd", "11th", "22nd".
    fn ordinal(day: u8) -> String {
        let suffix = match (day % 10, day % 100) {
            (_, 11..=13) => "th",
            (1, _) => "st",
            (2, _) => "nd",
            (3, _) => "rd",
            _ => "th",
        };
        format!("{day}{suffix}")
    }
}

/// Postcard encodes variants by position: only append new variants, never insert or reorder.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Frequency {
    Daily,
    Weekly { days: Vec<Weekday> },
    Monthly { day_of_month: u8 },
    Yearly,
    /// The same weekday in the same week of every month, e.g. the 2nd Tuesday.
    MonthlyWeekday { week: WeekOfMonth, weekday: Weekday },
}

/// Which week of the month a date is in: days 1–7 are the first, 8–14 the second, 15–21 the
/// third, 22–28 the fourth, and 29–31 the last.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WeekOfMonth {
    First,
    Second,
    Third,
    Fourth,
    Last,
}

impl WeekOfMonth {
    pub fn of(date: NaiveDate) -> WeekOfMonth {
        match date.day() {
            1..=7 => WeekOfMonth::First,
            8..=14 => WeekOfMonth::Second,
            15..=21 => WeekOfMonth::Third,
            22..=28 => WeekOfMonth::Fourth,
            _ => WeekOfMonth::Last,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            WeekOfMonth::First => "1st",
            WeekOfMonth::Second => "2nd",
            WeekOfMonth::Third => "3rd",
            WeekOfMonth::Fourth => "4th",
            WeekOfMonth::Last => "Last",
        }
    }

    /// This week's `weekday` in the month that starts on `first`. Every month has a 4th and a
    /// last one of each weekday.
    fn date_in(self, first: NaiveDate, weekday: Weekday) -> Option<NaiveDate> {
        let nth = |n| NaiveDate::from_weekday_of_month_opt(first.year(), first.month(), weekday, n);
        match self {
            WeekOfMonth::First => nth(1),
            WeekOfMonth::Second => nth(2),
            WeekOfMonth::Third => nth(3),
            WeekOfMonth::Fourth => nth(4),
            WeekOfMonth::Last => nth(5).or_else(|| nth(4)),
        }
    }
}

/// The Repeats step's choices, in form order. Never stored: each becomes a `Frequency`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Repeat {
    EveryDay,
    Weekdays,
    Weekends,
    SameWeekday,
    SameDate,
    SameWeekdayOfMonth,
    SameDayOfYear,
    Never,
}

impl Repeat {
    /// `Never` ("Doesn't repeat") is last, so the new-event forms can leave it off.
    pub fn options() -> [Repeat; 8] {
        [
            Repeat::EveryDay, Repeat::Weekdays, Repeat::Weekends, Repeat::SameWeekday,
            Repeat::SameDate, Repeat::SameWeekdayOfMonth, Repeat::SameDayOfYear, Repeat::Never,
        ]
    }

    /// The step is built before Date is typed, so the labels can't name the day.
    pub fn label(self) -> (&'static str, &'static str) {
        match self {
            Repeat::EveryDay => ("Every day", ""),
            Repeat::Weekdays => ("Every weekday", "Mon–Fri"),
            Repeat::Weekends => ("Every weekend", "Sat–Sun"),
            Repeat::SameWeekday => ("Every week", "On the same day"),
            Repeat::SameDate => ("Every month", "On the same date"),
            Repeat::SameWeekdayOfMonth => ("Every month", "On the same weekday, e.g. 2nd Tue"),
            Repeat::SameDayOfYear => ("Every year", "On the same date"),
            Repeat::Never => ("Doesn't repeat", ""),
        }
    }

    /// `None` for `Never`.
    pub fn frequency(self, start: NaiveDate) -> Option<Frequency> {
        use Weekday::{Fri, Mon, Sat, Sun, Thu, Tue, Wed};
        Some(match self {
            Repeat::EveryDay => Frequency::Daily,
            Repeat::Weekdays => Frequency::Weekly { days: vec![Mon, Tue, Wed, Thu, Fri] },
            Repeat::Weekends => Frequency::Weekly { days: vec![Sat, Sun] },
            Repeat::SameWeekday => Frequency::Weekly { days: vec![start.weekday()] },
            Repeat::SameDate => Frequency::Monthly { day_of_month: start.day() as u8 },
            Repeat::SameWeekdayOfMonth => Frequency::MonthlyWeekday {
                week: WeekOfMonth::of(start), weekday: start.weekday(),
            },
            Repeat::SameDayOfYear => Frequency::Yearly,
            Repeat::Never => return None,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Exception {
    Cancelled,
    Changed(Override),
}

/// What's different about one occurrence. `None` means "same as the series".
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Override {
    title: Option<String>,
    timing: Option<Timing>,
    location: Option<Option<String>>,
    info: Option<Option<String>>,
    video_link: Option<Option<String>>,
    attachments: Option<Vec<String>>,
    responses: BTreeMap<Name, Response>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Event {
    title: String,
    timing: Timing,
    location: Option<String>,
    info: Option<String>,
    attendees: Vec<Attendee>,
    video_link: Option<String>,
    attachments: Vec<String>,
    recurrence: Option<Recurrence>,
    category: Category,
    time_proposals: Vec<TimeProposal>,
    private: bool,
    deleted: bool,
}

impl Event {
    pub fn window_start() -> NaiveDate {
        Local::now().date_naive().with_day(1).unwrap()
    }

    pub fn date(input: &str) -> Option<NaiveDate> {
        Self::date_from(input, Local::now().date_naive())
    }

    /// The first `DD-MM` date on or after `earliest`: `earliest`'s year, else the next year.
    pub fn date_from(input: &str, earliest: NaiveDate) -> Option<NaiveDate> {
        [earliest.year(), earliest.year() + 1].into_iter()
            .filter_map(|year| {
                NaiveDate::parse_from_str(&format!("{input}-{year}"), "%d-%m-%Y").ok()
            })
            .find(|date| *date >= earliest)
    }

    pub fn time(input: &str) -> Option<NaiveTime> {
        NaiveTime::parse_from_str(input, "%H:%M").ok()
    }

    /// Reads any new-event form. They differ only in their timing steps and, for the repeating
    /// forms, the four repeat steps after them. Timed events use the device's time zone.
    pub fn from_states(states: &[State]) -> Event {
        let [
            State::Text(title), State::Enumerator(category), steps @ .., State::Text(location),
            State::Text(info),
        ] = states else {
            unreachable!()
        };
        let (timing, repeat) = match steps {
            [timing @ .., State::Enumerator(_), _, _, _] => steps.split_at(timing.len()),
            timing => (timing, &[][..]),
        };
        let timing = Timing::from_states(timing, Timing::device_zone());
        let start = timing.start_date();
        Event {
            title: title.trim().to_string(),
            timing,
            location: Self::optional(location),
            info: Self::optional(info),
            attendees: vec![],
            video_link: None,
            attachments: vec![],
            recurrence: (!repeat.is_empty()).then(|| Recurrence::from_states(repeat, start, start))
                .flatten(),
            category: Category::options()[*category],
            time_proposals: vec![],
            private: false,
            deleted: false,
        }
    }

    pub fn title(&self) -> &str { &self.title }
    pub fn timing(&self) -> &Timing { &self.timing }
    pub fn location(&self) -> Option<&str> { self.location.as_deref() }
    pub fn info(&self) -> Option<&str> { self.info.as_deref() }
    pub fn attendees(&self) -> &[Attendee] { &self.attendees }
    pub fn video_link(&self) -> Option<&str> { self.video_link.as_deref() }
    pub fn attachments(&self) -> &[String] { &self.attachments }
    pub fn recurrence(&self) -> Option<&Recurrence> { self.recurrence.as_ref() }
    pub fn category(&self) -> Category { self.category }
    pub fn time_proposals(&self) -> &[TimeProposal] { &self.time_proposals }
    pub fn private(&self) -> bool { self.private }

    pub fn optional(input: &str) -> Option<String> {
        let input = input.trim();
        (!input.is_empty()).then(|| input.to_string())
    }

    /// The timing of the occurrence that would normally start on `date` (its original date), or
    /// `None` if there's none. An event that doesn't repeat has one, on its start date.
    pub fn occurrence(&self, date: NaiveDate) -> Option<Timing> {
        let start = self.timing.start_date();
        let Some(recurrence) = &self.recurrence else {
            return (date == start).then_some(self.timing);
        };
        if !recurrence.occurs_on(date, start, recurrence.last(start)) { return None; }
        self.timing.moved_to(date)
    }

    /// The occurrences covering `date` in `zone`, with their original dates. Only start dates
    /// that could reach `date` are checked, not every date since the start: an occurrence ends
    /// `days` calendar days after its start date in its own zone, and zones differ by up to 26
    /// hours, so in `zone` it starts at most 2 days earlier or later. So it covers `date` only
    /// if it starts between `days + 2` days before `date` and 2 days after.
    fn covering(&self, date: NaiveDate, zone: Tz) -> Vec<(NaiveDate, Timing)> {
        let start = self.timing.start_date();
        let Some(recurrence) = &self.recurrence else {
            return match self.timing.covers(date, zone) {
                true => vec![(start, self.timing)],
                false => vec![],
            };
        };
        let last = recurrence.last(start);
        let earliest = date.checked_sub_days(Days::new(self.timing.days() + 2));
        let (Some(earliest), Some(latest)) = (earliest, date.checked_add_days(Days::new(2))) else {
            return vec![];
        };
        earliest.iter_days().take_while(|day| *day <= latest)
            .filter(|day| recurrence.occurs_on(*day, start, last))
            .filter_map(|day| Some((day, self.timing.moved_to(day)?)))
            .filter(|(_, timing)| timing.covers(date, zone))
            .collect()
    }
}

/// One appearance of an event on a day.
#[derive(Debug, Clone, Copy)]
pub struct Occurrence<'a> {
    id: EventId,
    event: &'a Event,
    date: NaiveDate,
    timing: Timing,
}

impl<'a> Occurrence<'a> {
    pub fn id(&self) -> EventId { self.id }
    pub fn event(&self) -> &'a Event { self.event }
    /// The date it would normally start on, in the event's own zone. It identifies the
    /// occurrence within its series.
    pub fn date(&self) -> NaiveDate { self.date }
    /// This occurrence's own timing; the event's `timing` is only the first one's.
    pub fn timing(&self) -> &Timing { &self.timing }
}

/// Postcard encodes variants by position: only append new variants, never insert or reorder.
#[derive(Debug, Clone, Hash, Serialize, Deserialize)]
pub enum CalendarMessage {
    CreateEvent { id: EventId, event: Box<Event> },
    DeleteEvent { id: EventId },
    SetTitle { id: EventId, title: String },
    SetTiming { id: EventId, timing: Timing },
    SetLocation { id: EventId, location: Option<String> },
    SetInfo { id: EventId, info: Option<String> },
    SetVideoLink { id: EventId, link: Option<String> },
    SetCategory { id: EventId, category: Category },
    SetPrivate { id: EventId, private: bool },
    AddAttachment { id: EventId, link: String },
    RemoveAttachment { id: EventId, link: String },
    SetRecurrence { id: EventId, recurrence: Option<Recurrence> },
}

/// The one AIR record per identity. Deleting only marks an event; AIR never erases data.
#[derive(Debug, Clone, Default, Hash, Serialize, Deserialize)]
pub struct Calendar {
    events: BTreeMap<EventId, Event>,
}

impl Calendar {
    /// The location is `hash((), signer)`, so this returns the existing calendar if there is one.
    pub fn open(ctx: &mut Context) -> Instance<Calendar> {
        ctx.air.instances::<Calendar>().create(())
    }

    pub fn create_event(calendar: &mut Instance<Calendar>, event: Event) {
        let id = EventId::random();
        calendar.send(CalendarMessage::CreateEvent { id, event: Box::new(event) });
    }

    pub fn delete_event(calendar: &mut Instance<Calendar>, id: EventId) {
        calendar.send(CalendarMessage::DeleteEvent { id });
    }

    pub fn set_title(calendar: &mut Instance<Calendar>, id: EventId, title: String) {
        calendar.send(CalendarMessage::SetTitle { id, title });
    }

    pub fn set_timing(calendar: &mut Instance<Calendar>, id: EventId, timing: Timing) {
        calendar.send(CalendarMessage::SetTiming { id, timing });
    }

    pub fn set_location(calendar: &mut Instance<Calendar>, id: EventId, location: Option<String>) {
        calendar.send(CalendarMessage::SetLocation { id, location });
    }

    pub fn set_info(calendar: &mut Instance<Calendar>, id: EventId, info: Option<String>) {
        calendar.send(CalendarMessage::SetInfo { id, info });
    }

    pub fn set_video_link(calendar: &mut Instance<Calendar>, id: EventId, link: Option<String>) {
        calendar.send(CalendarMessage::SetVideoLink { id, link });
    }

    pub fn set_category(calendar: &mut Instance<Calendar>, id: EventId, category: Category) {
        calendar.send(CalendarMessage::SetCategory { id, category });
    }

    pub fn set_private(calendar: &mut Instance<Calendar>, id: EventId, private: bool) {
        calendar.send(CalendarMessage::SetPrivate { id, private });
    }

    pub fn add_attachment(calendar: &mut Instance<Calendar>, id: EventId, link: String) {
        calendar.send(CalendarMessage::AddAttachment { id, link });
    }

    pub fn remove_attachment(calendar: &mut Instance<Calendar>, id: EventId, link: String) {
        calendar.send(CalendarMessage::RemoveAttachment { id, link });
    }

    /// `None` stops the event repeating.
    pub fn set_recurrence(calendar: &mut Instance<Calendar>, id: EventId, recurrence: Option<Recurrence>) {
        calendar.send(CalendarMessage::SetRecurrence { id, recurrence });
    }

    /// `None` if there's no such event or it was deleted.
    pub fn event(&self, id: EventId) -> Option<&Event> {
        self.events.get(&id).filter(|event| !event.deleted)
    }

    /// Occurrences on `date` in the viewer's time zone `zone`, including every occurrence of a
    /// repeating event that covers it: all-day ones first, then by start.
    pub fn on(&self, date: NaiveDate, zone: Tz) -> Vec<Occurrence<'_>> {
        let mut occurrences = self.events.iter()
            .filter(|(_, event)| !event.deleted)
            .flat_map(|(id, event)| {
                event.covering(date, zone).into_iter().map(|(original, timing)| {
                    Occurrence { id: *id, event, date: original, timing }
                })
            })
            .collect::<Vec<_>>();
        occurrences.sort_by_key(|occurrence| occurrence.timing.instants().map(|(start, _)| start));
        occurrences
    }

    /// The event an edit message may change: it exists and isn't deleted.
    fn editable(&mut self, id: EventId) -> Option<&mut Event> {
        self.events.get_mut(&id).filter(|event| !event.deleted)
    }
}

impl Contract for Calendar {
    type Init = ();
    type Message = CalendarMessage;
    type Result = ();

    fn id() -> Id { Id::hash("CoconutCalendarV2") }

    fn init(_init: (), _metadata: Metadata) -> Self { Calendar::default() }

    fn apply(&mut self, message: CalendarMessage, _metadata: Metadata) {
        match message {
            CalendarMessage::CreateEvent { id, event } => {
                if event.title.trim().is_empty() || !event.timing.is_valid() { return; }
                let recurrence = event.recurrence.clone().map(Recurrence::normalised);
                if let Some(recurrence) = &recurrence
                    && (!recurrence.is_valid(&event.timing) || !recurrence.exceptions.is_empty()) {
                    return;
                }
                let event = Event {
                    attendees: vec![],
                    recurrence,
                    time_proposals: vec![],
                    deleted: false,
                    ..*event
                };
                self.events.entry(id).or_insert(event);
            }
            CalendarMessage::DeleteEvent { id } => {
                if let Some(event) = self.events.get_mut(&id) { event.deleted = true; }
            }
            CalendarMessage::SetTitle { id, title } => {
                if let Some(event) = self.editable(id)
                    && let Some(title) = Event::optional(&title) {
                    event.title = title;
                }
            }
            CalendarMessage::SetTiming { id, timing } => {
                if let Some(event) = self.editable(id) && timing.is_valid() {
                    // Keep the repeat rule meaningful if the start date moves. If it no longer
                    // fits (an `until` before the new start), the whole message is ignored.
                    let (from, to) = (event.timing.start_date(), timing.start_date());
                    let recurrence = event.recurrence.as_ref()
                        .map(|recurrence| recurrence.moved(from, to));
                    if recurrence.as_ref().is_some_and(|recurrence| !recurrence.is_valid(&timing)) {
                        return;
                    }
                    event.timing = timing;
                    event.recurrence = recurrence;
                    event.time_proposals.clear();
                }
            }
            CalendarMessage::SetLocation { id, location } => {
                if let Some(event) = self.editable(id) {
                    event.location = location.as_deref().and_then(Event::optional);
                }
            }
            CalendarMessage::SetInfo { id, info } => {
                if let Some(event) = self.editable(id) {
                    event.info = info.as_deref().and_then(Event::optional);
                }
            }
            CalendarMessage::SetVideoLink { id, link } => {
                if let Some(event) = self.editable(id) {
                    event.video_link = link.as_deref().and_then(Event::optional);
                }
            }
            CalendarMessage::SetCategory { id, category } => {
                if let Some(event) = self.editable(id) { event.category = category; }
            }
            CalendarMessage::SetPrivate { id, private } => {
                if let Some(event) = self.editable(id) { event.private = private; }
            }
            CalendarMessage::AddAttachment { id, link } => {
                if let Some(event) = self.editable(id)
                    && let Some(link) = Event::optional(&link)
                    && !event.attachments.contains(&link) {
                    event.attachments.push(link);
                }
            }
            CalendarMessage::RemoveAttachment { id, link } => {
                if let Some(event) = self.editable(id) {
                    event.attachments.retain(|attachment| *attachment != link);
                }
            }
            CalendarMessage::SetRecurrence { id, recurrence } => {
                let Some(event) = self.editable(id) else { return };
                let Some(recurrence) = recurrence else {
                    event.recurrence = None;
                    return;
                };
                // Exceptions only change through their own messages (step 3b), so the event's
                // own are kept and any sent here are ignored.
                let exceptions = event.recurrence.as_ref()
                    .map(|current| current.exceptions.clone()).unwrap_or_default();
                let recurrence = Recurrence { exceptions, ..recurrence }.normalised();
                if recurrence.is_valid(&event.timing) { event.recurrence = Some(recurrence); }
            }
        }
    }
}
