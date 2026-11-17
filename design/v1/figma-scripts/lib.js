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
