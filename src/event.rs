use chk::{Context, State};
use chrono::{Datelike, Local, NaiveDate, NaiveDateTime, NaiveTime};
use ramp::prism::air::{Contract, Id, Instance, Metadata};
use ramp::prism::hardware::Data;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Event {
    title: String,
    datetime: NaiveDateTime,
}

impl Event {
    pub fn window_start() -> NaiveDate {Local::now().date_naive().with_day(1).unwrap()}

    pub fn date(input: &str) -> Option<NaiveDate> {
        let first = Self::window_start();
        let year = first.year();
        let parsed = NaiveDate::parse_from_str(&format!("{input}-{year}"), "%d-%m-%Y").ok()?;
        if parsed < first {parsed.with_year(year + 1)} else {Some(parsed)}
    }

    pub fn time(input: &str) -> Option<NaiveTime> {NaiveTime::parse_from_str(input, "%H:%M").ok()}

    pub fn from_states(states: &[State]) -> Event {
        let [State::Text(title), State::Number(date), State::Number(time)] = states else {
            unreachable!()
        };
        let datetime = Self::date(date).unwrap().and_time(Self::time(time).unwrap());
        Event {title: title.trim().to_string(), datetime}
    }

    pub fn title(&self) -> &str {&self.title}

    pub fn datetime(&self) -> NaiveDateTime {self.datetime}

    pub fn create(ctx: &mut Context, event: Event) {ctx.air.instances::<Event>().create(event);}
}

/// Each event is its own AIR instance; it never changes after creation.
impl Contract for Event {
    type Init = Event;
    type Message = ();
    type Result = ();

    fn id() -> Id {Id::hash("CoconutCalendarEvent")}

    fn init(init: Event, _metadata: Metadata) -> Self {init}

    fn apply(&mut self, _message: (), _metadata: Metadata) {}
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EventRegistry {
    events: Vec<Event>,
}

impl EventRegistry {
    pub fn new(instances: &[Instance<Event>]) -> Self {
        Self {events: instances.iter().map(|instance| instance.pending().clone()).collect()}
    }

    pub fn on(&self, date: NaiveDate) -> Vec<&Event> {
        let mut events = self.events.iter().filter(|event| event.datetime.date() == date)
            .collect::<Vec<_>>();
        events.sort_by_key(|event| event.datetime.time());
        events
    }

    /// Copies events saved by earlier versions into AIR. Repeating it creates no duplicates.
    pub fn import(ctx: &mut Context) {
        let Some(saved) = ctx.hardware.get_cache("coconut_calendar.events") else {return};
        let Data::Text(saved) = saved else {panic!("The saved events must be text")};
        let registry = serde_json::from_str::<Self>(&saved).expect("Invalid saved events");
        for event in registry.events {Event::create(ctx, event)}
    }
}
