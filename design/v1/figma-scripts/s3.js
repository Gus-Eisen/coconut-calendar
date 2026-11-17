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
