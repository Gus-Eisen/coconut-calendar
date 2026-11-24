use chk::profiles::Profile;
use chk::{
    Action, ActionItem, AvatarContent, AvatarPurpose, Bumper, Context, Display, Flow, FlowBuilder,
    Form, FormAction, FormComplete, FormItem, FormSubmit, FormValidState, Icons, ListItem, Offset,
    Page, PageType, Review, Root, State, Success, TableItem, Theme, Watch,
};
use ramp::prism::air::{Instance, Name};

use std::collections::BTreeMap;

use crate::{Calendar, Role};

use super::{DayScreen, EventScreen};

/// The People tab: everyone with a role on the calendar, and the pages that manage them. Only
/// a role that allows `Members` (an Owner) sees Add Person, Change role and Remove.
pub struct PeopleScreen;
impl PeopleScreen {
    /// The creator comes first, marked "created this calendar", with no detail page: there's
    /// nothing to change or remove. Everyone else follows by username.
    pub fn root(calendar: Instance<Calendar>) -> Root {
        Root::custom(Page::watching(
            (Watch::contract(calendar), Watch::list::<Profile>()),
            |ctx: &mut Context, _theme: &Theme, (calendar, _): (Instance<Calendar>, Vec<Instance<Profile>>)| {
                let (creator, members, mine) = {
                    let state = calendar.pending();
                    (state.creator(), state.members().clone(), state.role(ctx.air.me()))
                };
                let mut people = members.into_iter().map(|(name, role)| {
                    let (avatar, username) = Self::person(ctx, name);
                    (name, role, avatar, username)
                }).collect::<Vec<_>>();
                people.sort_by_key(|(name, _, _, username)| (*name != creator, username.clone()));
                let rows = people.into_iter().map(|(name, role, avatar, username)| {
                    let (subtitle, flow) = match name == creator {
                        true => (format!("{} · created this calendar", role.label()), None),
                        false => {
                            (role.label().to_string(), Some(Self::detail(calendar.clone(), name)))
                        }
                    };
                    ListItem::avatar(avatar, &username, &subtitle, None, flow)
                }).collect();
                let add = mine.is_some_and(|role| role.allows(crate::Action::Members)).then(|| {
                    let add = move |ctx: &mut Context, theme: &Theme| {
                        Flow::from_form(Self::add(ctx, theme, calendar.clone()))
                    };
                    ("Add Person".to_string(), Box::new(add) as Box<dyn FlowBuilder>)
                });
                let list = Display::list(None, rows, None);
                PageType::root("People", vec![], vec![list], None, add, None)
            },
        ))
    }

    /// `name`'s avatar and username, "{username} (you)" for this device's user. Someone with
    /// no profile here shows a short form of their key instead.
    pub fn person(ctx: &mut Context, name: Name) -> (AvatarContent, String) {
        let me = ctx.air.me();
        let profile = match name == me {
            true => Some(Profile::me(ctx)),
            false => Profile::try_from_name(ctx, name),
        };
        let (avatar, username) = match profile {
            Some(profile) => {
                let profile = profile.pending();
                (profile.avatar.clone(), profile.username.clone())
            }
            None => (AvatarContent::default(), Self::key(name)),
        };
        match name == me {
            true => (avatar, format!("{username} (you)")),
            false => (avatar, username),
        }
    }

    /// The start and end of `name`'s key, as chk's `Profile::name` shows it. A key is 66 hex
    /// digits, so the slices are always in range.
    fn key(name: Name) -> String {
        let key = name.to_string();
        format!("{}...{}", &key[..17], &key[key.len() - 4..])
    }

    /// One person's page: their role, what it sees and what it can do. If they're removed
    /// while it's open, it says so.
    fn detail(calendar: Instance<Calendar>, name: Name) -> Flow {
        Flow::new(vec![Page::watching(
            (Watch::contract(calendar), Watch::list::<Profile>()),
            move |ctx: &mut Context, theme: &Theme, (calendar, _): (Instance<Calendar>, Vec<Instance<Profile>>)| {
                let (role, mine) = {
                    let state = calendar.pending();
                    let role = state.role(name).filter(|_| name != state.creator());
                    (role, state.role(ctx.air.me()))
                };
                let Some(role) = role else {
                    let gone = vec![Display::instructions("They're no longer on this calendar.")];
                    return PageType::display("Person", gone, None, Bumper::None, Offset::Center);
                };
                let (avatar, username) = Self::person(ctx, name);
                let width = ctx.window.area().size.0;
                let rows = [
                    ("Role", role.label()), ("Sees", role.visibility().label()),
                    ("Can", role.abilities()),
                ]
                    .map(|(label, value)| {
                        TableItem::new(label, &DayScreen::fit_label(theme, width, value, true))
                    })
                    .to_vec();
                let actions = mine.is_some_and(|role| role.allows(crate::Action::Members)).then(|| {
                    let change = Self::change_role(theme, calendar.clone(), name, role);
                    let change = Flow::from_form(change);
                    let remove = Self::remove(calendar.clone(), name, avatar.clone(), &username);
                    vec![
                        ActionItem::new(Action::flow(change), "Change role", Icons::Edit),
                        ActionItem::new(Action::flow(remove), "Remove", Icons::Delete),
                    ]
                });
                let items = vec![
                    Display::avatar(avatar, AvatarPurpose::None),
                    Display::table("Access", rows, actions),
                ];
                PageType::display(&username, items, None, Bumper::None, Offset::Start)
            },
        )])
    }

    /// Add Person: pick people this device knows (or a new contact), then a role, then Review
    /// and Success. People already on the calendar aren't offered, and are skipped on submit.
    fn add(ctx: &mut Context, theme: &Theme, mut calendar: Instance<Calendar>) -> Form {
        let members = calendar.pending().members().keys().copied().collect::<Vec<_>>();
        let mut known = ctx.air.instances::<Profile>().list().into_values().filter_map(|profile| {
            let profile = profile.pending();
            let name = profile.name.filter(|name| !members.contains(name))?;
            Some((name, profile.avatar.clone(), profile.username.clone(), profile.name()))
        }).collect::<Vec<_>>();
        known.sort_by_key(|(name, ..)| *name);
        known.dedup_by_key(|(name, ..)| *name);
        let items = known.iter().map(|(name, avatar, username, key)| {
            (ListItem::avatar(avatar.clone(), username, key, None, None), *name)
        }).collect();
        // The review and success pages get no `Context`, so they read usernames from here. A
        // new contact isn't in it and shows as a short key.
        let usernames = known.into_iter().map(|(name, _, username, _)| (name, username))
            .collect::<BTreeMap<_, _>>();
        let username = move |name: &Name| {
            usernames.get(name).cloned().unwrap_or_else(|| Self::key(*name))
        };

        let new_contact = Action::flow(Flow::from_form(Self::new_contact(theme, members)));
        let new_contact = FormAction::Custom("New contact".into(), None, Icons::Add, new_contact);
        let search = FormItem::search("Add Person", "Search people", items, Some(vec![new_contact]));
        let roles = Role::options().map(|role| (role.label(), role.description())).to_vec();
        let inputs = vec![search, FormItem::enumerator("Role", roles)];

        let submit = Box::new(move |_ctx: &mut Context, states: &Vec<State>| {
            let [State::Search(names), State::Enumerator(index)] = states.as_slice() else {
                unreachable!()
            };
            for name in names {
                if calendar.pending().role(*name).is_none() {
                    Calendar::set_role(&mut calendar, *name, Role::options()[*index]);
                }
            }
            FormComplete::None
        }) as Box<dyn FormSubmit>;
        let review_username = username.clone();
        let review = move |states: &Vec<State>| {
            let [State::Search(names), State::Enumerator(index)] = states.as_slice() else {
                unreachable!()
            };
            let rows = names.iter().map(|name| TableItem::new("Person", &review_username(name)))
                .chain([TableItem::new("Role", Role::options()[*index].label())])
                .collect();
            vec![Display::table("New person", rows, None)]
        };
        let success = move |states: Vec<State>| {
            let [State::Search(names), State::Enumerator(index)] = states.as_slice() else {
                unreachable!()
            };
            let role = Role::options()[*index].label();
            let message = match names.as_slice() {
                [name] => format!("{} added as {role}", username(name)),
                names => format!("{} people added as {role}", names.len()),
            };
            (Icons::Checkmark, message)
        };
        let success = Success::new("Person Added", success);
        Form::flow(theme, inputs, Some(Review::new("Review", review)), Some(success), submit)
    }

    /// Paste or scan someone's AIR name. It makes them a profile on this device and selects
    /// them in the search, as chk's group "Add members" does.
    fn new_contact(theme: &Theme, members: Vec<Name>) -> Form {
        let submit = Box::new(|ctx: &mut Context, states: &Vec<State>| {
            let [State::Text(value)] = states.as_slice() else { unreachable!() };
            let Ok(name) = value.trim().parse::<Name>() else { return FormComplete::None };
            Profile::from_name(ctx, name);
            FormComplete::Return(Box::new(move |ctx: &mut Context, theme: &Theme| {
                Action::choose_search(name).get()(ctx, theme)
            }))
        }) as Box<dyn FormSubmit>;
        let actions = vec![FormAction::Paste, FormAction::scan_qr_code("Scan a profile QR code")];
        let name = FormItem::text(
            "New contact", "AIR name", Some(actions),
            move |_ctx: &mut Context, value: String| match value.trim().parse::<Name>() {
                Ok(name) if members.contains(&name) => {
                    FormValidState::InvalidWithData("Already on this calendar.".into())
                }
                Ok(_) => FormValidState::Valid,
                Err(_) => FormValidState::InvalidWithData("Enter a valid AIR name.".into()),
            },
        );
        Form::flow(theme, vec![name], None, None, submit)
    }

    /// chk selects the first option, so put the current role there on this form.
    fn change_role(theme: &Theme, mut calendar: Instance<Calendar>, name: Name, current: Role) -> Form {
        let roles = std::iter::once(current)
            .chain(Role::options().into_iter().filter(|role| *role != current))
            .collect::<Vec<_>>();
        let options = roles.iter().map(|role| (role.label(), role.description())).collect();
        let input = FormItem::enumerator("Change Role", options);
        let submit = Box::new(move |_ctx: &mut Context, states: &Vec<State>| {
            let [State::Enumerator(index)] = states.as_slice() else { unreachable!() };
            if roles[*index] != current && calendar.pending().role(name).is_some() {
                Calendar::set_role(&mut calendar, name, roles[*index]);
            }
            EventScreen::back()
        }) as Box<dyn FormSubmit>;
        Form::flow(theme, vec![input], None, None, submit)
    }

    /// Asks first. Remove closes this page and the person's page under it; Cancel only this one.
    fn remove(mut calendar: Instance<Calendar>, name: Name, avatar: AvatarContent, username: &str) -> Flow {
        let remove = Action::custom(move |ctx: &mut Context, theme: &Theme| {
            Calendar::remove_member(&mut calendar, name);
            FormComplete::Return(Box::new(|ctx: &mut Context, theme: &Theme| {
                EventScreen::back().run(ctx, theme)
            })).run(ctx, theme)
        });
        let items = vec![
            Display::avatar(avatar, AvatarPurpose::None),
            Display::confirmation_message(&format!("Remove {username}?")),
            Display::instructions("They'll no longer have a role on this calendar."),
        ];
        let bumper = Bumper::double("Remove", remove, "Cancel", Action::None);
        let page = PageType::display("Remove Person", items, None, bumper, Offset::Center);
        Flow::new(vec![Page::Static(Box::new(page))])
    }
}
