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
- **03 Regular event form:** no End Date step. Steps are Event Title, Category, Date,
  Start Time, End Time, Location, Info; an earlier End Time means the next day, an equal
  one is an error. v1's note says "step 6 of 8"; it should be "step 5 of 7". The
  Repeating form drops End Date too. (2026-11-16)
- **01 Month (home):** the spec's month row shows the start time only, as on 19
  ("09:30 Standup · Work"), not 01's start and end. (2026-11-16)
- **01 Month and 04 All-day form:** the month list runs 13 months, not 12, and a `DD-MM`
  date means the first such date on or after today, not "from the 1st of this month".
  Last Day (and Change time's End Date on 09) is the first such date on or after the start,
  at most 28 days later ("Must be on or after the start."). (2026-11-17)
