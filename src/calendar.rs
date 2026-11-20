use chk::{Context, State};
use chrono::{Datelike, Days, NaiveDate, NaiveTime};
use chrono_tz::Tz;
use ramp::prism::air::{Contract, Id, Instance, Metadata, Name};
use serde::{Deserialize, Serialize};

use std::collections::BTreeMap;

mod timing;
pub use timing::*;
mod recurrence;
pub use recurrence::*;
mod exception;
pub use exception::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Category {
    Work,
    Personal,
}

impl Category {
    /// In the order the new-event form lists them; `Personal` first, as new events default to it.
    pub fn options() -> [Category; 2] { [Category::Personal, Category::Work] }

    pub fn label(self) -> &'static str {
        match self {
            Category::Work => "Work",
            Category::Personal => "Personal",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Attendee {
    name: Name,
    response: Response,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Response {
    Pending,
    Accepted,
    Declined,
    Tentative,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Event {
    title: String,
    timing: Timing,
    location: Option<String>,
    info: Option<String>,
    attendees: Vec<Attendee>,
    link: Option<String>,
    recurrence: Option<Recurrence>,
    category: Category,
    time_proposals: Vec<TimeProposal>,
    private: bool,
    deleted: bool,
}

impl Event {
    pub fn window_start() -> NaiveDate {
        Timing::today().with_day(1).unwrap()
    }

    pub fn date(input: &str) -> Option<NaiveDate> {
        Self::date_from(input, Timing::today())
    }

    /// The first `DD-MM` date on or after `earliest`: `earliest`'s year, else the next year.
    pub fn date_from(input: &str, earliest: NaiveDate) -> Option<NaiveDate> {
        [earliest.year(), earliest.year() + 1].into_iter()
            .filter_map(|year| {
                NaiveDate::parse_from_str(&format!("{input}-{year}"), "%d-%m-%Y").ok()
            })
            .find(|date| *date >= earliest)
    }

    pub fn time(input: &str) -> Option<NaiveTime> {
        NaiveTime::parse_from_str(input, "%H:%M").ok()
    }

    /// Reads any new-event form. They differ only in their timing steps and, for the repeating
    /// forms, the four repeat steps after them; all end with Location, Info and Link. Timed
    /// events use the device's time zone. With `day` (the Day view's Create Event), the form
    /// skipped Date / First Day and the event starts on `day`.
    pub fn from_states(states: &[State], day: Option<NaiveDate>) -> Event {
        let [
            State::Text(title), State::Enumerator(category), steps @ .., State::Text(location),
            State::Text(info), State::Text(link),
        ] = states else {
            unreachable!()
        };
        let (timing, repeat) = match steps {
            [timing @ .., State::Enumerator(_), _, _, _] => steps.split_at(timing.len()),
            timing => (timing, &[][..]),
        };
        let timing = match day {
            Some(day) => Timing::from_start(day, timing, Timing::device_zone()),
            None => Timing::from_states(timing, Timing::device_zone()),
        };
        let start = timing.start_date();
        Event {
            title: title.trim().to_string(),
            timing,
            location: Self::optional(location),
            info: Self::optional(info),
            attendees: vec![],
            link: Self::optional(link),
            recurrence: (!repeat.is_empty()).then(|| Recurrence::from_states(repeat, start, start))
                .flatten(),
            category: Category::options()[*category],
            time_proposals: vec![],
            private: false,
            deleted: false,
        }
    }

    pub fn title(&self) -> &str { &self.title }
    pub fn timing(&self) -> &Timing { &self.timing }
    pub fn location(&self) -> Option<&str> { self.location.as_deref() }
    pub fn info(&self) -> Option<&str> { self.info.as_deref() }
    pub fn attendees(&self) -> &[Attendee] { &self.attendees }
    pub fn link(&self) -> Option<&str> { self.link.as_deref() }
    pub fn recurrence(&self) -> Option<&Recurrence> { self.recurrence.as_ref() }
    pub fn category(&self) -> Category { self.category }
    pub fn time_proposals(&self) -> &[TimeProposal] { &self.time_proposals }
    pub fn private(&self) -> bool { self.private }

    pub fn optional(input: &str) -> Option<String> {
        let input = input.trim();
        (!input.is_empty()).then(|| input.to_string())
    }

    /// The series' timing for the occurrence whose original date is `date`, ignoring any
    /// exception, or `None` if the rule doesn't pick `date`. An event that doesn't repeat has
    /// one, on its start date.
    pub fn series_timing(&self, date: NaiveDate) -> Option<Timing> {
        let start = self.timing.start_date();
        let Some(recurrence) = &self.recurrence else {
            return (date == start).then_some(self.timing);
        };
        if !recurrence.occurs_on(date, start, recurrence.last(start)) { return None; }
        self.timing.moved_to(date)
    }

    /// The exception on the occurrence whose original date is `date`, if it has one.
    pub fn exception(&self, date: NaiveDate) -> Option<&Exception> {
        self.recurrence.as_ref()?.exceptions().get(&date)
    }

    /// The occurrence's own changes, if it was changed (not cancelled).
    pub fn changes(&self, date: NaiveDate) -> Option<&Override> {
        match self.exception(date)? {
            Exception::Changed(changes) => Some(changes),
            Exception::Cancelled => None,
        }
    }

    /// The occurrence's effective timing: its own if it was moved, else the series' moved to
    /// `date`. `None` if there's no such occurrence or it was cancelled.
    pub fn occurrence(&self, date: NaiveDate) -> Option<Timing> {
        let series = self.series_timing(date)?;
        match self.exception(date) {
            Some(Exception::Cancelled) => None,
            Some(Exception::Changed(changes)) => Some(changes.timing().copied().unwrap_or(series)),
            None => Some(series),
        }
    }

    /// The occurrence as an event of its own: the series' fields with its `Override` applied
    /// (the views show these). `None` if there's no such occurrence or it was cancelled.
    pub fn effective(&self, date: NaiveDate) -> Option<Event> {
        let timing = self.occurrence(date)?;
        let mut event = Event { timing, ..self.clone() };
        let Some(changes) = self.changes(date) else { return Some(event) };
        if let Some(title) = changes.title() { event.title = title.to_string(); }
        if let Some(location) = changes.location() {
            event.location = location.map(str::to_string);
        }
        if let Some(info) = changes.info() { event.info = info.map(str::to_string); }
        if let Some(link) = changes.link() { event.link = link.map(str::to_string); }
        Some(event)
    }

    /// How many single changes `apply` would drop if the series became `timing` with the
    /// rule `recurrence`: those whose original date the new rule no longer picks. `None` for
    /// `recurrence` means it stops repeating, which drops them all.
    pub fn dropped(&self, timing: &Timing, recurrence: Option<&Recurrence>) -> usize {
        let Some(current) = &self.recurrence else { return 0 };
        let start = timing.start_date();
        let Some(recurrence) = recurrence else { return current.exceptions().len() };
        let last = recurrence.last(start);
        current.exceptions().keys().filter(|date| !recurrence.occurs_on(**date, start, last))
            .count()
    }

    /// `dropped` for a whole-series Change time, whose rule follows the new start.
    pub fn dropped_by_timing(&self, timing: &Timing) -> usize {
        self.dropped(timing, self.retimed(timing).as_ref())
    }

    /// The repeat rule once the series' timing becomes `timing` (see `Recurrence::moved`).
    fn retimed(&self, timing: &Timing) -> Option<Recurrence> {
        let (from, to) = (self.timing.start_date(), timing.start_date());
        self.recurrence.as_ref().map(|recurrence| recurrence.moved(from, to))
    }

    /// The occurrences covering `date` in `zone`, with their original dates. Only start dates
    /// that could reach `date` are checked, not every date since the start: an occurrence ends
    /// `days` calendar days after its start date in its own zone, and zones differ by up to 26
    /// hours, so in `zone` it starts at most 2 days earlier or later. So it covers `date` only
    /// if it starts between `days + 2` days before `date` and 2 days after.
    fn covering(&self, date: NaiveDate, zone: Tz) -> Vec<(NaiveDate, Timing)> {
        let start = self.timing.start_date();
        let Some(recurrence) = &self.recurrence else {
            return match self.timing.covers(date, zone) {
                true => vec![(start, self.timing)],
                false => vec![],
            };
        };
        let last = recurrence.last(start);
        // Original dates whose occurrence isn't at its series time: cancelled, or moved.
        let away = |day: &NaiveDate| match recurrence.exceptions().get(day) {
            Some(Exception::Cancelled) => true,
            Some(Exception::Changed(changes)) => changes.timing().is_some(),
            None => false,
        };
        // Moved occurrences can be anywhere, so each is checked on its own. Exceptions are few.
        let moved = recurrence.exceptions().iter()
            .filter_map(|(day, exception)| match exception {
                Exception::Changed(changes) => changes.timing().map(|timing| (*day, *timing)),
                Exception::Cancelled => None,
            })
            .filter(|(day, timing)| {
                recurrence.occurs_on(*day, start, last) && timing.covers(date, zone)
            });
        let earliest = date.checked_sub_days(Days::new(self.timing.days() + 2));
        let (Some(earliest), Some(latest)) = (earliest, date.checked_add_days(Days::new(2))) else {
            return moved.collect();
        };
        earliest.iter_days().take_while(|day| *day <= latest)
            .filter(|day| recurrence.occurs_on(*day, start, last) && !away(day))
            .filter_map(|day| Some((day, self.timing.moved_to(day)?)))
            .filter(|(_, timing)| timing.covers(date, zone))
            .chain(moved)
            .collect()
    }
}

/// One appearance of an event on a day.
#[derive(Debug, Clone, Copy)]
pub struct Occurrence<'a> {
    id: EventId,
    event: &'a Event,
    date: NaiveDate,
    timing: Timing,
}

impl<'a> Occurrence<'a> {
    pub fn id(&self) -> EventId { self.id }
    /// The series. Use `title` and `timing` for this occurrence's own values.
    pub fn event(&self) -> &'a Event { self.event }
    /// Its own title if it was renamed, else the series'.
    pub fn title(&self) -> &'a str {
        self.event.changes(self.date).and_then(Override::title).unwrap_or(&self.event.title)
    }
    /// The date it would normally start on, in the event's own zone. It identifies the
    /// occurrence within its series.
    pub fn date(&self) -> NaiveDate { self.date }
    /// This occurrence's own timing; the event's `timing` is only the first one's.
    pub fn timing(&self) -> &Timing { &self.timing }
}

/// Postcard encodes variants by position: only append new variants, never insert or reorder.
/// (They were reordered once, for `CoconutCalendarV3`, which started with no data.)
#[derive(Debug, Clone, Hash, Serialize, Deserialize)]
pub enum CalendarMessage {
    CreateEvent { id: EventId, event: Box<Event> },
    DeleteEvent { id: EventId },
    SetTitle { id: EventId, title: String },
    SetTiming { id: EventId, timing: Timing },
    SetLocation { id: EventId, location: Option<String> },
    SetInfo { id: EventId, info: Option<String> },
    SetLink { id: EventId, link: Option<String> },
    SetCategory { id: EventId, category: Category },
    SetPrivate { id: EventId, private: bool },
    SetRecurrence { id: EventId, recurrence: Option<Recurrence> },
    CancelOccurrence { id: EventId, date: NaiveDate },
    ChangeOccurrence { id: EventId, date: NaiveDate, changes: Override },
    RestoreOccurrence { id: EventId, date: NaiveDate },
}

/// The one AIR record per identity. Deleting only marks an event; AIR never erases data.
#[derive(Debug, Clone, Default, Hash, Serialize, Deserialize)]
pub struct Calendar {
    events: BTreeMap<EventId, Event>,
}

impl Calendar {
    /// The location is `hash((), signer)`, so this returns the existing calendar if there is one.
    pub fn open(ctx: &mut Context) -> Instance<Calendar> {
        ctx.air.instances::<Calendar>().create(())
    }

    pub fn create_event(calendar: &mut Instance<Calendar>, event: Event) {
        let id = EventId::random();
        calendar.send(CalendarMessage::CreateEvent { id, event: Box::new(event) });
    }

    pub fn delete_event(calendar: &mut Instance<Calendar>, id: EventId) {
        calendar.send(CalendarMessage::DeleteEvent { id });
    }

    pub fn set_title(calendar: &mut Instance<Calendar>, id: EventId, title: String) {
        calendar.send(CalendarMessage::SetTitle { id, title });
    }

    pub fn set_timing(calendar: &mut Instance<Calendar>, id: EventId, timing: Timing) {
        calendar.send(CalendarMessage::SetTiming { id, timing });
    }

    pub fn set_location(calendar: &mut Instance<Calendar>, id: EventId, location: Option<String>) {
        calendar.send(CalendarMessage::SetLocation { id, location });
    }

    pub fn set_info(calendar: &mut Instance<Calendar>, id: EventId, info: Option<String>) {
        calendar.send(CalendarMessage::SetInfo { id, info });
    }

    pub fn set_link(calendar: &mut Instance<Calendar>, id: EventId, link: Option<String>) {
        calendar.send(CalendarMessage::SetLink { id, link });
    }

    pub fn set_category(calendar: &mut Instance<Calendar>, id: EventId, category: Category) {
        calendar.send(CalendarMessage::SetCategory { id, category });
    }

    pub fn set_private(calendar: &mut Instance<Calendar>, id: EventId, private: bool) {
        calendar.send(CalendarMessage::SetPrivate { id, private });
    }

    /// `None` stops the event repeating.
    pub fn set_recurrence(calendar: &mut Instance<Calendar>, id: EventId, recurrence: Option<Recurrence>) {
        calendar.send(CalendarMessage::SetRecurrence { id, recurrence });
    }

    /// Cancels one occurrence for good, identified by its original date `date`.
    pub fn cancel_occurrence(calendar: &mut Instance<Calendar>, id: EventId, date: NaiveDate) {
        calendar.send(CalendarMessage::CancelOccurrence { id, date });
    }

    /// Replaces the occurrence's changes with `changes`.
    pub fn change_occurrence(calendar: &mut Instance<Calendar>, id: EventId, date: NaiveDate, changes: Override) {
        calendar.send(CalendarMessage::ChangeOccurrence { id, date, changes });
    }

    /// Removes the occurrence's changes, so it matches the series again.
    pub fn restore_occurrence(calendar: &mut Instance<Calendar>, id: EventId, date: NaiveDate) {
        calendar.send(CalendarMessage::RestoreOccurrence { id, date });
    }

    /// `None` if there's no such event or it was deleted.
    pub fn event(&self, id: EventId) -> Option<&Event> {
        self.events.get(&id).filter(|event| !event.deleted)
    }

    /// Occurrences on `date` in the viewer's time zone `zone`, including every occurrence of a
    /// repeating event that covers it: all-day ones first, then by start.
    pub fn on(&self, date: NaiveDate, zone: Tz) -> Vec<Occurrence<'_>> {
        let mut occurrences = self.events.iter()
            .filter(|(_, event)| !event.deleted)
            .flat_map(|(id, event)| {
                event.covering(date, zone).into_iter().map(|(original, timing)| {
                    Occurrence { id: *id, event, date: original, timing }
                })
            })
            .collect::<Vec<_>>();
        occurrences.sort_by_key(|occurrence| occurrence.timing.instants().map(|(start, _)| start));
        occurrences
    }

    /// The event an edit message may change: it exists and isn't deleted.
    fn editable(&mut self, id: EventId) -> Option<&mut Event> {
        self.events.get_mut(&id).filter(|event| !event.deleted)
    }

    /// The exceptions of a repeating event, if its rule picks `date`. Exceptions don't change
    /// which dates the rule picks, so a cancelled or moved occurrence still qualifies.
    fn exceptions(&mut self, id: EventId, date: NaiveDate) -> Option<&mut BTreeMap<NaiveDate, Exception>> {
        let event = self.editable(id)?;
        let start = event.timing.start_date();
        let recurrence = event.recurrence.as_mut()?;
        let picked = recurrence.occurs_on(date, start, recurrence.last(start));
        picked.then_some(recurrence.exceptions_mut())
    }
}

impl Contract for Calendar {
    type Init = ();
    type Message = CalendarMessage;
    type Result = ();

    fn id() -> Id { Id::hash("CoconutCalendarV3") }

    fn init(_init: (), _metadata: Metadata) -> Self { Calendar::default() }

    fn apply(&mut self, message: CalendarMessage, _metadata: Metadata) {
        match message {
            CalendarMessage::CreateEvent { id, event } => {
                if event.title.trim().is_empty() || !event.timing.is_valid() { return; }
                let recurrence = event.recurrence.clone().map(Recurrence::normalised);
                if let Some(recurrence) = &recurrence
                    && (!recurrence.is_valid(&event.timing)
                        || !recurrence.exceptions().is_empty()) {
                    return;
                }
                let event = Event {
                    title: event.title.trim().to_string(),
                    attendees: vec![],
                    location: event.location.as_deref().and_then(Event::optional),
                    info: event.info.as_deref().and_then(Event::optional),
                    link: event.link.as_deref().and_then(Event::optional),
                    recurrence,
                    time_proposals: vec![],
                    deleted: false,
                    ..*event
                };
                self.events.entry(id).or_insert(event);
            }
            CalendarMessage::DeleteEvent { id } => {
                if let Some(event) = self.events.get_mut(&id) { event.deleted = true; }
            }
            CalendarMessage::SetTitle { id, title } => {
                if let Some(event) = self.editable(id)
                    && let Some(title) = Event::optional(&title) {
                    event.title = title;
                }
            }
            CalendarMessage::SetTiming { id, timing } => {
                if let Some(event) = self.editable(id) && timing.is_valid() {
                    // Keep the repeat rule meaningful if the start date moves. If it no longer
                    // fits (an `until` before the new start), the whole message is ignored.
                    let recurrence = event.retimed(&timing);
                    if recurrence.as_ref().is_some_and(|recurrence| !recurrence.is_valid(&timing)) {
                        return;
                    }
                    // Single changes the new rule no longer picks are dropped.
                    event.recurrence = recurrence.map(|recurrence| {
                        recurrence.pruned(timing.start_date())
                    });
                    event.timing = timing;
                    event.time_proposals.clear();
                }
            }
            CalendarMessage::SetLocation { id, location } => {
                if let Some(event) = self.editable(id) {
                    event.location = location.as_deref().and_then(Event::optional);
                }
            }
            CalendarMessage::SetInfo { id, info } => {
                if let Some(event) = self.editable(id) {
                    event.info = info.as_deref().and_then(Event::optional);
                }
            }
            CalendarMessage::SetLink { id, link } => {
                if let Some(event) = self.editable(id) {
                    event.link = link.as_deref().and_then(Event::optional);
                }
            }
            CalendarMessage::SetCategory { id, category } => {
                if let Some(event) = self.editable(id) { event.category = category; }
            }
            CalendarMessage::SetPrivate { id, private } => {
                if let Some(event) = self.editable(id) { event.private = private; }
            }
            CalendarMessage::SetRecurrence { id, recurrence } => {
                let Some(event) = self.editable(id) else { return };
                let Some(mut recurrence) = recurrence else {
                    event.recurrence = None;
                    return;
                };
                // Exceptions only change through their own messages, so the event's own are kept
                // and any sent here are ignored.
                let exceptions = event.recurrence.as_ref()
                    .map(|current| current.exceptions().clone()).unwrap_or_default();
                *recurrence.exceptions_mut() = exceptions;
                let recurrence = recurrence.normalised();
                if recurrence.is_valid(&event.timing) {
                    // Single changes the new rule no longer picks are dropped.
                    event.recurrence = Some(recurrence.pruned(event.timing.start_date()));
                }
            }
            CalendarMessage::CancelOccurrence { id, date } => {
                if let Some(exceptions) = self.exceptions(id, date) {
                    exceptions.insert(date, Exception::Cancelled);
                }
            }
            CalendarMessage::ChangeOccurrence { id, date, changes } => {
                // Cancelling is permanent, so a cancelled occurrence can't be changed.
                if let Some(changes) = changes.normalised()
                    && let Some(exceptions) = self.exceptions(id, date)
                    && exceptions.get(&date) != Some(&Exception::Cancelled) {
                    exceptions.insert(date, Exception::Changed(changes));
                }
            }
            CalendarMessage::RestoreOccurrence { id, date } => {
                if let Some(exceptions) = self.exceptions(id, date)
                    && matches!(exceptions.get(&date), Some(Exception::Changed(_))) {
                    exceptions.remove(&date);
                }
            }
        }
    }
}
