use chk::{
    Action, ActionItem, Bumper, Context, Display, Flow, Form, FormAction, FormComplete, FormItem,
    FormSubmit, FormValidState, Icons, ListItem, NumberVariant, Offset, Page, PageType, Review,
    Root, State, Success, TableItem, Theme, Watch,
};
use chrono::{Datelike, Local, Months, NaiveDate, NaiveDateTime, NaiveTime};
use chrono_tz::Tz;
use ramp::prism::air::Instance;

use std::cell::RefCell;
use std::rc::Rc;

use crate::{Calendar, Category, Event, EventId, Recurrence, Repeat, Timing};

pub struct MonthScreen;
impl MonthScreen {
    pub fn root(ctx: &mut Context) -> Root {
        Root::custom(Page::watching(
            Watch::contract(Calendar::open(ctx)),
            |_ctx: &mut Context, _theme: &Theme, calendar: Instance<Calendar>| {
                let state = calendar.pending();
                let zone = Timing::device_zone();
                let first = Event::window_start();
                let months = (0..13u32)
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
                                        DayScreen::time_label(occurrence.timing(), day, zone),
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
                let rows = calendar
                    .pending()
                    .on(day, zone)
                    .into_iter()
                    .map(|occurrence| {
                        let event = occurrence.event();
                        let category = event.category().label();
                        // A repeating event says so instead of showing its location.
                        let subtitle = match (event.recurrence(), event.location()) {
                            (Some(_), _) => format!("{category} · Repeats"),
                            (None, Some(location)) => format!("{category} · {location}"),
                            (None, None) => category.to_string(),
                        };
                        let time = Self::time_label(occurrence.timing(), day, zone);
                        let detail =
                            EventScreen::flow(calendar.clone(), occurrence.id(), occurrence.date());
                        ListItem::plain(event.title(), &subtitle, Some(&time), Some(detail))
                    })
                    .collect();
                PageType::display(
                    &day.format("%A %-d %B").to_string(),
                    vec![Display::list(None, rows, None)],
                    None,
                    Bumper::None,
                    Offset::Start,
                )
            },
        )])
    }

    /// "All day", or the start and end in `zone`, with the date on any part that isn't on `day`.
    pub fn time_label(timing: &Timing, day: NaiveDate, zone: Tz) -> String {
        let Some((start, end)) = timing.in_zone(zone) else { return "All day".into() };
        let label = |time: NaiveDateTime| {
            let format = if time.date() == day { "%H:%M" } else { "%H:%M (%-d %b)" };
            time.format(format).to_string()
        };
        format!("{}–{}", label(start), label(end))
    }
}

/// One event's detail page and the flows that edit it.
pub struct EventScreen;
impl EventScreen {
    /// Rebuilds after every change to the calendar, so it always shows the latest values. `date`
    /// is the original date of the tapped occurrence; the Time row shows that occurrence, or the
    /// first one if an edit means there's no longer one on `date`.
    pub fn flow(calendar: Instance<Calendar>, id: EventId, date: NaiveDate) -> Flow {
        Flow::new(vec![Page::watching(
            Watch::contract(calendar),
            move |_ctx: &mut Context, theme: &Theme, calendar: Instance<Calendar>| {
                let Some(event) = calendar.pending().event(id).cloned() else {
                    let deleted = Display::instructions("This event was deleted.");
                    return PageType::display(
                        "Event", vec![deleted], None, Bumper::None, Offset::Center,
                    );
                };
                let zone = Timing::device_zone();
                let timing = event.occurrence(date).unwrap_or(*event.timing());
                let start = event.timing().start_date();
                let repeat = match event.recurrence() {
                    Some(recurrence) => ChangeRepeat::rows(Some(recurrence), start),
                    None => vec![],
                };
                let rows = [
                    vec![("Time", Self::when(&timing, zone))],
                    repeat,
                    [
                        Some(("Category", event.category().label().to_string())),
                        event.location().map(|location| ("Location", location.to_string())),
                        event.info().map(|info| ("Info", info.to_string())),
                        Some(("Private", if event.private() { "Yes" } else { "No" }.to_string())),
                        event.video_link().map(|link| ("Video link", link.to_string())),
                    ].into_iter().flatten().collect(),
                ];

                let edit = Flow::from_form(Self::edit_details(calendar.clone(), id, &event));
                let header_edit = edit.clone();
                let mut deleting = calendar.clone();
                let delete = Action::custom(move |ctx: &mut Context, theme: &Theme| {
                    Calendar::delete_event(&mut deleting, id);
                    Self::back().run(ctx, theme);
                });
                let forms = [
                    ("Change time", ChangeTime::picker(theme, calendar.clone(), id, &event)),
                    ("Change repeat", ChangeRepeat::form(theme, calendar.clone(), id, &event)),
                    ("Change category", Self::category(theme, calendar.clone(), id, &event)),
                    ("Change privacy", Self::privacy(theme, calendar.clone(), id, &event)),
                ];
                let actions = [ActionItem::new(Action::flow(edit), "Edit details", Icons::Edit)]
                    .into_iter()
                    .chain(forms.into_iter().map(|(label, form)| {
                        ActionItem::new(Action::flow(Flow::from_form(form)), label, Icons::Edit)
                    }))
                    .chain([ActionItem::new(delete, "Delete", Icons::Delete)])
                    .collect();
                let table = Display::table(
                    "Event",
                    rows.into_iter().flatten()
                        .map(|(label, value)| TableItem::new(label, &value)).collect(),
                    Some(actions),
                );

                let add_link = Self::add_link(theme, calendar.clone(), id);
                let add_link = Action::flow(Flow::from_form(add_link));
                let attachments = Display::data_list(
                    "Attachments",
                    event.attachments().is_empty().then_some("No links yet."),
                    vec![],
                    Some(vec![ActionItem::new(add_link, "Add link", Icons::Link)]),
                );
                let links = event.attachments().iter().map(|link| {
                    let (mut calendar, removed) = (calendar.clone(), link.clone());
                    let remove = Action::custom(move |_ctx: &mut Context, _theme: &Theme| {
                        Calendar::remove_attachment(&mut calendar, id, removed.clone())
                    });
                    let delete = ActionItem::new(remove, "Delete", Icons::Delete);
                    Display::cta(link, None, "Link", vec![delete])
                });

                let header = Box::new(move |_ctx: &mut Context, _theme: &Theme| {
                    header_edit.clone()
                });
                PageType::display(
                    event.title(),
                    [table, attachments].into_iter().chain(links).collect(),
                    Some((Icons::Edit, header)),
                    Bumper::None,
                    Offset::Start,
                )
            },
        )])
    }

    /// The detail page's time in `zone`: "14:00–15:00, Tue 3 Nov", or "All day, Tue 3 Nov".
    fn when(timing: &Timing, zone: Tz) -> String {
        let day = |date: NaiveDate| date.format("%a %-d %b").to_string();
        match *timing {
            Timing::AllDay { first_day, last_day } if first_day == last_day => {
                format!("All day, {}", day(first_day))
            }
            Timing::AllDay { first_day, last_day } => {
                format!("All day, {} – {}", day(first_day), day(last_day))
            }
            Timing::Timed { .. } => {
                // `apply` stores only valid timings and `occurrence` returns only valid ones, so
                // `in_zone` can't panic.
                let (start, _) = timing.in_zone(zone).unwrap();
                let time = DayScreen::time_label(timing, start.date(), zone);
                format!("{time}, {}", day(start.date()))
            }
        }
    }

    /// Ends the current flow and shows the page under it: the detail page, or the day screen
    /// after Delete.
    fn back() -> FormComplete { FormComplete::Return(Box::new(|_: &mut Context, _: &Theme| {})) }

    /// Save sends one message per field that changed, comparing after trimming.
    fn edit_details(mut calendar: Instance<Calendar>, id: EventId, event: &Event) -> Form {
        let save = Box::new(move |_ctx: &mut Context, states: &Vec<State>| {
            let [State::Text(title), State::Text(location), State::Text(info), State::Text(link)] =
                states.as_slice() else { unreachable!() };
            // The live event, not the one the page opened with, so a second Save on the same page
            // only sends what changed since the first.
            let Some(event) = calendar.pending().event(id).cloned() else {
                return FormComplete::None;
            };
            if let Some(title) = Event::optional(title) && title != event.title() {
                Calendar::set_title(&mut calendar, id, title);
            }
            let location = Event::optional(location);
            if location.as_deref() != event.location() {
                Calendar::set_location(&mut calendar, id, location);
            }
            let info = Event::optional(info);
            if info.as_deref() != event.info() { Calendar::set_info(&mut calendar, id, info); }
            let link = Event::optional(link);
            if link.as_deref() != event.video_link() {
                Calendar::set_video_link(&mut calendar, id, link);
            }
            FormComplete::None
        }) as Box<dyn FormSubmit>;

        let optional = |label: &str, value: Option<&str>| {
            FormItem::text_with_preset(
                label, value.unwrap_or(""), None,
                |_ctx: &mut Context, _value: String| FormValidState::Valid,
            )
        };
        let items = vec![
            FormItem::text_with_preset(
                "Title", event.title(), None,
                |_ctx: &mut Context, title: String| match title.trim().is_empty() {
                    true => FormValidState::Invalid,
                    false => FormValidState::Valid,
                },
            ),
            optional("Location", event.location()),
            optional("Info", event.info()),
            optional("Video link", event.video_link()),
        ];
        Form::page("Edit Event", items, vec![], save)
    }

    fn category(theme: &Theme, mut calendar: Instance<Calendar>, id: EventId, event: &Event) -> Form {
        let categories = Category::options();
        // `options` lists every category, so `position` always finds it.
        let current = categories.iter().position(|category| *category == event.category()).unwrap();
        Self::choice(theme, "Category", categories.map(Category::label), current, move |index| {
            Calendar::set_category(&mut calendar, id, categories[index])
        })
    }

    fn privacy(theme: &Theme, mut calendar: Instance<Calendar>, id: EventId, event: &Event) -> Form {
        let current = usize::from(event.private());
        Self::choice(theme, "Privacy", ["Not private", "Private"], current, move |index| {
            Calendar::set_private(&mut calendar, id, index == 1)
        })
    }

    /// A one-step radio. chk always selects the first option, so the event's value is marked
    /// "Current" instead. Picking it again sends nothing.
    fn choice(
        theme: &Theme, title: &str, labels: [&str; 2], current: usize,
        mut pick: impl FnMut(usize) + Clone + 'static,
    ) -> Form {
        let submit = Box::new(move |_ctx: &mut Context, states: &Vec<State>| {
            let [State::Enumerator(index)] = states.as_slice() else { unreachable!() };
            if *index != current { pick(*index); }
            Self::back()
        }) as Box<dyn FormSubmit>;

        let options = labels.iter().enumerate()
            .map(|(index, label)| (*label, if index == current { "Current" } else { "" }))
            .collect();
        Form::flow(theme, vec![FormItem::enumerator(title, options)], None, None, submit)
    }

    /// Links are plain text: anything that isn't blank is accepted.
    fn add_link(theme: &Theme, mut calendar: Instance<Calendar>, id: EventId) -> Form {
        let submit = Box::new(move |_ctx: &mut Context, states: &Vec<State>| {
            let [State::Text(link)] = states.as_slice() else { unreachable!() };
            Calendar::add_attachment(&mut calendar, id, link.clone());
            Self::back()
        }) as Box<dyn FormSubmit>;

        let link = FormItem::text(
            "Attachment", "Link", Some(vec![FormAction::Paste]),
            |_ctx: &mut Context, link: String| {
                if link.trim().is_empty() { FormValidState::Invalid } else { FormValidState::Valid }
            },
        );
        Form::flow(theme, vec![link], None, None, submit)
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
enum EventKind {
    Regular,
    AllDay,
    Repeating,
    RepeatingAllDay,
}

impl EventKind {
    fn options() -> [EventKind; 4] {
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

/// Change time: a Timed / All-day picker, then that kind's steps and a Current / New review.
/// Timed steps use the event's own time zone, so an event set in Tokyo stays in Tokyo time.
struct ChangeTime;
impl ChangeTime {
    /// A repeating event changes as a whole series (until step 3b): the times are the series'
    /// first occurrence's, and its start can't move past the repeat's Until.
    fn picker(theme: &Theme, calendar: Instance<Calendar>, id: EventId, event: &Event) -> Form {
        let current = *event.timing();
        let until = event.recurrence().and_then(|recurrence| recurrence.until());
        let submit = Box::new(move |_ctx: &mut Context, states: &Vec<State>| {
            let [State::Enumerator(index)] = states.as_slice() else { unreachable!() };
            let (kind, calendar) = (EventKind::options()[*index], calendar.clone());
            FormComplete::Return(Box::new(move |ctx: &mut Context, theme: &Theme| {
                let times = FormTimes { latest: until, ..FormTimes::default() };
                let times = Rc::new(RefCell::new(times));
                let form = match kind {
                    EventKind::AllDay => Self::all_day(theme, calendar.clone(), id, current, times),
                    _ => Self::timed(theme, calendar.clone(), id, current, times),
                };
                Flow::from_form(form).build(ctx, theme)(ctx, theme)
            }))
        }) as Box<dyn FormSubmit>;

        let now = format!("Now: {}", EventScreen::when(event.timing(), Timing::device_zone()));
        let kinds = vec![("Timed", now.as_str()), ("All-day", "Whole days, no times")];
        Form::flow(theme, vec![FormItem::enumerator("Timing", kinds)], None, None, submit)
    }

    /// An all-day event has no zone of its own, so it becomes timed in the device's zone. The
    /// step titles don't name the zone (chk cuts long titles short); the review does.
    fn timed(
        theme: &Theme, calendar: Instance<Calendar>, id: EventId, current: Timing,
        times: Rc<RefCell<FormTimes>>,
    ) -> Form {
        let zone = match current {
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
        Self::flow(theme, calendar, id, current, zone, steps)
    }

    fn all_day(
        theme: &Theme, calendar: Instance<Calendar>, id: EventId, current: Timing,
        times: Rc<RefCell<FormTimes>>,
    ) -> Form {
        let steps = vec![
            FormTimes::date_step("First Day", times.clone()),
            FormTimes::end_date_step("Last Day", times),
        ];
        Self::flow(theme, calendar, id, current, Timing::device_zone(), steps)
    }

    /// Continue sends `set_timing` unless nothing changed, then goes back to the detail page.
    fn flow(
        theme: &Theme, mut calendar: Instance<Calendar>, id: EventId, current: Timing, zone: Tz,
        steps: Vec<FormItem>,
    ) -> Form {
        let review = move |states: &Vec<State>| {
            let new = Timing::from_states(states, zone);
            vec![Self::table("Current", &current), Self::table("New", &new)]
        };
        let submit = Box::new(move |_ctx: &mut Context, states: &Vec<State>| {
            let timing = Timing::from_states(states, zone);
            if timing != current { Calendar::set_timing(&mut calendar, id, timing); }
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
struct ChangeRepeat;
impl ChangeRepeat {
    fn form(theme: &Theme, mut calendar: Instance<Calendar>, id: EventId, event: &Event) -> Form {
        let current = event.recurrence().cloned();
        let start = event.timing().start_date();
        let earliest = start.max(Local::now().date_naive());
        let times = FormTimes { date: Some(earliest), ..FormTimes::default() };
        let times = Rc::new(RefCell::new(times));

        let review_current = current.clone();
        let review = move |states: &Vec<State>| {
            let new = Recurrence::from_states(states, start, earliest);
            vec![
                Self::table("Current", review_current.as_ref(), start),
                Self::table("New", new.as_ref(), start),
            ]
        };
        let submit = Box::new(move |_ctx: &mut Context, states: &Vec<State>| {
            let new = Recurrence::from_states(states, start, earliest);
            if new != current { Calendar::set_recurrence(&mut calendar, id, new); }
            EventScreen::back()
        }) as Box<dyn FormSubmit>;
        let steps = FormTimes::repeat_steps(times, true);
        Form::flow(theme, steps, Some(Review::new("Change Repeat Review", review)), None, submit)
    }

    fn table(label: &str, recurrence: Option<&Recurrence>, start: NaiveDate) -> Display {
        let rows = Self::rows(recurrence, start).into_iter()
            .map(|(label, value)| TableItem::new(label, &value)).collect();
        Display::table(label, rows, None)
    }

    /// Repeats, On, First (only when it isn't the start date) and Ends, as the reviews and
    /// the detail page show them. Short rows, because chk clips long table values.
    fn rows(recurrence: Option<&Recurrence>, start: NaiveDate) -> Vec<(&'static str, String)> {
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
struct FormTimes {
    date: Option<NaiveDate>,
    start: Option<NaiveTime>,
    end_date: Option<NaiveDate>,
    until: Option<NaiveDate>,
    /// The latest Date allowed: a repeating event's Until, in Change time.
    latest: Option<NaiveDate>,
}

impl FormTimes {
    /// Date or First Day: on or after today, and not after `latest`.
    fn date_step(title: &str, times: Rc<RefCell<FormTimes>>) -> FormItem {
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
    fn repeat_steps(times: Rc<RefCell<FormTimes>>, never: bool) -> Vec<FormItem> {
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

    fn start_step(title: &str, times: Rc<RefCell<FormTimes>>) -> FormItem {
        FormItem::number(title, NumberVariant::Time, move |_ctx: &mut Context, input: String| {
            let start = Event::time(&input);
            times.borrow_mut().start = start;
            Self::validity(&input, start.ok_or("Not a valid time."))
        })
    }

    /// End Date or Last Day: the first such date on or after the start, at most 28 days later.
    /// The limit is a form rule only; `Timing::is_valid` and `apply` accept longer events.
    fn end_date_step(title: &str, times: Rc<RefCell<FormTimes>>) -> FormItem {
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
