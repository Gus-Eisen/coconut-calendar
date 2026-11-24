use chk::{
    Action, ActionItem, Bumper, Context, Display, Flow, Form, FormComplete, FormItem, FormSubmit,
    FormValidState, Icons, Offset, Page, PageType, State, TableItem, Theme, Watch,
};
use chrono::NaiveDate;
use chrono_tz::Tz;
use ramp::prism::air::Instance;

use crate::{Calendar, Category, Event, EventId, Exception, Override, Timing};

use super::{DayScreen, PeopleScreen};
use super::schedule::{ChangeRepeat, ChangeTime};

/// What an edit applies to: the whole series, or one occurrence by its original date.
#[derive(Clone, Copy)]
pub enum Scope {
    Series,
    Occurrence(NaiveDate),
}

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
            move |ctx: &mut Context, theme: &Theme, calendar: Instance<Calendar>| {
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

                let header_edit = Self::edit(theme, &calendar, id, date, &series);
                let width = ctx.window.area().size.0;
                let actions =
                    Self::actions(theme, &calendar, id, date, &series, header_edit.clone(), width);
                let people = [
                    ("Created by", series.created_by()),
                    ("Last changed by", series.last_changed_by()),
                ]
                    .map(|(label, name)| {
                        let (_, username) = PeopleScreen::person(ctx, name);
                        TableItem::new(label, &DayScreen::fit_label(theme, width, &username, true))
                    });
                let rows = Self::rows(&series, &event, date, changed, theme, width).into_iter()
                    .chain(people).collect();
                let table = Display::table("Event", rows, Some(actions));
                let changed_line = changed.then(|| {
                    Display::instructions("This occurrence was changed.")
                });
                let header = Box::new(move |_ctx: &mut Context, _theme: &Theme| {
                    header_edit.clone()
                });
                PageType::display(
                    event.title(),
                    changed_line.into_iter().chain([table]).collect(),
                    Some((Icons::Edit, header)),
                    Bumper::None,
                    Offset::Start,
                )
            },
        )])
    }

    /// The detail page's time in `zone`: "2:00 PM–3:00 PM, Tue 3 Nov", or "All day, Tue 3 Nov".
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
                let (start, end) = timing.in_zone(zone).unwrap();
                let end_time = Timing::format_time(end.time());
                let end_time = match end.date() == start.date() {
                    true => end_time,
                    false => format!("{end_time} ({})", end.format("%-d %b")),
                };
                format!("{}–{end_time}, {}", Timing::format_time(start.time()), day(start.date()))
            }
        }
    }

    /// Ends the current flow and shows the page under it: the detail page, or the day screen
    /// after Delete.
    pub fn back() -> FormComplete { FormComplete::Return(Box::new(|_: &mut Context, _: &Theme| {})) }

    /// Sends the occurrence's changes with `edit` applied, starting from its current ones.
    /// `edit` also gets the series, whose values mean "same as the series". If nothing is left
    /// that differs, it restores the occurrence instead. A cancelled occurrence is left alone.
    pub fn change(calendar: &mut Instance<Calendar>, id: EventId, date: NaiveDate, edit: impl FnOnce(&mut Override, &Event)) {
        let Some(series) = calendar.pending().event(id).cloned() else { return };
        let current = match series.exception(date) {
            Some(Exception::Cancelled) => return,
            Some(Exception::Changed(changes)) => Some(changes.clone()),
            None => None,
        };
        let mut changes = current.clone().unwrap_or_default();
        edit(&mut changes, &series);
        if changes.is_empty() && current.is_some() {
            Calendar::restore_occurrence(calendar, id, date);
        } else if !changes.is_empty() && current.as_ref() != Some(&changes) {
            Calendar::change_occurrence(calendar, id, date, changes);
        }
    }

    /// The detail table for `event`, the occurrence on `date`: Time (and Series time when this
    /// occurrence moved), the repeat rows, Category, Location, Info, Link and Private.
    fn rows(series: &Event, event: &Event, date: NaiveDate, changed: bool, theme: &Theme, width: f32) -> Vec<TableItem> {
        let zone = Timing::device_zone();
        let series_time = series.series_timing(date)
            .filter(|timing| changed && timing != event.timing())
            .map(|timing| ("Series time", Self::when(&timing, zone)));
        let start = series.timing().start_date();
        let repeat = series.recurrence()
            .map(|recurrence| ChangeRepeat::rows(recurrence, start)).unwrap_or_default();
        let rows = [
            [Some(("Time", Self::when(event.timing(), zone))), series_time]
                .into_iter().flatten().collect(),
            repeat,
            [
                Some(("Category", event.category().label().to_string())),
                event.location().map(|location| ("Location", location.to_string())),
                event.info().map(|info| ("Info", info.to_string())),
                event.link().map(|link| ("Link", link.to_string())),
                Some(("Private", if event.private() { "Yes" } else { "No" }.to_string())),
            ].into_iter().flatten().collect(),
        ];
        rows.into_iter().flatten().map(|(label, value)| {
            TableItem::new(label, &DayScreen::fit_label(theme, width, &value, true))
        }).collect()
    }

    /// The detail page's buttons. `edit` is the Edit details flow, shared with the header icon;
    /// Restore shows only when this occurrence was changed.
    fn actions(
        theme: &Theme, calendar: &Instance<Calendar>, id: EventId, date: NaiveDate, series: &Event,
        edit: Flow, width: f32,
    ) -> Vec<ActionItem> {
        let changed = series.changes(date).is_some();
        let time = |scope: Scope| {
            let (calendar, series) = (calendar.clone(), series.clone());
            Self::open(move |theme: &Theme| {
                ChangeTime::picker(theme, calendar.clone(), id, &series, scope)
            })
        };
        let time =
            Self::scoped(theme, series, date, time(Scope::Occurrence(date)), time(Scope::Series));
        let forms = [
            ("Change repeat", ChangeRepeat::form(theme, calendar.clone(), id, series, width)),
            ("Change category", Self::category(theme, calendar.clone(), id, series)),
            ("Change privacy", Self::privacy(theme, calendar.clone(), id, series)),
        ];
        let mut restoring = calendar.clone();
        let restore = Action::custom(move |_ctx: &mut Context, _theme: &Theme| {
            Calendar::restore_occurrence(&mut restoring, id, date)
        });
        [
            ActionItem::new(Action::flow(edit), "Edit details", Icons::Edit),
            ActionItem::new(time, "Change time", Icons::Edit),
        ]
            .into_iter()
            .chain(forms.into_iter().map(|(label, form)| {
                ActionItem::new(Action::flow(Flow::from_form(form)), label, Icons::Edit)
            }))
            .chain(changed.then(|| ActionItem::new(restore, "Restore", Icons::Back)))
            .chain([ActionItem::new(
                Self::delete(theme, calendar, id, date, series), "Delete", Icons::Delete,
            )])
            .collect()
    }

    /// The page once there's nothing left to show.
    fn gone(message: &str) -> PageType {
        let message = Display::instructions(message);
        PageType::display("Event", vec![message], None, Bumper::None, Offset::Center)
    }

    /// A callback that opens the form `make` builds. It runs after the scope step has closed,
    /// so the form opens on top of the detail page.
    fn open(make: impl Fn(&Theme) -> Form + Clone + 'static) -> impl FnMut(&mut Context, &Theme) + Clone + 'static {
        move |ctx: &mut Context, theme: &Theme| {
            Flow::from_form(make(theme)).build(ctx, theme)(ctx, theme)
        }
    }

    /// On a repeating event, the "Which Events?" step (`scope`). An event that doesn't repeat
    /// runs `whole` straight away.
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

    /// Edit details, as a flow because the header's edit icon opens it too: the scope step
    /// first on a repeating event.
    fn edit(theme: &Theme, calendar: &Instance<Calendar>, id: EventId, date: NaiveDate, series: &Event) -> Flow {
        let whole = {
            let (calendar, series) = (calendar.clone(), series.clone());
            Self::open(move |theme: &Theme| Self::edit_details(theme, calendar.clone(), id, &series))
        };
        let this = {
            let (calendar, series) = (calendar.clone(), series.clone());
            Self::open(move |theme: &Theme| {
                Self::edit_occurrence(theme, calendar.clone(), id, date, &series)
            })
        };
        match series.recurrence() {
            Some(_) => Self::scope(theme, date, this, whole),
            None => Flow::from_form(Self::edit_details(theme, calendar.clone(), id, series)),
        }
    }

    /// Delete. Whole series deletes the event at once; This occurrence opens the Cancel
    /// Occurrence review first, because cancelling can't be undone. Both end on the day screen.
    fn delete(theme: &Theme, calendar: &Instance<Calendar>, id: EventId, date: NaiveDate, series: &Event) -> Action {
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
                let width = ctx.window.area().size.0;
                let flow = Self::cancel(theme, calendar.clone(), id, date, &series, width);
                flow.clone().build(ctx, theme)(ctx, theme)
            }
        };
        Self::scoped(theme, series, date, this, whole)
    }

    /// The Cancel Occurrence review: the occurrence's time and a warning. Continue cancels it
    /// and returns to the day screen.
    fn cancel(theme: &Theme, mut calendar: Instance<Calendar>, id: EventId, date: NaiveDate, series: &Event, width: f32) -> Flow {
        let timing = series.occurrence(date).unwrap_or(*series.timing());
        let when = Self::when(&timing, Timing::device_zone());
        let when = DayScreen::fit_label(theme, width, &when, true);
        let title = series.effective(date)
            .map(|event| event.title().to_string()).unwrap_or_default();
        let title = DayScreen::fit_label(theme, width, &title, true);
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
    fn edit_details(theme: &Theme, mut calendar: Instance<Calendar>, id: EventId, event: &Event) -> Form {
        Self::details_page(theme, event, move |title, location, info, link| {
            // Compare against the live event so updates received while editing are considered.
            let Some(event) = calendar.pending().event(id).cloned() else { return };
            if title != event.title() { Calendar::set_title(&mut calendar, id, title); }
            let optional = [
                (location, event.location(), Calendar::set_location as fn(&mut _, _, _)),
                (info, event.info(), Calendar::set_info),
                (link, event.link(), Calendar::set_link),
            ];
            for (new, old, set) in optional {
                if new.as_deref() != old { set(&mut calendar, id, new); }
            }
        })
    }

    /// Edit details for one occurrence: filled with its own values. Save sends them as one
    /// `change_occurrence`; a field equal to the series' is stored as "same as the series".
    fn edit_occurrence(theme: &Theme, mut calendar: Instance<Calendar>, id: EventId, date: NaiveDate, series: &Event) -> Form {
        let event = series.effective(date).unwrap_or_else(|| series.clone());
        Self::details_page(theme, &event, move |title, location, info, link| {
            Self::change(&mut calendar, id, date, |changes, series| {
                changes.set_title(title, series.title());
                changes.set_location(location, series.location());
                changes.set_info(info, series.info());
                changes.set_link(link, series.link());
            });
        })
    }

    /// The Edit details page, filled with `event`'s values. Save trims every field once and
    /// hands `save` the title, location, info and link; a blank title saves nothing.
    fn details_page(
        theme: &Theme, event: &Event,
        mut save: impl FnMut(String, Option<String>, Option<String>, Option<String>) + Clone + 'static,
    ) -> Form {
        let theme = theme.clone();
        let submit = Box::new(move |ctx: &mut Context, states: &Vec<State>| {
            let [State::Text(title), State::Text(location), State::Text(info), State::Text(link)] =
                states.as_slice() else { unreachable!() };
            let Some(title) = Event::optional(title) else { return FormComplete::None };
            // Save is off while Link is invalid, so `web_link` is never `Err` here.
            let link = Event::web_link(link).ok().flatten();
            save(title, Event::optional(location), Event::optional(info), link);
            // chk edit pages discard the submit callback's return value.
            Self::back().run(ctx, &theme);
            FormComplete::None
        }) as Box<dyn FormSubmit>;
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
            FormItem::text_with_preset(
                "Link", event.link().unwrap_or(""), None,
                |_ctx: &mut Context, link: String| match Event::web_link(&link) {
                    Ok(_) => FormValidState::Valid,
                    Err(error) => FormValidState::InvalidWithData(error.into()),
                },
            ),
        ];
        Form::page("Edit Event", items, vec![], submit)
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
    fn choice(theme: &Theme, title: &str, labels: [&str; 2], current: usize, mut pick: impl FnMut(usize) + Clone + 'static) -> Form {
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
}
