use chk::{
    Context, Display, Flow, Form, FormComplete, FormItem, FormSubmit, FormValidState,
    NumberVariant, Review, State, TableItem, Theme,
};
use chrono::{Local, NaiveDate, NaiveTime};
use chrono_tz::Tz;
use ramp::prism::air::Instance;

use std::cell::RefCell;
use std::rc::Rc;

use crate::{Calendar, Event, EventId, Recurrence, Repeat, Timing};

use super::EventScreen;
use super::detail::Scope;
use super::new_event::EventKind;

/// Change time: a Timed / All-day picker, then that kind's steps and a Current / New review.
/// Timed steps use the event's own time zone, so an event set in Tokyo stays in Tokyo time.
pub struct ChangeTime;
impl ChangeTime {
    /// For the whole series, the times are the first occurrence's, and its start can't move
    /// past the repeat's Until. One occurrence starts from its own times and can move to any
    /// date the form allows, even past Until.
    pub fn picker(theme: &Theme, calendar: Instance<Calendar>, id: EventId, series: &Event, scope: Scope) -> Form {
        let (current, latest) = match scope {
            Scope::Series => {
                (*series.timing(), series.recurrence().and_then(|recurrence| recurrence.until()))
            }
            Scope::Occurrence(date) => (series.occurrence(date).unwrap_or(*series.timing()), None),
        };
        let series_now = series.clone();
        let submit = Box::new(move |_ctx: &mut Context, states: &Vec<State>| {
            let [State::Enumerator(index)] = states.as_slice() else { unreachable!() };
            let (kind, calendar, series) =
                (EventKind::options()[*index], calendar.clone(), series_now.clone());
            FormComplete::Return(Box::new(move |ctx: &mut Context, theme: &Theme| {
                let times = FormTimes { latest, ..FormTimes::default() };
                let times = Rc::new(RefCell::new(times));
                let edit = Edit {
                    calendar: calendar.clone(), id, series: series.clone(), scope, current,
                };
                let form = match kind {
                    EventKind::AllDay => Self::all_day(theme, edit, times),
                    _ => Self::timed(theme, edit, times),
                };
                Flow::from_form(form).build(ctx, theme)(ctx, theme)
            }))
        }) as Box<dyn FormSubmit>;

        let now = format!("Now: {}", EventScreen::when(&current, Timing::device_zone()));
        let kinds = vec![("Timed", now.as_str()), ("All-day", "Whole days, no times")];
        Form::flow(theme, vec![FormItem::enumerator("Timing", kinds)], None, None, submit)
    }

    /// An all-day event has no zone of its own, so it becomes timed in the device's zone. The
    /// step titles don't name the zone (chk cuts long titles short); the review does.
    fn timed(theme: &Theme, edit: Edit, times: Rc<RefCell<FormTimes>>) -> Form {
        let zone = match edit.current {
            Timing::Timed { time_zone, .. } => time_zone,
            Timing::AllDay { .. } => Timing::device_zone(),
        };
        let end_times = times.clone();

        let end_time = FormItem::number(
            "End Time",
            NumberVariant::Time,
            move |_ctx: &mut Context, input: String| {
                let times = end_times.borrow();
                let checked = Event::time(&input).ok_or("Not a valid time.").and_then(|end| {
                    let (Some(date), Some(start), Some(end_date)) =
                        (times.date, times.start, times.end_date) else { return Ok(()) };
                    let timing = Timing::Timed {
                        start: date.and_time(start), end: end_date.and_time(end), time_zone: zone,
                    };
                    if timing.is_valid() { Ok(()) } else { Err("Must be after the start.") }
                });
                FormTimes::validity(&input, checked)
            },
        );
        let steps = vec![
            FormTimes::date_step("Date", times.clone()),
            FormTimes::start_step("Start Time", times.clone()),
            FormTimes::end_date_step("End Date", times),
            end_time,
        ];
        Self::flow(theme, edit, zone, steps)
    }

    fn all_day(theme: &Theme, edit: Edit, times: Rc<RefCell<FormTimes>>) -> Form {
        let steps = vec![
            FormTimes::date_step("First Day", times.clone()),
            FormTimes::end_date_step("Last Day", times),
        ];
        Self::flow(theme, edit, Timing::device_zone(), steps)
    }

    /// Continue sends `set_timing` (whole series) or `change_occurrence` (one occurrence)
    /// unless nothing changed, then goes back to the detail page. A whole-series review warns
    /// about single changes the new time drops.
    fn flow(theme: &Theme, edit: Edit, zone: Tz, steps: Vec<FormItem>) -> Form {
        let Edit { mut calendar, id, series, scope, current } = edit;
        let review = move |states: &Vec<State>| {
            let new = Timing::from_states(states, zone);
            let dropped = match scope {
                Scope::Series => series.dropped_by_timing(&new),
                Scope::Occurrence(_) => 0,
            };
            [Self::table("Current", &current), Self::table("New", &new)].into_iter()
                .chain(ChangeRepeat::warning(dropped))
                .collect()
        };
        let submit = Box::new(move |_ctx: &mut Context, states: &Vec<State>| {
            let timing = Timing::from_states(states, zone);
            match scope {
                _ if timing == current => {}
                Scope::Series => Calendar::set_timing(&mut calendar, id, timing),
                Scope::Occurrence(date) => {
                    EventScreen::change(&mut calendar, id, date, |changes, series| {
                        changes.set_timing(timing, series.series_timing(date))
                    })
                }
            }
            EventScreen::back()
        }) as Box<dyn FormSubmit>;
        Form::flow(theme, steps, Some(Review::new("Change Time Review", review)), None, submit)
    }

    /// A timed table names the zone its times are in; all-day dates are the same everywhere.
    fn table(label: &str, timing: &Timing) -> Display {
        let rows = match *timing {
            Timing::AllDay { first_day, last_day } => vec![
                ("First day", first_day.format("%-d %b %Y").to_string()),
                ("Last day", last_day.format("%-d %b %Y").to_string()),
            ],
            Timing::Timed { time_zone, .. } => {
                // Stored timings and checked form timings are valid, so `in_zone` can't panic.
                let (start, end) = timing.in_zone(time_zone).unwrap();
                vec![
                    ("Start", start.format("%-d %b %Y, %H:%M").to_string()),
                    ("End", end.format("%-d %b %Y, %H:%M").to_string()),
                    ("Time zone", time_zone.name().to_string()),
                ]
            }
        };
        let rows = rows.into_iter().map(|(label, value)| TableItem::new(label, &value)).collect();
        Display::table(label, rows, None)
    }
}

/// Change repeat: Repeats (with "Doesn't repeat" last), Every, Until and Times, then a Current
/// / New review. Until is read from the later of the series' start and today, so it can't land
/// in the past for a series that started earlier.
pub struct ChangeRepeat;
impl ChangeRepeat {
    pub fn form(theme: &Theme, mut calendar: Instance<Calendar>, id: EventId, event: &Event) -> Form {
        // Compared without exceptions, as the form's rule never has any.
        let current = event.recurrence().map(Recurrence::rule);
        let start = event.timing().start_date();
        let earliest = start.max(Local::now().date_naive());
        let times = FormTimes { date: Some(earliest), ..FormTimes::default() };
        let times = Rc::new(RefCell::new(times));

        let (review_current, series) = (current.clone(), event.clone());
        let review = move |states: &Vec<State>| {
            let new = Recurrence::from_states(states, start, earliest);
            let dropped = match new == review_current {
                true => 0,
                false => series.dropped(series.timing(), new.as_ref()),
            };
            [
                Self::table("Current", review_current.as_ref(), start),
                Self::table("New", new.as_ref(), start),
            ].into_iter().chain(Self::warning(dropped)).collect()
        };
        let submit = Box::new(move |_ctx: &mut Context, states: &Vec<State>| {
            let new = Recurrence::from_states(states, start, earliest);
            if new != current { Calendar::set_recurrence(&mut calendar, id, new); }
            EventScreen::back()
        }) as Box<dyn FormSubmit>;
        let steps = FormTimes::repeat_steps(times, true);
        Form::flow(theme, steps, Some(Review::new("Change Repeat Review", review)), None, submit)
    }

    /// The line a whole-series review shows when the change drops single changes; Change time
    /// uses it too.
    fn warning(dropped: usize) -> Option<Display> {
        match dropped {
            0 => None,
            1 => Some(Display::instructions("1 single change will be removed.")),
            dropped => {
                Some(Display::instructions(&format!("{dropped} single changes will be removed.")))
            }
        }
    }

    fn table(label: &str, recurrence: Option<&Recurrence>, start: NaiveDate) -> Display {
        let rows = Self::rows(recurrence, start).into_iter()
            .map(|(label, value)| TableItem::new(label, &value)).collect();
        Display::table(label, rows, None)
    }

    /// Repeats, On, First (only when it isn't the start date) and Ends, as the reviews and
    /// the detail page show them. Short rows, because chk clips long table values.
    pub fn rows(recurrence: Option<&Recurrence>, start: NaiveDate) -> Vec<(&'static str, String)> {
        let Some(recurrence) = recurrence else {
            return vec![("Repeats", "Doesn't repeat".into())];
        };
        let (often, days) = recurrence.describe(start);
        let first = match recurrence.first(start) {
            Some(first) if first == start => None,
            Some(first) => Some(first.format("%a %-d %b %Y").to_string()),
            None => Some("No date matches".to_string()),
        };
        [
            Some(("Repeats", often)), days.map(|days| ("On", days)),
            first.map(|first| ("First", first)), Some(("Ends", recurrence.ends())),
        ].into_iter().flatten().collect()
    }
}

/// The form's answers so far. chk only shows each validator its own step, so later steps read
/// the earlier answers from here. The new-event forms, Change time and Change repeat share these
/// steps.
#[derive(Default)]
pub struct FormTimes {
    date: Option<NaiveDate>,
    start: Option<NaiveTime>,
    end_date: Option<NaiveDate>,
    until: Option<NaiveDate>,
    /// The latest Date allowed: a repeating event's Until, in Change time.
    latest: Option<NaiveDate>,
}

impl FormTimes {
    /// No answers yet, except the start date when it's already known: the Day view's Create
    /// Event skips Date / First Day, and Last Day, Until and End Time read the start from here.
    pub fn starting(date: Option<NaiveDate>) -> FormTimes {
        FormTimes { date, ..FormTimes::default() }
    }

    pub fn date(&self) -> Option<NaiveDate> { self.date }
    pub fn start(&self) -> Option<NaiveTime> { self.start }

    /// Date or First Day: on or after today, and not after `latest`.
    pub fn date_step(title: &str, times: Rc<RefCell<FormTimes>>) -> FormItem {
        FormItem::number(title, NumberVariant::Date, move |_ctx: &mut Context, input: String| {
            let mut times = times.borrow_mut();
            let date = Event::date(&input);
            times.date = date;
            let checked = match date {
                None => Err("Not a valid date."),
                Some(date) if times.latest.is_some_and(|latest| date > latest) => {
                    Err("Must be on or before the repeat ends.")
                }
                Some(date) => Ok(date),
            };
            Self::validity(&input, checked)
        })
    }

    /// Repeats, Every, Until and Times. Until is read from `date`: the start, or Change repeat's
    /// earliest date. Blank is allowed on the last three; chk's keypads can't show a custom hint,
    /// so what blank means only shows on the review.
    pub fn repeat_steps(times: Rc<RefCell<FormTimes>>, never: bool) -> Vec<FormItem> {
        let options = Repeat::options().into_iter()
            .filter(|repeat| never || *repeat != Repeat::Never)
            .map(Repeat::label)
            .collect();
        let until_times = times.clone();
        vec![
            FormItem::enumerator("Repeats", options),
            FormItem::number("Every", NumberVariant::Number, |_ctx: &mut Context, input: String| {
                Self::count_validity(&input, 99)
            }),
            FormItem::number(
                "Until",
                NumberVariant::Date,
                move |_ctx: &mut Context, input: String| {
                    let mut times = until_times.borrow_mut();
                    times.until = times.date.and_then(|start| Event::date_from(&input, start));
                    match times.until {
                        _ if input.is_empty() => FormValidState::Valid,
                        Some(_) => FormValidState::Valid,
                        None => FormValidState::InvalidWithData("Not a valid date.".into()),
                    }
                },
            ),
            FormItem::number(
                "Times",
                NumberVariant::Number,
                move |_ctx: &mut Context, input: String| {
                    let set = input.parse::<u16>().is_ok_and(|count| count > 0);
                    if set && times.borrow().until.is_some() {
                        return FormValidState::InvalidWithData(
                            "Use Until or Times, not both.".into(),
                        );
                    }
                    Self::count_validity(&input, 999)
                },
            ),
        ]
    }

    /// Blank (chk's number keypad shows "0" until something is typed), or 1 to `max`.
    fn count_validity(input: &str, max: u16) -> FormValidState {
        match input.parse::<u16>() {
            _ if input.is_empty() => FormValidState::Valid,
            Ok(count) if count <= max => FormValidState::Valid,
            _ => FormValidState::InvalidWithData(format!("Must be 1 to {max}.")),
        }
    }

    pub fn start_step(title: &str, times: Rc<RefCell<FormTimes>>) -> FormItem {
        FormItem::number(title, NumberVariant::Time, move |_ctx: &mut Context, input: String| {
            let start = Event::time(&input);
            times.borrow_mut().start = start;
            Self::validity(&input, start.ok_or("Not a valid time."))
        })
    }

    /// End Date or Last Day: the first such date on or after the start, at most 28 days later.
    /// The limit is a form rule only; `Timing::is_valid` and `apply` accept longer events.
    pub fn end_date_step(title: &str, times: Rc<RefCell<FormTimes>>) -> FormItem {
        FormItem::number(title, NumberVariant::Date, move |_ctx: &mut Context, input: String| {
            let mut times = times.borrow_mut();
            let end = times.date.and_then(|start| Event::date_from(&input, start));
            let checked = match times.date.zip(end) {
                None => Err("Not a valid date."),
                Some((start, end)) if (end - start).num_days() > 28 => {
                    Err("Must be on or after the start.")
                }
                Some((_, end)) => Ok(end),
            };
            times.end_date = checked.ok();
            Self::validity(&input, checked)
        })
    }

    /// Blank turns Continue off without an error; otherwise `checked`'s error shows inline.
    fn validity<T>(input: &str, checked: Result<T, &str>) -> FormValidState {
        match checked {
            _ if input.is_empty() => FormValidState::Invalid,
            Ok(_) => FormValidState::Valid,
            Err(error) => FormValidState::InvalidWithData(error.into()),
        }
    }
}

/// What a Change time flow edits, and the timing it starts from.
#[derive(Clone)]
struct Edit {
    calendar: Instance<Calendar>,
    id: EventId,
    series: Event,
    scope: Scope,
    current: Timing,
}
