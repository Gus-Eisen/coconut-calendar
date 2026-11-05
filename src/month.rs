use chk::{
    Action, ActionItem, Bumper, Context, Display, Flow, Form, FormComplete, FormItem, FormSubmit,
    FormValidState, Icons, ListItem, NumberVariant, Offset, Page, PageType, Review, Root, State,
    Success, TableItem, Theme, Watch,
};
use chrono::{Datelike, Months, NaiveDate};
use ramp::prism::air::Instance;

use crate::{Event, EventRegistry};

pub struct MonthScreen;
impl MonthScreen {
    pub fn root() -> Root {
        Root::custom(Page::watching(
            Watch::list::<Event>(),
            |_ctx: &mut Context, _theme: &Theme, events: Vec<Instance<Event>>| {
                let registry = EventRegistry::new(&events);
                let first = Event::window_start();
                let months = (0..12u32)
                    .map(|month| {
                        let start = first + Months::new(month);
                        let end = start + Months::new(1);
                        let days = start
                            .iter_days()
                            .take_while(|day| *day < end)
                            .map(|day| {
                                let events = registry.on(day);
                                let summary = match events.as_slice() {
                                    [] => None,
                                    [(_, event)] => Some(format!(
                                        "{} {}",
                                        event.datetime().format("%H:%M"),
                                        event.title()
                                    )),
                                    events => Some(format!("{} events", events.len())),
                                };
                                ListItem::plain(
                                    &day.day().to_string(),
                                    &day.format("%A").to_string(),
                                    summary.as_deref(),
                                    summary.is_some().then(|| DayScreen::flow(day)),
                                )
                            })
                            .collect();
                        Display::list(Some(&start.format("%B %Y").to_string()), days, None)
                    })
                    .collect();
                let new_event = Box::new(|_ctx: &mut Context, theme: &Theme| {
                    Flow::from_form(NewEvent::form(theme))
                });
                PageType::root(
                    "Calendar",
                    vec![],
                    months,
                    None,
                    Some(("Create Event".into(), new_event)),
                    None,
                )
            },
        ))
    }
}

pub struct DayScreen;
impl DayScreen {
    pub fn flow(day: NaiveDate) -> Flow {
        Flow::new(vec![Page::watching(
            Watch::list::<Event>(),
            move |_ctx: &mut Context, _theme: &Theme, events: Vec<Instance<Event>>| {
                let registry = EventRegistry::new(&events);
                let items = registry
                    .on(day)
                    .into_iter()
                    .map(|(instance, event)| {
                        let mut instance = instance.clone();
                        let delete = Action::custom(move |_ctx: &mut Context, _theme: &Theme| {
                            Event::delete(&mut instance)
                        });
                        Display::table(
                            event.title(),
                            vec![TableItem::new("Time", &event.datetime().format("%H:%M").to_string())],
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
}

pub struct NewEvent;
impl NewEvent {
    pub fn form(theme: &Theme) -> Form {
        let submit = Box::new(|ctx: &mut Context, states: &Vec<State>| {
            Event::create(ctx, Event::from_states(states));
            FormComplete::None
        }) as Box<dyn FormSubmit>;

        let review = |states: &Vec<State>| {
            let event = Event::from_states(states);
            vec![Display::table(
                "New event",
                vec![
                    TableItem::new("Title", event.title()),
                    TableItem::new("Date", &event.datetime().format("%-d %b %Y").to_string()),
                    TableItem::new("Time", &event.datetime().format("%H:%M").to_string()),
                ],
                None,
            )]
        };

        let success = |states: Vec<State>| {
            let event = Event::from_states(&states);
            (
                Icons::Checkmark,
                format!(
                    "Event saved for {} at {}",
                    event.datetime().format("%-d %b %Y"),
                    event.datetime().format("%H:%M")
                ),
            )
        };

        let items = vec![
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
            FormItem::number(
                "Date",
                NumberVariant::Date,
                |_ctx: &mut Context, date: String| {
                    if date.is_empty() {
                        FormValidState::Invalid
                    } else if Event::date(&date).is_some() {
                        FormValidState::Valid
                    } else {
                        FormValidState::InvalidWithData("Not a valid date.".into())
                    }
                },
            ),
            FormItem::number(
                "Time",
                NumberVariant::Time,
                |_ctx: &mut Context, time: String| {
                    if time.is_empty() {
                        FormValidState::Invalid
                    } else if Event::time(&time).is_some() {
                        FormValidState::Valid
                    } else {
                        FormValidState::InvalidWithData("Not a valid time.".into())
                    }
                },
            ),
        ];

        Form::flow(
            theme,
            items,
            Some(Review::new("New Event Review", review)),
            Some(Success::new("New Event", success)),
            submit,
        )
    }
}
