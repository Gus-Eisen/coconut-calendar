use chk::{Context, State};
use chrono::{Datelike, Local, NaiveDate, NaiveDateTime, NaiveTime, TimeDelta};
use ramp::prism::air::{Contract, Id, Instance, Metadata};
use serde::{Deserialize, Serialize};

use std::collections::BTreeMap;

/// Chosen by the creating device, so it is known before AIR confirms the event.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct EventId(Id);

impl EventId {
    pub fn random() -> Self { EventId(Id::random()) }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Event {
    title: String,
    start: NaiveDateTime,
    duration_minutes: u32,
    location: Option<String>,
    info: Option<String>,
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

    /// Reads `HH:MM` as hours and minutes; accepts 00:01 up to 24:00.
    pub fn duration(input: &str) -> Option<u32> {
        let (hours, minutes) = input.split_once(':')?;
        let (hours, minutes) = (hours.parse::<u32>().ok()?, minutes.parse::<u32>().ok()?);
        let total = hours * 60 + minutes;
        (minutes < 60 && (1..=24 * 60).contains(&total)).then_some(total)
    }

    pub fn from_states(states: &[State]) -> Event {
        let [
            State::Text(title), State::Number(date), State::Number(time), State::Number(duration),
            State::Text(location), State::Text(info),
        ] = states else {
            unreachable!()
        };
        Event {
            title: title.trim().to_string(),
            start: Self::date(date).unwrap().and_time(Self::time(time).unwrap()),
            duration_minutes: Self::duration(duration).unwrap(),
            location: Self::optional(location),
            info: Self::optional(info),
            deleted: false,
        }
    }

    pub fn title(&self) -> &str { &self.title }
    pub fn start(&self) -> NaiveDateTime { self.start }
    pub fn end(&self) -> NaiveDateTime {
        self.start + TimeDelta::minutes(self.duration_minutes.into())
    }
    pub fn duration_minutes(&self) -> u32 { self.duration_minutes }
    pub fn location(&self) -> Option<&str> { self.location.as_deref() }
    pub fn info(&self) -> Option<&str> { self.info.as_deref() }

    fn optional(input: &str) -> Option<String> {
        let input = input.trim();
        (!input.is_empty()).then(|| input.to_string())
    }
}

/// Postcard encodes variants by position: only append new variants, never insert or reorder.
#[derive(Debug, Clone, Hash, Serialize, Deserialize)]
pub enum CalendarMessage {
    CreateEvent { id: EventId, event: Event },
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
        calendar.send(CalendarMessage::CreateEvent { id: EventId::random(), event });
    }

    pub fn delete_event(calendar: &mut Instance<Calendar>, id: EventId) {
        calendar.send(CalendarMessage::DeleteEvent { id });
    }

    pub fn on(&self, date: NaiveDate) -> Vec<(EventId, &Event)> {
        let mut events = self.events.iter()
            .filter(|(_, event)| !event.deleted && event.start.date() == date)
            .map(|(id, event)| (*id, event))
            .collect::<Vec<_>>();
        events.sort_by_key(|(_, event)| event.start);
        events
    }
}

impl Contract for Calendar {
    type Init = ();
    type Message = CalendarMessage;
    type Result = ();

    fn id() -> Id { Id::hash("CoconutCalendar") }

    fn init(_init: (), _metadata: Metadata) -> Self { Calendar::default() }

    fn apply(&mut self, message: CalendarMessage, _metadata: Metadata) {
        match message {
            CalendarMessage::CreateEvent { id, event } => {
                self.events.entry(id).or_insert(Event { deleted: false, ..event });
            }
            CalendarMessage::DeleteEvent { id } => {
                if let Some(event) = self.events.get_mut(&id) { event.deleted = true; }
            }
        }
    }
}
