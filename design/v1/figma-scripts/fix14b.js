const sec = await figma.getNodeByIdAsync('4:1112');
const f14b = await figma.getNodeByIdAsync('4:1038');
const f15 = await figma.getNodeByIdAsync('4:1070');
const content = f14b.children.find(c => c.name === 'Content');
const oldKids = content.children.map(c => c.id);
const notes14b = sec.children.filter(n => n.type === 'INSTANCE' && n.name === 'Note · ' + f14b.name);
const firstNote = notes14b[0].id;
// 15: change the typed-days values to preset values
const changed15 = [];
f15.query('TEXT').each(t => {
  if (t.characters === 'Weekly on Tue') { changed15.push([t.characters, 'Weekly, same day as the start (Tue)']); }
  if (t.characters === 'Weekly on Tue, Thu') { changed15.push([t.characters, 'Weekly, weekdays (Mon–Fri)']); }
});
const map = Object.fromEntries(changed15);
const fonts = [{ family: 'Outfit', style: 'Regular' }, { family: 'Outfit', style: 'Bold' }];
for (const fn of fonts) await figma.loadFontAsync(fn);
f15.query('TEXT').each(t => { if (map[t.characters]) t.characters = map[t.characters]; });
// 14b: replace the typed field with a single-select radio list (FormItem::enumerator)
for (const id of oldKids) (await figma.getNodeByIdAsync(id)).remove();
const OPTS = ['Same day as the start', 'Weekdays (Mon–Fri)', 'Weekends (Sat–Sun)', 'Every day'];
$fig.get(content.id).append($fig.autoLayout({ name: 'RadioSelector', layoutMode: 'VERTICAL', layoutSizingHorizontal: 'FILL', fills: [] },
  OPTS.map((t, i) => $fig.instance('3:73', { layoutSizingHorizontal: 'FILL', props: { Selected: i === 0 ? 'Yes' : 'No', Title: t, Subtitle: '', 'Show Subtitle': false } }))));
$fig.get(firstNote).set({ props: { Heading: 'chk: single-select only', Note: "Repeat Days is a FormItem::enumerator radio list, and chk's radio list is single-select, so custom day sets (e.g. Mon and Wed only) aren't possible yet. 'Same day as the start' is first, so it's the default. Gus has asked the chk developer to add multi-select; it will replace this list once it lands, but the delivery date is unknown." } });
await $fig.done();
// re-stack notes beside 14b
let y = f14b.y;
for (const n of notes14b) { n.x = f14b.x + 393 + 24; n.y = y; y += n.height + 16; }
$fig.get(f14b.id).screenshot({ scale: 0.5 });
return { removed: oldKids.length, changed15, notes: notes14b.map(n => n.id) };
