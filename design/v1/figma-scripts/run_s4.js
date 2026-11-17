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
