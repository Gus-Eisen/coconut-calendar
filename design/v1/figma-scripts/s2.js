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
