// use std::sync::{Arc, Mutex};
//
// use crate::event_editor_screen::EventEditorScreen;
// use crate::objects::EventForEES;
// use crate::objects::EventRegistry;
// use crate::PageFlow;
//
// use pelican_ui::drawable::{Component, SizedTree};
// use pelican_ui::event::{Event, OnEvent};
// use pelican_ui::components::RadioSelector;
// use pelican_ui::interface::general::{Bumper, Content, Page};
// use pelican_ui::interface::navigation::{AppPage, NavigationEvent};
// use pelican_ui::interface::general::Header;
// use pelican_ui::layout::{Offset, Stack};
// use pelican_ui::theme::Theme;
// use pelican_ui::{Callback, Context, Request};
//
// pub mod year_selector_screen_block {
//
//     use super::*;
//
//     const Y2026: u8 = 1;
//
//     #[derive(Debug, Component, Clone)]
//     pub struct YearSelectorScreen(
//         Stack,
//         Page,
//         #[skip] Arc<Mutex<EventForEES>>,
//         #[skip] Arc<Mutex<EventRegistry>>,
//     );
//
//     impl OnEvent for YearSelectorScreen {
//         fn on_event(
//             &mut self,
//             _ctx: &mut Context,
//             _sized: &SizedTree,
//             event: Box<dyn Event>,
//         ) -> Vec<Box<dyn Event>> {
//             vec![event]
//         }
//     }
//
//     impl AppPage for YearSelectorScreen {}
//
//     impl YearSelectorScreen {
//         pub fn new(
//             theme: &Theme,
//             event_for_ees: Arc<Mutex<EventForEES>>,
//             event_registry: Arc<Mutex<EventRegistry>>,
//         ) -> Self {
//             let efees_for_radio = event_for_ees.clone();
//             let year_radioselector = RadioSelector::new(
//                 theme,
//                 usize::MAX,
//                 vec![(
//                     "2026",
//                     "",
//                     Box::new({
//                         let efees = efees_for_radio.clone();
//                         move |_ctx: &mut Context, _: &Theme| {
//                             efees.lock().unwrap().set_year(Y2026);
//                             println!("2026 selected.");
//                         }
//                     }) as Box<dyn Callback>,
//                 )],
//             );
//
//             let efees_for_bumper = event_for_ees.clone();
//             let event_reg_for_bumper = event_registry.clone();
//             let content = Content::new(
//                 Offset::Start,
//                 vec![Box::new(year_radioselector)],
//                 Box::new(|_| true),
//             );
//             let bumper = Bumper::stack(
//                 theme,
//                 Some("Save Year"),
//                 move |ctx: &mut Context, theme: &Theme| {
//                     let page = Box::new(EventEditorScreen::new(
//                         theme,
//                         None,
//                         None,
//                         None,
//                         efees_for_bumper.clone(),
//                         event_reg_for_bumper.clone(),
//                     ));
//                     ctx.send(Request::event(NavigationEvent::push(PageFlow::new(page))));
//                     println!("Save Year bumper clicked.");
//                 },
//                 None,
//             );
//
//             YearSelectorScreen(
//                 Stack::default(),
//                 Page::new(Header::stack(theme, "Select Year", None), content, Some(bumper)),
//                 event_for_ees,
//                 event_registry,
//             )
//         }
//     }
// }
//
// pub mod month_selector_screen_block {
//
//     use super::*;
//
//     const JAN: u8 = 1;
//     const FEB: u8 = 2;
//     const MAR: u8 = 3;
//     const APR: u8 = 4;
//     const MAY: u8 = 5;
//     const JUN: u8 = 6;
//     const JUL: u8 = 7;
//     const AUG: u8 = 8;
//     const SEP: u8 = 9;
//     const OCT: u8 = 10;
//     const NOV: u8 = 11;
//     const DEC: u8 = 12;
//
//     #[derive(Debug, Component, Clone)]
//     pub struct MonthSelectorScreen(
//         Stack,
//         Page,
//         #[skip] Arc<Mutex<EventForEES>>,
//         #[skip] Arc<Mutex<EventRegistry>>,
//     );
//
//     impl OnEvent for MonthSelectorScreen {
//         fn on_event(
//             &mut self,
//             _ctx: &mut Context,
//             _sized: &SizedTree,
//             event: Box<dyn Event>,
//         ) -> Vec<Box<dyn Event>> {
//             vec![event]
//         }
//     }
//
//     impl AppPage for MonthSelectorScreen {}
//
//     impl MonthSelectorScreen {
//         pub fn new(
//             theme: &Theme,
//             event_for_ees: Arc<Mutex<EventForEES>>,
//             event_registry: Arc<Mutex<EventRegistry>>,
//         ) -> Self {
//             let make_cb = |efees: Arc<Mutex<EventForEES>>, index: u8, name: &'static str| {
//                 Box::new(move |_ctx: &mut Context, _: &Theme| {
//                     efees.lock().unwrap().set_month(index);
//                     println!("Selected {}.", name);
//                 }) as Box<dyn Callback>
//             };
//
//             let efr = event_for_ees.clone();
//             let month_radioselector = RadioSelector::new(
//                 theme,
//                 usize::MAX,
//                 vec![
//                     ("January",   "1",  make_cb(efr.clone(), JAN, "January")),
//                     ("February",  "2",  make_cb(efr.clone(), FEB, "February")),
//                     ("March",     "3",  make_cb(efr.clone(), MAR, "March")),
//                     ("April",     "4",  make_cb(efr.clone(), APR, "April")),
//                     ("May",       "5",  make_cb(efr.clone(), MAY, "May")),
//                     ("June",      "6",  make_cb(efr.clone(), JUN, "June")),
//                     ("July",      "7",  make_cb(efr.clone(), JUL, "July")),
//                     ("August",    "8",  make_cb(efr.clone(), AUG, "August")),
//                     ("September", "9",  make_cb(efr.clone(), SEP, "September")),
//                     ("October",   "10", make_cb(efr.clone(), OCT, "October")),
//                     ("November",  "11", make_cb(efr.clone(), NOV, "November")),
//                     ("December",  "12", make_cb(efr.clone(), DEC, "December")),
//                 ],
//             );
//
//             let efees_for_bumper = event_for_ees.clone();
//             let event_reg_for_bumper = event_registry.clone();
//             let content = Content::new(
//                 Offset::Start,
//                 vec![Box::new(month_radioselector)],
//                 Box::new(|_| true),
//             );
//             let bumper = Bumper::stack(
//                 theme,
//                 Some("Save Month"),
//                 move |ctx: &mut Context, theme: &Theme| {
//                     let page = Box::new(EventEditorScreen::new(
//                         theme,
//                         None,
//                         None,
//                         None,
//                         efees_for_bumper.clone(),
//                         event_reg_for_bumper.clone(),
//                     ));
//                     ctx.send(Request::event(NavigationEvent::push(PageFlow::new(page))));
//                     println!("Save Month button clicked.");
//                 },
//                 None,
//             );
//
//             MonthSelectorScreen(
//                 Stack::default(),
//                 Page::new(Header::stack(theme, "Select Month", None), content, Some(bumper)),
//                 event_for_ees,
//                 event_registry,
//             )
//         }
//     }
// }
//
// pub mod day_selector_screen_block {
//     use crate::objects::Month;
//     use chrono::{Datelike, NaiveDate, Weekday};
//
//     use super::*;
//
//     #[derive(Component, Debug, Clone)]
//     pub struct DaySelectorScreen(
//         Stack,
//         Page,
//         #[skip] Arc<Mutex<EventForEES>>,
//         #[skip] Arc<Mutex<EventRegistry>>,
//     );
//
//     impl OnEvent for DaySelectorScreen {
//         fn on_event(
//             &mut self,
//             _ctx: &mut Context,
//             _sized: &SizedTree,
//             event: Box<dyn Event>,
//         ) -> Vec<Box<dyn Event>> {
//             vec![event]
//         }
//     }
//
//     impl AppPage for DaySelectorScreen {}
//
//     impl DaySelectorScreen {
//         pub fn new(
//             theme: &Theme,
//             event_for_ees: Arc<Mutex<EventForEES>>,
//             event_registry: Arc<Mutex<EventRegistry>>,
//         ) -> Self {
//             let day_radioselector =
//                 Self::day_radioselector_builder(theme, event_for_ees.clone());
//
//             let efees_for_bumper = event_for_ees.clone();
//             let event_reg_for_bumper = event_registry.clone();
//             let content = Content::new(
//                 Offset::Start,
//                 vec![Box::new(day_radioselector)],
//                 Box::new(|_| true),
//             );
//             let bumper = Bumper::stack(
//                 theme,
//                 Some("Save Day"),
//                 move |ctx: &mut Context, theme: &Theme| {
//                     let page = Box::new(EventEditorScreen::new(
//                         theme,
//                         None,
//                         None,
//                         None,
//                         efees_for_bumper.clone(),
//                         event_reg_for_bumper.clone(),
//                     ));
//                     ctx.send(Request::event(NavigationEvent::push(PageFlow::new(page))));
//                     println!("Save Day button clicked.");
//                 },
//                 None,
//             );
//
//             DaySelectorScreen(
//                 Stack::default(),
//                 Page::new(Header::stack(theme, "Select Day", None), content, Some(bumper)),
//                 event_for_ees,
//                 event_registry,
//             )
//         }
//
//         pub fn amt_of_days(event_for_ees: &Arc<Mutex<EventForEES>>) -> u8 {
//             let efees = event_for_ees.lock().unwrap();
//             match efees.get_month().unwrap() {
//                 Month::January => 31,
//                 //TODO: code in logic for leap year.
//                 Month::February => 28,
//                 Month::March => 31,
//                 Month::April => 30,
//                 Month::May => 31,
//                 Month::June => 30,
//                 Month::July => 31,
//                 Month::August => 31,
//                 Month::September => 30,
//                 Month::October => 31,
//                 Month::November => 30,
//                 Month::December => 31,
//             }
//         }
//
//         pub fn day_u8_to_stringliteral(day: u8) -> &'static str {
//             match day {
//                 1 => "1", 2 => "2", 3 => "3", 4 => "4", 5 => "5",
//                 6 => "6", 7 => "7", 8 => "8", 9 => "9", 10 => "10",
//                 11 => "11", 12 => "12", 13 => "13", 14 => "14", 15 => "15",
//                 16 => "16", 17 => "17", 18 => "18", 19 => "19", 20 => "20",
//                 21 => "21", 22 => "22", 23 => "23", 24 => "24", 25 => "25",
//                 26 => "26", 27 => "27", 28 => "28", 29 => "29", 30 => "30",
//                 31 => "31",
//                 _ => panic!("Out of bounds day."),
//             }
//         }
//
//         pub fn days_of_week_determiner(event_for_ees: &Arc<Mutex<EventForEES>>) -> Vec<Weekday> {
//             let ees = event_for_ees.lock().unwrap();
//             let mut date =
//                 NaiveDate::from_ymd_opt(ees.get_year_as_i32(), ees.get_month_as_u32(), 1)
//                     .unwrap();
//             let mut weekdays = Vec::new();
//             while date.month() == ees.get_month_as_u32() {
//                 weekdays.push(date.weekday());
//                 date = date.succ_opt().unwrap();
//             }
//             weekdays
//         }
//
//         pub fn day_radioselector_builder(
//             theme: &Theme,
//             event_for_ees: Arc<Mutex<EventForEES>>,
//         ) -> RadioSelector {
//             let amt_of_days = Self::amt_of_days(&event_for_ees);
//             let vec_of_weekday = Self::days_of_week_determiner(&event_for_ees);
//
//             let vec_of_radioselector: Vec<(&str, &str, Box<dyn Callback>)> = (1..=amt_of_days)
//                 .map(|day| {
//                     let day_str = Self::day_u8_to_stringliteral(day);
//                     let weekday_str = vec_of_weekday
//                         .get((day - 1) as usize)
//                         .map(|wd| match wd {
//                             Weekday::Mon => "Mon",
//                             Weekday::Tue => "Tue",
//                             Weekday::Wed => "Wed",
//                             Weekday::Thu => "Thu",
//                             Weekday::Fri => "Fri",
//                             Weekday::Sat => "Sat",
//                             Weekday::Sun => "Sun",
//                         })
//                         .expect("Weekday variant not found.");
//
//                     let efees_clone = event_for_ees.clone();
//                     (
//                         day_str,
//                         weekday_str,
//                         Box::new(move |_ctx: &mut Context, _: &Theme| {
//                             efees_clone.lock().unwrap().set_day(day.to_string());
//                             println!("Selected day {}", day);
//                         }) as Box<dyn Callback>,
//                     )
//                 })
//                 .collect();
//
//             RadioSelector::new(theme, usize::MAX, vec_of_radioselector)
//         }
//     }
// }
//
// pub mod time_selector_screen_block {
//     use super::*;
//
//     const TIME_DISPLAYS: [&str; 96] = [
//         "12:00 a.m.", "12:15 a.m.", "12:30 a.m.", "12:45 a.m.",
//         "1:00 a.m.",  "1:15 a.m.",  "1:30 a.m.",  "1:45 a.m.",
//         "2:00 a.m.",  "2:15 a.m.",  "2:30 a.m.",  "2:45 a.m.",
//         "3:00 a.m.",  "3:15 a.m.",  "3:30 a.m.",  "3:45 a.m.",
//         "4:00 a.m.",  "4:15 a.m.",  "4:30 a.m.",  "4:45 a.m.",
//         "5:00 a.m.",  "5:15 a.m.",  "5:30 a.m.",  "5:45 a.m.",
//         "6:00 a.m.",  "6:15 a.m.",  "6:30 a.m.",  "6:45 a.m.",
//         "7:00 a.m.",  "7:15 a.m.",  "7:30 a.m.",  "7:45 a.m.",
//         "8:00 a.m.",  "8:15 a.m.",  "8:30 a.m.",  "8:45 a.m.",
//         "9:00 a.m.",  "9:15 a.m.",  "9:30 a.m.",  "9:45 a.m.",
//         "10:00 a.m.", "10:15 a.m.", "10:30 a.m.", "10:45 a.m.",
//         "11:00 a.m.", "11:15 a.m.", "11:30 a.m.", "11:45 a.m.",
//         "12:00 p.m.", "12:15 p.m.", "12:30 p.m.", "12:45 p.m.",
//         "1:00 p.m.",  "1:15 p.m.",  "1:30 p.m.",  "1:45 p.m.",
//         "2:00 p.m.",  "2:15 p.m.",  "2:30 p.m.",  "2:45 p.m.",
//         "3:00 p.m.",  "3:15 p.m.",  "3:30 p.m.",  "3:45 p.m.",
//         "4:00 p.m.",  "4:15 p.m.",  "4:30 p.m.",  "4:45 p.m.",
//         "5:00 p.m.",  "5:15 p.m.",  "5:30 p.m.",  "5:45 p.m.",
//         "6:00 p.m.",  "6:15 p.m.",  "6:30 p.m.",  "6:45 p.m.",
//         "7:00 p.m.",  "7:15 p.m.",  "7:30 p.m.",  "7:45 p.m.",
//         "8:00 p.m.",  "8:15 p.m.",  "8:30 p.m.",  "8:45 p.m.",
//         "9:00 p.m.",  "9:15 p.m.",  "9:30 p.m.",  "9:45 p.m.",
//         "10:00 p.m.", "10:15 p.m.", "10:30 p.m.", "10:45 p.m.",
//         "11:00 p.m.", "11:15 p.m.", "11:30 p.m.", "11:45 p.m.",
//     ];
//
//     const TIME_CODES: [&str; 96] = [
//         "0000", "0015", "0030", "0045", "0100", "0115", "0130", "0145",
//         "0200", "0215", "0230", "0245", "0300", "0315", "0330", "0345",
//         "0400", "0415", "0430", "0445", "0500", "0515", "0530", "0545",
//         "0600", "0615", "0630", "0645", "0700", "0715", "0730", "0745",
//         "0800", "0815", "0830", "0845", "0900", "0915", "0930", "0945",
//         "1000", "1015", "1030", "1045", "1100", "1115", "1130", "1145",
//         "1200", "1215", "1230", "1245", "1300", "1315", "1330", "1345",
//         "1400", "1415", "1430", "1445", "1500", "1515", "1530", "1545",
//         "1600", "1615", "1630", "1645", "1700", "1715", "1730", "1745",
//         "1800", "1815", "1830", "1845", "1900", "1915", "1930", "1945",
//         "2000", "2015", "2030", "2045", "2100", "2115", "2130", "2145",
//         "2200", "2215", "2230", "2245", "2300", "2315", "2330", "2345",
//     ];
//
//     #[derive(Component, Debug, Clone)]
//     pub struct TimeSelectorScreen(
//         Stack,
//         Page,
//         #[skip] Arc<Mutex<EventForEES>>,
//         #[skip] Arc<Mutex<EventRegistry>>,
//     );
//
//     impl OnEvent for TimeSelectorScreen {
//         fn on_event(
//             &mut self,
//             _ctx: &mut Context,
//             _sized: &SizedTree,
//             event: Box<dyn Event>,
//         ) -> Vec<Box<dyn Event>> {
//             vec![event]
//         }
//     }
//
//     impl AppPage for TimeSelectorScreen {}
//
//     impl TimeSelectorScreen {
//         pub fn new(
//             theme: &Theme,
//             event_for_ees: Arc<Mutex<EventForEES>>,
//             event_registry: Arc<Mutex<EventRegistry>>,
//         ) -> Self {
//             let time_builder = vec_for_timeselector_builder(event_for_ees.clone());
//             let time_radioselector = RadioSelector::new(theme, usize::MAX, time_builder);
//
//             let efees_for_bumper = event_for_ees.clone();
//             let event_reg_for_bumper = event_registry.clone();
//             let content = Content::new(
//                 Offset::Start,
//                 vec![Box::new(time_radioselector)],
//                 Box::new(|_| true),
//             );
//             let bumper = Bumper::stack(
//                 theme,
//                 Some("Save Time"),
//                 move |ctx: &mut Context, theme: &Theme| {
//                     let page = Box::new(EventEditorScreen::new(
//                         theme,
//                         None,
//                         None,
//                         None,
//                         efees_for_bumper.clone(),
//                         event_reg_for_bumper.clone(),
//                     ));
//                     ctx.send(Request::event(NavigationEvent::push(PageFlow::new(page))));
//                     println!("Save Time bumper clicked.");
//                 },
//                 None,
//             );
//
//             TimeSelectorScreen(
//                 Stack::default(),
//                 Page::new(
//                     Header::stack(theme, "Select Time", None),
//                     content,
//                     Some(bumper),
//                 ),
//                 event_for_ees,
//                 event_registry,
//             )
//         }
//     }
//
//     fn vec_for_timeselector_builder(
//         event_for_ees: Arc<Mutex<EventForEES>>,
//     ) -> Vec<(&'static str, &'static str, Box<dyn Callback>)> {
//         (0..96)
//             .map(|i| {
//                 let display_str = TIME_DISPLAYS[i];
//                 let time_code = TIME_CODES[i];
//                 let efees = event_for_ees.clone();
//                 (
//                     display_str,
//                     "",
//                     Box::new(move |_ctx: &mut Context, _: &Theme| {
//                         efees.lock().unwrap().set_time(time_code.to_string());
//                         println!("{} selected.", display_str);
//                     }) as Box<dyn Callback>,
//                 )
//             })
//             .collect()
//     }
// }
