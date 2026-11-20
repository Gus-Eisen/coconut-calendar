use chk::{
    Context, Display, Flow, Form, FormComplete, FormItem, FormSubmit, FormValidState, Icons,
    NumberVariant, Review, State, Success, TableItem, Theme,
};
use chrono::NaiveDate;
use ramp::prism::air::Instance;

use std::cell::RefCell;
use std::rc::Rc;

use crate::{Calendar, Category, Event, Timing};

use super::schedule::{ChangeRepeat, FormTimes};

pub struct NewEvent;
impl NewEvent {
    /// Asks which kind of event to create, then resets navigation and opens that kind's form.
    pub fn picker(theme: &Theme, calendar: Instance<Calendar>) -> Form {
        let submit = Box::new(move |_ctx: &mut Context, states: &Vec<State>| {
            let [State::Enumerator(index)] = states.as_slice() else { unreachable!() };
            let (kind, calendar) = (EventKind::options()[*index], calendar.clone());
            FormComplete::Return(Box::new(move |ctx: &mut Context, theme: &Theme| {
                let form = match kind {
                    EventKind::Regular => Self::form(theme, calendar.clone(), false),
                    EventKind::AllDay => Self::all_day_form(theme, calendar.clone(), false),
                    EventKind::Repeating => Self::form(theme, calendar.clone(), true),
                    EventKind::RepeatingAllDay => Self::all_day_form(theme, calendar.clone(), true),
                };
                Flow::from_form(form).build(ctx, theme)(ctx, theme)
            }))
        }) as Box<dyn FormSubmit>;

        let kinds = EventKind::options().map(|kind| (kind.label(), kind.description())).to_vec();
        Form::flow(theme, vec![FormItem::enumerator("Event Type", kinds)], None, None, submit)
    }

    /// The Regular form, or with `repeating` the Repeating form: the same steps plus Repeats,
    /// Every, Until and Times after End Time.
    pub fn form(theme: &Theme, calendar: Instance<Calendar>, repeating: bool) -> Form {
        let review = |states: &Vec<State>| {
            let event = Event::from_states(states);
            let Timing::Timed { time_zone, .. } = *event.timing() else { unreachable!() };
            // End Time's validator checked `is_valid`, so `in_zone` can't panic.
            let (start, end) = event.timing().in_zone(time_zone).unwrap();
            Self::review(&event, vec![
                ("Start", start.format("%-d %b %Y, %H:%M").to_string()),
                ("End", end.format("%-d %b %Y, %H:%M").to_string()),
                ("Time zone", time_zone.name().to_string()),
            ])
        };

        let success = |states: Vec<State>| {
            let event = Event::from_states(&states);
            let Timing::Timed { time_zone, .. } = *event.timing() else { unreachable!() };
            // End Time's validator checked `is_valid`, so `in_zone` can't panic.
            let (start, _) = event.timing().in_zone(time_zone).unwrap();
            let time = start.format("%H:%M");
            let message = match Self::first(&event) {
                Some(first) => {
                    format!("Repeating event saved, from {} at {time}", first.format("%-d %b %Y"))
                }
                None => format!("Event saved for {} at {time}", start.format("%-d %b %Y")),
            };
            (Icons::Checkmark, message)
        };

        let times = Rc::new(RefCell::new(FormTimes::default()));
        let end_times = times.clone();

        let mut timing = vec![
            FormTimes::date_step("Date", times.clone()),
            FormTimes::start_step("Start Time", times.clone()),
            FormItem::number(
                "End Time",
                NumberVariant::Time,
                move |_ctx: &mut Context, time: String| {
                    let times = end_times.borrow();
                    let timing = match (times.date(), times.start(), Event::time(&time)) {
                        _ if time.is_empty() => return FormValidState::Invalid,
                        (_, _, None) => {
                            return FormValidState::InvalidWithData("Not a valid time.".into());
                        }
                        (Some(date), Some(start), Some(end)) => {
                            Timing::from_times(date, start, end, Timing::device_zone())
                        }
                        _ => return FormValidState::Valid,
                    };
                    match timing {
                        None => FormValidState::InvalidWithData(
                            "Must be different from the start.".into(),
                        ),
                        Some(timing) if timing.is_valid() => FormValidState::Valid,
                        Some(_) => {
                            FormValidState::InvalidWithData("Must be after the start.".into())
                        }
                    }
                },
            ),
        ];
        if repeating { timing.extend(FormTimes::repeat_steps(times, false)); }

        Self::flow(
            theme,
            calendar,
            timing,
            Review::new("New Event Review", review),
            Success::new("New Event", success),
        )
    }

    /// The All-day form, or with `repeating` the Repeating all-day form.
    pub fn all_day_form(theme: &Theme, calendar: Instance<Calendar>, repeating: bool) -> Form {
        let review = |states: &Vec<State>| {
            let event = Event::from_states(states);
            let Timing::AllDay { first_day, last_day } = *event.timing() else { unreachable!() };
            Self::review(&event, vec![
                ("First day", first_day.format("%-d %b %Y").to_string()),
                ("Last day", last_day.format("%-d %b %Y").to_string()),
            ])
        };

        let success = |states: Vec<State>| {
            let event = Event::from_states(&states);
            let Timing::AllDay { first_day, last_day } = *event.timing() else { unreachable!() };
            let days = if first_day == last_day {
                first_day.format("%-d %b %Y").to_string()
            } else {
                format!("{} to {}", first_day.format("%-d %b %Y"), last_day.format("%-d %b %Y"))
            };
            let message = match Self::first(&event) {
                Some(first) => {
                    format!("Repeating all-day event saved, from {}", first.format("%-d %b %Y"))
                }
                None => format!("All-day event saved for {days}"),
            };
            (Icons::Checkmark, message)
        };

        let times = Rc::new(RefCell::new(FormTimes::default()));
        let mut timing = vec![
            FormTimes::date_step("First Day", times.clone()),
            FormTimes::end_date_step("Last Day", times.clone()),
        ];
        if repeating { timing.extend(FormTimes::repeat_steps(times, false)); }

        // chk cuts long titles short, so every new-event review has the same short title.
        Self::flow(
            theme,
            calendar,
            timing,
            Review::new("New Event Review", review),
            Success::new("New All-Day Event", success),
        )
    }

    /// A repeating event's first occurrence date, or `None` if it doesn't repeat or no date
    /// matches. A weekly set that skips the start's weekday starts later than Date.
    fn first(event: &Event) -> Option<NaiveDate> {
        event.recurrence()?.first(event.timing().start_date())
    }

    /// Puts the steps both forms share around `timing`, so `Event::from_states` can read either.
    fn flow(
        theme: &Theme, mut calendar: Instance<Calendar>, timing: Vec<FormItem>, review: Review,
        success: Success,
    ) -> Form {
        let submit = Box::new(move |_ctx: &mut Context, states: &Vec<State>| {
            Calendar::create_event(&mut calendar, Event::from_states(states));
            FormComplete::None
        }) as Box<dyn FormSubmit>;

        let start = vec![
            FormItem::text(
                "Event Title",
                "Event title",
                None,
                |_ctx: &mut Context, title: String| {
                    if title.trim().is_empty() {
                        FormValidState::Invalid
                    } else {
                        FormValidState::Valid
                    }
                },
            ),
            FormItem::enumerator(
                "Category",
                Category::options().map(|category| (category.label(), "")).to_vec(),
            ),
        ];
        let end = vec![
            FormItem::text(
                "Location",
                "Location (optional)",
                None,
                |_ctx: &mut Context, _location: String| FormValidState::Valid,
            ),
            FormItem::text(
                "Info",
                "Info (optional)",
                None,
                |_ctx: &mut Context, _info: String| FormValidState::Valid,
            ),
            FormItem::text(
                "Link",
                "Link (optional)",
                None,
                |_ctx: &mut Context, _link: String| FormValidState::Valid,
            ),
        ];

        let items = [start, timing, end].into_iter().flatten().collect();
        Form::flow(theme, items, Some(review), Some(success), submit)
    }

    fn review(event: &Event, timing: Vec<(&str, String)>) -> Vec<Display> {
        let start = event.timing().start_date();
        let repeat = match event.recurrence() {
            Some(recurrence) => ChangeRepeat::rows(Some(recurrence), start),
            None => vec![],
        };
        let rows = [
            vec![
                ("Title", event.title().to_string()),
                ("Category", event.category().label().to_string()),
            ],
            timing,
            repeat,
            [
                event.location().map(|location| ("Location", location.to_string())),
                event.info().map(|info| ("Info", info.to_string())),
                event.link().map(|link| ("Link", link.to_string())),
            ].into_iter().flatten().collect(),
        ];
        vec![Display::table(
            "New event",
            rows.into_iter().flatten()
                .map(|(label, value)| TableItem::new(label, &value)).collect(),
            None,
        )]
    }
}

/// The choices on the event-type picker. UI-only: never stored. Change time's Timed / All-day
/// picker uses the first two.
#[derive(Clone, Copy)]
pub enum EventKind {
    Regular,
    AllDay,
    Repeating,
    RepeatingAllDay,
}

impl EventKind {
    pub fn options() -> [EventKind; 4] {
        [EventKind::Regular, EventKind::AllDay, EventKind::Repeating, EventKind::RepeatingAllDay]
    }

    fn label(self) -> &'static str {
        match self {
            EventKind::Regular => "Regular",
            EventKind::AllDay => "All-Day",
            EventKind::Repeating => "Repeating",
            EventKind::RepeatingAllDay => "Repeating All-Day",
        }
    }

    fn description(self) -> &'static str {
        match self {
            EventKind::Regular => "Starts and ends at set times",
            EventKind::AllDay => "Lasts one or more whole days",
            EventKind::Repeating => "Happens again at set times",
            EventKind::RepeatingAllDay => "Whole days that repeat, like a birthday",
        }
    }
}
