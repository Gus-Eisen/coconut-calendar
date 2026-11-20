use chk::State;
use chrono::{Datelike, Days, Months, NaiveDate, Weekday};
use serde::{Deserialize, Serialize};

use std::collections::BTreeMap;

use super::{Event, Exception, Timing};

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
        let (interval, count) = (number(every).unwrap_or(1), number(times));
        Some(Recurrence { frequency, interval, until, count, exceptions: BTreeMap::new() })
    }

    pub fn until(&self) -> Option<NaiveDate> { self.until }
    pub fn exceptions(&self) -> &BTreeMap<NaiveDate, Exception> { &self.exceptions }
    pub fn exceptions_mut(&mut self) -> &mut BTreeMap<NaiveDate, Exception> { &mut self.exceptions }

    /// The rule without its exceptions, to compare with a rule a form built (forms never set
    /// exceptions).
    pub fn rule(&self) -> Recurrence { Recurrence { exceptions: BTreeMap::new(), ..self.clone() } }

    /// Keeps only the exceptions whose original date this rule still picks (from `start`).
    /// A dropped cancellation's date is no longer in the series, so nothing comes back.
    pub fn pruned(mut self, start: NaiveDate) -> Recurrence {
        let last = self.last(start);
        let rule = self.clone();
        self.exceptions.retain(|date, _| rule.occurs_on(*date, start, last));
        self
    }

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
    pub fn normalised(mut self) -> Recurrence {
        if let Frequency::Weekly { days } = &mut self.frequency {
            days.sort_by_key(|day| day.num_days_from_monday());
            days.dedup();
        }
        self
    }

    /// The rule once the start moves from `from` to `to`: monthly rules take the new start's
    /// day, and the "same day" weekly rule takes its weekday. Other weekly sets stay.
    pub fn moved(&self, from: NaiveDate, to: NaiveDate) -> Recurrence {
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
    pub fn occurs_on(&self, date: NaiveDate, start: NaiveDate, last: Option<NaiveDate>) -> bool {
        date >= start && last.is_none_or(|last| date <= last) && self.matches(date, start)
    }

    /// The last date it can happen on: `until`, the date of the `count`th occurrence, or `None`
    /// if it never ends. The count is walked one occurrence at a time, at most 999 steps, so
    /// `on` never has to walk from the start.
    pub fn last(&self, start: NaiveDate) -> Option<NaiveDate> {
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
