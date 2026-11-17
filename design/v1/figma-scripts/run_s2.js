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
const S = [];
S.push(screen('07 · Event detail', { title: 'Design review', right: 'edit',
  content: [
    table('Event', [['Time', '14:00–15:00, Tue 3 Nov'], ['Category', 'Work'], ['Location', 'Room 4B'], ['Info', 'Bring the Q4 mockups'], ['Private', 'No'], ['Video link', 'meet.example.com/q4-review']],
      [['Edit details', 'edit'], ['Change time', 'edit'], ['Change category', 'edit'], ['Delete', 'delete']]),
    dataList('Attachments', [{ title: 'docs.example.com/q4-plan', sub: 'Link' }], [['Add link', 'link']]),
  ],
  notes: [
    ['New page', 'Today each event is a table inside the Day view. A separate PageType::display keeps the day short and has room for attendees, links and proposals later.'],
    ['chk forces: edits are split', 'Only text inputs can be pre-filled, so text fields share one edit page (08) and time, category and privacy each get a small flow (09–11). Google opens one sheet with everything filled in.'],
    ['Delete', 'Permanent, no confirmation or undo (your decision).'],
  ] }));
S.push(screen('08 · Edit details', { title: 'Edit Event',
  content: [
    input({ label: 'Title', value: 'Design review' }),
    input({ label: 'Location', value: 'Room 5A' }),
    input({ label: 'Info', value: 'Bring the Q4 mockups' }),
    input({ label: 'Video link', value: 'meet.example.com/q4-review' }),
  ], bumper: [['Save']],
  notes: [
    ['chk: one edit page', 'Form::page / PageType::edit_and_display with FormItem::text_with_preset for each field. Shown after Location changed from Room 4B, so Save is on. Save stays off until something changes and every field is valid.'],
    ['What it cannot hold', "Date, time and the radio picker can't be pre-filled, so they aren't on this page."],
  ] }));
S.push(screen('09 · Change time (review)', { title: 'Change Time Review',
  content: [
    table('Current', [['Start', '3 Nov 2026, 14:00'], ['End', '3 Nov 2026, 15:00']]),
    table('New', [['Start', '3 Nov 2026, 16:00'], ['End', '3 Nov 2026, 17:00'], ['Time zone', 'America/Chicago']]),
  ], bumper: [['Continue']],
  notes: [
    ['chk forces: blank steps', 'Flow: Date → Start Time → End Date → End Time → Review. The number inputs have no preset and a form step holds only its input, so the current time first appears here on the review.'],
  ] }));
S.push(screen('10 · Change to all-day / to timed', { title: 'Timing',
  content: [radios([['Timed', 'Now: 14:00–15:00, Tue 3 Nov'], ['All-day', 'Whole days, no times']])], bumper: [['Continue']],
  notes: [
    ['chk forces: picker, then flow', 'Same pattern as the type picker (02). Submit returns FormComplete::Return and opens the matching timing flow (09, or First Day → Last Day).'],
    ["Can't preselect", 'The radio always starts on the first option, so the current timing is written in its subtitle.'],
  ] }));
S.push(screen('11 · Change category / privacy', { title: 'Category',
  content: [radios([['Personal', ''], ['Work', 'Current']])], bumper: [['Continue']],
  notes: [
    ["chk: can't preselect", "The radio always starts on item 0 (Personal), even though this event is Work. The current value is shown as the option's subtitle. Submit calls set_category and returns to the detail page."],
    ['Text label, no color', "Category is shown as text everywhere; chk can't color rows."],
    ['Privacy', "Same one-step page: 'Not private' / 'Private'."],
  ] }));
S.push(screen('12 · Add attachment link', { title: 'Attachment',
  content: [input({ value: 'docs.example.com/q4-plan' }), actions([['Paste', 'paste']])], bumper: [['Continue']],
  notes: [
    ['Links only', 'FormItem::text with FormAction::Paste. No file upload (attachments are Vec<String> links).'],
    ['Removing', 'Each link in the detail page (07) gets a Delete ActionItem (remove_attachment).'],
  ] }));
return await place('Step 2 · Editing — SAMPLE DATA', 1200, S);
