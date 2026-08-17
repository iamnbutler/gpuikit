# Prerequisite: decide what Select, Dropdown and their popups are called

## The problem

`src/elements/select.rs` and `src/elements/dropdown.rs` are two components that
overlap, and the overlap is structural rather than cosmetic:

- `select.rs` imports `DropdownMenu` and `DropdownOption` **from** `dropdown.rs`
  — one component is built on the other's internals.
- `ELEMENT_COVERAGE` in `examples/showcase.rs` maps both modules to the same
  `dropdown` showcase page.
- Both render a trigger with a chevron and a popup list of options, and both
  now take the same `ControlSize`.

The names do not distinguish them either. "Dropdown" describes a *presentation*;
"Select" describes a *purpose*; neither says which one is a chooser and which
one is a menu of actions.

## Why it blocks work

`docs/issues/combobox.md` is a third component in the same neighbourhood, and
it is hard-blocked on this. Adding it before the naming is settled produces a
fourth overlapping popup implementation and a third component nobody can pick
between. `docs/issues/command.md` is a fourth.

## The distinction that matters

In accessibility terms these are genuinely different things, and the platform
enforces the difference:

- A **listbox** presents *values* to choose between. It has a selected option,
  and the selection persists. `Select` is this.
- A **menu** presents *actions* to invoke. Nothing stays selected; typeahead,
  submenus and separators are menu things. `ContextMenu` is this.

`Dropdown` as it stands is a listbox wearing a menu's name, and its popup type
is called `DropdownMenu`.

## What has to be decided

1. **Whether `Select` and `Dropdown` are one component or two.** If they are
   one, one of them is deleted and the other takes its API. If they are two,
   the difference has to be nameable in one sentence in each module's docs.
2. **What the shared popup is called**, and whether it is public. Today
   `DropdownMenu` is public and reachable from both.
3. **Whether the menu family (`ContextMenu`) and the listbox family share
   anything.** They should probably share placement (see
   `docs/issues/portal-adopt-or-delete.md`) and nothing else.
4. **The showcase pages.** One `ELEMENT_COVERAGE` row per module, and a page
   per component that exists after the decision.

## Scope

A naming and layering decision, the rename, and the doc sentence in each module
that makes the distinction re-checkable. Breaking changes are fine — see
`README.md` — but the CHANGELOG has to spell out the migration.

## Blocks

- `docs/issues/combobox.md` — **hard block**.
- `docs/issues/command.md` — should follow it.

---

*This is a prerequisite rather than a component, so it has no rung and no
showcase page of its own. It is reachable from
[`docs/component-triage.md`](../component-triage.md).*
