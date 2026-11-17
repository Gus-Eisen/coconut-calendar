const A = [];
A.push(screen('24 · Invite people', { title: 'Invite People',
  content: [input({ placeholder: 'Search people...' }), list(null, [
    { title: 'Alice', sub: 'Selected', avatar: true, chev: false },
    { title: 'Dana', sub: 'Selected', avatar: true, chev: false },
    { title: 'Eve', sub: 'Not selected', avatar: true, chev: false },
  ]), actions([['New contact', 'add_user']])], bumper: [['Continue']],
  notes: [
    ['chk: FormItem::search', "Orange's 'Add members' pattern. Submit calls add_attendee for each person picked."],
    ['Google', 'Type any name or email, mark guests optional, "Find a time". Coconut invites Coconut users only (email guests are after 1.0), with no optional flag.'],
  ] }));
A.push(screen('25 · Attendees', { title: 'Attendees',
  content: [dataList('Design review · Tue 3 Nov', [
    { title: 'Alice', sub: 'Accepted', avatar: true, chev: false },
    { title: 'Charlie', sub: 'Tentative', avatar: true, chev: false },
    { title: 'Dana', sub: 'Pending', avatar: true, chev: false },
    { title: 'Eve', sub: 'Declined', avatar: true, chev: false },
  ], [['Invite people', 'add_user'], ['Remove someone', 'delete']])],
  notes: [['Display::data_list', "One ListItem::avatar per attendee with their answer as the subtitle. Removing goes through an ActionItem. Attendees can always read this list (AIR's read model), so Google's 'hide guest list' isn't possible."]] }));
A.push(screen('26 · Invitations list', { type: 'Home', title: 'Invites', tab: 'Invites',
  content: [
    list('Waiting for you', [
      { title: 'Book club', sub: 'From Alice · Thu 19 Nov, 19:00', badge: true, chev: true },
      { title: 'Quarterly planning', sub: 'From Charlie · Mon 7 Dec, 10:00', badge: true, chev: true },
    ]),
    list('Answered', [{ title: 'Team offsite', sub: 'From Alice · Accepted', chev: true }]),
  ],
  notes: [
    ['Root tab', 'Its own root, since invites are checked often. Unanswered invites use ListItem::with_badge(true).'],
    ['Spec gap', 'There is no way to remove or hide an unwanted invite from your own calendar (Google allows it). In 1.0, anyone can invite you.'],
  ] }));
A.push(screen('27 · Invitation (invitee view)', { title: 'Book club',
  content: [table('Invitation', [['From', 'Alice'], ['When', 'Thu 19 Nov 2026, 19:00–21:00'], ['Location', "Alice's place"], ['Your answer', 'Pending']],
    [['Accept', 'checkmark'], ['Tentative', 'info'], ['Decline', 'close'], ['Propose new time', 'edit']])],
  notes: [
    ['chk forces: action buttons', 'Four choices are too many for a bumper (max 2), so they are QuickActions (Display::actions / ActionItems). Google shows Yes / No / Maybe at the bottom.'],
    ['Repeating invites', 'For a repeating event, a one-step picker first asks "This occurrence" or "All" (respond takes Option<NaiveDate>). No RSVP note in the spec.'],
  ] }));
A.push(screen('28 · Propose new time (review)', { title: 'Propose Time Review',
  content: [table('Current', [['When', 'Thu 19 Nov 2026, 19:00–21:00']]), table('Proposed', [['When', 'Thu 19 Nov 2026, 18:00–20:00']])], bumper: [['Continue']],
  notes: [
    ['Flow', '(If it repeats: This occurrence / Whole event) → Date → Start Time → End Date → End Time → Review → Success.'],
    ['Differs from Google', "No message field. The spec allows proposals for all-day events; Google doesn't."],
  ] }));
A.push(screen('29 · Proposal review (counterparty)', { title: 'Proposed Time',
  content: [instructions('Charlie proposed a new time for Design review.'),
    table('Current', [['When', 'Tue 3 Nov 2026, 14:00–15:00']]),
    table('Proposed', [['When', 'Tue 3 Nov 2026, 10:00–11:00']]),
    dataList('Answers', [{ title: 'Bob (you)', sub: 'Waiting', avatar: true, chev: false }, { title: 'Alice', sub: 'Agreed', avatar: true, chev: false }])],
  bumper: [['Agree'], ['Decline', 'Secondary']],
  notes: [
    ['Coconut rule differs', 'Every accepted attendee plus the owner must agree, not just the organizer (Google: organizer accepts with one tap).'],
    ['Bumper::double', 'Agree / Decline fit the two-button bumper.'],
  ] }));
const r6 = await place('Step 6 · Invites and time proposals — SAMPLE DATA', 7200, A);
const Bs = [];
Bs.push(screen('30 · Shared calendars', { type: 'Home', title: 'Shared', tab: 'Shared',
  content: [list(null, [
    { title: 'Alice', sub: 'You see: event details', avatar: true, chev: true },
    { title: 'Charlie', sub: 'You see: busy times only', avatar: true, chev: true },
  ])],
  notes: [['Needs a decision', "The spec says shared calendars appear in the instance list; this root tab lists their owners. Google shows them under 'Other calendars', side by side on the web."]] }));
Bs.push(screen("31 · Someone else's month", { title: "Charlie's Calendar",
  content: [list('November 2026', [
    { title: '16', sub: 'Monday', right: 'Busy 09:00–10:00' },
    { title: '17', sub: 'Tuesday', right: '2 busy times' },
    { title: '18', sub: 'Wednesday' },
    { title: '19', sub: 'Thursday', right: 'Busy all day' },
    { title: '20', sub: 'Friday', right: 'Busy 13:00–14:30' },
    { title: '21', sub: 'Saturday' },
  ])],
  notes: [
    ['Read-only', 'Same layout as the Month (19) and Day (18) views, with no Create Event, edit or delete.'],
    ['FreeBusy', "With FreeBusy visibility you see 'Busy' rows with times only, no titles. With Details you see titles too."],
  ] }));
const r7 = await place('Step 7 · Visibility — SAMPLE DATA', 8400, Bs);
return { r6, r7 };
