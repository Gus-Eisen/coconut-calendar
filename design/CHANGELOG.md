# Coconut design changelog

Each version is a frozen folder under design/. Never edit an old one; a Figma
change gets a new folder (v2, v3, ...). Screens are named by screen number, so
"see 21b" in the spec means design/vN/screens/21b-*.png.

Live Figma file (may be newer than the latest folder):
https://www.figma.com/design/ep0UXdTZLDWLQJgeVrg8Hb

## v1 (2026-11-16)

First snapshot. Screens 01-31 plus 14b and 21b, grouped by plan step
(1 new events, 2 editing, 3a repeats, 3b exceptions, 4 views, 5 roles,
6 invites, 7 visibility). Based on the Google Calendar study.

- all-screens.png: the whole Mockups page, including the yellow notes.
- screens/: one PNG per screen, cut from all-screens.png.
- figma-scripts/: the scripts that built the page through the Figma plugin API.
  No .fig file: Save local copy isn't available on a View seat.

Decisions shown: category as a text label (no colors); editing as one
pre-filled text page plus separate time and category flows; Invites, People
and Shared calendars as root tabs; Repeat Days as a single-select preset list
(14b, until chk gets multi-select); Colleague first and default on the role
step (21b).

The spec has not been updated to match v1 yet.

## Where the spec is ahead of v1 (the spec wins)

These were decided after the v1 snapshot. Don't "fix" the spec back to the mockups; fold
them into the next design version instead.

- **07 Event detail:** gets a fifth action, Change privacy, opening 11's privacy radio.
  v1 shows only Edit details, Change time, Change category and Delete. (2026-11-16)
- **03 Regular event form:** no End Date step. An earlier End Time means the next day, an equal
  one is an error. v1's note says "step 6 of 8"; after dropping End Date it was "step 5 of 7".
  A Link step was added later (2026-11-20, below). The Repeating form drops End Date too.
  (2026-11-16)
- **01 Month (home):** the spec's month row shows the start time only, not 01's
  start and end. (2026-11-16)
- **01, 18, 19 Views:** the month row shows time and title only, with no
  category (v1 19 shows "09:30 Standup · Work"). Future days with several
  events show the first event, not 19's "2 events". The Day title is the short
  date, e.g. "Wed 25 Nov", not 18's "Tuesday 3 November". (2026-11-20)
- **01 Month and 04 All-day form:** the month list runs 13 months, not 12, and a `DD-MM`
  date means the first such date on or after today, not "from the 1st of this month".
  Last Day (and Change time's End Date on 09) is the first such date on or after the start,
  at most 28 days later ("Must be on or after the start."). (2026-11-17)
- **13, 14, 14b, 15 Repeating events:** the picker has four options (Repeating All-Day after
  Repeating). Frequency and Repeat Days are one "Repeats" radio of seven options; Every,
  Until and Times steps follow. No End Date step. The review shows Repeats, On, First (only
  when the first occurrence isn't the typed date) and Ends rows, not one long Repeats line,
  because chk clips long values. Change repeat's Repeats list adds "Doesn't repeat" last.
  Step titles are plain ("Start Time", "End Time"); chk clips longer ones. (2026-11-17)
- **16, 17, 07 Exceptions:** the scope step is titled "Which Events?" and has two choices,
  This occurrence ("Only Tue 1 Dec") and Whole series ("Every occurrence"); no "This and
  following" yet. Change category, Change privacy and Change repeat have no scope step (they
  always change the series). Delete > This occurrence opens a "Cancel Occurrence" review with
  "This can't be undone." The changed occurrence's action is "Restore", not "Restore to
  series", and its line is "This occurrence was changed." Plain short titles, as chk clips
  long ones. (2026-11-17)
- **03, 04, 13, 14 forms:** all four new-event forms have an optional Link step after
  Info (placeholder "Link (optional)"). Regular is 8 steps, All-day 7, Repeating 12,
  Repeating all-day 11. The review shows a Link row when it's set. v1 has no Link step,
  and 03's note should now read "step 5 of 8". (2026-11-20)
- **07 Event detail and 08 Edit details:** the detail-page row and the Edit details
  field are "Link", not 08's "Video link". The Attachments section and its Add link
  action are gone from 07. (2026-11-20)
- **12 Add link:** the Add link flow and the links list are removed; each link's
  Delete goes with them. (2026-11-20)
