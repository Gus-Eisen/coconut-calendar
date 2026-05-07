// use std::sync::{Arc, Mutex};
//
// use chrono::{DateTime, Local, Utc};
// use pelican_ui::components::TextInput;
// use pelican_ui::components::button::SecondaryButton;
// use pelican_ui::drawable::{Component, SizedTree};
// use pelican_ui::event::{Event, OnEvent, TickEvent};
// use pelican_ui::interface::general::{Bumper, Content, Header, Page};
// use pelican_ui::interface::navigation::{AppPage, NavigationEvent};
// use pelican_ui::layout::{Offset, Stack};
// use pelican_ui::theme::Theme;
// use pelican_ui::{Context, Request};
//
// use crate::PageFlow;
// use crate::objects::{EventForEES, EventForER, EventRegistry};
// use crate::various_date_selector_screens::day_selector_screen_block::DaySelectorScreen;
// use crate::various_date_selector_screens::month_selector_screen_block::MonthSelectorScreen;
// use crate::various_date_selector_screens::time_selector_screen_block::TimeSelectorScreen;
// use crate::various_date_selector_screens::year_selector_screen_block::YearSelectorScreen;
use crate::Display;

use chk::{Context, Flow, Form, FormItem, FormSubmit, Review, State, Theme};

pub struct EventEditorScreen;

impl EventEditorScreen {
    #![allow(clippy::new_ret_no_self)]
    pub fn new(theme: &Theme) -> Flow {
        let on_submit = Box::new(|_ctx: &mut Context, objects: &Vec<State>| {
            println!("New event: {:?}", objects);
            //TODO: put event-saving logic to event registry here.
        }) as Box<dyn FormSubmit>;

        let getter = |states: &Vec<State>| {
            let title = match states.first() {
                Some(State::Text(t)) => t,
                _ => "(no title)",
            };
            let date = match states.get(1) {
                Some(State::Number(d)) => d,
                _ => "(no date)",
            };
            let time = match states.get(2) {
                Some(State::Number(t)) => t,
                _ => "(no time)",
            };
            vec![
                Display::review("Event Title", title, ""),
                Display::review("Date", date, ""),
                Display::review("Time", time, ""),
            ]
        };

        let form = Form::new(
            theme,
            vec![
                FormItem::text("Event Title"),
                FormItem::number("Date", chk::NumberVariant::Date),
                FormItem::number("Time", chk::NumberVariant::Time),
            ],
            Some(Review::new("New Event Review", getter)),
            None,
            on_submit,
        );

        Flow::from_form(form)
    }
}
// #[derive(Debug, Component, Clone)]
// pub struct EventEditorScreen(
//     Stack,
//     Page,
//     #[skip] Arc<Mutex<EventForEES>>,
//     #[skip] Arc<Mutex<EventRegistry>>,
// );
//
// impl OnEvent for EventEditorScreen {
//     fn on_event(
//         &mut self,
//         _ctx: &mut Context,
//         _sized: &SizedTree,
//         event: Box<dyn Event>,
//     ) -> Vec<Box<dyn Event>> {
//         if event.downcast_ref::<TickEvent>().is_some() {
//             if let Some(input) = self.1.content.find::<TextInput>() {
//                 let val = input.value();
//                 if !val.is_empty() {
//                     self.2.lock().unwrap().event_title = Some(val);
//                 }
//             }
//         }
//         vec![event]
//     }
// }
//
// impl AppPage for EventEditorScreen {}
//
// impl EventEditorScreen {
//     pub fn new(
//         theme: &Theme,
//         year: Option<u16>,
//         month: Option<u8>,
//         day: Option<u8>,
//         event_for_ees: Arc<Mutex<EventForEES>>,
//         event_registry: Arc<Mutex<EventRegistry>>,
//     ) -> Self {
//         {
//             let mut efees = event_for_ees.lock().unwrap();
//             if let (Some(y), Some(m), Some(d)) = (year, month, day) {
//                 efees.set_year((y - 2025) as u8);
//                 efees.set_month(m);
//                 efees.set_day(d.to_string());
//             }
//         }
//
//         let (title_val, year_label, month_label, day_label, time_label) = {
//             let efees = event_for_ees.lock().unwrap();
//             (
//                 efees.event_title.clone(),
//                 efees.year.as_ref().map(|y| y.as_str().to_string()),
//                 efees.month.as_ref().map(|m| m.as_str().to_string()),
//                 efees.day.clone(),
//                 efees.time.clone(),
//             )
//         };
//
//         let event_title = if let Some(ref t) = title_val {
//             TextInput::new(
//                 theme,
//                 None,
//                 Some("Event Title"),
//                 Some(t.as_str()),
//                 None,
//                 None,
//             )
//         } else {
//             TextInput::new(
//                 theme,
//                 None,
//                 Some("Event Title"),
//                 Some("Enter Event Title here"),
//                 Some("Ex.: Strategy meeting with Satoshi"),
//                 None,
//             )
//         };
//
//         let efees_for_year = event_for_ees.clone();
//         let event_reg_for_year = event_registry.clone();
//         let year_str = year_label
//             .as_deref()
//             .unwrap_or("Select year here")
//             .to_string();
//         let year_btn = SecondaryButton::medium(
//             theme,
//             "right",
//             &year_str,
//             None,
//             move |ctx: &mut Context, theme: &Theme| {
//                 let page = Box::new(YearSelectorScreen::new(
//                     theme,
//                     efees_for_year.clone(),
//                     event_reg_for_year.clone(),
//                 ));
//                 ctx.send(Request::event(NavigationEvent::push(PageFlow::new(page))));
//                 println!("year clicked.");
//             },
//         );
//
//         let efees_for_month = event_for_ees.clone();
//         let event_reg_for_month = event_registry.clone();
//         let month_str = month_label
//             .as_deref()
//             .unwrap_or("Select month here")
//             .to_string();
//         let month_btn = SecondaryButton::medium(
//             theme,
//             "right",
//             &month_str,
//             None,
//             move |ctx: &mut Context, theme: &Theme| {
//                 let page = Box::new(MonthSelectorScreen::new(
//                     theme,
//                     efees_for_month.clone(),
//                     event_reg_for_month.clone(),
//                 ));
//                 ctx.send(Request::event(NavigationEvent::push(PageFlow::new(page))));
//                 println!("month clicked.");
//             },
//         );
//
//         let efees_for_day = event_for_ees.clone();
//         let event_reg_for_day = event_registry.clone();
//         let day_str = day_label
//             .as_deref()
//             .unwrap_or("Select day here")
//             .to_string();
//         let day_btn = SecondaryButton::medium(
//             theme,
//             "right",
//             &day_str,
//             None,
//             move |ctx: &mut Context, theme: &Theme| {
//                 let page = Box::new(DaySelectorScreen::new(
//                     theme,
//                     efees_for_day.clone(),
//                     event_reg_for_day.clone(),
//                 ));
//                 ctx.send(Request::event(NavigationEvent::push(PageFlow::new(page))));
//                 println!("day clicked.");
//             },
//         );
//
//         let efees_for_time = event_for_ees.clone();
//         let event_reg_for_time = event_registry.clone();
//         let time_str = time_label
//             .as_deref()
//             .unwrap_or("Select time here")
//             .to_string();
//         let time_btn = SecondaryButton::medium(
//             theme,
//             "right",
//             &time_str,
//             None,
//             move |ctx: &mut Context, theme: &Theme| {
//                 let page = Box::new(TimeSelectorScreen::new(
//                     theme,
//                     efees_for_time.clone(),
//                     event_reg_for_time.clone(),
//                 ));
//                 ctx.send(Request::event(NavigationEvent::push(PageFlow::new(page))));
//                 println!("time clicked.");
//             },
//         );
//
//         let content = Content::new(
//             Offset::Start,
//             vec![
//                 Box::new(event_title),
//                 Box::new(year_btn),
//                 Box::new(month_btn),
//                 Box::new(day_btn),
//                 Box::new(time_btn),
//             ],
//             Box::new(|_| true),
//         );
//
//         let efees_arc = event_for_ees.clone();
//         let event_reg_arc = event_registry.clone();
//         let bumper = Bumper::stack(
//             theme,
//             Some("Save Event"),
//             move |ctx: &mut Context, _: &Theme| {
//                 let mut efees = efees_arc.lock().unwrap();
//                 if efees.all_some() {
//                     let title = efees.get_title().unwrap();
//                     let year = efees.get_year_as_i32().to_string();
//                     let month = efees.get_month_as_u32().to_string();
//                     let day = efees.get_day_as_u32().to_string();
//                     let time = efees.get_time().unwrap();
//                     let fmt_for_datetime = Self::formatter(&year, &month, &day, &time);
//                     println!("{:?}", &fmt_for_datetime);
//                     efees.set_all_none();
//                     drop(efees);
//                     let datetime = DateTime::parse_from_str(&fmt_for_datetime, "%Y %m %d %H%M %z")
//                         .expect("Failed to parse into DateTime")
//                         .with_timezone(&Utc);
//                     println!("{:?}", &datetime);
//                     let event = EventForER::new(title, datetime);
//                     event_reg_arc.lock().unwrap().push(Some(event));
//                     ctx.send(Request::event(NavigationEvent::Reset));
//                     println!("Save Event button clicked.");
//                 }
//             },
//             None,
//         );
//
//         EventEditorScreen(
//             Stack::default(),
//             Page::new(
//                 Header::stack(theme, "New Event", None),
//                 content,
//                 Some(bumper),
//             ),
//             event_for_ees,
//             event_registry,
//         )
//     }
//
//     pub fn formatter(year: &str, month: &str, day: &str, time: &str) -> String {
//         let local_time = Local::now();
//         let tz_string = local_time.format("%z").to_string();
//         format!("{year} {month} {day} {time} {tz_string}")
//     }
// }
