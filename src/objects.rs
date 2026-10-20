use std::collections::HashSet;

use chrono::{Datelike, NaiveDate, NaiveDateTime, NaiveTime};

#[derive(Debug, Clone)]
pub enum Month {
    January,
    February,
    March,
    April,
    May,
    June,
    July,
    August,
    September,
    October,
    November,
    December,
}

impl Month {
    pub fn as_str(&self) -> &str {
        match self {
            Month::January => "January",
            Month::February => "February",
            Month::March => "March",
            Month::April => "April",
            Month::May => "May",
            Month::June => "June",
            Month::July => "July",
            Month::August => "August",
            Month::September => "September",
            Month::October => "October",
            Month::November => "November",
            Month::December => "December",
        }
    }

    pub fn as_u32(&self) -> u32 {
        match self {
            Month::January => 1,
            Month::February => 2,
            Month::March => 3,
            Month::April => 4,
            Month::May => 5,
            Month::June => 6,
            Month::July => 7,
            Month::August => 8,
            Month::September => 9,
            Month::October => 10,
            Month::November => 11,
            Month::December => 12,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Year {
    Y2025,
    Y2026,
}

impl Year {
    pub fn as_str(&self) -> &str {
        match self {
            Year::Y2025 => "2025",
            Year::Y2026 => "2026",
        }
    }
    pub fn as_i32(&self) -> i32 {
        match self {
            Year::Y2025 => 2025,
            Year::Y2026 => 2026,
        }
    }
}

//Event(s) are stored in local time (no timezone attached).
#[derive(Debug, Clone)]
pub struct Event {
    title: String,
    datetime: NaiveDateTime,
}

impl Event {
    pub fn new(title: String, datetime: NaiveDateTime) -> Self {
        Self { title, datetime }
    }

    /// Builds an Event from the Create Event form's strings.
    /// `date` is "DD-MM" and `time` is "HH:MM", as produced by chk's date/time inputs.
    pub fn from_form(title: &str, date: &str, time: &str, year: i32) -> Result<Event, String> {
        let title = title.trim();
        if title.is_empty() {
            return Err("Title can't be empty".to_string());
        }
        let date = NaiveDate::parse_from_str(&format!("{date}-{year}"), "%d-%m-%Y")
            .map_err(|e| format!("Invalid date \"{date}\": {e}"))?;
        let time = NaiveTime::parse_from_str(time, "%H:%M")
            .map_err(|e| format!("Invalid time \"{time}\": {e}"))?;

        Ok(Event::new(title.to_string(), date.and_time(time)))
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn datetime(&self) -> NaiveDateTime {
        self.datetime
    }
}

#[derive(Debug, Default)]
pub struct EventRegistry {
    events: Vec<Event>,
}

impl EventRegistry {
    pub fn push(&mut self, event: Event) {
        self.events.push(event)
    }

    pub fn days_with_events(&self, year: i32, month: u32) -> HashSet<u32> {
        self.events
            .iter()
            .filter(|event| event.datetime.year() == year && event.datetime.month() == month)
            .map(|event| event.datetime.day())
            .collect()
    }

    pub fn events_for_day(&self, year: i32, month: u32, day: u32) -> Vec<&Event> {
        let mut events: Vec<&Event> = self
            .events
            .iter()
            .filter(|event| {
                event.datetime.year() == year
                    && event.datetime.month() == month
                    && event.datetime.day() == day
            })
            .collect();
        events.sort_by_key(|event| event.datetime);
        events
    }
}

#[derive(Debug, Clone)]
pub struct EventForEES {
    pub event_title: Option<String>,
    pub year: Option<Year>,
    pub month: Option<Month>,
    pub day: Option<String>,
    pub time: Option<String>,
}

impl EventForEES {
    pub fn new(
        event_title: Option<String>,
        year: Option<Year>,
        month: Option<Month>,
        day: Option<String>,
        time: Option<String>,
    ) -> Self {
        Self {
            event_title,
            year,
            month,
            day,
            time,
        }
    }

    pub fn get_title(&self) -> Option<String> {
        self.event_title.clone()
    }

    pub fn get_year_as_i32(&self) -> i32 {
        self.year.clone().unwrap().as_i32()
    }

    pub fn get_month_as_u32(&self) -> u32 {
        self.month.clone().unwrap().as_u32()
    }

    pub fn get_day_as_u32(&self) -> u32 {
        self.day
            .clone()
            .unwrap()
            .parse::<u32>()
            .expect("Unable to parse Day from EventForEES")
    }

    pub fn get_time(&self) -> Option<String> {
        self.time.clone()
    }

    pub fn all_some(&self) -> bool {
        self.event_title.is_some()
            && self.year.is_some()
            && self.month.is_some()
            && self.day.is_some()
            && self.time.is_some()
    }

    pub fn set_year(&mut self, index: u8) {
        match index {
            0 => {
                self.year = Some(Year::Y2025);
                println!("Year: {:?}", self.year);
            }
            1 => {
                self.year = Some(Year::Y2026);
                println!("Year: {:?}", self.year);
            }
            _ => panic!("Invalid year index: {}", index),
        }
    }

    pub fn set_month(&mut self, index: u8) {
        match index {
            1 => {
                self.month = Some(Month::January);
                println!("Month: {:?}", self.month);
            }
            2 => {
                self.month = Some(Month::February);
                println!("Month: {:?}", self.month);
            }
            3 => {
                self.month = Some(Month::March);
                println!("Month: {:?}", self.month);
            }
            4 => {
                self.month = Some(Month::April);
                println!("Month: {:?}", self.month);
            }
            5 => {
                self.month = Some(Month::May);
                println!("Month: {:?}", self.month);
            }
            6 => {
                self.month = Some(Month::June);
                println!("Month: {:?}", self.month);
            }
            7 => {
                self.month = Some(Month::July);
                println!("Month: {:?}", self.month);
            }
            8 => {
                self.month = Some(Month::August);
                println!("Month: {:?}", self.month);
            }
            9 => {
                self.month = Some(Month::September);
                println!("Month: {:?}", self.month);
            }
            10 => {
                self.month = Some(Month::October);
                println!("Month: {:?}", self.month);
            }
            11 => {
                self.month = Some(Month::November);
                println!("Month: {:?}", self.month);
            }
            12 => {
                self.month = Some(Month::December);
                println!("Month: {:?}", self.month);
            }
            _ => panic!("Invalid month index: {}", index),
        }
    }

    pub fn set_day(&mut self, day: String) {
        self.day = Some(day)
    }

    pub fn set_time(&mut self, time: String) {
        self.time = Some(time)
    }

    pub fn get_month(&self) -> Option<Month> {
        self.month.clone()
    }

    pub fn set_all_none(&mut self) {
        self.event_title = None;
        self.year = None;
        self.month = None;
        self.day = None;
        self.time = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_form_valid_input() {
        let event = Event::from_form("Dentist", "14-11", "09:30", 2026).unwrap();
        let expected = NaiveDate::from_ymd_opt(2026, 11, 14)
            .unwrap()
            .and_hms_opt(9, 30, 0)
            .unwrap();

        assert_eq!(event.title(), "Dentist");
        assert_eq!(event.datetime(), expected);
    }

    #[test]
    fn from_form_rejects_empty_title() {
        let err = Event::from_form("", "14-11", "09:30", 2026).unwrap_err();
        assert_eq!(err, "Title can't be empty");
    }

    #[test]
    fn from_form_rejects_whitespace_title() {
        let err = Event::from_form("   ", "14-11", "09:30", 2026).unwrap_err();
        assert_eq!(err, "Title can't be empty");
    }

    #[test]
    fn from_form_rejects_zero_date() {
        let err = Event::from_form("Dentist", "00-00", "09:30", 2026).unwrap_err();
        assert!(err.contains("Invalid date"), "{err}");
    }

    #[test]
    fn from_form_rejects_february_31st() {
        let err = Event::from_form("Dentist", "31-02", "09:30", 2026).unwrap_err();
        assert!(err.contains("Invalid date"), "{err}");
    }

    #[test]
    fn from_form_rejects_hour_25() {
        let err = Event::from_form("Dentist", "14-11", "25:00", 2026).unwrap_err();
        assert!(err.contains("Invalid time"), "{err}");
    }
}
