const SECS = ['4:455','4:852','4:1112','4:1285','5:1030','5:1419','6:1701','6:2007'];
const report = [];
for (const sid of SECS) {
  const s = await figma.getNodeByIdAsync(sid);
  const frames = s.children.filter(n => n.type === 'FRAME');
  for (const f of frames) {
    const notes = s.children.filter(n => n.type === 'INSTANCE' && n.name === 'Note · ' + f.name);
    let y = f.y; for (const n of notes) { n.x = f.x + 393 + 24; n.y = y; y += n.height + 16; }
    const content = f.children.find(c => c.name === 'Content');
    const inner = content.children.reduce((a, c) => a + c.height, 0) + Math.max(0, content.children.length - 1) * 24 + 24;
    const tight = [];
    content.query('INSTANCE').each(i => { if (i.name.startsWith('ListItem')) { const L = i.children.find(c => c.name === 'Left'); if (L && L.width < 60) tight.push(i.name + ':' + Math.round(L.width)); } });
    if (inner > content.height || tight.length || y > f.y + 852) report.push({ f: f.name, inner: Math.round(inner), avail: Math.round(content.height), tight, notesBottom: Math.round(y - f.y) });
  }
}
return report;
