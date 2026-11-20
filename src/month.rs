use chk::{
    Action, Bumper, Context, Display, Flow, ListItem, Offset, Page, PageType, Root, Theme, Watch,
};
use ramp::prism::drawable::Color;
use ramp::prism::drawable::text::{Align, Text};
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
            |ctx: &mut Context, theme: &Theme, calendar: Instance<Calendar>| {
                let (state, now, first) = (calendar.pending(), Utc::now(), Event::window_start());
                let zone = Timing::device_zone();
                let months = (0..13u32)
                    .map(|month| first + Months::new(month))
                    .map(|start| {
                        Self::month_display(ctx, theme, &calendar, &state, start, now, zone)
                    })
                    .collect();
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
    fn month_display(
        ctx: &Context, theme: &Theme, calendar: &Instance<Calendar>, state: &Calendar,
        start: NaiveDate, now: DateTime<Utc>, zone: Tz,
    ) -> Display {
        let width = ctx.window.area().size.0;
        // `Timing::today()`, at the same instant as the summaries' "has it ended?".
        let today = now.with_timezone(&zone).date_naive();
        let end = start + Months::new(1);
        let days = start.iter_days().take_while(|day| *day < end).map(|day| {
            let summary = Self::summary(&state.on(day, zone), day, now, zone)
                .map(|summary| DayScreen::fit_label(theme, width, &summary, false));
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
        // The start; on a later day of an event that started earlier, when it ends instead.
        let time = match occurrence.timing().in_zone(zone) {
            None => "All day".to_string(),
            Some((start, _)) if start.date() == day => Timing::format_short_time(start.time()),
            Some((_, end)) if end.date() == day => {
                format!("until {}", Timing::format_short_time(end.time()))
            }
            Some((_, end)) => format!("until {}", end.format("%-d %b")),
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
            move |ctx: &mut Context, theme: &Theme, calendar: Instance<Calendar>| {
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
                        let width = ctx.window.area().size.0;
                        let title = Self::fit_label(theme, width, occurrence.title(), false);
                        let time = Self::time_label(occurrence.timing(), day, zone);
                        let time = Self::fit_label(theme, width, &time, false);
                        let detail =
                            EventScreen::flow(calendar.clone(), occurrence.id(), occurrence.date());
                        ListItem::plain(&title, &subtitle, Some(&time), Some(detail))
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

    /// `text` shortened to fit one side of a list row, or a table value with `table`, in a
    /// window `width` wide, ending in "…" when it was cut. pelican would clip it mid-letter
    /// instead. Measured with the renderer's own layout. Temporary: it goes once chk's renderer
    /// draws its own ellipsis.
    pub fn fit_label(theme: &Theme, width: f32, text: &str, table: bool) -> String {
        // pelican's content is at most 423 pt wide with 24 pt padding each side. A row's
        // forward icon takes 32 pt and its two columns share the rest, 8 pt apart; a table row
        // has no icon, and a long value gets half after the 8 pt gap. 2 pt spare covers the
        // renderer rounding widths up.
        let content = width.min(423.0) - 48.0;
        let (font, size, half) = match table {
            false => ("FontStyle::Heading", "TextSize::H5", (content - 32.0 - 8.0) / 2.0),
            true => ("FontStyle::Text", "TextSize::Sm", (content - 8.0) / 2.0),
        };
        let max_width = (half - 2.0).max(0.0);
        let Some(font) = theme.fonts().get_font(font) else { return text.to_string() };
        let size = theme.fonts().get_size(size);
        let fits = |text: &str| {
            let text = Text::new(text, font.clone(), size, Color::transparent(), Align::Left);
            text.size().0.ceil() <= max_width
        };
        if max_width <= 0.0 || fits(text) { return text.to_string(); }
        // The longest start of `text` that still fits with "…" after it.
        let cut = |end: usize| format!("{}…", text[..end].trim_end());
        let ends = text.char_indices().map(|(end, _)| end).collect::<Vec<_>>();
        match ends.partition_point(|end| fits(&cut(*end))) {
            0 => "…".to_string(),
            fitting => cut(ends[fitting - 1]),
        }
    }

    /// The Day view's time in `zone`: "9:30 PM–10 PM" on one day; an event running past
    /// midnight shows "from 11 PM" on its first day, then "until 1 AM", or "until 6 Dec" on a
    /// day it neither starts nor ends.
    fn time_label(timing: &Timing, day: NaiveDate, zone: Tz) -> String {
        let Some((start, end)) = timing.in_zone(zone) else { return "All day".into() };
        let short = |time: NaiveDateTime| Timing::format_short_time(time.time());
        match (start.date() == day, end.date() == day) {
            (true, true) => format!("{}–{}", short(start), short(end)),
            (true, false) => format!("from {}", short(start)),
            (false, true) => format!("until {}", short(end)),
            (false, false) => format!("until {}", end.format("%-d %b")),
        }
    }
}
