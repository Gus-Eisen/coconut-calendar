use serde::{Deserialize, Serialize};

/// What a person on a calendar may do, and how much of it they see. Least powerful first.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Role {
    Colleague,
    Viewer,
    Editor,
    Owner,
}

impl Role {
    /// In the order the role lists show them: least powerful first, so it's the default.
    pub fn options() -> [Role; 4] { [Role::Colleague, Role::Viewer, Role::Editor, Role::Owner] }

    pub fn label(self) -> &'static str {
        match self {
            Role::Colleague => "Colleague",
            Role::Viewer => "Viewer",
            Role::Editor => "Editor",
            Role::Owner => "Owner",
        }
    }

    /// The role list's subtitle.
    pub fn description(self) -> &'static str {
        match self {
            Role::Colleague => "Can propose new times. Sees when you're busy.",
            Role::Viewer => "Sees event details. Can't make changes.",
            Role::Editor => "Can create and change events. Doesn't see private events.",
            Role::Owner => "Full control; sees private events.",
        }
    }

    /// The person page's "Can" row.
    pub fn abilities(self) -> &'static str {
        match self {
            Role::Colleague => "Propose new times",
            Role::Viewer => "Nothing; view only",
            Role::Editor => "Create and change events",
            Role::Owner => "Everything",
        }
    }

    pub fn visibility(self) -> Visibility {
        match self {
            Role::Colleague => Visibility::FreeBusy,
            Role::Viewer | Role::Editor => Visibility::Details,
            Role::Owner => Visibility::Private,
        }
    }

    pub fn allows(self, action: Action) -> bool {
        match self {
            Role::Colleague => action == Action::ProposeTime,
            Role::Viewer => false,
            Role::Editor => !matches!(action, Action::Privacy | Action::Members),
            Role::Owner => true,
        }
    }
}

/// How much of a calendar a role sees. Not enforced on reads yet (step 7).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Visibility {
    None,
    FreeBusy,
    Details,
    Private,
}

impl Visibility {
    /// The person page's "Sees" row.
    pub fn label(self) -> &'static str {
        match self {
            Visibility::None => "Nothing",
            Visibility::FreeBusy => "Busy times only",
            Visibility::Details => "Event details",
            Visibility::Private => "Private events",
        }
    }
}

/// What a message does, for the role check in `Calendar::apply`. Not chk's UI `Action`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Create,
    Delete,
    Title,
    Time,
    Info,
    Location,
    Recurrence,
    Attendee,
    Link,
    ProposeTime,
    Privacy,
    Category,
    /// Adding people, changing their roles and removing them.
    Members,
}
