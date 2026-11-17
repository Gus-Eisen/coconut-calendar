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
A.push(screen('13 · Type picker with Repeating', { title: 'Event Type',
  content: [radios([['Regular', 'Starts and ends at set times'], ['All-Day', 'Lasts one or more whole days'], ['Repeating', 'Happens daily, weekly, monthly or yearly']])], bumper: [['Continue']],
  notes: [['Step 3a', 'Same one-step picker as 02 with a third option. Regular stays first, so it stays the default.']] }));
A.push(screen('14 · Repeating event form (Frequency step)', { title: 'Frequency',
  content: [radios([['Daily', ''], ['Weekly', ''], ['Monthly', ''], ['Yearly', '']])], bumper: [['Continue']],
  notes: [
    ['chk forces: every step always shows', 'Steps: Event Title → Category → Date → Start Time → End Date → End Time → Frequency → Repeat Days → Until → Location → Info → Review. Repeat Days appears even for Daily, because a form can’t skip a step.'],
    ['Google', "'Does not repeat' is a menu inside the one sheet, with custom rules (every 2 weeks, 3rd Tuesday, after N times). The spec has 4 frequencies and an optional end date."],
  ] }));
A.push(screen('14b · Repeating event form (Repeat Days step)', { title: 'Repeat Days',
  content: [input({ value: 'Tue, Thu' })], bumper: [['Continue']],
  notes: [
    ['chk: no multi-select', "There's no checkbox list (FormItem::search only picks people), so weekly days are typed (e.g. 'Tue, Thu'; blank = the start's weekday). A form step can't show extra help text, so the rule must fit the placeholder. Presets ('Same weekday', 'Weekdays') or a form per frequency are the alternatives (open question 4)."],
    ['Until', 'Next step: Until (DD-MM). Leaving it blank means it never ends.'],
  ] }));
A.push(screen('15 · Repeat settings on edit (review)', { title: 'Change Repeat Review',
  content: [table('Current', [['Repeats', 'Weekly on Tue'], ['Until', 'Never']]), table('New', [['Repeats', 'Weekly on Tue, Thu'], ['Until', '31 Mar 2027']])],
  bumper: [['Continue']],
  notes: [['set_recurrence', 'Flow: Frequency → Repeat Days → Until → Review, without the event fields. As with time (09), the current value can only appear on the review page.']] }));
const r1 = await place('Step 3a · Repeating events — SAMPLE DATA', 2400, A);
const Bs = [];
Bs.push(screen('16 · Scope chooser', { title: 'Change Which Events?',
  content: [instructions('Gym repeats every Tuesday. You chose Tue 10 Nov.'), radios([['This event', 'Only Tue 10 Nov'], ['This and following events', 'From Tue 10 Nov on'], ['All events', 'Every Tuesday']])],
  bumper: [['Continue']],
  notes: [
    ['chk forces: ask first', 'Google asks after you tap Save. Here the edit is its own flow, so the scope is chosen before it. Submit opens the matching edit flow, or deletes right away.'],
    ['Why a radio page', 'A bumper holds at most two buttons, and there are three choices. Hide "This and following" on the first occurrence (as Google does).'],
    ['Form page limit', 'A form step holds only its input; the instruction line here assumes a display-and-input page.'],
  ] }));
Bs.push(screen('17 · Occurrence detail (changed occurrence)', { title: 'Gym', right: 'edit',
  content: [
    instructions('This occurrence was changed. Others in the series follow the series.'),
    table('Event', [['Time', '07:00–07:45, Tue 10 Nov'], ['Series time', '08:00–08:45'], ['Repeats', 'Weekly on Tue'], ['Category', 'Personal']],
      [['Edit', 'edit'], ['Restore to series', 'left'], ['Delete', 'delete']]),
  ],
  notes: [['restore_occurrence', 'Like the detail page (07) with one more ActionItem, "Restore to series". pelican has no "undo" icon; the mockup reuses Back.']] }));
const r2 = await place('Step 3b · Exceptions — SAMPLE DATA', 3600, Bs);
return { r1, r2 };
