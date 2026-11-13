use chk::{Context, State};
use chrono::offset::LocalResult;
use chrono::{
    DateTime, Datelike, Local, NaiveDate, NaiveDateTime, NaiveTime, TimeDelta, TimeZone, Utc,
    Weekday,
};
use chrono_tz::Tz;
use ramp::prism::air::{Contract, Id, Instance, Metadata, Name};
use serde::{Deserialize, Serialize};

use std::collections::{BTreeMap, BTreeSet};

/// Chosen by the creating device, so it is known before AIR confirms the event.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct EventId(Id);

impl EventId {
    pub fn random() -> Self { EventId(Id::random()) }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Timing {
    AllDay {
        first_day: NaiveDate,
        last_day: NaiveDate,
    },
    /// `start` and `end` are clock times in `time_zone`.
    Timed {
        start: NaiveDateTime,
        end: NaiveDateTime,
        time_zone: Tz,
    },
}

impl Timing {
    /// The device's time zone, or UTC if the device reports one that isn't recognised.
    pub fn device_zone() -> Tz {
        iana_time_zone::get_timezone().ok().and_then(|zone| zone.parse().ok()).unwrap_or(Tz::UTC)
    }

    pub fn is_valid(&self) -> bool {
        match self {
            Timing::AllDay { first_day, last_day } => last_day >= first_day,
            Timing::Timed { .. } => self.instants().is_some_and(|(start, end)| end > start),
        }
    }

    /// The start and end as clock times in `zone`; `None` for an all-day event.
    pub fn in_zone(&self, zone: Tz) -> Option<(NaiveDateTime, NaiveDateTime)> {
        let (start, end) = self.instants()?;
        Some((start.with_timezone(&zone).naive_local(), end.with_timezone(&zone).naive_local()))
    }

    /// Whether any part of the event falls on `date` in `zone`. All-day events ignore `zone`.
    pub fn covers(&self, date: NaiveDate, zone: Tz) -> bool {
        match self {
            Timing::AllDay { first_day, last_day } => (*first_day..=*last_day).contains(&date),
            Timing::Timed { .. } => self.instants().is_some_and(|(start, end)| {
                let midnight = |date: NaiveDate| Self::instant(date.and_time(NaiveTime::MIN), zone);
                start < midnight(date.succ_opt().unwrap()) && end > midnight(date)
            }),
        }
    }

    fn instants(&self) -> Option<(DateTime<Utc>, DateTime<Utc>)> {
        match self {
            Timing::AllDay { .. } => None,
            Timing::Timed { start, end, time_zone } => {
                Some((Self::instant(*start, *time_zone), Self::instant(*end, *time_zone)))
            }
        }
    }

    /// A clock time that doesn't exist (clocks spring forward) moves forward one hour; one that
    /// happens twice (clocks fall back) uses the first.
    fn instant(local: NaiveDateTime, zone: Tz) -> DateTime<Utc> {
        match zone.from_local_datetime(&local) {
            LocalResult::Single(time) | LocalResult::Ambiguous(time, _) => time.to_utc(),
            LocalResult::None => Self::instant(local + TimeDelta::hours(1), zone),
        }
    }
}

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
pub struct TimeProposal {
    proposer: Name,
    occurrence: Option<NaiveDate>,
    timing: Timing,
    approvals: BTreeSet<Name>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Recurrence {
    frequency: Frequency,
    until: Option<NaiveDate>,
    /// Keyed by the date the occurrence would normally start on.
    exceptions: BTreeMap<NaiveDate, Exception>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Frequency {
    Daily,
    Weekly { days: Vec<Weekday> },
    Monthly { day_of_month: u8 },
    Yearly,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Exception {
    Cancelled,
    Changed(Override),
}

/// What's different about one occurrence. `None` means "same as the series".
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Override {
    title: Option<String>,
    timing: Option<Timing>,
    location: Option<Option<String>>,
    info: Option<Option<String>>,
    video_link: Option<Option<String>>,
    attachments: Option<Vec<String>>,
    responses: BTreeMap<Name, Response>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Event {
    title: String,
    timing: Timing,
    location: Option<String>,
    info: Option<String>,
    attendees: Vec<Attendee>,
    video_link: Option<String>,
    attachments: Vec<String>,
    recurrence: Option<Recurrence>,
    category: Category,
    time_proposals: Vec<TimeProposal>,
    private: bool,
    deleted: bool,
}

impl Event {
    pub fn window_start() -> NaiveDate {
        Local::now().date_naive().with_day(1).unwrap()
    }

    pub fn date(input: &str) -> Option<NaiveDate> {
        let first = Self::window_start();
        let year = first.year();
        let parsed = NaiveDate::parse_from_str(&format!("{input}-{year}"), "%d-%m-%Y").ok()?;
        if parsed < first {
            parsed.with_year(year + 1)
        } else {
            Some(parsed)
        }
    }

    pub fn time(input: &str) -> Option<NaiveTime> {
        NaiveTime::parse_from_str(input, "%H:%M").ok()
    }

    /// The new-event form creates timed events in the device's time zone.
    pub fn from_states(states: &[State]) -> Event {
        let [
            State::Text(title), State::Enumerator(category), State::Number(date),
            State::Number(start), State::Number(end_date), State::Number(end),
            State::Text(location), State::Text(info),
        ] = states else {
            unreachable!()
        };
        Event {
            title: title.trim().to_string(),
            timing: Timing::Timed {
                start: Self::date(date).unwrap().and_time(Self::time(start).unwrap()),
                end: Self::date(end_date).unwrap().and_time(Self::time(end).unwrap()),
                time_zone: Timing::device_zone(),
            },
            location: Self::optional(location),
            info: Self::optional(info),
            attendees: vec![],
            video_link: None,
            attachments: vec![],
            recurrence: None,
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
    pub fn video_link(&self) -> Option<&str> { self.video_link.as_deref() }
    pub fn attachments(&self) -> &[String] { &self.attachments }
    pub fn recurrence(&self) -> Option<&Recurrence> { self.recurrence.as_ref() }
    pub fn category(&self) -> Category { self.category }
    pub fn time_proposals(&self) -> &[TimeProposal] { &self.time_proposals }
    pub fn private(&self) -> bool { self.private }

    fn optional(input: &str) -> Option<String> {
        let input = input.trim();
        (!input.is_empty()).then(|| input.to_string())
    }
}

/// One appearance of an event on a day.
#[derive(Debug, Clone, Copy)]
pub struct Occurrence<'a> {
    id: EventId,
    event: &'a Event,
}

impl<'a> Occurrence<'a> {
    pub fn id(&self) -> EventId { self.id }
    pub fn event(&self) -> &'a Event { self.event }
}

/// Postcard encodes variants by position: only append new variants, never insert or reorder.
#[derive(Debug, Clone, Hash, Serialize, Deserialize)]
pub enum CalendarMessage {
    CreateEvent { id: EventId, event: Box<Event> },
    DeleteEvent { id: EventId },
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

    /// Events on `date` in the viewer's time zone `zone`: all-day events first, then by start.
    pub fn on(&self, date: NaiveDate, zone: Tz) -> Vec<Occurrence<'_>> {
        let mut occurrences = self.events.iter()
            .filter(|(_, event)| !event.deleted && event.timing.covers(date, zone))
            .map(|(id, event)| Occurrence { id: *id, event })
            .collect::<Vec<_>>();
        occurrences.sort_by_key(|occurrence| {
            occurrence.event.timing.instants().map(|(start, _)| start)
        });
        occurrences
    }
}

impl Contract for Calendar {
    type Init = ();
    type Message = CalendarMessage;
    type Result = ();

    fn id() -> Id { Id::hash("CoconutCalendarV2") }

    fn init(_init: (), _metadata: Metadata) -> Self { Calendar::default() }

    fn apply(&mut self, message: CalendarMessage, _metadata: Metadata) {
        match message {
            CalendarMessage::CreateEvent { id, event } => {
                if event.title.trim().is_empty() || !event.timing.is_valid() { return; }
                let event = Event {
                    attendees: vec![],
                    recurrence: None,
                    time_proposals: vec![],
                    deleted: false,
                    ..*event
                };
                self.events.entry(id).or_insert(event);
            }
            CalendarMessage::DeleteEvent { id } => {
                if let Some(event) = self.events.get_mut(&id) { event.deleted = true; }
            }
        }
    }
}
