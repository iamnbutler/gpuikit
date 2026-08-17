# Prerequisite: adopt src/traits/portal.rs or delete it

## The problem

`src/traits/portal.rs` is **486 lines of overlay positioning math with zero
callers, zero implementors and zero tests.** Nothing in `src/`, nothing in
`examples/`, nothing in the test suite refers to it.

Meanwhile six elements place overlays, and each hand-rolls its own
`anchored()`/`deferred()` pair:

- `src/elements/dialog.rs`
- `src/elements/dropdown.rs`
- `src/elements/select.rs`
- `src/elements/popover.rs`
- `src/elements/context_menu.rs`
- `src/elements/toast.rs`

#146's blocker table lists "overlay positioning system" as resolved and cites
this file. That is the one row of the table that does not hold: the file exists,
and the problem it was written for is being solved six other times.

## Why it blocks work

Three of the ten components surviving the #59 re-triage are anchored overlays —
`docs/issues/command.md`, `docs/issues/combobox.md` and
`docs/issues/date-picker.md`. Left alone, each will add a seventh, eighth and
ninth hand-rolled placement, and the shared behaviour that is currently missing
everywhere (flipping when there is no room below, staying on screen near an
edge, matching the trigger's width) will be missing from each of them
differently.

## What has to be decided

There are exactly two honest outcomes:

**Adopt.** Convert at least two of the six existing overlays to it, in the same
change. One conversion proves nothing — a trait with a single implementor is
still an untested abstraction. Two different call sites (a `Dialog`, which is
centred and modal, and a `Dropdown`, which is anchored to a trigger) is the
minimum that exercises the shape. Tests come with the conversion:
`src/elements/context_menu.rs` already measures laid-out positions with
`debug_selector`/`debug_bounds`, which is the local pattern for asserting where
an overlay ended up.

**Delete.** If the six call sites do not want what it offers, the file is a
486-line answer to a question nobody asked, and its presence actively misleads —
it is why #146's blocker table says this is resolved. Deleting it and writing
down that overlays are hand-rolled on purpose is a legitimate outcome and a
better one than leaving it.

What is *not* an outcome is leaving it as it is.

## How to decide

Read the file against the six call sites and ask, per call site, what it would
have saved. If the answer is "nothing" four or more times, delete.

## Scope

Whichever outcome, in one change, with the CHANGELOG entry that goes with it.
Do not adopt it partially.

---

*This is a prerequisite rather than a component, so it has no rung and no
showcase page of its own. It is reachable from
[`docs/component-triage.md`](../component-triage.md).*
