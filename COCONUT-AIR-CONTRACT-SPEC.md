# Coconut Calendar Air Contract Spec

Mockups: design v1 (2026-11-16), in `design/v1/`. See `design/CHANGELOG.md`.
Figma (live, may be newer): https://www.figma.com/design/ep0UXdTZLDWLQJgeVrg8Hb
Status: matches design v1, except where it is ahead of it (listed in `design/CHANGELOG.md`). Where they differ, the spec wins.

Bob owns his Coconut Calendar. Ownership means the power to create, modify, and delete events. Bob can extend ownership permissions to others (bots and humans). This means that a bot has varying levels of access to the calendar, as Bob sees fit.

The primary object in Coconut is the Event, it is technically described below, although this technical definition is a work in progress. Events are collected into a BTree. Coconut's codebase pulls these events from the BTree and populates them into appropriate dates in the UI. The "calendar" is an object too: `Calendar` holds the BTree of events, and the UI renders them on-screen.

Every event is stored and synced through AIR, the ramp stack's storage and sync
layer, so an event created on one device shows up on the user's other devices.
The code lives in `src/calendar.rs`.

This spec leads the code: changes are made here first, then the code is updated
to match.

The spec has six sections:

1. [Air Objects and Fields](#1-air-objects-and-fields)
2. [Event methods](#2-event-methods)
3. [Groups](#3-groups)
4. [What each group can do](#4-what-each-group-can-do)
5. [1.0 scope](#5-10-scope)
6. [Open questions from design v1](#6-open-questions-from-design-v1)

Screen numbers such as "see 21b" refer to `design/v1/screens/21b-*.png`.

------

## 1. Air Objects and Fields

`Event` is a public struct with twelve private fields:

| Field              | Type             | Meaning                                                  | Example                       |
|--------------------|------------------|----------------------------------------------------------|-------------------------------|
| `title`            | `String`         | The name of the event, trimmed of surrounding spaces.    | `"Team standup"`              |
| `timing`           | `Timing`         | When the event happens: all day, or at set times in a time zone. Can span several days. | `Timing::Timed { start: 2026-11-10 09:30:00, end: 2026-11-10 10:00:00, time_zone: America/New_York }` |
| `location`         | `Option<String>` | Where the event happens, trimmed. `None` if left blank.  | `Some("Room 4B")`             |
| `info`             | `Option<String>` | Extra notes about the event, trimmed. `None` if left blank. | `Some("Bring the slides")` |
| `attendees`        | `Vec<Attendee>`  | The users invited to the event, each with their response. Empty if there are none. | `vec![Attendee { name: alice, response: Response::Accepted }]` |
| `video_link`       | `Option<String>` | A link to the event's video call, trimmed. `None` if left blank. | `Some("https://meet.example.com/abc-123")` |
| `attachments`      | `Vec<String>`    | Links to files attached to the event. Empty if there are none. | `vec!["https://files.example.com/slides.pdf"]` |
| `recurrence`       | `Option<Recurrence>` | How the event repeats, until when, and which occurrences were cancelled or changed. `None` if it happens once. | `Some(Recurrence { frequency: Frequency::Weekly { days: vec![Weekday::Mon] }, interval: 1, until: Some(2027-06-30), count: None, exceptions: {} })` |
| `category`         | `Category`       | Whether it's a work or personal event. Shown as a text label (see `Category`). | `Category::Work` |
| `time_proposals`   | `Vec<TimeProposal>` | New times that users have proposed and that are still waiting for everyone to agree. Empty if there are none. | `vec![TimeProposal { proposer: charlie, timing: Timing::Timed { .. }, approvals: {bob} }]` |
| `private`          | `bool`           | `true` if only users with `Visibility::Private` can see the event (see section 3). Starts `false`. | `false` |
| `deleted`          | `bool`           | `true` once the whole event has been deleted, including every occurrence if it repeats. Starts `false`. | `false` |

### Supporting types

#### `Timing`

`Timing` is when an event happens. An event is either all-day or at set times,
never both, so it's an enum:

```rust
pub enum Timing {
    AllDay {
        first_day: NaiveDate,
        last_day: NaiveDate,       // same as first_day for a one-day event
    },
    Timed {
        start: NaiveDateTime,      // local time in `time_zone`
        end: NaiveDateTime,        // local time in `time_zone`
        time_zone: Tz,             // e.g. America/New_York
    },
}
```

- **Multi-day events:** an event can last any number of days. For `AllDay`,
  `last_day` must be on or after `first_day`. For `Timed`, `end` must be after
  `start`, and they can be on different days (a 3-day trip, or a flight that
  lands the next morning).
- **Time zones:** a `Timed` event is stored as the clock time where it happens,
  plus that place's time zone. Users see it converted to their own time zone. A
  9:00 meeting in New York shows as 15:00 for someone in Paris.
- **Repeating events keep their clock time:** a weekly 9:00 meeting in New York
  stays at 9:00 New York time when daylight saving time starts or ends. Storing
  the time zone, not just a fixed offset, is what makes this possible.
- **All-day events have no time zone.** A birthday on 10 November is on
  10 November everywhere.
- **Times that don't exist** because clocks spring forward are shifted forward
  one hour: a 2:30 AM start on that day becomes 3:30 AM. This applies when an
  occurrence is worked out, and to what the form shows.
- **Times that happen twice** because clocks fall back (e.g. 1:30 AM) use the
  first one.
- **Times that can't be worked out** at all, because they're past the ends of
  the range of dates chrono supports, make the `Timing` invalid, so `apply`
  ignores the event.
- **Days are half-open:** a day runs from its midnight up to, but not including,
  the next midnight. An event ending exactly at midnight doesn't show on the
  next day.
- `Tz` is the time zone type from the `chrono-tz` crate, with its `serde`
  feature turned on so it can be stored in AIR.

#### `Attendee`

`Attendee` is one invited user and their answer to the invite:

```rust
pub struct Attendee {
    name: Name,         // AIR's user id: the user's public key
    response: Response,
}

pub enum Response {
    Pending,            // hasn't answered yet; every attendee starts here
    Accepted,
    Declined,
    Tentative,
}
```

#### `TimeProposal`

`TimeProposal` is a new time that a user has suggested for the event:

```rust
pub struct TimeProposal {
    proposer: Name,                  // the user who proposed it
    occurrence: Option<NaiveDate>,   // original date of one occurrence; None for the whole event or series
    timing: Timing,                  // the proposed new time
    approvals: BTreeSet<Name>,       // the counterparties who have agreed so far
}
```

A proposal can be for the whole event, or for one occurrence of a repeating
event. A user can have only one open proposal per event and occurrence.
Proposing again replaces their earlier one. That way a proposal can be
identified by its proposer and occurrence. When a proposal for one occurrence
goes through, it becomes an exception that changes that occurrence's `timing`.
For a proposal about one occurrence, an attendee's response for that occurrence
(in `Override::responses`) decides whether they're a counterparty.

The time only changes if every counterparty agrees:

- **Counterparties** are the event's owner and every attendee whose `response`
  is `Accepted`, except the proposer. Proposing a time counts as agreeing to it.
  - Attendees who declined the event have no say.
  - Attendees who haven't accepted the event's current time (`Pending` or
    `Tentative`) have no say either. This includes attendees added after the proposal was made: they
    only have to agree if they've already accepted the current time.
  - The counterparties are worked out each time someone answers, not fixed when
    the proposal is made. So if an attendee accepts the event while a proposal
    is open, they become a counterparty and have to agree too.
- Each counterparty answers for themselves: agree or decline. Anyone with the
  `Owner` role can answer for the owner, and one answer is enough.
- When the last counterparty agrees, the event's `timing` is set to the proposed
  one and the proposal is removed.
- If any counterparty declines, the proposal is removed and the time stays the
  same.
- When an attendee declines the event, check every open proposal: if all
  remaining counterparties have agreed, the time changes.
- If someone with the `Time` action changes the time directly, every open
  proposal on the event is cancelled.

UI: see 28 (proposing a new time) and 29 (a counterparty agreeing or
declining).

#### `Recurrence`

`Recurrence` is how an event repeats, when it stops, and which occurrences were
cancelled or changed. A repeating event never ends, ends on a set date
(`until`), or ends after a number of times (`count`). `Weekday` is
`chrono::Weekday`.

```rust
pub struct Recurrence {
    frequency: Frequency,
    interval: u16,                // every N days, weeks, months or years; 1 to 99
    until: Option<NaiveDate>,     // last date it can start on; None means no end date
    count: Option<u16>,           // how many times in total, 1 to 999; never set with `until`
    exceptions: BTreeMap<NaiveDate, Exception>,  // keyed by the date the occurrence would normally start on
}

pub enum Frequency {
    Daily,
    Weekly { days: Vec<Weekday> },
    Monthly { day_of_month: u8 },
    Yearly,
    MonthlyWeekday { week: WeekOfMonth, weekday: Weekday },  // appended in step 3a
}

pub enum WeekOfMonth { First, Second, Third, Fourth, Last }
```

Postcard stores enum variants by position, so new variants are only appended.
Adding `interval`, `count` and `MonthlyWeekday` in step 3a kept the contract id
`CoconutCalendarV2`: before 3a, `CreateEvent` always stored `recurrence: None`,
which is one byte with no `Recurrence` inside, so no stored data had the old
layout (checked by encoding with the old types and decoding with the new).

A repeating event is stored once; the app works out each occurrence from
`timing` and `recurrence` when it shows the calendar.

**Which dates it picks.** The *start date* is `first_day` (all-day) or the
date of `start` in the event's own zone (timed). The rule picks dates on or
after the start date, and on or before `until`:

- `Daily`: every day. `Weekly`: the date's weekday is in `days`. `Monthly`:
  the date's day is `day_of_month`. `Yearly`: the same month and day as the
  start date. `MonthlyWeekday`: the given weekday in the given week of the
  month.
- The week of the month comes from the day: 1–7 is `First`, 8–14 `Second`,
  15–21 `Third`, 22–28 `Fourth` and 29–31 `Last`. So a start on Mon 30 Nov
  means "last Monday", and one on Mon 23 Nov means "4th Monday" (choosing 4th
  or last is a later option).
- With `interval` N, only every Nth period counts, from the start: days since
  the start date, weeks since the start date's week (weeks start on Monday),
  months since the start month, years since the start year.
- With `count` N, only the first N picked dates count.
- The first occurrence is the first picked date on or after the start date.
  It can be later than the start date (for example weekdays from a Saturday),
  and the stored `timing` keeps the typed date.

Dates a month or year doesn't have are skipped:

- A monthly event on the 31st doesn't happen in months with fewer days (April,
  June, September, November, February). The same goes for the 29th and 30th in
  February.
- A yearly event on 29 February happens only in leap years.

Each occurrence has the same clock times as the first (timed) or the same
number of days (all-day), moved to its date. Times are resolved in the event's
zone with `Timing`'s usual rules, so a 09:00 event stays at 09:00 across
daylight saving. An occurrence whose times can't be worked out is skipped.

**Creating a repeating event (step 3a, decided 2026-11-17).** The event-type
picker has four options: Regular (first, so it's the default), All-Day,
**Repeating** ("Happens again at set times") and **Repeating All-Day** ("Whole
days that repeat, like a birthday") (UI: see 13). Each opens its own linear
form:

| Form | Steps |
|------|-------|
| Repeating | Event Title, Category, Date, Start Time, End Time, Repeats, Every, Until, Times, Location, Info |
| Repeating All-Day | Event Title, Category, First Day, Last Day, Repeats, Every, Until, Times, Location, Info |

- **Repeats** is one single-select radio list (UI: see 14, 14b), replacing the
  old Frequency and Repeat Days steps:

  | Option | Subtitle | Stored as |
  |--------|----------|-----------|
  | Every day | | `Daily` |
  | Every weekday | Mon–Fri | `Weekly { days: Mon..Fri }` |
  | Every weekend | Sat–Sun | `Weekly { days: [Sat, Sun] }` |
  | Every week | On the same day | `Weekly { days: [start weekday] }` |
  | Every month | On the same date | `Monthly { day_of_month: start day }` |
  | Every month | On the same weekday, e.g. 2nd Tue | `MonthlyWeekday` from the start date |
  | Every year | On the same date | `Yearly` |

  The form is built before Date is typed, so the labels can't name the day;
  the review does. *Every day* is stored as `Daily`, never as a 7-day weekly
  set. chk's radio list is single-select, so custom day sets (Monday and
  Wednesday only) wait for chk multi-select.
- **Every** is a number: every N days, weeks, months or years (weekday and
  weekend sets: every N weeks), 1 to 99. Blank means 1.
- **Until** is a `DD-MM` date read with `Event::date_from` from the start date,
  so it's on or after the start and at most a year later. Until on the start
  date means one occurrence. Blank means no end date.
- **Times** is a number, 1 to 999: how many times in total. Blank means no
  limit. With Until also set, the error is "Use Until or Times, not both." Both
  blank means it never ends. Cancelled occurrences (3b) still count.
- Every, Until and Times use chk's number and date keypads, which show fixed
  hints ("Enter number", "Enter date") and start at "0" for a number. "0"
  counts as blank. What blank means shows on the review instead (Ends:
  "Never").
- End Time uses the overnight rule; there's no End Date step. Last Day uses
  the 28-day rule, and every occurrence lasts the same number of days.
- The review adds **Repeats** (how often: "Daily", "Weekly", "Every 2 weeks",
  "Monthly", "Yearly"), **On** (which days: "Tuesday", "Mon–Fri", "The 31st",
  "3rd Tuesday", "Last Monday", "24 Dec"; none for daily), **First** (only when
  it isn't the start date: "Mon 23 Nov 2026", or "No date matches") and
  **Ends** ("Never", "On 30 Jun 2027", "After 10 times"). chk clips long table
  values, so how often and which days are separate rows (`Recurrence::describe`).
- The success message is "Repeating event saved, from 23 Nov 2026 at 09:00"
  or "Repeating all-day event saved, from 24 Dec 2026", naming the first
  occurrence.

**Changing how it repeats** (`set_recurrence`, UI: see 15) is a flow of
Repeats (with an eighth option, **Doesn't repeat**, last), Every, Until and
Times, then a "Change Repeat Review" with Current and New tables of the rows
above. Until is read from the later of the event's start date and today.
Continue sends `set_recurrence` unless nothing changed, and returns to the
detail page. As with changing the time, the current value can only appear on
the review page.

**Edits apply to the whole series in 3a.** Edit details, Change time, Change
category, Change privacy, links and Delete act on the whole series until 3b
adds the scope chooser (UI: see 16).

#### Exceptions

Like Google Calendar, one occurrence of a repeating event can be cancelled or
changed without affecting the rest. An exception is stored on the series, keyed
by the date the occurrence would normally start on (its *original date*), so
it stays part of the same event: same attendees, same AIR instance.

```rust
pub enum Exception {
    Cancelled,
    Changed(Override),
}

/// What's different about one occurrence. `None` means "same as the series".
pub struct Override {
    title: Option<String>,
    timing: Option<Timing>,                  // e.g. moved from Monday to Tuesday
    location: Option<Option<String>>,        // Some(None) clears it for this occurrence
    info: Option<Option<String>>,
    video_link: Option<Option<String>>,
    attachments: Option<Vec<String>>,
    responses: BTreeMap<Name, Response>,     // attendees' answers for this occurrence only
}
```

- **Fields that can't differ per occurrence:** `attendees`, `category`,
  `private` and `recurrence`. Attendees can still answer differently for one
  occurrence through `responses`, for example declining just one Monday.
- **Inviting someone to one occurrence:** the occurrence is *detached*. In one
  message, it's cancelled in the series and a standalone event (no
  `recurrence`) is created with the same details: the series' fields with the
  occurrence's `Override` applied, the series' attendees and their responses
  for that occurrence. An open time proposal for that occurrence moves to the
  standalone event as a whole-event proposal (`occurrence: None`), with its
  approvals kept. The new person is then invited to the standalone event.
  Because it has its own AIR instance, they only ever hold the key to that one
  event. A detached event no longer follows later changes to the series (for
  example a rename). Google behaves the same way.
- **Cancelling is permanent.** A cancelled occurrence can't be restored or
  changed. `restore_occurrence` and `change_occurrence` do nothing on it.
- **`Option<Option<String>>`** is needed for the optional fields because there
  are three cases: same as the series (`None`), cleared for this occurrence
  (`Some(None)`), or a different value (`Some(Some(..))`).
- **Changing the series later:** an occurrence keeps its overridden fields;
  every other field follows the series. For example, if one Monday was moved to
  Tuesday and the series is then renamed, that occurrence stays on Tuesday and
  takes the new name. If the series' `timing` or
  `recurrence` changes so that an exception's original date is no longer an
  occurrence, that exception no longer applies.
- **Original date, not new date:** a Monday occurrence moved to Tuesday is
  still keyed by the Monday. That's how the app knows the Monday occurrence
  shouldn't also appear.

Exceptions are stored on the series, not as separate events. A change to one
occurrence is then one message to one instance, and can never get out of step
with the series.

Every edit or delete of a repeating event offers three choices, as in Google
Calendar:

| Choice | What happens |
| -------- | -------------- |
| **This event** | Adds or updates the exception for that occurrence. |
| **This and following events** | Splits the series in one message (see `split_series` in section 2). |
| **All events** | Changes the series itself. For a delete, the whole event is deleted (`deleted` becomes `true`). |

In design v1 the choice is a radio page shown *before* the edit flow opens,
because each edit is its own flow and a bumper holds only two buttons. Its
submit opens the matching edit flow, or deletes right away (UI: see 16; see
section 6 for what's still open). A changed occurrence's detail page says it
was changed, shows the series time next to its own, and has a **Restore to
series** action (`restore_occurrence`) (UI: see 17).

#### `Category`

`Category` separates work events from personal ones. The views show it as a
text label, "Work" or "Personal". There are no category colors, because chk
can't color list rows or tables (UI: see 07, 11, 18, 19).

```rust
pub enum Category {
    Work,
    Personal,
}
```

New events are `Personal` unless the user picks `Work`.

------

## 2. Event methods

### Methods on `Event`

Getters for every field except `deleted`:

| Method | Signature | What it does |
| -------- | ----------- | -------------- |
| `title` | `fn title(&self) -> &str` | Returns the title. |
| `timing` | `fn timing(&self) -> &Timing` | Returns when the event happens. |
| `location` | `fn location(&self) -> Option<&str>` | Returns the location, if there is one. |
| `info` | `fn info(&self) -> Option<&str>` | Returns the notes, if there are any. |
| `attendees` | `fn attendees(&self) -> &[Attendee]` | Returns the attendees. |
| `video_link` | `fn video_link(&self) -> Option<&str>` | Returns the video call link, if there is one. |
| `attachments` | `fn attachments(&self) -> &[String]` | Returns the attachment links. |
| `recurrence` | `fn recurrence(&self) -> Option<&Recurrence>` | Returns how the event repeats, if it does. |
| `category` | `fn category(&self) -> Category` | Returns work or personal. |
| `time_proposals` | `fn time_proposals(&self) -> &[TimeProposal]` | Returns the open time proposals. |
| `private` | `fn private(&self) -> bool` | Returns whether the event is private. |

There's no public getter for `deleted`; only `Calendar` (in the same module)
checks it.

Helpers for the new-event forms:

| Method | Signature | What it does |
| -------- | ----------- | -------------- |
| `window_start` | `fn window_start() -> NaiveDate` | The first day of the current month, where the month list starts. |
| `date` | `fn date(input: &str) -> Option<NaiveDate>` | Reads a `DD-MM` date as the first such date on or after today: `date_from(input, today)`. Used for every first date (Date, First Day). `None` if the input isn't a valid date. |
| `date_from` | `fn date_from(input: &str, earliest: NaiveDate) -> Option<NaiveDate>` | Reads a `DD-MM` date as the first such date on or after `earliest`: it tries `earliest`'s year, then the next year. `None` if neither gives a valid date on or after `earliest` (e.g. `29-02` when the next 29 February is more than a year away). |
| `time` | `fn time(input: &str) -> Option<NaiveTime>` | Reads an `HH:MM` time (24-hour). `None` if invalid. |
| `from_states` | `fn from_states(states: &[State]) -> Event` | Builds an `Event` from either new-event form's inputs, reading the timing steps with `Timing::from_states` in the device's time zone. Sets `deleted: false`. The form checks the inputs first, so this panics if they're invalid. It only ever reads form inputs, never AIR data. |
| `optional` | `fn optional(input: &str) -> Option<String>` | Trims a text. Returns `None` if it's empty. The forms and `apply` both use it, so they share one rule. |

### Methods on `Timing`

| Method | Signature | What it does |
| -------- | ----------- | -------------- |
| `device_zone` | `fn device_zone() -> Tz` | The device's time zone, or UTC if the device reports one that `chrono-tz` doesn't recognise. |
| `is_valid` | `fn is_valid(&self) -> bool` | `AllDay`: `last_day` is on or after `first_day`. `Timed`: both times can be worked out in `time_zone`, and `end` is after `start`. `apply` and both forms use this one rule. |
| `from_times` | `fn from_times(date: NaiveDate, start: NaiveTime, end: NaiveTime, time_zone: Tz) -> Option<Timing>` | The regular form's timing: ends on `date`, or the next day if `end` is earlier than `start`. `None` if they're equal. Callers still check `is_valid`. |
| `from_states` | `fn from_states(states: &[State], time_zone: Tz) -> Timing` | Reads a form's timing steps: First Day and Last Day (`AllDay`, Last Day by `Event::date_from` the first day); Date, Start Time and End Time (the overnight rule of `from_times`); or Date, Start Time, End Date and End Time (Change time, End Date by `Event::date_from` the date). The form checks the inputs first, so this panics if they're invalid. |
| `in_zone` | `fn in_zone(&self, zone: Tz) -> Option<(NaiveDateTime, NaiveDateTime)>` | The start and end as clock times in `zone`. `None` for an all-day event. |
| `covers` | `fn covers(&self, date: NaiveDate, zone: Tz) -> bool` | Whether any part of the event falls on `date` in `zone` (days are half-open). All-day events ignore `zone`. |

`in_zone` and `covers` expect a `Timing` that `is_valid`, which is true of
every stored event because `apply` checks it. On a `Timed` value whose times
can't be worked out, they panic.

### Methods on `Category`

| Method | Signature | What it does |
| -------- | ----------- | -------------- |
| `options` | `fn options() -> [Category; 2]` | The categories in the order the forms list them, `Personal` first. |
| `label` | `fn label(self) -> &'static str` | `"Work"` or `"Personal"`. |

### The new-event forms

The month screen has one button, **Create Event** (UI: see 01). It opens an
**event-type picker**: a one-step `Form::flow` with a radio list
(`FormItem::enumerator`) of *Regular* ("Starts and ends at set times") and
*All-Day* ("Lasts one or more whole days") (UI: see 02). Regular is first, so
it's the default. The picker has no review or success page; its submit returns
`FormComplete::Return`, which resets navigation and opens the chosen form.
Repeating and Repeating All-Day were added in step 3a (see `Recurrence`).

A chk form can't have optional or conditional steps, so each kind of event has
its own form, one field per page (UI: see 03 and 04):

| Form | Steps |
|------|-------|
| Regular (timed) | Event Title, Category, Date, Start Time, End Time, Location, Info |
| All-day | Event Title, Category, First Day, Last Day, Location, Info |

- Dates are `DD-MM` (`Event::date`), and times are `HH:MM` (`Event::time`).
- The new-event forms have no End Date step (Gus, 2026-11-16). The end date
  is worked out from End Time: if End Time is later than Start Time, the event
  ends on Date; if it's earlier, the event ends the next day (23:00 to 01:00
  is an overnight event). An End Time equal to Start Time is an error ("Must
  be different from the start."). So a timed event made with this form lasts
  under 24 hours. Longer timed events are made by changing the end with
  Change time (UI: see 09), which keeps its End Date step; events with set
  hours on several days are a daily repeating event (step 3a); and whole days
  are an All-day event.
- Date and First Day are the first such date on or after today (`Event::date`),
  so new events run from today to about a year ahead.
- Last Day (and Change time's End Date) is the first such date on or after
  the start date (`Event::date_from`). If that's more than 28 days after the
  start, the error is "Must be on or after the start.": a date typed before the
  start comes out about a year later, so it hits this error. For example, First
  Day `30-12` and Last Day `02-01` is 2 Jan of the next year; First Day `17-11`
  and Last Day `16-11` is an error. The 28-day limit is a form rule only: it
  isn't in `Timing::is_valid` or `apply`, so longer events from elsewhere are
  still accepted. End Time is checked with `Timing::is_valid`.
- A chk validator only sees its own step. So the later steps read the earlier
  answers from state the form's validators share. An invalid answer shows an
  inline error and turns Continue off (UI: see 03).
- Date and time inputs can't be pre-filled, so End Time is typed every time.
- The review shows the full end date and time, so a wrong overnight guess is
  caught before saving.
- The review shows Title, Category, the timing (Start, End and Time zone, or
  First day and Last day), and Location and Info if they're filled in. All-day
  events have no time-zone row, because their dates are the same everywhere
  (UI: see 05).
- The success message names the start date and time, or the first day (and
  the last day if it's different). Done returns to the month screen (UI: see
  06).

`DayScreen::time_label(timing, day, zone) -> String` is how the views show an
occurrence's time: "All day", or the start and end in `zone`. Any part that isn't on
`day` also shows its date, e.g. "23:00–01:00 (16 Nov)".

### Methods on `Calendar`

All of an owner's events live in their `Calendar`, which holds a
`BTreeMap<EventId, Event>`. Every change goes through it as a message to AIR.
When a message is applied, `Calendar` checks that the sender's role allows the
action (section 4) and ignores it if not. Some messages need several actions
(for example `split_series`). The sender's role must allow *every* one of them,
or the whole message is ignored, so a message is never half-applied. AIR checks each message's signature
before it's applied, so the sender (`metadata.signer`) can be trusted, and
every device makes the same decision.

AIR stores data with postcard, which can't read old data once a stored type
changes. So changing `Event` or `Calendar` needs a new `Calendar` contract id.
That's fine before release, because nobody uses Coconut yet.

**Opening and viewing:**

| Method | Signature | What it does |
|--------|-----------|--------------|
| `open` | `fn open(ctx: &mut Context) -> Instance<Calendar>` | Opens the user's own calendar, creating it the first time. It finds the calendar by its own location (`hash(init, signer)`), never by taking the first one in the user's list: once calendars are shared, other people's calendars show up in the same list. |
| `event` | `fn event(&self, id: EventId) -> Option<&Event>` | The event with `id`, for its detail page. `None` if there's none, or it's deleted. |
| `on` | `fn on(&self, date: NaiveDate, zone: Tz) -> Vec<Occurrence<'_>>` | Everything happening on `date` in the viewer's time zone `zone`, sorted with all-day events first, then by start time. Includes multi-day events that cover `date`, and each occurrence of repeating events. Leaves out deleted events and cancelled occurrences. |

`Occurrence` is one appearance of an event on a day: the event's `EventId`, a
reference to the `Event`, the occurrence's original date (needed to change or
cancel just that occurrence), and its `Override` if it has one. For an event
that doesn't repeat, there's only one. A moved occurrence appears on its new
date, not its original one. Since step 3a, `Occurrence` has `date()` (the
original date, in the event's zone) and `timing()` (this occurrence's own
`Timing`; the event's `timing` is only the first one's). Views use `timing()`,
and `on` sorts by it. The `Override` comes with exceptions (step 3b).

`on` doesn't walk a series from its start. For each event it checks only the
start dates that could reach `date`: from `date` minus the event's length in
days, minus 2, to `date` plus 2. Zones are up to 26 hours apart, so moving an
occurrence into the viewer's zone shifts it by at most 2 days. With `count`,
the last occurrence is found once per check by stepping through at most
`count` occurrences, a period at a time, and then treated like `until`.

**Creating and deleting:**

| Method | Signature | Action | What it does |
| -------- | ----------- | -------- | -------------- |
| `create_event` | `fn create_event(calendar: &mut Instance<Calendar>, event: Event)` | `Create` | Gives the event a random `EventId` and sends it to AIR as `CalendarMessage::CreateEvent { id, event: Box<Event> }`. The `Event` is boxed because it's much bigger than the other messages (clippy's `large_enum_variant`). `apply` ignores an event with a blank title or a `Timing` that isn't valid. |
| `delete_event` | `fn delete_event(calendar: &mut Instance<Calendar>, id: EventId)` | `Delete` | Deletes the whole event. For a repeating event, cancels every occurrence. |
| `cancel_occurrence` | `fn cancel_occurrence(calendar: &mut Instance<Calendar>, id: EventId, date: NaiveDate)` | `Delete` | Cancels one occurrence of a repeating event, identified by its original date. The others are unaffected. |

**Changing one occurrence:**

| Method | Signature | Action | What it does |
| -------- | ----------- | -------- | -------------- |
| `change_occurrence` | `fn change_occurrence(calendar: &mut Instance<Calendar>, id: EventId, date: NaiveDate, changes: Override)` | The action for each field it changes (e.g. `Time` for `timing`) | Sets the exception for the occurrence on its original date `date`. Replaces any earlier changes to that occurrence. Does nothing if the occurrence is cancelled. |
| `restore_occurrence` | `fn restore_occurrence(calendar: &mut Instance<Calendar>, id: EventId, date: NaiveDate)` | The actions for the fields it had changed | Removes a `Changed` exception, so the occurrence goes back to matching the series. Does nothing if the occurrence is cancelled. |
| `detach_occurrence` | `fn detach_occurrence(calendar: &mut Instance<Calendar>, id: EventId, date: NaiveDate)` | `Delete` and `Create` | Turns one occurrence into a standalone event (see "Inviting someone to one occurrence" in section 1). Sends `CalendarMessage::DetachOccurrence { id, date, new_id }`, with a random `new_id` for the new event. |
| `split_series` | `fn split_series(calendar: &mut Instance<Calendar>, id: EventId, from: NaiveDate, changes: Override)` | `Recurrence`, `Create`, and the action for each field it changes | "This and following events". Sends `CalendarMessage::SplitSeries { id, from, new_id, changes }`, with a random `new_id` for the new series. |

`split_series` must be one message. As two messages (change `until`, then
create the new series), the series would silently stop early if the second
never arrived, and AIR's pending state has no way to report that. When
`SplitSeries` is applied, all of this happens at once:

- The old series' `until` is set to the day before `from`.
- A new series is created starting on `from`, with `changes` applied and the
  same attendees.
- Every exception dated on or after `from` is **moved** to the new series, so
  none are left behind where they no longer apply. Open time proposals for
  occurrences on or after `from` move with them.
- Open time proposals for the whole series (`occurrence: None`) are cancelled.
  Nobody agreed to a time for the new series, and keeping them on only the old
  half would quietly change what people agreed to.
- Attendees' responses to the series carry over to the new series.

**Editing:** each method replaces one field, or adds or removes one item.

| Method | Signature | Action |
| -------- | ----------- | -------- |
| `set_title` | `fn set_title(calendar: &mut Instance<Calendar>, id: EventId, title: String)` | `Title` |
| `set_timing` | `fn set_timing(calendar: &mut Instance<Calendar>, id: EventId, timing: Timing)` | `Time` |
| `set_info` | `fn set_info(calendar: &mut Instance<Calendar>, id: EventId, info: Option<String>)` | `Info` |
| `set_location` | `fn set_location(calendar: &mut Instance<Calendar>, id: EventId, location: Option<String>)` | `Location` |
| `set_recurrence` | `fn set_recurrence(calendar: &mut Instance<Calendar>, id: EventId, recurrence: Option<Recurrence>)` | `Recurrence` |
| `add_attendee` | `fn add_attendee(calendar: &mut Instance<Calendar>, id: EventId, name: Name)` | `Attendee` |
| `remove_attendee` | `fn remove_attendee(calendar: &mut Instance<Calendar>, id: EventId, name: Name)` | `Attendee` |
| `set_video_link` | `fn set_video_link(calendar: &mut Instance<Calendar>, id: EventId, link: Option<String>)` | `VideoLink` |
| `add_attachment` | `fn add_attachment(calendar: &mut Instance<Calendar>, id: EventId, link: String)` | `Attachment` |
| `remove_attachment` | `fn remove_attachment(calendar: &mut Instance<Calendar>, id: EventId, link: String)` | `Attachment` |
| `set_category` | `fn set_category(calendar: &mut Instance<Calendar>, id: EventId, category: Category)` | `Category` |
| `set_private` | `fn set_private(calendar: &mut Instance<Calendar>, id: EventId, private: bool)` | `Privacy` |

`set_timing` also cancels every open time proposal on the event.

`set_recurrence` sends `CalendarMessage::SetRecurrence { id, recurrence }`,
appended after `RemoveAttachment` in step 3a. `None` stops the event
repeating.

**Checking a recurrence.** `Recurrence::is_valid(&self, timing: &Timing) ->
bool` is the one rule the forms and `apply` share. A recurrence is valid when:

- `Weekly { days }` isn't empty. `apply` first sorts `days` Mon..Sun and drops
  duplicates (it normalises, it doesn't reject).
- `Monthly { day_of_month }` equals the start date's day (so it's 1 to 31).
- `MonthlyWeekday { week, weekday }` matches the start date's week of the
  month and weekday.
- `interval` is 1 to 99, `count` (if set) is 1 to 999, `until` (if set) is on
  or after the start date, and `until` and `count` aren't both set.

In `apply`:

- `CreateEvent` keeps a valid recurrence. An invalid one, or one with
  exceptions, makes it ignore the whole event.
- `SetRecurrence` ignores the incoming exceptions and keeps the event's own
  (empty in 3a; in 3b only their own messages change them). It ignores the
  message if the result isn't valid.
- `SetTiming` on a repeating event keeps the rule meaningful when the start
  date moves: `Monthly` takes the new start's day, `MonthlyWeekday` the new
  start's week and weekday, and a one-day `Weekly` set on the old start's
  weekday (the "same day" option) moves to the new start's weekday. Other
  weekly sets stay. If the result isn't valid (for example `until` is now
  before the start), it ignores the whole message, so Change time's Date step
  checks this too ("Must be on or before the repeat ends.").

**Editing screens (step 2).** Tapping an event in the day view opens its
**event detail** page (UI: see 07): a table of Time, Category, Location, Info,
Private and Video link, the attachment links, and the actions Edit details,
Change time, Change category, Change privacy and Delete. Since step 3a, the
Time row is the tapped occurrence's time, a repeating event adds Repeats, On,
First and Ends rows after Time, and there's a Change repeat action. On the day
screen, a repeating event's subtitle is its category plus "Repeats". chk can only pre-fill text inputs, so
editing is split:

| Screen | What it changes | How |
|--------|-----------------|-----|
| Edit details (UI: see 08) | Title, Location, Info, Video link | One page (`Form::page` / `PageType::edit_and_display`) with a `FormItem::text_with_preset` for each field, filled with the current values. Save stays off until something changes and every field is valid. Calls the setter for each changed field. |
| Change time (UI: see 09) | `timing` | A flow: Date, Start Time, End Date, End Time, then a review showing the current and new times. Step titles are plain ("Start Time", "End Time"): chk clips longer ones. Date is on or after today; End Date uses the same rule as Last Day (on or after the start, at most 28 days later). Times are typed and shown in the event's own time zone (the `time_zone` in its `Timing`), and the review names it. An all-day event changed to timed uses the device's zone. Calls `set_timing`, unless the timing is unchanged. |
| Change to all-day or timed (UI: see 10) | `timing` | A picker like the event-type picker (Timed / All-day, with the current timing in the first option's subtitle), then the matching timing flow. |
| Change category or privacy (UI: see 11) | `category`, `private` | A one-step radio page: Personal / Work, or Not private / Private. The radio always starts on its first option, so the current value is shown as that option's subtitle. Calls `set_category` or `set_private`. |
| Add attachment link (UI: see 12) | `attachments` | A text step with a Paste action. Calls `add_attachment`. Each link on the detail page has a Delete action (`remove_attachment`). No file upload. |
| Change repeat (UI: see 15) | `recurrence` | See `Recurrence`. |

Delete on the detail page has no confirmation step and no undo (UI: see 07).

**Answering and proposing:** these depend on who the sender is, not their role.

| Method | Signature | Who can use it |
| -------- | ----------- | ---------------- |
| `respond` | `fn respond(calendar: &mut Instance<Calendar>, id: EventId, occurrence: Option<NaiveDate>, response: Response)` | An attendee, for their own response only. With `occurrence`, answers for that one occurrence. |
| `propose_time` | `fn propose_time(calendar: &mut Instance<Calendar>, id: EventId, occurrence: Option<NaiveDate>, timing: Timing)` | Anyone with the `ProposeTime` action. With `occurrence`, proposes moving just that one. |
| `answer_proposal` | `fn answer_proposal(calendar: &mut Instance<Calendar>, id: EventId, proposer: Name, occurrence: Option<NaiveDate>, agree: bool)` | A counterparty (see `TimeProposal`). |

Deleting doesn't remove anything, because data sent to AIR is permanent. A
deleted event or cancelled occurrence is only marked, and the app stops showing
it. Deletion and cancellation are:

- **Permanent:** there's no undo or restore.
- **Synced:** they go through AIR like every other change, so they show on
  every device.
- **Immediate on this device:** the app shows changes this device has sent
  before AIR confirms them.

### Inviting people

In 1.0, only other Coconut users can be invited. Each user is identified by
their AIR `Name`.

1. **Each event with attendees gets its own AIR instance.** If attendees were
   given the owner's calendar instance, they could read every event in it.
   Instead, an event with attendees lives in a separate instance shared only
   with them. The owner's calendar keeps a reference to it.
2. **The invite is delivered through AIR's inbox.** Adding an attendee calls
   `Instance::share(name)` on the event's instance (ramp2), which sends its
   `Location` to the attendee's `Name` through AIR's `Inbox`. Nothing needs
   setting up. On the attendee's side, the shared instance appears in their
   list of instances, and Coconut shows the event with the response `Pending`.
   AIR has no accept step and no filter: every shared instance arrives.
   In 1.0, Coconut shows invites from anyone.
3. **Attendees answer inside the event's instance.** `respond`, `propose_time`
   and `answer_proposal` are messages to that instance, signed by the attendee.
   So the owner sees each answer, and who gave it.
4. **Removing an attendee** takes them off the list, but they still have the
   instance's key. To stop them seeing later changes, the event has to move to a
   new instance, which is then sent to the remaining attendees.

Screens (step 6):

- **Invite people** (UI: see 24): `FormItem::search` over the people this
  device knows, with a "New contact" action, the same pattern as Orange's "Add
  members". Submit calls `add_attendee` for each person picked. There's no
  "optional guest" flag.
- **Attendees** (UI: see 25): one row per attendee with their response, plus
  Invite people and Remove someone actions. Attendees can always read this list,
  because they hold the event's instance.
- **Invites** is a root tab (UI: see 26). It lists invites waiting for an answer
  (marked with a dot) and answered ones.
- **Invitation** (UI: see 27): the invitee's view of an event, with the actions
  Accept, Tentative, Decline and Propose new time (`respond`, `propose_time`).
  For a repeating event, a one-step picker first asks "This occurrence" or
  "All".
- **Propose new time** (UI: see 28): for a repeating event, a choice of this
  occurrence or the whole event first, then Date, Start Time, End Date, End
  Time, a review of the current and proposed times, and Success. There's no
  message field.
- **Proposal review** (UI: see 29): what a counterparty sees. It shows who
  proposed, the current and proposed times, and who has agreed so far, with
  Agree and Decline buttons (`answer_proposal`).

Open questions:

- **For AIR's developer:** where does a display name come from? A
  `Name` is only a public key, and nothing in AIR or ramp2 provides a name. If
  AIR won't, Coconut could store display names itself.

------

## 3. Groups

Bob has friends and colleagues who need to interact with his calendar (more specifically, his events). Bob has the option to assign roles to any person (or bot) that he wants. The assignment of roles is best understood as ownership permissions. Bob might favor his personal assistant, Alice, to have full control over his calendar. This means the ability to create, modify, and delete events as Alice sees fit, without Bob's explicit approval prior to each action. Bob has a colleague, Charlie, who needs to ability to propose new event times. Bob can thus assign the role "colleague" to Charlie. This role is more limited in scope than "personal assistant" in that it does not have the power to create or delete events, it only can propose a new time for an event.

An Coconut event is digital and thus differs from the material world. The creator of an event is the owner by default. Ownership is defined by control, ie, the power to modify the event's title, info box, time, and attendee list.

I'm also (currently) imagining roles as an enum:

```rust
pub enum Role {
    Owner,
    Colleague,
}
```

| Role | Who | Visibility | Example |
|------|-----|------------|---------|
| `Owner` | The calendar's owner, and anyone the owner trusts with full control, such as a personal assistant or the owner's own agent. | `Private` | Bob, Alice |
| `Colleague` | Someone who only needs to propose new event times. | `Details` | Charlie |

The calendar's creator can never lose the `Owner` role. Other Owners can't remove
it, and neither can anyone else, including the creator themselves. The creator is the user who created the
calendar's AIR instance; AIR passes their `Name` to the contract's `init` as
`metadata.signer`.

Screens (step 5):

- **People** is a root tab (UI: see 20). It lists everyone with a role on the
  calendar and their role, with the creator marked "created this calendar", and
  has an Add Person button.
- **Add Person** (UI: see 21, 21b): `FormItem::search` over the people this
  device knows (with "New contact"), then a **Role** radio list, then Review and
  Success. The role list puts **Colleague first**, so the least powerful role is
  the default and giving someone full control always takes a deliberate tap.
  Each option's subtitle says what the role can do: Colleague "Can propose new
  times. Sees event details."; Owner "Full control: sees private events."
- **Person detail** (UI: see 22): their role, what they see and what they can
  do, with Change role and Remove. Remove asks for confirmation (Remove /
  Cancel). The creator's page has no Remove.

### Visibility

Each role has a visibility: how much of the calendar its users can see.

```rust
pub enum Visibility {
    None,       // sees nothing
    FreeBusy,   // only when the owner is busy: start and end times, no details
    Details,    // full details of events that aren't private
    Private,    // everything, including private events
}
```

Roles control *changes* by checking the sender's role when a message is applied.
That can't control *reading*: anyone who has an AIR instance's `Location` has
its key and can read everything in it. So visibility is enforced by splitting
the calendar into separate AIR instances, and only giving a user the instances
their visibility allows:

| Instance | Contains | Given to |
| ---------- | ---------- | ---------- |
| Busy times | Start and end times of every event, with no other details | `FreeBusy`, `Details`, `Private` |
| Events | Full details of events that aren't private | `Details`, `Private` |
| Private events | Full details of private events | `Private` |

Events with attendees also have their own instance (see "Inviting people" in
section 2). The "Events" or "Private events" instance holds a reference to it.

Costs of this design:

- **One change can touch several instances.** For example, moving an event that
  isn't private updates it in both "Events" and "Busy times", and the two must
  stay in sync.
- **Access can't simply be taken away.** Once a user has an instance's
  `Location`, they have its key for good. Removing their role means creating a
  new instance with a new key, copying the data over, and re-sharing it with
  everyone else who should still have it.
- **Changes still need role checks.** Visibility only covers reading.

Screens (step 7):

- **Shared** is a root tab (UI: see 30). It lists the people whose calendars
  are shared with the user, and what the user can see of each ("event details",
  "busy times only").
- Tapping one opens that calendar's month and day views, read-only: no Create
  Event, edit or delete (UI: see 31). With `FreeBusy` visibility, days show
  "Busy" rows with times only, no titles.

------

## 4. What each group can do

Bob's powers over events are described as actions, ie, the list of options Bob has at his disposal. I'm currently imagining actions as an enum, defined below.

```rust
pub enum Action {
    Create,
    Delete,
    Title,
    Time,
    Info,
    Location,
    Recurrence,
    Attendee,
    VideoLink,
    Attachment,
    ProposeTime,
    Privacy,
    Category,
}
```

| Action | `Owner` | `Colleague` |
| -------- | :--------: | :-----------: |
| `Create` | ✅ | ❌ |
| `Delete` | ✅ | ❌ |
| `Title` | ✅ | ❌ |
| `Time` | ✅ | ❌ |
| `Info` | ✅ | ❌ |
| `Location` | ✅ | ❌ |
| `Recurrence` | ✅ | ❌ |
| `Attendee` | ✅ | ❌ |
| `VideoLink` | ✅ | ❌ |
| `Attachment` | ✅ | ❌ |
| `ProposeTime` | ✅ | ✅ |
| `Privacy` | ✅ | ❌ |
| `Category` | ✅ | ❌ |

| Action | Changes the field |
| -------- | ------------------- |
| `Create` | Adds a new event |
| `Delete` | `deleted` (the whole event), or a `Cancelled` exception (one occurrence) |
| `Title` | `title` |
| `Time` | `timing`, directly, without a proposal. Includes switching between all-day and set times. |
| `Info` | `info` |
| `Location` | `location` |
| `Recurrence` | `recurrence`'s `frequency` and `until` |
| `Attendee` | `attendees` (adding and removing attendees) |
| `VideoLink` | `video_link` |
| `Attachment` | `attachments` (adding and removing links) |
| `ProposeTime` | `time_proposals` (adding a proposal) |
| `Privacy` | `private` (moves the event between the "Events" and "Private events" instances) |
| `Category` | `category` |

### Who made each change

Every change is recorded with the user who made it (`metadata.signer`). Changes
an agent makes are attributed to the agent, not the owner.

Agents and other users join the owner's calendar with their own identity, never
with the owner's credentials. AIR signs every message with the sender's key, so a
user can't make a change look like someone else made it.

### Open questions

- Assigning roles isn't an `Action` yet, though design v1 has screens for it
  (UI: see 21, 21b, 22). Presumably only `Owner` can do it.
- Nothing enforces roles in the code yet. Only the owner writes to their own
  `Calendar`. Roles will be enforced in `Calendar::apply`, and `init` will store
  the creator from `metadata.signer`.
- **For AIR's developer:** will AIR get a way to revoke someone's
  access to an instance or rotate its key? Nothing exists in air 4.0.1 or
  ramp2 3.0.4. Until then, removing a role means rebuilding instances (see
  section 3, Visibility).

------

## 5. 1.0 scope

The goal for 1.0 is the functionality of Google Calendar, with Coconut's
ownership and permissions model.

### In 1.0

- **One calendar per user.** There's no support for several calendars. Work and
  personal events are told apart by `Category` and its text label instead.
- **Time zones and multi-day events** (see `Timing`).
- **Repeating events**, with exceptions: any occurrence can be changed or
  cancelled on its own, as in Google Calendar ("this event", "this and
  following events", "all events").
- **Editing events** (the editing methods in section 2).
- **Inviting other Coconut users** (see "Inviting people" in section 2).
- **Roles, visibility and time proposals** (sections 3 and 4).
- **Root tabs:** Calendar (the month view), Invites, People and Shared are
  separate roots, which pelican shows as a tab bar on iPhone (UI: see 19, 20,
  26, 30). pelican has no calendar or envelope icon, so the tabs use Home,
  Messages, Group and Explore.
- **Two views,** plus the event detail page (see "Editing screens" in
  section 2):

| View | Shows |
|------|-------|
| Month | chk has no grid component, so it's a list, not a grid: one section per month, for 13 months from `Event::window_start()` (so a date nearly a year ahead is on screen), and one row per day with the day number, the weekday and a summary on the right. A day with one event shows its time (or "All day"), title and category label, e.g. "09:30 Standup · Work". Today is marked with a dot and "Today" in the subtitle. A multi-day event appears on every day it covers. Tapping a day with events opens the day view. The root's one header icon is free (UI: see 01, 19). |
| Day | Every occurrence on one day, from `Calendar::on`: all-day events first, then the rest by start time. It's a list, not an hour grid. Each row shows the title, the category label (with the location, or that it repeats), and the time in the viewer's time zone (with the date too if it starts or ends on another day). Tapping a row opens the event detail page, which has edit and delete. Editing or deleting an occurrence of a repeating event asks: this event, this and following events, or all events (UI: see 16). A display page can't swipe, so going to another day means going back to the month (UI: see 18). |

**Month summary for a day with several events (planned, likely step 4).** The
row shows the next event that hasn't ended yet. An event that's in progress
stays shown until it ends. For example, with Foo at 09:00 for 30 minutes and
Bar at 14:00, the row shows Foo at 09:15 and Bar at 11:00. Until this is built,
such a day shows "N events" (UI: see 01, 19).

### Order of work

Agreed with Will. Steps 1 to 5 keep every event in the one `Calendar`
instance. The per-event instances (step 6) and the visibility split (step 7)
change *where* events are sent, not what an `Event` is.

1. Data model: `Timing` (with `chrono-tz`), `Category`, the new fields and
   their getters, the event-type picker and the new-event forms (UI: see
   01–06).
2. Editing methods and their `CalendarMessage` variants, and the event detail
   and editing screens (UI: see 07–12).
3. a. Repeating events: `Recurrence`, working out occurrences in
   `Calendar::on`, the skipped-date and daylight-saving rules, and the
   Repeating option and form (UI: see 13–15).

   b. Exceptions: `Exception`, `cancel_occurrence`, `change_occurrence`,
   `restore_occurrence`, `SplitSeries`, `DetachOccurrence` (UI: see 16, 17).
4. Views: day and month, including the month summary for days with several
   events (UI: see 18, 19).
5. Attribution and roles, and the People tab (UI: see 20–23).
6. Invites and time proposals, and the Invites tab. A test of
   `Instance::share` between two identities comes first (UI: see 24–29).
7. Visibility, and the Shared tab (UI: see 30, 31).

### After 1.0

- Reminders and notifications
- Search
- Week view
- Inviting people who don't use Coconut (for example by email)
- Only showing invites from the user's contacts. This needs a contacts app in
  the ramp stack, which doesn't exist yet.

------

## 6. Open questions from design v1

Things the mockups show, or the Google Calendar study raised, that the spec
doesn't decide yet. The open questions in sections 2 and 4 still stand.

**New events**

- **Ending a regular event.** Decided 2026-11-16: no End Date step; the
  end date comes from the overnight rule (see the new-event forms). Still
  open: a default length instead of typing End Time? (UI: see 03)
- **Time zone in the forms.** Change time: decided 2026-11-17, it uses the
  event's own time zone and names it on the review (an all-day event becoming
  timed takes the device's zone). Still open: the new-event forms use the
  device's time zone (UI: see 05). Is there a time-zone step in 1.0, and if so,
  a typed zone name or a short list?
- **Date window.** Decided 2026-11-17: `DD-MM` has no year, so events can be
  created from today to about a year ahead (`Event::date`). A past day can't be
  picked (UI: see 04).
- **Quick create, search, schedule view.** Are any of these wanted early? The
  month root's one header icon is free and could open Search or "Go to date"
  (UI: see 19).

**Editing**

- **Saving several fields at once.** Decided 2026-11-17: one Save on Edit
  details (UI: see 08) sends one message per changed field, comparing after
  trimming, so an unchanged field sends nothing.
- **Changing privacy.** Decided: 07 gets a Change privacy action that opens screen 11's privacy radio. The Figma mockup of 07 still needs the button added.

**Repeating events and exceptions**

- **Repeat Days when Frequency isn't Weekly.** Decided 2026-11-17: Frequency
  and Repeat Days are one Repeats step with seven options (see `Recurrence`).
- **Repeating all-day events.** Decided 2026-11-17: a fourth picker option,
  Repeating All-Day, with its own form.
- **Repeat options.** Decided 2026-11-17: every N, the nth weekday of the
  month, and ending after N times are in 1.0. Still open: choosing 4th or
  last weekday when the start is on the 22nd–28th and also the month's last.
- **A start date that doesn't match the rule.** Decided 2026-11-17: the first
  occurrence is the first matching date on or after it; the review shows First.
- **Editing a repeating event in 3a.** Decided 2026-11-17: every edit applies
  to the whole series until 3b.
- **Ask the scope before or after editing?** Design v1 asks first (UI: see 16);
  Google asks after Save. Is asking first OK?
- **"This and following" on the first occurrence.** The mockup note suggests
  hiding it there, as Google does (UI: see 16). `split_series` from the first
  date would leave an old series that ends before it starts.
- **"Google behaves the same way"** (detaching an occurrence when inviting
  someone to it, in `Exception`). Google's API guide suggests it keeps such an
  occurrence as an exception of the series. Check or reword.

**Views**

- **Month summary after the day's events have ended.** With the planned
  next-event rule, what does the row show once every event that day has ended?
  And does an all-day event count as not ended for the whole day?

**Invites and proposals**

- **Proposals for all-day events.** Google doesn't allow them; the spec does.
  Keep? (UI: see 28)
- **Removing an unwanted invite.** An attendee has no way to remove or hide an
  invite from their own calendar, and in 1.0 anyone can invite them (UI: see
  26).
- **Declined events.** What does an attendee see after declining? Google has a
  "Show declined events" setting.

**Roles and history**

- **Display names.** chk's `Profile` contract stores a `username` and `avatar`
  for each `Name`, and the people pickers use it (UI: see 21, 24). That may
  answer the display-name question in section 2, if Coconut can read the same
  profiles.
- **Change history.** Screen 23 sketches a list of who changed what, from each
  message's `metadata.signer`, but it isn't in the order of work (UI: see 23).

------

## Notes

Roles might become confusing to Bob. I need to explain to Bob exactly which actions each role is capable of performing. (Design v1: each option on the role step says what the role can do. See 21b.)

I should implement a color system (red for work events, blue for personal, etc). This would allow for various roles to have access to different types of events in Bob's calendar. Work colleagues should not have the ability to see Bob's doctor appointments. (Design v1: chk can't color rows, so the category is a text label instead. See 11, 18, 19.)

If Bob has a trusted agent, he in theory has the ability to only go "hands on" one initial time with Coconut (assigning Owner to his agent). Bob could then only interact with his agent in his it's own UI, who would then interact with Coconut as need be.

I need to implement a changelog so Bob can trace blame for mistaken actions. (Design v1 sketches it as screen 23; see section 6.)

There are two visions of the future as humans interact with software, both of which will be realized with Coconut.

1. Most interactions a human has with software will be with his own agent in it's own UI (Grok Bot).

2. The current limitations of software mean that agents currently have to use Bob's personal credentials to interact with said software. This should change. Bots should, for all intents and purposes, be treated as humans. This means that agents should have their own creds. Software moving forward will thus need to reflect various ownership properties. It will be multiuser.
