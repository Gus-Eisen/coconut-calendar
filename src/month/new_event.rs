use chk::{
    Context, Display, Flow, Form, FormAction, FormComplete, FormItem, FormSubmit, FormValidState,
    Icons, Review, State, Success, TableItem, Theme,
};
use chrono::NaiveDate;
use ramp::prism::air::Instance;

use std::cell::RefCell;
use std::rc::Rc;

use crate::{Calendar, Category, Event, Timing};

use super::DayScreen;
use super::schedule::{ChangeRepeat, ChangeTime, FormTimes};

/// The choices on the event-type picker. UI-only: never stored.
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

    fn all_day(self) -> bool { matches!(self, EventKind::AllDay | EventKind::RepeatingAllDay) }
    fn repeating(self) -> bool { matches!(self, EventKind::Repeating | EventKind::RepeatingAllDay) }
}

pub struct NewEvent;
impl NewEvent {
    /// Asks which kind of event to create, then opens that kind's form in the picker's place:
    /// chk's Return pops the picker and pushes the form, so Back and Done go to the page the
    /// picker was opened from (the month, or a Day view). From a Day view, `day` is that day:
    /// the forms skip Date / First Day and start on it.
    pub fn picker(theme: &Theme, calendar: Instance<Calendar>, day: Option<NaiveDate>) -> Form {
        let submit = Box::new(move |_ctx: &mut Context, states: &Vec<State>| {
            let [State::Enumerator(index)] = states.as_slice() else { unreachable!() };
            let (kind, calendar) = (EventKind::options()[*index], calendar.clone());
            FormComplete::Return(Box::new(move |ctx: &mut Context, theme: &Theme| {
                let form = Self::form(theme, calendar.clone(), kind, day, ctx.window.area().size.0);
                Flow::from_form(form).build(ctx, theme)(ctx, theme)
            }))
        }) as Box<dyn FormSubmit>;

        let kinds = EventKind::options().map(|kind| (kind.label(), kind.description())).to_vec();
        Form::flow(theme, vec![FormItem::enumerator("Event Type", kinds)], None, None, submit)
    }

    /// One kind's form: Start Time and End Time, or Last Day for an all-day kind, after Date /
    /// First Day; a repeating kind adds Repeats, Every, Until and Times. With `day`, there's no
    /// Date / First Day step.
    fn form(theme: &Theme, calendar: Instance<Calendar>, kind: EventKind, day: Option<NaiveDate>, width: f32) -> Form {
        let review_theme = theme.clone();
        let review = move |states: &Vec<State>| match Event::from_states(states, day) {
            Some(event) => Self::review(&event, &review_theme, width),
            None => vec![Display::instructions("Go back and check the dates and times.")],
        };

        // Continue saves nothing when the steps fail their checks, and this says so.
        let success = move |states: Vec<State>| {
            let Some(event) = Event::from_states(&states, day) else {
                return (Icons::Error, "Event not saved.".to_string());
            };
            // A weekly set that skips the start's weekday starts later than Date.
            let start = event.timing().start_date();
            let first = event.recurrence().and_then(|rule| rule.first(start));
            let date = |date: NaiveDate| date.format("%-d %b %Y").to_string();
            let message = match (*event.timing(), first) {
                (Timing::AllDay { .. }, Some(first)) => {
                    format!("Repeating all-day event saved, from {}", date(first))
                }
                (Timing::AllDay { first_day, last_day }, None) if first_day == last_day => {
                    format!("All-day event saved for {}", date(first_day))
                }
                (Timing::AllDay { first_day, last_day }, None) => {
                    format!("All-day event saved for {} to {}", date(first_day), date(last_day))
                }
                (Timing::Timed { time_zone, .. }, first) => {
                    // `Event::from_states` checked `is_valid`, so `in_zone` can't panic.
                    let (start, _) = event.timing().in_zone(time_zone).unwrap();
                    let time = Timing::format_time(start.time());
                    match first {
                        Some(first) => {
                            format!("Repeating event saved, from {} at {time}", date(first))
                        }
                        None => format!("Event saved for {} at {time}", date(start.date())),
                    }
                }
            };
            (Icons::Checkmark, message)
        };

        let times = Rc::new(RefCell::new(FormTimes::starting(day)));
        let (date_title, mut timing) = match kind.all_day() {
            true => ("First Day", vec![FormTimes::end_date_step("Last Day", times.clone())]),
            false => ("Date", vec![
                FormTimes::start_step("Start Time", times.clone()),
                FormTimes::end_time_step(times.clone(), Timing::device_zone()),
            ]),
        };
        if day.is_none() { timing.insert(0, FormTimes::date_step(date_title, times.clone())); }
        if kind.repeating() { timing.extend(FormTimes::repeat_steps(times, false)); }

        // chk cuts long titles short, so every new-event review has the same short title.
        let title = if kind.all_day() { "New All-Day Event" } else { "New Event" };
        let review = Review::new("New Event Review", review);
        Self::flow(theme, calendar, day, timing, review, Success::new(title, success))
    }

    /// Puts the steps every kind shares around `timing`, so `Event::from_states` can read either.
    fn flow(theme: &Theme, mut calendar: Instance<Calendar>, day: Option<NaiveDate>, timing: Vec<FormItem>, review: Review, success: Success) -> Form {
        let submit = Box::new(move |_ctx: &mut Context, states: &Vec<State>| {
            if let Some(event) = Event::from_states(states, day) {
                Calendar::create_event(&mut calendar, event);
            }
            FormComplete::None
        }) as Box<dyn FormSubmit>;

        let start = vec![
            FormItem::text(
                "Event Title", "Event title", None,
                |_ctx: &mut Context, title: String| match title.trim().is_empty() {
                    true => FormValidState::Invalid,
                    false => FormValidState::Valid,
                },
            ),
            FormItem::enumerator(
                "Category",
                Category::options().map(|category| (category.label(), "")).to_vec(),
            ),
        ];
        let end = vec![
            FormItem::text(
                "Location", "Location (optional)", None,
                |_ctx: &mut Context, _location: String| FormValidState::Valid,
            ),
            FormItem::text(
                "Info", "Info (optional)", None,
                |_ctx: &mut Context, _info: String| FormValidState::Valid,
            ),
            FormItem::text(
                "Link", "Link (optional)", Some(vec![FormAction::Paste]),
                |_ctx: &mut Context, link: String| match Event::web_link(&link) {
                    Ok(_) => FormValidState::Valid,
                    Err(error) => FormValidState::InvalidWithData(error.into()),
                },
            ),
        ];

        let items = [start, timing, end].into_iter().flatten().collect();
        Form::flow(theme, items, Some(review), Some(success), submit)
    }

    /// The table's values are cut to fit a window `width` wide.
    fn review(event: &Event, theme: &Theme, width: f32) -> Vec<Display> {
        let start = event.timing().start_date();
        let repeat = event.recurrence()
            .map(|recurrence| ChangeRepeat::rows(recurrence, start)).unwrap_or_default();
        let rows = [
            vec![
                ("Title", event.title().to_string()),
                ("Category", event.category().label().to_string()),
            ],
            ChangeTime::rows(event.timing()),
            repeat,
            [
                event.location().map(|location| ("Location", location.to_string())),
                event.info().map(|info| ("Info", info.to_string())),
                event.link().map(|link| ("Link", link.to_string())),
            ].into_iter().flatten().collect(),
        ];
        vec![Display::table(
            "New event",
            rows.into_iter().flatten().map(|(label, value)| {
                TableItem::new(label, &DayScreen::fit_label(theme, width, &value, true))
            }).collect(),
            None,
        )]
    }
}

