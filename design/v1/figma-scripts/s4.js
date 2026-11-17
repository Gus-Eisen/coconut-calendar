const A = [];
A.push(screen('18 · Day view (final)', { title: 'Tuesday 3 November',
  content: [list(null, [
    { title: "Mum's birthday", sub: 'Personal', right: 'All day' },
    { title: 'Gym', sub: 'Personal · repeats weekly', right: '08:00–08:45' },
    { title: 'Design review', sub: 'Work · Room 4B', right: '14:00–15:00' },
  ])],
  notes: [
    ['chk: a list, not an hour grid', 'PageType::display with one ListItem per occurrence, all-day first. Each opens the event detail (07). Google draws an hour grid you can swipe.'],
    ['No swipe to the next day', 'A display page has no swipe or arrows, so going to another day means back to the Month.'],
    ['Category as text', "Category is part of the subtitle ('Work', 'Personal'); no colors."],
  ] }));
A.push(screen('19 · Month view (final)', { type: 'Home', title: 'Calendar', right: 'search', tab: 'Calendar',
  content: [list('November 2026', [
    { title: '13', sub: 'Friday' },
    { title: '14', sub: 'Saturday', right: 'All day Market · Personal' },
    { title: '15', sub: 'Sunday' },
    { title: '16', sub: 'Monday · Today', badge: true, right: '09:30 Standup · Work' },
    { title: '17', sub: 'Tuesday', right: '2 events' },
    { title: '18', sub: 'Wednesday', right: '12:30 Lunch · Personal' },
    { title: '19', sub: 'Thursday' },
  ])], bumper: [['Create Event']],
  notes: [
    ['Today marker', "Uses ListItem::with_badge(true) (the brand dot Orange uses for unread) plus 'Today' in the subtitle. Shown scrolled to mid-month; chk can't scroll there for you."],
    ['Category as text', "chk fixes the right-hand subtitle to 'Details', so the category label goes into the summary ('· Work')."],
    ['Header icon', "The root's one header icon could open 'Go to date' or Search. pelican has no calendar icon; Search is shown."],
    ['Tabs', 'Calendar, Invites, People and Shared are separate roots, which pelican shows as a tab bar on iPhone. pelican has no calendar or envelope icon; the tabs use Home, Messages, Group and Explore.'],
  ] }));
const r4 = await place('Step 4 · Views — SAMPLE DATA', 4800, A);
const Bs = [];
Bs.push(screen('20 · People and roles', { type: 'Home', title: 'People', tab: 'People',
  content: [list(null, [
    { title: 'Bob (you)', sub: 'Owner · created this calendar', avatar: true, chev: false },
    { title: 'Alice', sub: 'Owner', avatar: true, chev: true },
    { title: "Bob's agent", sub: 'Owner · agent', avatar: true, chev: true },
    { title: 'Charlie', sub: 'Colleague', avatar: true, chev: true },
  ])], bumper: [['Add Person']],
  notes: [
    ['New root tab', 'Its own root (RootInfo with an icon) instead of crowding the Month header. A Display::list of ListItem::avatar rows, plus an "Add Person" bumper.'],
    ['Google', 'Settings → the calendar → "Shared with", with five permission levels. Coconut has two roles: Owner and Colleague.'],
  ] }));
Bs.push(screen('21 · Add person (search step)', { title: 'Add Person',
  content: [input({ placeholder: 'Search people...' }), list(null, [
    { title: 'Charlie', sub: 'Selected', avatar: true, chev: false },
    { title: 'Dana', sub: 'Not selected', avatar: true, chev: false },
    { title: 'Eve', sub: 'Not selected', avatar: true, chev: false },
  ]), actions([['New contact', 'add_user']])], bumper: [['Continue']],
  notes: [
    ['chk: FormItem::search', "The same picker as Orange's 'Add members': the people on this device (Profile), giving State::Search(Vec<Name>). 'New contact' adds someone by name or QR code."],
    ['Approximated', "pelican's SearchBar is drawn here as a text field plus avatar rows; how it marks a selected person isn't mocked exactly."],
    ['Next steps', 'Role (21b) → Review → Success.'],
  ] }));
Bs.push(screen('21b · Add person (role step)', { title: 'Role',
  content: [radios([['Owner', 'Full control: create, edit and delete events. Sees private events.'], ['Colleague', 'Can propose new times. Sees event details.']])], bumper: [['Continue']],
  notes: [
    ['Roles explained', "Each option's subtitle says what the role can do, as the spec's Notes ask. Owner is first, so it's the default; consider putting Colleague first so a misclick doesn't give someone full control."],
    ['Spec gap', 'Assigning roles has no Action yet.'],
  ] }));
Bs.push(screen('22 · Person detail / remove', { title: 'Charlie',
  content: [bigAvatar(), table('Access', [['Role', 'Colleague'], ['Sees', 'Event details'], ['Can', 'Propose new times']], [['Change role', 'edit'], ['Remove', 'delete']])],
  notes: [
    ['Remove = confirm page', "Remove opens a confirmation in Orange's Flow::action_target style with Bumper::double('Remove', 'Cancel'). Removing someone means rebuilding instances (spec section 3)."],
    ['Creator', "The calendar's creator can never lose Owner, so their page has no Remove."],
  ] }));
Bs.push(screen('23 · Change history', { title: 'History',
  content: [list('This week', [
    { title: 'Alice', sub: 'Moved Design review to 16:00', avatar: true, right: 'Tue 3 Nov' },
    { title: "Bob's agent", sub: 'Created Gym (weekly)', avatar: true, right: 'Mon 2 Nov' },
    { title: 'Charlie', sub: 'Proposed 10:00 for Team standup', avatar: true, right: 'Mon 2 Nov' },
  ])],
  notes: [['Open question', 'Not in the plan yet. It would come from each message’s metadata.signer. Google has no user-facing history.']] }));
const r5 = await place('Step 5 · Attribution and roles — SAMPLE DATA', 6000, Bs);
return { r4, r5 };
