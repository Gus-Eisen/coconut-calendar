use chk::{
    Action, ActionItem, Bumper, Context, Display, Flow, Form, FormComplete, FormItem, FormSubmit,
    FormValidState, Icons, ListItem, NumberVariant, Offset, Page, PageType, Review, Root, State,
    Success, TableItem, Theme, Watch,
};
use chrono::{Datelike, Months, NaiveDate, NaiveDateTime, NaiveTime};
use chrono_tz::Tz;
use ramp::prism::air::Instance;

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use crate::{Calendar, Category, Event, Timing};

pub struct MonthScreen;
impl MonthScreen {
    pub fn root(ctx: &mut Context) -> Root {
        Root::custom(Page::watching(
            Watch::contract(Calendar::open(ctx)),
            |_ctx: &mut Context, _theme: &Theme, calendar: Instance<Calendar>| {
                let state = calendar.pending();
                let zone = Timing::device_zone();
                let first = Event::window_start();
                let months = (0..12u32)
                    .map(|month| {
                        let start = first + Months::new(month);
                        let end = start + Months::new(1);
                        let days = start
                            .iter_days()
                            .take_while(|day| *day < end)
                            .map(|day| {
                                let occurrences = state.on(day, zone);
                                let summary = match occurrences.as_slice() {
                                    [] => None,
                                    [occurrence] => Some(format!(
                                        "{} {}",
                                        DayScreen::time_label(occurrence.event(), day, zone),
                                        occurrence.event().title()
                                    )),
                                    occurrences => Some(format!("{} events", occurrences.len())),
                                };
                                ListItem::plain(
                                    &day.day().to_string(),
                                    &day.format("%A").to_string(),
                                    summary.as_deref(),
                                    summary.is_some()
                                        .then(|| DayScreen::flow(calendar.clone(), day)),
                                )
                            })
                            .collect();
                        Display::list(Some(&start.format("%B %Y").to_string()), days, None)
                    })
                    .collect();
                drop(state);
                let new_event = Box::new(move |_ctx: &mut Context, theme: &Theme| {
                    Flow::from_form(NewEvent::picker(theme, calendar.clone()))
                });
                PageType::root(
                    "Calendar", vec![], months, None,
                    Some(("Create Event".into(), new_event)), None,
                )
            },
        ))
    }
}

pub struct DayScreen;
impl DayScreen {
    pub fn flow(calendar: Instance<Calendar>, day: NaiveDate) -> Flow {
        Flow::new(vec![Page::watching(
            Watch::contract(calendar),
            move |_ctx: &mut Context, _theme: &Theme, calendar: Instance<Calendar>| {
                let zone = Timing::device_zone();
                let items = calendar
                    .pending()
                    .on(day, zone)
                    .into_iter()
                    .map(|occurrence| {
                        let (id, event) = (occurrence.id(), occurrence.event());
                        let mut calendar = calendar.clone();
                        let delete = Action::custom(move |_ctx: &mut Context, _theme: &Theme| {
                            Calendar::delete_event(&mut calendar, id)
                        });
                        let rows = [
                            Some(("Time", Self::time_label(event, day, zone))),
                            event.location().map(|location| ("Location", location.to_string())),
                            event.info().map(|info| ("Info", info.to_string())),
                        ];
                        Display::table(
                            event.title(),
                            rows.into_iter().flatten()
                                .map(|(label, value)| TableItem::new(label, &value)).collect(),
                            Some(vec![ActionItem::new(delete, "Delete", Icons::Delete)]),
                        )
                    })
                    .collect();
                PageType::display(
                    &day.format("%A %-d %B").to_string(),
                    items,
                    None,
                    Bumper::None,
                    Offset::Start,
                )
            },
        )])
    }

    /// "All day", or the start and end in `zone`, with the date on any part that isn't on `day`.
    pub fn time_label(event: &Event, day: NaiveDate, zone: Tz) -> String {
        let Some((start, end)) = event.timing().in_zone(zone) else { return "All day".into() };
        let label = |time: NaiveDateTime| {
            let format = if time.date() == day { "%H:%M" } else { "%H:%M (%-d %b)" };
            time.format(format).to_string()
        };
        format!("{}–{}", label(start), label(end))
    }
}

pub struct NewEvent;
impl NewEvent {
    /// Asks which kind of event to create, then resets navigation and opens that kind's form.
    pub fn picker(theme: &Theme, calendar: Instance<Calendar>) -> Form {
        let submit = Box::new(move |_ctx: &mut Context, states: &Vec<State>| {
            let [State::Enumerator(index)] = states.as_slice() else { unreachable!() };
            let (kind, calendar) = (EventKind::options()[*index], calendar.clone());
            FormComplete::Return(Box::new(move |ctx: &mut Context, theme: &Theme| {
                let form = match kind {
                    EventKind::Regular => Self::form(theme, calendar.clone()),
                    EventKind::AllDay => Self::all_day_form(theme, calendar.clone()),
                };
                Flow::from_form(form).build(ctx, theme)(ctx, theme)
            }))
        }) as Box<dyn FormSubmit>;

        let kinds = EventKind::options().map(|kind| (kind.label(), kind.description())).to_vec();
        Form::flow(theme, vec![FormItem::enumerator("Event Type", kinds)], None, None, submit)
    }

    pub fn form(theme: &Theme, calendar: Instance<Calendar>) -> Form {
        let review = |states: &Vec<State>| {
            let event = Event::from_states(states);
            let Timing::Timed { start, end, time_zone } = *event.timing() else { unreachable!() };
            Self::review(&event, vec![
                ("Start", start.format("%-d %b %Y, %H:%M").to_string()),
                ("End", end.format("%-d %b %Y, %H:%M").to_string()),
                ("Time zone", time_zone.name().to_string()),
            ])
        };

        let success = |states: Vec<State>| {
            let event = Event::from_states(&states);
            let Timing::Timed { start, .. } = *event.timing() else { unreachable!() };
            (
                Icons::Checkmark,
                format!(
                    "Event saved for {} at {}",
                    start.format("%-d %b %Y"),
                    start.format("%H:%M")
                ),
            )
        };

        let times = Rc::new(RefCell::new(FormTimes::default()));
        let (date_times, start_times, end_times) = (times.clone(), times.clone(), times);

        let timing = vec![
            FormItem::number(
                "Date",
                NumberVariant::Date,
                move |_ctx: &mut Context, date: String| {
                    let parsed = Event::date(&date);
                    date_times.borrow_mut().date = parsed;
                    if date.is_empty() {
                        FormValidState::Invalid
                    } else if parsed.is_some() {
                        FormValidState::Valid
                    } else {
                        FormValidState::InvalidWithData("Not a valid date.".into())
                    }
                },
            ),
            FormItem::number(
                "Start Time",
                NumberVariant::Time,
                move |_ctx: &mut Context, time: String| {
                    let parsed = Event::time(&time);
                    start_times.borrow_mut().start = parsed;
                    if time.is_empty() {
                        FormValidState::Invalid
                    } else if parsed.is_some() {
                        FormValidState::Valid
                    } else {
                        FormValidState::InvalidWithData("Not a valid time.".into())
                    }
                },
            ),
            FormItem::number(
                "End Time",
                NumberVariant::Time,
                move |_ctx: &mut Context, time: String| {
                    let times = end_times.borrow();
                    let timing = match (times.date, times.start, Event::time(&time)) {
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

        Self::flow(
            theme,
            calendar,
            timing,
            Review::new("New Event Review", review),
            Success::new("New Event", success),
        )
    }

    pub fn all_day_form(theme: &Theme, calendar: Instance<Calendar>) -> Form {
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
            (Icons::Checkmark, format!("All-day event saved for {days}"))
        };

        let first_day = Rc::new(Cell::new(None));
        let first_day_step = first_day.clone();

        let timing = vec![
            FormItem::number(
                "First Day",
                NumberVariant::Date,
                move |_ctx: &mut Context, date: String| {
                    let parsed = Event::date(&date);
                    first_day_step.set(parsed);
                    if date.is_empty() {
                        FormValidState::Invalid
                    } else if parsed.is_some() {
                        FormValidState::Valid
                    } else {
                        FormValidState::InvalidWithData("Not a valid date.".into())
                    }
                },
            ),
            FormItem::number(
                "Last Day",
                NumberVariant::Date,
                move |_ctx: &mut Context, date: String| {
                    let timing = match (first_day.get(), Event::date(&date)) {
                        _ if date.is_empty() => return FormValidState::Invalid,
                        (_, None) => {
                            return FormValidState::InvalidWithData("Not a valid date.".into());
                        }
                        (Some(first_day), Some(last_day)) => Timing::AllDay { first_day, last_day },
                        _ => return FormValidState::Valid,
                    };
                    if timing.is_valid() {
                        FormValidState::Valid
                    } else {
                        FormValidState::InvalidWithData("Must be on or after the first day.".into())
                    }
                },
            ),
        ];

        Self::flow(
            theme,
            calendar,
            timing,
            Review::new("New All-Day Event Review", review),
            Success::new("New All-Day Event", success),
        )
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
        ];

        let items = [start, timing, end].into_iter().flatten().collect();
        Form::flow(theme, items, Some(review), Some(success), submit)
    }

    fn review(event: &Event, timing: Vec<(&str, String)>) -> Vec<Display> {
        let rows = [
            vec![
                ("Title", event.title().to_string()),
                ("Category", event.category().label().to_string()),
            ],
            timing,
            [
                event.location().map(|location| ("Location", location.to_string())),
                event.info().map(|info| ("Info", info.to_string())),
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

/// The choices on the event-type picker. UI-only: never stored.
#[derive(Clone, Copy)]
enum EventKind {
    Regular,
    AllDay,
}

impl EventKind {
    fn options() -> [EventKind; 2] { [EventKind::Regular, EventKind::AllDay] }

    fn label(self) -> &'static str {
        match self {
            EventKind::Regular => "Regular",
            EventKind::AllDay => "All-Day",
        }
    }

    fn description(self) -> &'static str {
        match self {
            EventKind::Regular => "Starts and ends at set times",
            EventKind::AllDay => "Lasts one or more whole days",
        }
    }
}

/// The form's answers so far. chk only shows each validator its own step, so End Time reads
/// the start from here.
#[derive(Default)]
struct FormTimes {
    date: Option<NaiveDate>,
    start: Option<NaiveTime>,
}
