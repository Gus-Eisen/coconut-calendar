// use std::sync::{Arc, Mutex};
//
// use chrono::{DateTime, Local, Utc};
// use pelican_ui::canvas::Align;
// use pelican_ui::components::text::{Text, TextSize, TextStyle};
// use pelican_ui::drawable::Component;
// use pelican_ui::event::OnEvent;
// use pelican_ui::interface::general::{Bumper, Content, Header, Page};
// use pelican_ui::interface::navigation::{AppPage, NavigationEvent};
// use pelican_ui::layout::{Offset, Stack};
// use pelican_ui::theme::Theme;
// use pelican_ui::{Context, Request};
//
// use crate::PageFlow;
// use crate::event_editor_screen::EventEditorScreen;
// use crate::objects::{EventForEES, EventRegistry};
//
// #[derive(Debug, Component, Clone)]
// pub struct DayViewScreen(Stack, Page);
//
// impl OnEvent for DayViewScreen {}
//
// impl AppPage for DayViewScreen {}
//
// impl DayViewScreen {
//     pub fn new(
//         theme: &Theme,
//         year: i32,
//         month: u32,
//         day: u32,
//         event_registry: Arc<Mutex<EventRegistry>>,
//         event_for_ees: Arc<Mutex<EventForEES>>,
//     ) -> Result<Self, String> {
//         let events: Vec<(String, DateTime<Utc>)> = {
//             let registry = event_registry.lock().unwrap();
//             registry
//                 .events_for_day(year, month, day)
//                 .iter()
//                 .map(|e| (e.title().to_string(), e.datetime()))
//                 .collect()
//         };
//
//         for (title, datetime) in &events {
//             println!("Event: '{}' on {}", title, datetime);
//         }
//
//         let month_for_header = Self::month_name(month).unwrap();
//         let title = format!("{} {}, {}", month_for_header, day, year);
//         println!("DEBUG HEADER: '{}'", title);
//         let header = Header::stack(theme, &title, None);
//
//         let content_items: Vec<Box<dyn pelican_ui::drawable::Drawable>> =
//             Self::vec_of_text(theme, events)
//                 .into_iter()
//                 .map(|t| Box::new(t) as Box<dyn pelican_ui::drawable::Drawable>)
//                 .collect();
//
//         let ees_for_bumper = event_for_ees.clone();
//         let reg_for_bumper = event_registry.clone();
//         let bumper = Bumper::stack(
//             theme,
//             Some("Create New Event"),
//             move |ctx: &mut Context, theme: &Theme| {
//                 let page = Box::new(EventEditorScreen::new(
//                     theme,
//                     Some(year as u16),
//                     Some(month as u8),
//                     Some(day as u8),
//                     ees_for_bumper.clone(),
//                     reg_for_bumper.clone(),
//                 ));
//                 ctx.send(Request::event(NavigationEvent::push(PageFlow::new(page))));
//             },
//             None,
//         );
//
//         let content = Content::new(Offset::Start, content_items, Box::new(|_| true));
//
//         Ok(Self(Stack::default(), Page::new(header, content, Some(bumper))))
//     }
//
//     pub fn month_name(month: u32) -> Option<&'static str> {
//         const MONTHS: [&str; 12] = [
//             "January",
//             "February",
//             "March",
//             "April",
//             "May",
//             "June",
//             "July",
//             "August",
//             "September",
//             "October",
//             "November",
//             "December",
//         ];
//         MONTHS.get((month - 1) as usize).copied()
//     }
//
//     pub fn vec_of_text(theme: &Theme, events: Vec<(String, DateTime<Utc>)>) -> Vec<Text> {
//         events
//             .into_iter()
//             .map(|(title, datetime)| {
//                 let local_datetime = datetime.with_timezone(&Local);
//                 let label = format!("{} - {}", local_datetime.format("%I:%M %p"), title);
//                 Text::new(theme, &label, TextSize::Md, TextStyle::Primary, Align::Left, None)
//             })
//             .collect()
//     }
// }
