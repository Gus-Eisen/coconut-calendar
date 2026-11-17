// icons scale with their instance
const ICON_IDS = {"left":"2:3","close":"2:6","forward":"2:9","radio":"2:12","radio_filled":"2:15","home":"2:18","notification":"2:23","group":"2:26","explore":"2:29","edit":"2:32","delete":"2:35","add":"2:38","add_user":"2:41","checkmark":"2:48","link":"2:51","paste":"2:54","copy":"2:57","backspace":"2:60","search":"2:63","info":"2:66","profile":"2:69","settings":"2:72"};
for (const id of Object.values(ICON_IDS)) {
  const c = await figma.getNodeByIdAsync(id);
  c.query('*').each(n => { if ('constraints' in n) n.constraints = { horizontal: 'SCALE', vertical: 'SCALE' }; });
}
const V = (id) => $fig.getVar(id);
const C = { bg: V('VariableID:1:3'), bg2: V('VariableID:1:4'), txt: V('VariableID:1:5'), txt2: V('VariableID:1:6'), head: V('VariableID:1:7'), out2: V('VariableID:1:9'), btnOut: V('VariableID:1:10'), ghostDis: V('VariableID:1:11'), brand: V('VariableID:1:12'), danger: V('VariableID:1:13'), annBg: V('VariableID:1:15'), annTxt: V('VariableID:1:16') };
const P = (v) => [{ type: 'SOLID', color: v }];
const B = { family: 'Outfit', style: 'Bold' }, R = { family: 'Outfit', style: 'Regular' };
const T = (chars, size, font, color, o = {}) => $fig.text({ characters: chars, fontSize: size, fontName: font, fills: P(color), ...o });
const icon = (n, s, o = {}) => $fig.instance(ICON_IDS[n], { width: s, height: s, name: 'icon/' + n, ...o });

// 1 Status bar
const statusBar = $fig.component({ name: 'iOS Status Bar', layoutMode: 'HORIZONTAL', width: 393, height: 54, paddingLeft: 40, paddingRight: 32, primaryAxisAlignItems: 'SPACE_BETWEEN', counterAxisAlignItems: 'CENTER', fills: P(C.bg) }, [
  T('9:41', 17, B, C.txt),
  $fig.autoLayout({ name: 'Indicators', itemSpacing: 6, counterAxisAlignItems: 'CENTER', fills: [] }, [
    $fig.autoLayout({ name: 'Signal', itemSpacing: 2, counterAxisAlignItems: 'MAX', fills: [] }, [4, 6, 8, 10].map(h => $fig.rectangle({ width: 3, height: h, cornerRadius: 1, fills: P(C.txt) }))),
    $fig.rectangle({ name: 'Battery', width: 24, height: 11, cornerRadius: 3, fills: P(C.txt) }),
  ]),
]);
// 2 Home indicator
const homeInd = $fig.component({ name: 'iOS Home Indicator', layoutMode: 'HORIZONTAL', width: 393, height: 34, primaryAxisAlignItems: 'CENTER', counterAxisAlignItems: 'CENTER', fills: P(C.bg) }, [
  $fig.rectangle({ width: 134, height: 5, cornerRadius: 3, fills: P(C.txt) }),
]);
// 3 Header variants
const slot = (name, child) => $fig.autoLayout({ name, width: 48, height: 48, primaryAxisAlignItems: 'CENTER', counterAxisAlignItems: 'CENTER', fills: [] }, child ? [child] : []);
const headerVariant = (type) => $fig.component({ name: 'Type=' + type, layoutMode: 'HORIZONTAL', width: 393, height: 80, paddingLeft: 24, paddingRight: 24, counterAxisAlignItems: 'CENTER', fills: P(C.bg) }, [
  slot('Left', type === 'Stack' ? icon('left', 48) : type === 'StackEnd' ? icon('close', 48) : null),
  T(type === 'Home' ? 'Calendar' : 'Title', type === 'Home' ? 24 : 20, B, C.head, { textAlignHorizontal: 'CENTER', layoutSizingHorizontal: 'FILL' }).textProp('Title'),
  slot('Right', icon('edit', 48, { visible: false }).instanceSwapProp('Right Icon').booleanProp('Show Right')),
]);
const header = $fig.variants({ name: 'Header' }, ['Home', 'Stack', 'StackEnd'].map(headerVariant));
// 4 Button variants (Large)
const btn = (style) => $fig.component({ name: 'Style=' + style, layoutMode: 'HORIZONTAL', height: 48, paddingLeft: 24, paddingRight: 24, cornerRadius: 24, primaryAxisAlignItems: 'CENTER', counterAxisAlignItems: 'CENTER',
  fills: style === 'Primary' ? P(C.brand) : style === 'Disabled' ? [figma.util.solidPaint('#443F3F')] : [],
  strokes: style === 'Secondary' ? P(C.btnOut) : [], strokeWeight: 1 }, [
  T('Continue', 20, B, style === 'Disabled' ? C.bg : C.txt).textProp('Label'),
]);
const button = $fig.variants({ name: 'Button (Large)' }, ['Primary', 'Secondary', 'Disabled'].map(btn));
// 5 Quick action (SecondaryButton::medium)
const quick = $fig.component({ name: 'Quick Action', layoutMode: 'HORIZONTAL', height: 32, paddingLeft: 12, paddingRight: 12, itemSpacing: 4, cornerRadius: 16, counterAxisAlignItems: 'CENTER', fills: [], strokes: P(C.btnOut), strokeWeight: 1 }, [
  icon('edit', 20).instanceSwapProp('Icon'),
  T('Edit', 16, B, C.txt).textProp('Label'),
]);
// 7 Avatar (Md 48, icon content)
const avatar = $fig.component({ name: 'Avatar (Md, icon)', layoutMode: 'HORIZONTAL', width: 48, height: 48, cornerRadius: 24, primaryAxisAlignItems: 'CENTER', counterAxisAlignItems: 'CENTER', fills: P(C.bg2) }, [ icon('profile', 32) ]);
// 6 List item
const listItem = $fig.component({ name: 'List Item', layoutMode: 'HORIZONTAL', width: 327, paddingTop: 16, paddingBottom: 16, itemSpacing: 16, counterAxisAlignItems: 'CENTER', fills: [] }, [
  $fig.instance(avatar, { name: 'Avatar', visible: false }).booleanProp('Show Avatar'),
  $fig.autoLayout({ name: 'Left', layoutMode: 'VERTICAL', itemSpacing: 4, layoutSizingHorizontal: 'FILL', fills: [] }, [
    $fig.autoLayout({ name: 'Title Row', itemSpacing: 4, counterAxisAlignItems: 'CENTER', fills: [] }, [
      T('16', 16, B, C.head).textProp('Title'),
      $fig.ellipse({ name: 'Badge', width: 8, height: 8, fills: P(C.brand), visible: false }).booleanProp('Show Badge'),
    ]),
    T('Monday', 12, R, C.txt2).textProp('Subtitle'),
  ]),
  $fig.autoLayout({ name: 'Right', layoutMode: 'VERTICAL', itemSpacing: 4, counterAxisAlignItems: 'MAX', fills: [] }, [
    T('09:30–10:00 Standup', 16, B, C.head, { textAlignHorizontal: 'RIGHT' }).textProp('Right Title'),
    T('Details', 12, R, C.txt2, { textAlignHorizontal: 'RIGHT' }).textProp('Right Subtitle'),
  ]).booleanProp('Show Right'),
  icon('forward', 24, { name: 'Chevron' }).booleanProp('Show Chevron'),
]);
// 8 Radio row
const radioRow = (sel) => $fig.component({ name: 'Selected=' + sel, layoutMode: 'HORIZONTAL', width: 327, paddingTop: 16, paddingBottom: 16, itemSpacing: 16, counterAxisAlignItems: 'CENTER', fills: [] }, [
  $fig.autoLayout({ name: 'Left', layoutMode: 'VERTICAL', itemSpacing: 4, layoutSizingHorizontal: 'FILL', fills: [] }, [
    T('Regular', 16, B, C.head).textProp('Title'),
    T('A timed event', 12, R, C.txt2).textProp('Subtitle').booleanProp('Show Subtitle'),
  ]),
  icon(sel === 'Yes' ? 'radio_filled' : 'radio', 32),
]);
const radio = $fig.variants({ name: 'Radio Row (RadioSelector item)' }, ['Yes', 'No'].map(radioRow));
// 9 Table row (Tabular)
const tableRow = $fig.component({ name: 'Table Row', layoutMode: 'HORIZONTAL', width: 327, paddingTop: 4, paddingBottom: 4, itemSpacing: 8, fills: [] }, [
  T('Time', 14, R, C.txt, { layoutSizingHorizontal: 'FILL' }).textProp('Name'),
  T('09:30–10:00', 14, R, C.txt, { textAlignHorizontal: 'RIGHT' }).textProp('Data'),
]);
// 10 Text input
const textInput = (filled) => $fig.component({ name: 'Filled=' + filled, layoutMode: 'VERTICAL', width: 327, itemSpacing: 16, fills: [] }, [
  T('Title', 16, B, C.head, { visible: false }).textProp('Label').booleanProp('Show Label'),
  $fig.autoLayout({ name: 'Field', layoutSizingHorizontal: 'FILL', paddingLeft: 16, paddingRight: 16, paddingTop: 16, paddingBottom: 16, cornerRadius: 8, fills: [], strokes: P(C.out2), strokeWeight: 1 }, [
    T(filled === 'Yes' ? 'Team standup' : 'Enter event title...', 16, R, filled === 'Yes' ? C.txt : C.txt2, { layoutSizingHorizontal: 'FILL' }).textProp('Value'),
  ]),
  T('Error', 14, R, C.danger, { visible: false }).textProp('Error').booleanProp('Show Error'),
]);
const input = $fig.variants({ name: 'Text Input' }, ['Yes', 'No'].map(textInput));
// 11 Number display (NumericalInput date/time)
const numDisplay = $fig.component({ name: 'Number Input (date/time)', layoutMode: 'VERTICAL', width: 327, paddingTop: 32, paddingBottom: 16, itemSpacing: 24, counterAxisAlignItems: 'CENTER', fills: [] }, [
  $fig.autoLayout({ name: 'Slots', fills: [] }, [
    T('16-1', 48, B, C.head).textProp('Value'),
    T('1', 48, B, C.ghostDis).textProp('Ghost'),
  ]),
  T('Enter date', 20, R, C.txt2, { textAlignHorizontal: 'CENTER' }).textProp('Instructions'),
  T('Not a valid date.', 20, R, C.danger, { textAlignHorizontal: 'CENTER', visible: false }).textProp('Error').booleanProp('Show Error'),
]);
// 12 Keypad
const key = $fig.component({ name: 'Keypad Key', layoutMode: 'HORIZONTAL', width: 96, height: 48, primaryAxisAlignItems: 'CENTER', counterAxisAlignItems: 'CENTER', fills: [] }, [ T('1', 24, B, C.txt).textProp('Key') ]);
const keyRows = [['1','2','3'],['4','5','6'],['7','8','9'],['.','0','⌫']];
const keypad = $fig.component({ name: 'Keypad', layoutMode: 'VERTICAL', width: 327, itemSpacing: 16, counterAxisAlignItems: 'CENTER', fills: [] },
  keyRows.map((r, i) => $fig.autoLayout({ name: 'Row ' + (i + 1), itemSpacing: 16, fills: [] }, r.map(k => k === '⌫'
    ? $fig.autoLayout({ name: 'Delete', width: 96, height: 48, primaryAxisAlignItems: 'CENTER', counterAxisAlignItems: 'CENTER', fills: [] }, [icon('backspace', 32)])
    : $fig.instance(key, { name: 'Key ' + k, props: { Key: k } })))));
// 13 Tab bar
const TABS = [['Calendar', 'home'], ['Invites', 'notification'], ['People', 'group'], ['Shared', 'explore']];
const tabBar = $fig.variants({ name: 'Tab Bar (Navigator::mobile)' }, TABS.map(([sel]) => $fig.component({ name: 'Selected=' + sel, layoutMode: 'HORIZONTAL', width: 393, height: 56, primaryAxisAlignItems: 'CENTER', counterAxisAlignItems: 'CENTER', fills: P(C.bg) },
  TABS.map(([name, ic]) => $fig.autoLayout({ name: 'Tab ' + name, width: 90, height: 48, primaryAxisAlignItems: 'CENTER', counterAxisAlignItems: 'CENTER', fills: [] }, [ icon(ic, 32, { opacity: name === sel ? 1 : 0.35 }) ])))));
// 14 Annotation
const note = $fig.component({ name: 'Annotation', layoutMode: 'VERTICAL', width: 300, paddingTop: 16, paddingBottom: 16, paddingLeft: 16, paddingRight: 16, itemSpacing: 8, cornerRadius: 8, fills: P(C.annBg) }, [
  T('chk constraint', 14, B, C.annTxt, { layoutSizingHorizontal: 'FILL', textAutoResize: 'HEIGHT' }).textProp('Heading'),
  T('Note', 14, R, C.annTxt, { layoutSizingHorizontal: 'FILL', textAutoResize: 'HEIGHT', lineHeight: { unit: 'PIXELS', value: 20 } }).textProp('Note'),
]);

await $fig.done();
const sec = $fig.section({ name: 'Components · chk / pelican_ui (dark, brand #EB343A)', x: -1400, y: 600 });
await $fig.done();
const section = sec.node;
// grid variant sets
for (const set of [header.node, button.node, radio.node, input.node, tabBar.node]) {
  const vs = set.children; let y = 20;
  vs.forEach(v => { v.x = 20; v.y = y; y += v.height + 20; });
  set.resizeWithoutConstraints(Math.max(...vs.map(v => v.width)) + 40, y);
}
const items = [statusBar, homeInd, header, button, quick, avatar, listItem, radio, tableRow, input, numDisplay, key, keypad, tabBar, note].map(p => p.node);
let x = 40, y = 40, rowH = 0; const out = {};
for (const n of items) {
  section.appendChild(n);
  if (x + n.width > 1300) { x = 40; y += rowH + 40; rowH = 0; }
  n.x = x; n.y = y; x += n.width + 40; rowH = Math.max(rowH, n.height);
  out[n.name] = n.id;
}
section.resizeWithoutConstraints(1340, y + rowH + 40);
return { section: section.id, comps: out };
