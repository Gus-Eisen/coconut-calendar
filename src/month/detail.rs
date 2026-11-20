use chk::{
    Action, ActionItem, Bumper, Context, Display, Flow, Form, FormAction, FormComplete, FormItem,
    FormSubmit, FormValidState, Icons, Offset, Page, PageType, State, TableItem, Theme, Watch,
};
use chrono::NaiveDate;
use chrono_tz::Tz;
use ramp::prism::air::Instance;

use crate::{Calendar, Category, Event, EventId, Exception, Override, Timing};

use super::DayScreen;
use super::schedule::{ChangeRepeat, ChangeTime};

/// One event's detail page and the flows that edit it.
pub struct EventScreen;
impl EventScreen {
    /// Rebuilds after every change to the calendar, so it always shows the latest values. `date`
    /// is the original date of the tapped occurrence, and the page shows that occurrence's own
    /// values. If an edit to the series means there's no longer an occurrence on `date`, it
    /// shows the first one.
    pub fn flow(calendar: Instance<Calendar>, id: EventId, date: NaiveDate) -> Flow {
        Flow::new(vec![Page::watching(
            Watch::contract(calendar),
            move |_ctx: &mut Context, theme: &Theme, calendar: Instance<Calendar>| {
                let Some(series) = calendar.pending().event(id).cloned() else {
                    return Self::gone("This event was deleted.");
                };
                if series.exception(date) == Some(&Exception::Cancelled) {
                    return Self::gone("This occurrence was cancelled.");
                }
                let start = series.timing().start_date();
                let date = match series.series_timing(date) {
                    Some(_) => date,
                    None => series.recurrence().and_then(|rule| rule.first(start)).unwrap_or(start),
                };
                let event = series.effective(date).unwrap_or_else(|| series.clone());
                let changed = series.changes(date).is_some();
                let zone = Timing::device_zone();

                let series_time = series.series_timing(date)
                    .filter(|timing| changed && timing != event.timing())
                    .map(|timing| ("Series time", Self::when(&timing, zone)));
                let repeat = match series.recurrence() {
                    Some(recurrence) => ChangeRepeat::rows(Some(recurrence), start),
                    None => vec![],
                };
                let rows = [
                    [Some(("Time", Self::when(event.timing(), zone))), series_time]
                        .into_iter().flatten().collect(),
                    repeat,
                    [
                        Some(("Category", event.category().label().to_string())),
                        event.location().map(|location| ("Location", location.to_string())),
                        event.info().map(|info| ("Info", info.to_string())),
                        Some(("Private", if event.private() { "Yes" } else { "No" }.to_string())),
                        event.video_link().map(|link| ("Video link", link.to_string())),
                    ].into_iter().flatten().collect(),
                ];

                let header_edit = Self::edit(theme, &calendar, id, date, &series);
                let edit = Action::flow(header_edit.clone());
                let time = Self::scoped(
                    theme, &series, date,
                    Self::open({
                        let (calendar, series) = (calendar.clone(), series.clone());
                        move |theme: &Theme| {
                            let scope = Scope::Occurrence(date);
                            ChangeTime::picker(theme, calendar.clone(), id, &series, scope)
                        }
                    }),
                    Self::open({
                        let (calendar, series) = (calendar.clone(), series.clone());
                        move |theme: &Theme| {
                            ChangeTime::picker(theme, calendar.clone(), id, &series, Scope::Series)
                        }
                    }),
                );
                let forms = [
                    ("Change repeat", ChangeRepeat::form(theme, calendar.clone(), id, &series)),
                    ("Change category", Self::category(theme, calendar.clone(), id, &series)),
                    ("Change privacy", Self::privacy(theme, calendar.clone(), id, &series)),
                ];
                let mut restoring = calendar.clone();
                let restore = Action::custom(move |_ctx: &mut Context, _theme: &Theme| {
                    Calendar::restore_occurrence(&mut restoring, id, date)
                });
                let actions = [
                    ActionItem::new(edit, "Edit details", Icons::Edit),
                    ActionItem::new(time, "Change time", Icons::Edit),
                ]
                    .into_iter()
                    .chain(forms.into_iter().map(|(label, form)| {
                        ActionItem::new(Action::flow(Flow::from_form(form)), label, Icons::Edit)
                    }))
                    .chain(changed.then(|| ActionItem::new(restore, "Restore", Icons::Back)))
                    .chain([ActionItem::new(
                        Self::delete(theme, &calendar, id, date, &series), "Delete", Icons::Delete,
                    )])
                    .collect();
                let changed_line = changed.then(|| {
                    Display::instructions("This occurrence was changed.")
                });
                let table = Display::table(
                    "Event",
                    rows.into_iter().flatten()
                        .map(|(label, value)| TableItem::new(label, &value)).collect(),
                    Some(actions),
                );

                let attachments = event.attachments().to_vec();
                let add_link = Self::scoped(
                    theme, &series, date,
                    Self::open({
                        let (calendar, attachments) = (calendar.clone(), attachments.clone());
                        move |theme: &Theme| {
                            let (mut calendar, mut links) = (calendar.clone(), attachments.clone());
                            Self::add_link(theme, move |link: String| {
                                if !links.contains(&link) { links.push(link); }
                                let links = links.clone();
                                Self::change(&mut calendar, id, date, |changes, series| {
                                    changes.set_attachments(links, series.attachments())
                                })
                            })
                        }
                    }),
                    Self::open({
                        let calendar = calendar.clone();
                        move |theme: &Theme| {
                            let mut calendar = calendar.clone();
                            Self::add_link(theme, move |link: String| {
                                Calendar::add_attachment(&mut calendar, id, link)
                            })
                        }
                    }),
                );
                let attachments_list = Display::data_list(
                    "Attachments",
                    attachments.is_empty().then_some("No links yet."),
                    vec![],
                    Some(vec![ActionItem::new(add_link, "Add link", Icons::Link)]),
                );
                let links = attachments.iter().map(|link| {
                    let this = {
                        let mut calendar = calendar.clone();
                        let kept = attachments.iter().filter(|kept| *kept != link)
                            .cloned().collect::<Vec<_>>();
                        move |_ctx: &mut Context, _theme: &Theme| {
                            let kept = kept.clone();
                            Self::change(&mut calendar, id, date, |changes, series| {
                                changes.set_attachments(kept, series.attachments())
                            })
                        }
                    };
                    // If the occurrence has its own list, the link goes from that too, or it
                    // would stay on screen. One that follows the series needs nothing more.
                    let whole = {
                        let (mut calendar, removed) = (calendar.clone(), link.clone());
                        let mut this = this.clone();
                        move |ctx: &mut Context, theme: &Theme| {
                            Calendar::remove_attachment(&mut calendar, id, removed.clone());
                            let own = calendar.pending().event(id)
                                .and_then(|series| series.changes(date))
                                .and_then(Override::attachments).is_some();
                            if own { this(ctx, theme); }
                        }
                    };
                    let remove = Self::scoped(theme, &series, date, this, whole);
                    let delete = ActionItem::new(remove, "Delete", Icons::Delete);
                    Display::cta(link, None, "Link", vec![delete])
                });

                let header = Box::new(move |_ctx: &mut Context, _theme: &Theme| {
                    header_edit.clone()
                });
                PageType::display(
                    event.title(),
                    changed_line.into_iter().chain([table, attachments_list]).chain(links)
                        .collect(),
                    Some((Icons::Edit, header)),
                    Bumper::None,
                    Offset::Start,
                )
            },
        )])
    }

    /// The page once there's nothing left to show.
    fn gone(message: &str) -> PageType {
        let message = Display::instructions(message);
        PageType::display("Event", vec![message], None, Bumper::None, Offset::Center)
    }

    /// The detail page's time in `zone`: "14:00–15:00, Tue 3 Nov", or "All day, Tue 3 Nov".
    pub fn when(timing: &Timing, zone: Tz) -> String {
        let day = |date: NaiveDate| date.format("%a %-d %b").to_string();
        match *timing {
            Timing::AllDay { first_day, last_day } if first_day == last_day => {
                format!("All day, {}", day(first_day))
            }
            Timing::AllDay { first_day, last_day } => {
                format!("All day, {} – {}", day(first_day), day(last_day))
            }
            Timing::Timed { .. } => {
                // `apply` stores only valid timings and `occurrence` returns only valid ones, so
                // `in_zone` can't panic.
                let (start, _) = timing.in_zone(zone).unwrap();
                let time = DayScreen::time_label(timing, start.date(), zone);
                format!("{time}, {}", day(start.date()))
            }
        }
    }

    /// Ends the current flow and shows the page under it: the detail page, or the day screen
    /// after Delete.
    pub fn back() -> FormComplete { FormComplete::Return(Box::new(|_: &mut Context, _: &Theme| {})) }

    /// A callback that opens the form `make` builds. It runs after the scope step has closed,
    /// so the form opens on top of the detail page.
    fn open(
        make: impl Fn(&Theme) -> Form + Clone + 'static,
    ) -> impl FnMut(&mut Context, &Theme) + Clone + 'static {
        move |ctx: &mut Context, theme: &Theme| {
            Flow::from_form(make(theme)).build(ctx, theme)(ctx, theme)
        }
    }

    /// On a repeating event, the "Which Events?" step (`scope`). An event that doesn't repeat
    /// runs `whole` straight away, as before step 3b.
    fn scoped(
        theme: &Theme, series: &Event, date: NaiveDate,
        this: impl FnMut(&mut Context, &Theme) + Clone + 'static,
        whole: impl FnMut(&mut Context, &Theme) + Clone + 'static,
    ) -> Action {
        match series.recurrence() {
            Some(_) => Action::flow(Self::scope(theme, date, this, whole)),
            None => Action::custom(whole),
        }
    }

    /// The "Which Events?" step: it runs `this` (This occurrence) or `whole` (Whole series) once
    /// it closes. chk always selects the first option, so This occurrence is first.
    fn scope(
        theme: &Theme, date: NaiveDate,
        this: impl FnMut(&mut Context, &Theme) + Clone + 'static,
        whole: impl FnMut(&mut Context, &Theme) + Clone + 'static,
    ) -> Flow {
        let submit = Box::new(move |_ctx: &mut Context, states: &Vec<State>| {
            let [State::Enumerator(index)] = states.as_slice() else { unreachable!() };
            match index {
                0 => FormComplete::Return(Box::new(this.clone())),
                _ => FormComplete::Return(Box::new(whole.clone())),
            }
        }) as Box<dyn FormSubmit>;
        let only = format!("Only {}", date.format("%a %-d %b"));
        let options =
            vec![("This occurrence", only.as_str()), ("Whole series", "Every occurrence")];
        let scope = FormItem::enumerator("Which Events?", options);
        Flow::from_form(Form::flow(theme, vec![scope], None, None, submit))
    }

    /// Sends the occurrence's changes with `edit` applied, starting from its current ones.
    /// `edit` also gets the series, whose values mean "same as the series". If nothing is left
    /// that differs, it restores the occurrence instead. A cancelled occurrence is left alone.
    pub fn change(
        calendar: &mut Instance<Calendar>, id: EventId, date: NaiveDate,
        edit: impl FnOnce(&mut Override, &Event),
    ) {
        let Some(series) = calendar.pending().event(id).cloned() else { return };
        let current = match series.exception(date) {
            Some(Exception::Cancelled) => return,
            Some(Exception::Changed(changes)) => Some(changes.clone()),
            None => None,
        };
        let mut changes = current.clone().unwrap_or_default();
        edit(&mut changes, &series);
        match changes.is_empty() {
            true if current.is_some() => Calendar::restore_occurrence(calendar, id, date),
            true => {}
            false if current.as_ref() != Some(&changes) => {
                Calendar::change_occurrence(calendar, id, date, changes)
            }
            false => {}
        }
    }

    /// Edit details, as a flow because the header's edit icon opens it too: the scope step
    /// first on a repeating event.
    fn edit(
        theme: &Theme, calendar: &Instance<Calendar>, id: EventId, date: NaiveDate, series: &Event,
    ) -> Flow {
        let whole = {
            let (calendar, series) = (calendar.clone(), series.clone());
            Self::open(move |_theme: &Theme| Self::edit_details(calendar.clone(), id, &series))
        };
        let this = {
            let (calendar, series) = (calendar.clone(), series.clone());
            Self::open(move |_theme: &Theme| {
                Self::edit_occurrence(calendar.clone(), id, date, &series)
            })
        };
        match series.recurrence() {
            Some(_) => Self::scope(theme, date, this, whole),
            None => Flow::from_form(Self::edit_details(calendar.clone(), id, series)),
        }
    }

    /// Delete. Whole series deletes the event at once, as before; This occurrence opens the
    /// Cancel Occurrence review first, because cancelling can't be undone. Both end on the day
    /// screen.
    fn delete(
        theme: &Theme, calendar: &Instance<Calendar>, id: EventId, date: NaiveDate, series: &Event,
    ) -> Action {
        let whole = {
            let mut calendar = calendar.clone();
            move |ctx: &mut Context, theme: &Theme| {
                Calendar::delete_event(&mut calendar, id);
                Self::back().run(ctx, theme);
            }
        };
        let this = {
            let (calendar, series) = (calendar.clone(), series.clone());
            move |ctx: &mut Context, theme: &Theme| {
                let flow = Self::cancel(calendar.clone(), id, date, &series);
                flow.clone().build(ctx, theme)(ctx, theme)
            }
        };
        Self::scoped(theme, series, date, this, whole)
    }

    /// The Cancel Occurrence review: the occurrence's time and a warning. Continue cancels it
    /// and returns to the day screen.
    fn cancel(mut calendar: Instance<Calendar>, id: EventId, date: NaiveDate, series: &Event) -> Flow {
        let timing = series.occurrence(date).unwrap_or(*series.timing());
        let when = Self::when(&timing, Timing::device_zone());
        let title = series.effective(date)
            .map(|event| event.title().to_string()).unwrap_or_default();
        let review = move |_states: &Vec<State>| {
            let rows = vec![TableItem::new("Title", &title), TableItem::new("Time", &when)];
            vec![
                Display::table("Occurrence", rows, None),
                Display::instructions("This can't be undone."),
            ]
        };
        let submit = Box::new(move |_ctx: &mut Context, _states: &Vec<State>| {
            Calendar::cancel_occurrence(&mut calendar, id, date);
            // Closes the review, then the detail page under it.
            FormComplete::Return(Box::new(|ctx: &mut Context, theme: &Theme| {
                Self::back().run(ctx, theme)
            }))
        }) as Box<dyn FormSubmit>;
        let page = PageType::review("Cancel Occurrence", Box::new(review), submit);
        Flow::new(vec![Page::Static(Box::new(page))])
    }

    /// Save sends one message per field that changed, comparing after trimming.
    fn edit_details(mut calendar: Instance<Calendar>, id: EventId, event: &Event) -> Form {
        let save = Box::new(move |_ctx: &mut Context, states: &Vec<State>| {
            let [State::Text(title), State::Text(location), State::Text(info), State::Text(link)] =
                states.as_slice() else { unreachable!() };
            // The live event, not the one the page opened with, so a second Save on the same page
            // only sends what changed since the first.
            let Some(event) = calendar.pending().event(id).cloned() else {
                return FormComplete::None;
            };
            if let Some(title) = Event::optional(title) && title != event.title() {
                Calendar::set_title(&mut calendar, id, title);
            }
            let location = Event::optional(location);
            if location.as_deref() != event.location() {
                Calendar::set_location(&mut calendar, id, location);
            }
            let info = Event::optional(info);
            if info.as_deref() != event.info() { Calendar::set_info(&mut calendar, id, info); }
            let link = Event::optional(link);
            if link.as_deref() != event.video_link() {
                Calendar::set_video_link(&mut calendar, id, link);
            }
            FormComplete::None
        }) as Box<dyn FormSubmit>;
        Self::details_page(event, save)
    }

    /// Edit details for one occurrence: filled with its own values. Save sends them as one
    /// `change_occurrence`; a field equal to the series' is stored as "same as the series".
    fn edit_occurrence(mut calendar: Instance<Calendar>, id: EventId, date: NaiveDate, series: &Event) -> Form {
        let event = series.effective(date).unwrap_or_else(|| series.clone());
        let save = Box::new(move |_ctx: &mut Context, states: &Vec<State>| {
            let [State::Text(title), State::Text(location), State::Text(info), State::Text(link)] =
                states.as_slice() else { unreachable!() };
            let Some(title) = Event::optional(title) else { return FormComplete::None };
            let (location, info, link) =
                (Event::optional(location), Event::optional(info), Event::optional(link));
            Self::change(&mut calendar, id, date, |changes, series| {
                changes.set_title(title, series.title());
                changes.set_location(location, series.location());
                changes.set_info(info, series.info());
                changes.set_video_link(link, series.video_link());
            });
            FormComplete::None
        }) as Box<dyn FormSubmit>;
        Self::details_page(&event, save)
    }

    /// The Edit details page, filled with `event`'s values.
    fn details_page(event: &Event, save: Box<dyn FormSubmit>) -> Form {
        let optional = |label: &str, value: Option<&str>| {
            FormItem::text_with_preset(
                label, value.unwrap_or(""), None,
                |_ctx: &mut Context, _value: String| FormValidState::Valid,
            )
        };
        let items = vec![
            FormItem::text_with_preset(
                "Title", event.title(), None,
                |_ctx: &mut Context, title: String| match title.trim().is_empty() {
                    true => FormValidState::Invalid,
                    false => FormValidState::Valid,
                },
            ),
            optional("Location", event.location()),
            optional("Info", event.info()),
            optional("Video link", event.video_link()),
        ];
        Form::page("Edit Event", items, vec![], save)
    }

    fn category(theme: &Theme, mut calendar: Instance<Calendar>, id: EventId, event: &Event) -> Form {
        let categories = Category::options();
        // `options` lists every category, so `position` always finds it.
        let current = categories.iter().position(|category| *category == event.category()).unwrap();
        Self::choice(theme, "Category", categories.map(Category::label), current, move |index| {
            Calendar::set_category(&mut calendar, id, categories[index])
        })
    }

    fn privacy(theme: &Theme, mut calendar: Instance<Calendar>, id: EventId, event: &Event) -> Form {
        let current = usize::from(event.private());
        Self::choice(theme, "Privacy", ["Not private", "Private"], current, move |index| {
            Calendar::set_private(&mut calendar, id, index == 1)
        })
    }

    /// A one-step radio. chk always selects the first option, so the event's value is marked
    /// "Current" instead. Picking it again sends nothing.
    fn choice(
        theme: &Theme, title: &str, labels: [&str; 2], current: usize,
        mut pick: impl FnMut(usize) + Clone + 'static,
    ) -> Form {
        let submit = Box::new(move |_ctx: &mut Context, states: &Vec<State>| {
            let [State::Enumerator(index)] = states.as_slice() else { unreachable!() };
            if *index != current { pick(*index); }
            Self::back()
        }) as Box<dyn FormSubmit>;

        let options = labels.iter().enumerate()
            .map(|(index, label)| (*label, if index == current { "Current" } else { "" }))
            .collect();
        Form::flow(theme, vec![FormItem::enumerator(title, options)], None, None, submit)
    }

    /// Links are plain text: anything that isn't blank is accepted. `save` gets it trimmed.
    fn add_link(theme: &Theme, mut save: impl FnMut(String) + Clone + 'static) -> Form {
        let submit = Box::new(move |_ctx: &mut Context, states: &Vec<State>| {
            let [State::Text(link)] = states.as_slice() else { unreachable!() };
            if let Some(link) = Event::optional(link) { save(link); }
            Self::back()
        }) as Box<dyn FormSubmit>;

        let link = FormItem::text(
            "Attachment", "Link", Some(vec![FormAction::Paste]),
            |_ctx: &mut Context, link: String| {
                if link.trim().is_empty() { FormValidState::Invalid } else { FormValidState::Valid }
            },
        );
        Form::flow(theme, vec![link], None, None, submit)
    }
}

/// What an edit applies to: the whole series, or one occurrence by its original date.
#[derive(Clone, Copy)]
pub enum Scope {
    Series,
    Occurrence(NaiveDate),
}
