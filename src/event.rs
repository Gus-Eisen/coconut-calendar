use chk::{Context, State};
use chrono::{Datelike, Local, NaiveDate, NaiveDateTime, NaiveTime};
use ramp::prism::air::{Contract, Id, Instance, Metadata};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Event {
    title: String,
    datetime: NaiveDateTime,
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

    pub fn from_states(states: &[State]) -> Event {
        let [State::Text(title), State::Number(date), State::Number(time)] = states else {
            unreachable!()
        };
        let datetime = Self::date(date)
            .unwrap()
            .and_time(Self::time(time).unwrap());
        Event {
            title: title.trim().to_string(),
            datetime,
            deleted: false,
        }
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn datetime(&self) -> NaiveDateTime {
        self.datetime
    }

    pub fn create(ctx: &mut Context, event: Event) {
        ctx.air.instances::<Event>().create(event);
    }

    pub fn delete(instance: &mut Instance<Event>) {
        instance.send(EventMessage::Delete);
    }
}

#[derive(Debug, Clone, Hash, Serialize, Deserialize)]
pub enum EventMessage {
    Delete,
}

/// Each event is its own AIR instance. Deleting only marks it; AIR never erases data.
impl Contract for Event {
    type Init = Event;
    type Message = EventMessage;
    type Result = ();

    fn id() -> Id {
        Id::hash("CoconutCalendarEventV2")
    }

    fn init(init: Event, _metadata: Metadata) -> Self {
        init
    }

    fn apply(&mut self, message: EventMessage, _metadata: Metadata) {
        match message {
            EventMessage::Delete => self.deleted = true,
        }
    }
}

pub struct EventRegistry {
    events: Vec<(Instance<Event>, Event)>,
}

impl EventRegistry {
    pub fn new(instances: &[Instance<Event>]) -> Self {
        let events = instances
            .iter()
            .filter_map(|instance| {
                let event = instance.pending().clone();
                (!event.deleted).then(|| (instance.clone(), event))
            })
            .collect();
        Self { events }
    }

    pub fn on(&self, date: NaiveDate) -> Vec<&(Instance<Event>, Event)> {
        let mut events = self
            .events
            .iter()
            .filter(|(_, event)| event.datetime.date() == date)
            .collect::<Vec<_>>();
        events.sort_by_key(|(_, event)| event.datetime.time());
        events
    }
}
