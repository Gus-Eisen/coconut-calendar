const ICON = {"left":"2:3","close":"2:6","forward":"2:9","radio":"2:12","radio_filled":"2:15","home":"2:18","notification":"2:23","group":"2:26","explore":"2:29","edit":"2:32","delete":"2:35","add":"2:38","add_user":"2:41","checkmark":"2:48","link":"2:51","paste":"2:54","copy":"2:57","backspace":"2:60","search":"2:63","info":"2:66","profile":"2:69","settings":"2:72","messages":"3:222"};
const IDS = { status: '3:2', home: '3:11', header: '3:13', button: '3:41', quick: '3:48', avatar: '3:53', li: '3:57', radio: '3:73', row: '3:88', input: '3:91', num: '3:102', keypad: '3:110', tab: '3:141', note: '3:218' };
const V = (id) => $fig.getVar(id);
const C = { bg: V('VariableID:1:3'), bg2: V('VariableID:1:4'), txt: V('VariableID:1:5'), txt2: V('VariableID:1:6'), head: V('VariableID:1:7'), brand: V('VariableID:1:12'), danger: V('VariableID:1:13') };
const P = (v) => [{ type: 'SOLID', color: v }];
const B = { family: 'Outfit', style: 'Bold' }, R = { family: 'Outfit', style: 'Regular' };
const T = (chars, size, font, color, o = {}) => $fig.text({ characters: chars, fontSize: size, fontName: font, fills: P(color), ...o });
const icon = (n, s, o = {}) => $fig.instance(ICON[n], { width: s, height: s, name: 'icon/' + n, ...o });
const FILL = { layoutSizingHorizontal: 'FILL' };
// ---- content builders (chk Display / Input equivalents)
const label = (t) => T(t, 16, B, C.head, { ...FILL, textAutoResize: 'HEIGHT' });                       // H5 heading
const text = (t) => T(t, 16, R, C.txt, { ...FILL, textAutoResize: 'HEIGHT' });                        // Display::text
const instructions = (t) => T(t, 16, R, C.txt2, { ...FILL, textAutoResize: 'HEIGHT', textAlignHorizontal: 'CENTER' }); // Display::instructions
const li = (o) => $fig.instance(IDS.li, { ...FILL, name: 'ListItem ' + o.title, props: {
  Title: o.title, Subtitle: o.sub || '', 'Right Title': o.right || '', 'Right Subtitle': o.rsub || 'Details',
  'Show Right': !!o.right, 'Show Chevron': o.chev ?? !!o.right, 'Show Avatar': !!o.avatar, 'Show Badge': !!o.badge } });
const list = (lbl, items) => $fig.autoLayout({ name: 'Display::list ' + (lbl || ''), layoutMode: 'VERTICAL', ...FILL, itemSpacing: 8, fills: [] }, [
  ...(lbl ? [label(lbl)] : []),
  $fig.autoLayout({ name: 'Items', layoutMode: 'VERTICAL', ...FILL, itemSpacing: 0, fills: [] }, items.map(li)) ]);
const quick = (lbl, ic) => $fig.instance(IDS.quick, { name: 'Action ' + lbl, props: { Label: lbl, Icon: ICON[ic] } });
const actions = (acts) => $fig.autoLayout({ name: 'QuickActions', ...FILL, layoutWrap: 'WRAP', itemSpacing: 8, counterAxisSpacing: 8, fills: [] }, acts.map(([l, i]) => quick(l, i)));
const table = (lbl, rows, acts) => $fig.autoLayout({ name: 'Display::table ' + lbl, layoutMode: 'VERTICAL', ...FILL, itemSpacing: 16, fills: [] }, [
  label(lbl),
  $fig.autoLayout({ name: 'Rows', layoutMode: 'VERTICAL', ...FILL, itemSpacing: 0, fills: [] }, rows.map(([n, d]) => $fig.instance(IDS.row, { ...FILL, props: { Name: n, Data: d } }))),
  ...(acts ? [actions(acts)] : []) ]);
const dataList = (lbl, items, acts) => $fig.autoLayout({ name: 'Display::data_list ' + lbl, layoutMode: 'VERTICAL', ...FILL, itemSpacing: 8, fills: [] }, [
  label(lbl), $fig.autoLayout({ name: 'Items', layoutMode: 'VERTICAL', ...FILL, fills: [] }, items.map(li)), ...(acts ? [actions(acts)] : []) ]);
const radios = (items, sel = 0) => $fig.autoLayout({ name: 'RadioSelector', layoutMode: 'VERTICAL', ...FILL, fills: [] }, items.map(([t, s], i) =>
  $fig.instance(IDS.radio, { ...FILL, props: { Selected: i === sel ? 'Yes' : 'No', Title: t, Subtitle: s || '', 'Show Subtitle': !!s } })));
const input = (o) => $fig.instance(IDS.input, { ...FILL, props: { Filled: o.value ? 'Yes' : 'No', Value: o.value || o.placeholder, Label: o.label || '', 'Show Label': !!o.label, Error: o.error || '', 'Show Error': !!o.error } });
const number = (o) => $fig.instance(IDS.num, { ...FILL, props: { Value: o.value, Ghost: o.ghost, Instructions: o.instr, Error: o.error || '', 'Show Error': !!o.error } });
const keypad = () => $fig.instance(IDS.keypad, { ...FILL });
const bigAvatar = () => $fig.autoLayout({ name: 'Display::avatar (Xxl)', width: 128, height: 128, cornerRadius: 64, primaryAxisAlignItems: 'CENTER', counterAxisAlignItems: 'CENTER', fills: P(C.bg2) }, [icon('profile', 96)]);
const success = (msg) => [icon('checkmark', 128), T(msg, 20, B, C.head, { ...FILL, textAutoResize: 'HEIGHT', textAlignHorizontal: 'CENTER' })];
const bumper = (btns) => $fig.autoLayout({ name: 'Bumper', ...FILL, paddingLeft: 24, paddingRight: 24, paddingTop: 16, paddingBottom: 16, itemSpacing: 16, fills: P(C.bg) },
  btns.map(([l, s]) => $fig.instance(IDS.button, { ...FILL, props: { Style: s || 'Primary', Label: l } })));
// ---- screen frame: status bar, chk header, content (375 max incl. 24 padding), bumper, optional tab bar, home indicator
function screen(name, o) {
  const kids = [ $fig.instance(IDS.status, FILL),
    $fig.instance(IDS.header, { ...FILL, props: { Type: o.type || 'Stack', Title: o.title, 'Show Right': !!o.right, ...(o.right ? { 'Right Icon': ICON[o.right] } : {}) } }),
    $fig.autoLayout({ name: 'Content', layoutMode: 'VERTICAL', ...FILL, layoutSizingVertical: 'FILL', paddingLeft: 33, paddingRight: 33, paddingTop: 8, paddingBottom: 16, itemSpacing: 24,
      primaryAxisAlignItems: o.center ? 'CENTER' : 'MIN', counterAxisAlignItems: 'CENTER', clipsContent: true, fills: [] }, o.content) ];
  if (o.bumper) kids.push(bumper(o.bumper));
  if (o.tab) kids.push($fig.instance(IDS.tab, { ...FILL, props: { Selected: o.tab } }));
  kids.push($fig.instance(IDS.home, FILL));
  const f = $fig.autoLayout({ name, layoutMode: 'VERTICAL', width: 393, height: 852, layoutSizingVertical: 'FIXED', itemSpacing: 0, clipsContent: true, fills: P(C.bg) }, kids);
  const notes = (o.notes || []).map(([h, n]) => $fig.instance(IDS.note, { name: 'Note · ' + name, props: { Heading: h, Note: n } }));
  return { f, notes };
}
async function place(sectionName, y, screens) {
  const sec = $fig.section({ name: sectionName, x: 0, y });
  await $fig.done();
  const s = sec.node; let x = 80; const ids = {};
  for (const { f, notes } of screens) {
    const fn = f.node; s.appendChild(fn); fn.x = x; fn.y = 100; ids[fn.name] = fn.id;
    let ny = 100;
    for (const n of notes) { const nn = n.node; s.appendChild(nn); nn.x = x + 393 + 24; nn.y = ny; ny += nn.height + 16; }
    x += 393 + (notes.length ? 24 + 300 : 0) + 100;
  }
  s.resizeWithoutConstraints(x, 852 + 200);
  s.fills = [figma.util.solidPaint('#1C1A19')];
  return { section: s.id, ids };
}
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
