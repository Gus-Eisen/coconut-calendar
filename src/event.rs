use chk::{Context, State, Watcher};
use chrono::{Datelike, Local, NaiveDate, NaiveDateTime, NaiveTime};
use ramp::prism::hardware::Data;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct EventRegistry {
    events: Vec<Event>,
}

impl EventRegistry {
    pub fn load(ctx: &Context) -> Self {
        match ctx.hardware.get_cache("coconut_calendar.events") {
            Some(Data::Text(saved)) => serde_json::from_str(&saved).expect("Invalid saved events"),
            Some(_) => panic!("The saved events must be text"),
            None => Self::default(),
        }
    }

    pub fn save(&self, ctx: &Context) {
        let saved = serde_json::to_string(self).expect("Could not encode the events");
        ctx.hardware.set_cache("coconut_calendar.events", &Data::Text(saved));
    }

    pub fn push(&mut self, event: Event) {self.events.push(event)}

    pub fn on(&self, date: NaiveDate) -> Vec<&Event> {
        let mut events = self.events.iter().filter(|event| event.datetime.date() == date)
            .collect::<Vec<_>>();
        events.sort_by_key(|event| event.datetime.time());
        events
    }

    pub fn add(ctx: &mut Context, event: Event) {
        let mut registry = Self::load(ctx);
        registry.push(event);
        registry.save(ctx);
    }
}

impl Watcher for EventRegistry {
    type Output = EventRegistry;

    fn poll(&mut self, ctx: &mut Context) -> bool {Self::load(ctx) != *self}

    fn get(&mut self, ctx: &mut Context) -> EventRegistry {
        *self = Self::load(ctx);
        self.clone()
    }
}
