use chk::{
    Action, Bumper, Context, Display, Flow, ListItem, Offset, Page, PageType, Root, Theme, Watch,
};
use chrono::{DateTime, Datelike, Months, NaiveDate, NaiveDateTime, Utc};
use chrono_tz::Tz;
use ramp::prism::air::Instance;

use crate::{Calendar, Event, Occurrence, Timing};

mod detail;
pub use detail::EventScreen;
mod new_event;
pub use new_event::NewEvent;
mod schedule;

pub struct MonthScreen;
impl MonthScreen {
    pub fn root(ctx: &mut Context) -> Root {
        Root::custom(Page::watching(
            Watch::contract(Calendar::open(ctx)),
            |_ctx: &mut Context, _theme: &Theme, calendar: Instance<Calendar>| {
                let (state, now, first) = (calendar.pending(), Utc::now(), Event::window_start());
                let months = (0..13u32).map(|month| {
                    Self::month_display(&calendar, &state, first + Months::new(month), now)
                }).collect();
                drop(state);
                let new_event = Box::new(move |_ctx: &mut Context, theme: &Theme| {
                    Flow::from_form(NewEvent::picker(theme, calendar.clone(), None))
                });
                PageType::root(
                    "Calendar", vec![], months, None,
                    Some(("Create Event".into(), new_event)), None,
                )
            },
        ))
    }

    /// One month's list, from its first day `start`: a row per day, each opening that day.
    fn month_display(calendar: &Instance<Calendar>, state: &Calendar, start: NaiveDate, now: DateTime<Utc>) -> Display {
        let (zone, today) = (Timing::device_zone(), Timing::today());
        let end = start + Months::new(1);
        let days = start.iter_days().take_while(|day| *day < end).map(|day| {
            let summary = Self::summary(&state.on(day, zone), day, now, zone);
            let weekday = match day == today {
                true => format!("{} · Today", day.format("%A")),
                false => day.format("%A").to_string(),
            };
            let flow = DayScreen::flow(calendar.clone(), day);
            ListItem::plain(&day.day().to_string(), &weekday, summary.as_deref(), Some(flow))
                .with_badge(day == today)
        }).collect();
        Display::list(Some(&start.format("%B %Y").to_string()), days, None)
    }

    /// The day's row summary in `zone`: its one event, even after it has ended. With several,
    /// the first timed event that hasn't ended at `now`, else (today or later) the first all-day
    /// one, else "N events" once they've all ended. `None` for a day with no events.
    fn summary(occurrences: &[Occurrence<'_>], day: NaiveDate, now: DateTime<Utc>, zone: Tz) -> Option<String> {
        let today = now.with_timezone(&zone).date_naive();
        // `on` lists all-day events first, then timed ones by start, so `find` takes the first.
        let shown = match occurrences {
            [] => return None,
            [occurrence] => Some(occurrence),
            occurrences => {
                let unended = |occurrence: &&Occurrence| {
                    occurrence.timing().instants().is_some_and(|(_, end)| end > now)
                };
                let all_day = |occurrence: &&Occurrence| occurrence.timing().instants().is_none();
                occurrences.iter().find(unended)
                    .or_else(|| occurrences.iter().find(all_day).filter(|_| day >= today))
            }
        };
        let Some(occurrence) = shown else { return Some(format!("{} events", occurrences.len())) };
        // The start only; on a later day of an event that started earlier, with its date.
        let time = match occurrence.timing().in_zone(zone) {
            None => "All day".to_string(),
            Some((start, _)) => DayScreen::clock_label(start, day),
        };
        Some(format!("{time} {}", occurrence.title()))
    }
}

pub struct DayScreen;
impl DayScreen {
    /// Every day on the month opens one, even with no events. Today and later days have a
    /// Create Event button that starts the new-event flow on this day.
    pub fn flow(calendar: Instance<Calendar>, day: NaiveDate) -> Flow {
        Flow::new(vec![Page::watching(
            Watch::contract(calendar),
            move |_ctx: &mut Context, theme: &Theme, calendar: Instance<Calendar>| {
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
                        ListItem::plain(occurrence.title(), &subtitle, Some(&time), Some(detail))
                    })
                    .collect::<Vec<_>>();
                // Deleting or cancelling the day's last event leaves the page open with nothing.
                let (items, offset) = match rows.is_empty() {
                    true => (Display::instructions("No events on this day."), Offset::Center),
                    false => (Display::list(None, rows, None), Offset::Start),
                };
                let title = day.format("%a %-d %b").to_string();
                // A past day can't get a new event: the forms only make today or later.
                let bumper = match day >= Timing::today() {
                    true => {
                        let picker = NewEvent::picker(theme, calendar.clone(), Some(day));
                        Bumper::custom("Create Event", Action::flow(Flow::from_form(picker)))
                    }
                    false => Bumper::None,
                };
                PageType::display(&title, vec![items], None, bumper, offset)
            },
        )])
    }

    /// "All day", or the start and end in `zone`, with the date on any part that isn't on `day`.
    pub fn time_label(timing: &Timing, day: NaiveDate, zone: Tz) -> String {
        let Some((start, end)) = timing.in_zone(zone) else { return "All day".into() };
        format!("{}–{}", Self::clock_label(start, day), Self::clock_label(end, day))
    }

    /// A time as "HH:MM", with its date when it isn't on `day`: "HH:MM (3 Nov)".
    fn clock_label(time: NaiveDateTime, day: NaiveDate) -> String {
        time.format(if time.date() == day { "%H:%M" } else { "%H:%M (%-d %b)" }).to_string()
    }
}
