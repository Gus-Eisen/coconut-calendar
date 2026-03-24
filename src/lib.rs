mod day_view_screen;
mod event_editor_screen;
mod objects;
mod various_date_selector_screens;

use std::sync::{Arc, Mutex};

use chrono::{DateTime, Months, NaiveDate};
use chrono::{Datelike, Local};

#![allow(clippy::new_ret_no_self)]
use ramp::prism;

use chk::{
    Bumper, ChkTheme, Color, Context, Display, Flow, Form, FormItem, FormSubmit, Icons, Message, NumberVariant, Offset, OnEvent, PageBuilder, PageType, Profile, Review, Root, RootInfo, Screen, State, Success, Theme
};

use chk::items::{Action, ListItem, TableItem};
use ramp::prism::canvas::{Span, Text};
use ramp::prism::drawable::Component;
use ramp::prism::layout::{Column, Padding};

use crate::day_view_screen::DayViewScreen;
use crate::objects::{EventForEES, EventRegistry};

chk::run! { |_ctx: &mut Context| Calendar }

pub struct Calendar;

impl chk::App for Calendar {
    fn roots(&self, ctx: &mut Context, theme: &Theme) -> Vec<RootInfo> {
        vec![RootInfo::icon(
            ctx,
            theme,
            Icons::Home,
            "Home",
            MonthScreen::new(theme),
        )]
    }

    fn theme(&self) -> ChkTheme {
        ChkTheme::Dark(Color::from_hex("#eb343a", 255))
    }
}

// Define the first screen of the app
#[derive(Debug, Clone)]
pub struct MonthScreen;

impl MonthScreen {
    pub fn new(
        ctx: &mut Context,
        theme: &Theme,
    ) -> PageType {
        if ctx.state().get::<EventForEES>().is_none() {
            let event_for_ees = EventForEES::new(None, None, None, None, None);
            ctx.state().set(event_for_ees);
        }

        if ctx.state().get::<EventRegistry>().is_none() {
            let event_registry = EventRegistry::new(vec![None]);
            ctx.state().set(event_registry);
        }

        let now = Local::now();

        let current_month = now.format("%B").to_string();
        let current_year = now.format("%Y").to_string();
        let current_month_and_year = format!("{} {}", current_month, current_year);

        // let current_month_and_year_for_text = Text::new(
        //     theme,
        //     &current_month_and_year,
        //     TextSize::Md,
        //     TextStyle::Primary,
        //     Align::Left,
        //     None,
        // );
        let current_month_and_year_for_text = Text::new(Span::new(current_month_and_year, 0.5, None, , color, kerning), width, align, max_lines)

        let next_11_months = Self::next_11_months_determiner(&now);

        let next_11_months_1 = Text::new(
            theme,
            next_11_months.first().unwrap(),
            TextSize::Md,
            TextStyle::Secondary,
            Align::Left,
            None,
        );

        let next_11_months_2 = Text::new(
            theme,
            next_11_months.get(1).unwrap(),
            TextSize::Md,
            TextStyle::Secondary,
            Align::Left,
            None,
        );

        let next_11_months_3 = Text::new(
            theme,
            next_11_months.get(2).unwrap(),
            TextSize::Md,
            TextStyle::Secondary,
            Align::Left,
            None,
        );

        let next_11_months_4 = Text::new(
            theme,
            next_11_months.get(3).unwrap(),
            TextSize::Md,
            TextStyle::Secondary,
            Align::Left,
            None,
        );

        let next_11_months_5 = Text::new(
            theme,
            next_11_months.get(4).unwrap(),
            TextSize::Md,
            TextStyle::Secondary,
            Align::Left,
            None,
        );

        let next_11_months_6 = Text::new(
            theme,
            next_11_months.get(5).unwrap(),
            TextSize::Md,
            TextStyle::Secondary,
            Align::Left,
            None,
        );

        let next_11_months_7 = Text::new(
            theme,
            next_11_months.get(6).unwrap(),
            TextSize::Md,
            TextStyle::Secondary,
            Align::Left,
            None,
        );

        let next_11_months_8 = Text::new(
            theme,
            next_11_months.get(7).unwrap(),
            TextSize::Md,
            TextStyle::Secondary,
            Align::Left,
            None,
        );

        let next_11_months_9 = Text::new(
            theme,
            next_11_months.get(8).unwrap(),
            TextSize::Md,
            TextStyle::Secondary,
            Align::Left,
            None,
        );

        let next_11_months_10 = Text::new(
            theme,
            next_11_months.get(9).unwrap(),
            TextSize::Md,
            TextStyle::Secondary,
            Align::Left,
            None,
        );

        let next_11_months_11 = Text::new(
            theme,
            next_11_months.get(10).unwrap(),
            TextSize::Md,
            TextStyle::Secondary,
            Align::Left,
            None,
        );

        let lig_for_1 = ListItemGroup::new(Self::listitem_builder_plus_n(
            theme,
            next_11_months.first().unwrap(),
            registry_snapshot.clone(),
            event_registry.clone(),
            event_for_ees.clone(),
        ));

        let lig_for_2 = ListItemGroup::new(Self::listitem_builder_plus_n(
            theme,
            next_11_months.get(1).unwrap(),
            registry_snapshot.clone(),
            event_registry.clone(),
            event_for_ees.clone(),
        ));

        let lig_for_3 = ListItemGroup::new(Self::listitem_builder_plus_n(
            theme,
            next_11_months.get(2).unwrap(),
            registry_snapshot.clone(),
            event_registry.clone(),
            event_for_ees.clone(),
        ));

        let lig_for_4 = ListItemGroup::new(Self::listitem_builder_plus_n(
            theme,
            next_11_months.get(3).unwrap(),
            registry_snapshot.clone(),
            event_registry.clone(),
            event_for_ees.clone(),
        ));

        let lig_for_5 = ListItemGroup::new(Self::listitem_builder_plus_n(
            theme,
            next_11_months.get(4).unwrap(),
            registry_snapshot.clone(),
            event_registry.clone(),
            event_for_ees.clone(),
        ));

        let lig_for_6 = ListItemGroup::new(Self::listitem_builder_plus_n(
            theme,
            next_11_months.get(5).unwrap(),
            registry_snapshot.clone(),
            event_registry.clone(),
            event_for_ees.clone(),
        ));

        let lig_for_7 = ListItemGroup::new(Self::listitem_builder_plus_n(
            theme,
            next_11_months.get(6).unwrap(),
            registry_snapshot.clone(),
            event_registry.clone(),
            event_for_ees.clone(),
        ));

        let lig_for_8 = ListItemGroup::new(Self::listitem_builder_plus_n(
            theme,
            next_11_months.get(7).unwrap(),
            registry_snapshot.clone(),
            event_registry.clone(),
            event_for_ees.clone(),
        ));

        let lig_for_9 = ListItemGroup::new(Self::listitem_builder_plus_n(
            theme,
            next_11_months.get(8).unwrap(),
            registry_snapshot.clone(),
            event_registry.clone(),
            event_for_ees.clone(),
        ));

        let lig_for_10 = ListItemGroup::new(Self::listitem_builder_plus_n(
            theme,
            next_11_months.get(9).unwrap(),
            registry_snapshot.clone(),
            event_registry.clone(),
            event_for_ees.clone(),
        ));

        let lig_for_11 = ListItemGroup::new(Self::listitem_builder_plus_n(
            theme,
            next_11_months.get(10).unwrap(),
            registry_snapshot.clone(),
            event_registry.clone(),
            event_for_ees.clone(),
        ));

        let header = Header::home(theme, "Calendar", None);

        let listitemgroup_for_cmayft = ListItemGroup::new(Self::listitem_builder(
            theme,
            now,
            registry_snapshot,
            event_registry.clone(),
            event_for_ees.clone(),
        ));

        let content = Content::new(
            Offset::Start,
            vec![
                Box::new(current_month_and_year_for_text),
                Box::new(listitemgroup_for_cmayft),
                Box::new(next_11_months_1),
                Box::new(lig_for_1),
                Box::new(next_11_months_2),
                Box::new(lig_for_2),
                Box::new(next_11_months_3),
                Box::new(lig_for_3),
                Box::new(next_11_months_4),
                Box::new(lig_for_4),
                Box::new(next_11_months_5),
                Box::new(lig_for_5),
                Box::new(next_11_months_6),
                Box::new(lig_for_6),
                Box::new(next_11_months_7),
                Box::new(lig_for_7),
                Box::new(next_11_months_8),
                Box::new(lig_for_8),
                Box::new(next_11_months_9),
                Box::new(lig_for_10),
                Box::new(next_11_months_11),
                Box::new(lig_for_11),
            ],
            Box::new(|_| true),
        );

        Self(
            Column::new(1.0, Offset::Start, prism::layout::Size::Fit, Padding::new(1.0), None),
            Page::new(header, content, None),
        )
    }

    // Creates User's current month.
    fn listitem_builder(
        theme: &Theme,
        now: DateTime<chrono::Local>,
        registry_snapshot: EventRegistry,
        event_registry: Arc<Mutex<EventRegistry>>,
        event_for_ees: Arc<Mutex<EventForEES>>,
    ) -> Vec<ListItem> {
        let range = Self::num_of_days_in_month(now);
        eprintln!("{:?}", now);

        (1..=range)
            .map(|d| {
                let day_of_week = now.with_day(d as u32).unwrap().weekday();
                let day_of_month = d as u32;
                let month = now.month();
                let year = now.year();
                let events_with_days = registry_snapshot.days_with_events(year, month);
                let has_event = events_with_days.contains(&(d as u32));
                let first_event_title = has_event
                    .then(|| {
                        registry_snapshot
                            .events_for_day(year, month, d as u32)
                            .into_iter()
                            .next()
                            .map(|e| TitleSubtitle::new(e.title(), None))
                    })
                    .flatten();

                let reg_clone = event_registry.clone();
                let ees_clone = event_for_ees.clone();
                let flair = if d == now.day() as i32 {
                    Some(("notification", Color::from_hex("#FF0000", 255)))
                } else {
                    None
                };

                ListItem::new(
                    theme,
                    None,
                    ListItemInfoLeft::new(
                        //TODO: create colored current day here.
                        &d.to_string(),
                        Some(&day_of_week.to_string()),
                        None,
                        flair,
                    ),
                    first_event_title,
                    None,
                    None,
                    move |ctx: &mut Context, theme: &Theme| {
                        let page = Box::new(
                            DayViewScreen::new(
                                theme,
                                year,
                                month,
                                day_of_month,
                                reg_clone.clone(),
                                ees_clone.clone(),
                            )
                            .unwrap(),
                        );
                        ctx.send(Request::event(NavigationEvent::push(PageFlow::new(page))));
                    },
                )
            })
            .collect()
    }

    fn next_11_months_determiner(now: &DateTime<Local>) -> Vec<String> {
        (1..=11)
            .map(|i| {
                let future = *now + Months::new(i);
                future.format("%B %Y").to_string()
            })
            .collect()
    }

    // Creates next 11 months after current month.
    pub fn listitem_builder_plus_n(
        theme: &Theme,
        chosen_month_and_year: &str,
        registry_snapshot: EventRegistry,
        event_registry: Arc<Mutex<EventRegistry>>,
        event_for_ees: Arc<Mutex<EventForEES>>,
    ) -> Vec<ListItem> {
        let month_and_year: Vec<&str> = chosen_month_and_year.split_whitespace().collect();
        let range = Self::num_of_days_in_month_for_vec_str(&month_and_year);

        (1..=range)
            .map(|d| {
                let day_of_month = d as u32;
                let year = month_and_year.get(1).unwrap().parse::<i32>().unwrap();
                let month = match *month_and_year.first().unwrap() {
                    "January" => 1,
                    "February" => 2,
                    "March" => 3,
                    "April" => 4,
                    "May" => 5,
                    "June" => 6,
                    "July" => 7,
                    "August" => 8,
                    "September" => 9,
                    "October" => 10,
                    "November" => 11,
                    "December" => 12,
                    _ => panic!("month in listitem_builder_plus_n is borked."),
                };
                let events_with_days = registry_snapshot.days_with_events(year, month);
                let has_event = events_with_days.contains(&(d as u32));
                let first_event_title = has_event
                    .then(|| {
                        registry_snapshot
                            .events_for_day(year, month, d as u32)
                            .into_iter()
                            .next()
                            .map(|e| TitleSubtitle::new(e.title(), None))
                    })
                    .flatten();

                let reg_clone = event_registry.clone();
                let ees_clone = event_for_ees.clone();
                let day_of_week = NaiveDate::from_ymd_opt(year, month, d as u32)
                    .unwrap()
                    .weekday();
                ListItem::new(
                    theme,
                    None,
                    ListItemInfoLeft::new(
                        &d.to_string(),
                        Some(&day_of_week.to_string()),
                        None,
                        None,
                    ),
                    first_event_title,
                    None,
                    None,
                    move |ctx: &mut Context, theme: &Theme| {
                        let page = Box::new(
                            DayViewScreen::new(
                                theme,
                                year,
                                month,
                                day_of_month,
                                reg_clone.clone(),
                                ees_clone.clone(),
                            )
                            .unwrap(),
                        );
                        ctx.send(Request::event(NavigationEvent::push(PageFlow::new(page))));
                    },
                )
            })
            .collect()
    }

    fn is_leap_year(year: i32) -> bool {
        (year % 4 == 0) && (year % 100 != 0 || year % 400 == 0)
    }

    pub fn num_of_days_in_month(now: chrono::DateTime<chrono::Local>) -> i32 {
        match now.month() {
            1 => 31,
            2 => {
                if Self::is_leap_year(now.year()) {
                    29
                } else {
                    28
                }
            }
            3 => 31,
            4 => 30,
            5 => 31,
            6 => 30,
            7 => 31,
            8 => 31,
            9 => 30,
            10 => 31,
            11 => 30,
            12 => 31,
            _ => panic!("Something went wrong with num_of_days_in_month()."),
        }
    }

    //HACK: Having two functions that do the same thing (fn num_of_days_in_month) is stupid.
    //Combine / Refactor.
    fn num_of_days_in_month_for_vec_str(month_and_year: &[&str]) -> i32 {
        match *month_and_year.first().unwrap() {
            "January" => 31,
            "February" => {
                if Self::is_leap_year(month_and_year.get(1).unwrap().parse::<i32>().unwrap()) {
                    29
                } else {
                    28
                }
            }
            "March" => 31,
            "April" => 30,
            "May" => 31,
            "June" => 30,
            "July" => 31,
            "August" => 31,
            "September" => 30,
            "October" => 31,
            "November" => 30,
            "December" => 31,
            _ => panic!("Something went wrong with month_and_year."),
        }
    }
}

struct Calendar {
    canvas: Canvas,
    atlas: Atlas,
    ui: Interface,
    sized: SizedTree,
    prism_ctx: Context,
    instance: Instance,
    cursor: (f32, f32),
}

impl Services for Calendar {}

impl Application for Calendar {
    async fn new(ctx: &mut Context) -> Self {
        let (w, h) = ctx.window.size;
        let canvas = Canvas::new(ctx.window.handle.clone(), w, h).await;
        let atlas = Atlas::default();
        let (prism_ctx, receiver) = Context::new();
        let instance = Instance::new(receiver);
        let ui = crate::app();
        let req = ui.request_size();
        let sized = ui.build((w as f32, h as f32), req);
        CalendarApp {
            canvas,
            atlas,
            ui,
            sized,
            prism_ctx,
            instance,
            cursor: (0.0, 0.0),
        }
    }

    async fn on_event(&mut self, ctx: &mut maverick_os::Context, event: WinEvent) {
        let (w, h) = ctx.window.size;
        let sf = ctx.window.scale_factor as f32;
        let size = (w as f32, h as f32);

        match event {
            WinEvent::Input(Input::CursorMoved { position, .. }) => {
                self.cursor = (position.0 as f32 / sf, position.1 as f32 / sf);
                let e = Box::new(MouseEvent {
                    position: Some(self.cursor),
                    state: MouseState::Moved,
                });
                self.ui.event(&mut self.prism_ctx, &self.sized, e);
            }
            WinEvent::Input(Input::Mouse {
                state: ElementState::Pressed,
                ..
            }) => {
                let e = Box::new(MouseEvent {
                    position: Some(self.cursor),
                    state: MouseState::Pressed,
                });
                self.ui.event(&mut self.prism_ctx, &self.sized, e);
            }
            WinEvent::Input(Input::Mouse {
                state: ElementState::Released,
                ..
            }) => {
                let e = Box::new(MouseEvent {
                    position: Some(self.cursor),
                    state: MouseState::Released,
                });
                self.ui.event(&mut self.prism_ctx, &self.sized, e);
            }
            WinEvent::Input(Input::MouseWheel { delta, .. }) => {
                let (dx, dy) = match delta {
                    MouseScrollDelta::LineDelta(x, y) => (x * 20.0, y * 20.0),
                    MouseScrollDelta::PixelDelta(p) => (p.x as f32, p.y as f32),
                };
                let e = Box::new(MouseEvent {
                    position: Some(self.cursor),
                    state: MouseState::Scroll(dx, dy),
                });
                self.ui.event(&mut self.prism_ctx, &self.sized, e);
            }
            WinEvent::Input(Input::Keyboard {
                event:
                    KeyEvent {
                        logical_key,
                        state,
                        repeat,
                        ..
                    },
                ..
            }) => {
                let key = match &logical_key {
                    maverick_os::window::Key::Named(WinitNamedKey::Enter) => {
                        Key::Named(NamedKey::Enter)
                    }
                    maverick_os::window::Key::Named(WinitNamedKey::Tab) => {
                        Key::Named(NamedKey::Tab)
                    }
                    maverick_os::window::Key::Named(WinitNamedKey::Space) => {
                        Key::Named(NamedKey::Space)
                    }
                    maverick_os::window::Key::Named(WinitNamedKey::ArrowUp) => {
                        Key::Named(NamedKey::ArrowUp)
                    }
                    maverick_os::window::Key::Named(WinitNamedKey::ArrowDown) => {
                        Key::Named(NamedKey::ArrowDown)
                    }
                    maverick_os::window::Key::Named(WinitNamedKey::ArrowLeft) => {
                        Key::Named(NamedKey::ArrowLeft)
                    }
                    maverick_os::window::Key::Named(WinitNamedKey::ArrowRight) => {
                        Key::Named(NamedKey::ArrowRight)
                    }
                    maverick_os::window::Key::Named(WinitNamedKey::Backspace) => {
                        Key::Named(NamedKey::Delete)
                    }
                    maverick_os::window::Key::Named(WinitNamedKey::Delete) => {
                        Key::Named(NamedKey::Delete)
                    }
                    maverick_os::window::Key::Character(s) => Key::Character(s.to_string()),
                    _ => return,
                };
                let kb_state = match (state, repeat) {
                    (ElementState::Released, _) => KeyboardState::Released,
                    (ElementState::Pressed, true) => KeyboardState::Repeated,
                    (ElementState::Pressed, false) => KeyboardState::Pressed,
                };
                self.ui.event(
                    &mut self.prism_ctx,
                    &self.sized,
                    Box::new(KeyboardEvent {
                        key,
                        state: kb_state,
                    }),
                );
            }
            WinEvent::Lifetime(Lifetime::Resized) => {
                self.canvas.resize(Option::<Arc<Window>>::None, w, h);
                let req = self.ui.request_size();
                self.sized = self.ui.build(size, req);
            }
            WinEvent::Lifetime(Lifetime::Draw) => {
                self.instance.handle_requests();
                self.instance.tick(&mut self.prism_ctx);
                while let Some(e) = self.instance.events.pop_front() {
                    self.ui.event(&mut self.prism_ctx, &self.sized, e);
                }
                self.ui
                    .event(&mut self.prism_ctx, &self.sized, Box::new(TickEvent));
                let req = self.ui.request_size();
                self.sized = self.ui.build(size, req);
                let items = self
                    .ui
                    .draw(&self.sized, (0.0, 0.0), (0.0, 0.0, size.0, size.1));
                self.canvas.draw(&mut self.atlas, items);
            }
            _ => {}
        }
    }
}

maverick_os::start!(CalendarApp);
