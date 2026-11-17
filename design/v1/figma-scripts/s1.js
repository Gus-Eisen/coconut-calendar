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
