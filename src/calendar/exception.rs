use ramp::prism::air::Name;
use serde::{Deserialize, Serialize};

use std::collections::BTreeMap;

use super::{Action, Event, Response, Timing};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Exception {
    Cancelled,
    Changed(Override),
}

/// What's different about one occurrence. `None` means "same as the series".
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Override {
    title: Option<String>,
    timing: Option<Timing>,
    location: Option<Option<String>>,
    info: Option<Option<String>>,
    link: Option<Option<String>>,
    responses: BTreeMap<Name, Response>,
}

impl Override {
    pub fn title(&self) -> Option<&str> { self.title.as_deref() }
    pub fn timing(&self) -> Option<&Timing> { self.timing.as_ref() }
    pub fn location(&self) -> Option<Option<&str>> { self.location.as_ref().map(Option::as_deref) }
    pub fn info(&self) -> Option<Option<&str>> { self.info.as_ref().map(Option::as_deref) }
    pub fn link(&self) -> Option<Option<&str>> { self.link.as_ref().map(Option::as_deref) }

    /// Nothing differs from the series.
    pub fn is_empty(&self) -> bool { *self == Override::default() }

    /// The actions setting or clearing these changes needs: one per field that differs.
    pub fn actions(&self) -> Vec<Action> {
        [
            (self.title.is_some(), Action::Title), (self.timing.is_some(), Action::Time),
            (self.location.is_some(), Action::Location), (self.info.is_some(), Action::Info),
            (self.link.is_some(), Action::Link),
        ]
            .into_iter().filter_map(|(set, action)| set.then_some(action)).collect()
    }

    // Each setter takes the series' value too: a value equal to it is stored as `None` ("same
    // as the series"), so the occurrence keeps following the series for that field.

    pub fn set_title(&mut self, title: String, series: &str) {
        self.title = (title != series).then_some(title);
    }

    pub fn set_timing(&mut self, timing: Timing, series: Option<Timing>) {
        self.timing = (Some(timing) != series).then_some(timing);
    }

    pub fn set_location(&mut self, location: Option<String>, series: Option<&str>) {
        self.location = (location.as_deref() != series).then_some(location);
    }

    pub fn set_info(&mut self, info: Option<String>, series: Option<&str>) {
        self.info = (info.as_deref() != series).then_some(info);
    }

    pub fn set_link(&mut self, link: Option<String>, series: Option<&str>) {
        self.link = (link.as_deref() != series).then_some(link);
    }

    /// Trims the text fields as the series setters do. `None` if nothing is left to change, or
    /// a field can't be stored (a blank title, an invalid timing, or responses before step 6).
    pub fn normalised(self) -> Option<Override> {
        let text = |field: Option<Option<String>>| {
            field.map(|value| value.as_deref().and_then(Event::optional))
        };
        let title = self.title.map(|title| Event::optional(&title).ok_or(())).transpose().ok()?;
        if self.timing.is_some_and(|timing| !timing.is_valid()) || !self.responses.is_empty() {
            return None;
        }
        let changes = Override {
            title,
            timing: self.timing,
            location: text(self.location),
            info: text(self.info),
            link: text(self.link),
            responses: BTreeMap::new(),
        };
        (!changes.is_empty()).then_some(changes)
    }
}
