use chk::{
    Action, ActionItem, Bumper, Context, Display, Flow, Form, FormComplete, FormItem, FormSubmit,
    FormValidState, Icons, ListItem, NumberVariant, Offset, Page, PageType, Review, Root, State,
    Success, TableItem, Theme, Watch,
};
use chrono::{Datelike, Months, NaiveDate};
use ramp::prism::air::Instance;

use crate::{Calendar, Event};

pub struct MonthScreen;
impl MonthScreen {
    pub fn root(ctx: &mut Context) -> Root {
        Root::custom(Page::watching(
            Watch::contract(Calendar::open(ctx)),
            |_ctx: &mut Context, _theme: &Theme, calendar: Instance<Calendar>| {
                let state = calendar.pending();
                let first = Event::window_start();
                let months = (0..12u32)
                    .map(|month| {
                        let start = first + Months::new(month);
                        let end = start + Months::new(1);
                        let days = start
                            .iter_days()
                            .take_while(|day| *day < end)
                            .map(|day| {
                                let events = state.on(day);
                                let summary = match events.as_slice() {
                                    [] => None,
                                    [(_, event)] => Some(format!(
                                        "{} {}",
                                        event.start().format("%H:%M"),
                                        event.title()
                                    )),
                                    events => Some(format!("{} events", events.len())),
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
                    Flow::from_form(NewEvent::form(theme, calendar.clone()))
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
    pub fn flow(calendar: Instance<Calendar>, day: NaiveDate) -> Flow {
        Flow::new(vec![Page::watching(
            Watch::contract(calendar),
            move |_ctx: &mut Context, _theme: &Theme, calendar: Instance<Calendar>| {
                let items = calendar
                    .pending()
                    .on(day)
                    .into_iter()
                    .map(|(id, event)| {
                        let mut calendar = calendar.clone();
                        let delete = Action::custom(move |_ctx: &mut Context, _theme: &Theme| {
                            Calendar::delete_event(&mut calendar, id)
                        });
                        let (start, end) = (event.start(), event.end());
                        let end_format =
                            if end.date() == start.date() { "%H:%M" } else { "%H:%M (%-d %b)" };
                        let time = format!("{}–{}", start.format("%H:%M"), end.format(end_format));
                        let rows = [
                            Some(("Time", time)),
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
}

pub struct NewEvent;
impl NewEvent {
    pub fn form(theme: &Theme, mut calendar: Instance<Calendar>) -> Form {
        let submit = Box::new(move |_ctx: &mut Context, states: &Vec<State>| {
            Calendar::create_event(&mut calendar, Event::from_states(states));
            FormComplete::None
        }) as Box<dyn FormSubmit>;

        let review = |states: &Vec<State>| {
            let event = Event::from_states(states);
            let duration = event.duration_minutes();
            let rows = [
                Some(("Title", event.title().to_string())),
                Some(("Date", event.start().format("%-d %b %Y").to_string())),
                Some(("Time", event.start().format("%H:%M").to_string())),
                Some(("Duration", format!("{}h {}m", duration / 60, duration % 60))),
                event.location().map(|location| ("Location", location.to_string())),
                event.info().map(|info| ("Info", info.to_string())),
            ];
            vec![Display::table(
                "New event",
                rows.into_iter().flatten()
                    .map(|(label, value)| TableItem::new(label, &value)).collect(),
                None,
            )]
        };

        let success = |states: Vec<State>| {
            let event = Event::from_states(&states);
            (
                Icons::Checkmark,
                format!(
                    "Event saved for {} at {}",
                    event.start().format("%-d %b %Y"),
                    event.start().format("%H:%M")
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
            FormItem::number(
                "Duration (hours:minutes)",
                NumberVariant::Time,
                |_ctx: &mut Context, duration: String| {
                    if duration.is_empty() {
                        FormValidState::Invalid
                    } else if Event::duration(&duration).is_some() {
                        FormValidState::Valid
                    } else {
                        FormValidState::InvalidWithData("Between 00:01 and 24:00.".into())
                    }
                },
            ),
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

        Form::flow(
            theme,
            items,
            Some(Review::new("New Event Review", review)),
            Some(Success::new("New Event", success)),
            submit,
        )
    }
}
