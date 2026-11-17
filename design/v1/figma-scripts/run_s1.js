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
const nov = [
  { title: '1', sub: 'Sunday' },
  { title: '2', sub: 'Monday', right: '09:30–10:00 Team standup' },
  { title: '3', sub: 'Tuesday', right: '3 events' },
  { title: '4', sub: 'Wednesday' },
  { title: '5', sub: 'Thursday', right: '12:30–13:30 Lunch with Sam' },
  { title: '6', sub: 'Friday' },
  { title: '7', sub: 'Saturday', right: 'All day Lake cabin weekend' },
  { title: '8', sub: 'Sunday', right: 'All day Lake cabin weekend' },
];
const S = [];
S.push(screen('01 · Month (home, after 1c)', { type: 'Home', title: 'Calendar', content: [list('November 2026', nov)], bumper: [['Create Event']],
  notes: [
    ['chk: no calendar grid', 'Each month is a Display::list with one ListItem::plain per day (day, weekday, summary). Google shows a month grid or a schedule list.'],
    ['chk: summary sits on the right', "ListItem puts the secondary text on the right with a fixed 'Details' subtitle. One event shows 'time + title'; more show 'N events'."],
    ['chk: list starts on the 1st', "The list starts at the 1st of this month (window_start) and runs 12 months. chk can't scroll a root list to today. Google has 'Jump to today'."],
    ['1c: one bumper', "Only 'Create Event' now (bumper_a). The root's one header-icon slot is still free."],
  ] }));
S.push(screen('02 · Event type picker', { title: 'Event Type', content: [radios([['Regular', 'Starts and ends at set times'], ['All-Day', 'Lasts one or more whole days']])], bumper: [['Continue']],
  notes: [
    ['chk forces: pick the type first', "A one-step Form::flow with FormItem::enumerator. Regular is first, so it's the default (the radio always starts on item 0)."],
    ['How it opens the form', "Submit returns FormComplete::Return(callback). Navigation resets, then the chosen linear form opens (the groups.rs new_contact pattern). Google: an All-day toggle inside one sheet, and switching type keeps what you typed."],
  ] }));
S.push(screen('03 · Regular event form (step 6 of 8: End Time)', { title: 'End Time',
  content: [number({ value: '15:00', ghost: '', instr: 'Enter time', error: 'Must be after the start.' }), keypad()], bumper: [['Continue', 'Disabled']],
  notes: [
    ['chk forces: one field per page', 'Steps: Event Title → Category → Date → Start Time → End Date → End Time → Location → Info → Review. A linear form: no skipping or branching.'],
    ['Cross-step check', "Each validator sees only its own step, so End Time reads Date / Start / End Date from a shared Rc<RefCell<FormTimes>>. Example: start 16-11 15:00, end typed 15:00 → inline error, Continue disabled."],
    ['No presets on numbers', "End Date and End Time are typed every time. Google fills the end from a default length."],
  ] }));
S.push(screen('04 · All-day event form (step 3 of 6: First Day)', { title: 'First Day',
  content: [number({ value: '07-11', ghost: '', instr: 'Enter date' }), keypad()], bumper: [['Continue']],
  notes: [
    ['chk forces: a separate form', 'Steps: Event Title → Category → First Day → Last Day → Location → Info → Review. No time-zone step. Google uses an All-day toggle in the same sheet.'],
    ['DD-MM, no year', 'Dates are DD-MM and the year is inferred, so events fit from the 1st of this month to about a year ahead. The keypad is chk’s (it includes a “.” key).'],
  ] }));
S.push(screen('05 · New event review', { title: 'New Event Review',
  content: [table('New event', [['Title', 'Dentist appointment'], ['Category', 'Personal'], ['Start', '16 Nov 2026, 15:00'], ['End', '16 Nov 2026, 15:45'], ['Time zone', 'America/Chicago'], ['Location', 'Dr. Reyes, 5th floor']])],
  bumper: [['Continue']],
  notes: [['chk preset page', "PageType::review with one Display::table. Its button is chk's default 'Continue' (ReviewPage passes no label). Location and Info rows appear only if filled in."]] }));
S.push(screen('06 · New event saved', { type: 'StackEnd', title: 'New Event', center: true,
  content: success('Event saved for 16 Nov 2026 at 15:00'), bumper: [['Done', 'Secondary']],
  notes: [['chk preset page', "PageType::success: a 128 pt checkmark and an H4 message. The close icon and Done both return to the Month."]] }));
return await place('Step 1 · Data model and new-event forms (built; 1c in progress) — SAMPLE DATA', 0, S);
