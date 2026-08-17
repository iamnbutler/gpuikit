# Menus and listboxes

This is both a decision record — why `src/elements/dropdown.rs` was deleted
rather than kept beside `src/elements/select.rs` (#154) — and the convention
that replaces it. It is held to the crate by `mod family_coverage` in
`src/elements.rs`, because a naming convention nothing checks is a naming
convention that lasts until the next component.

**A listbox presents *values* to choose between, and the choice persists. A
menu presents *actions* to invoke, and nothing stays selected afterwards.**

That sentence is the whole distinction, and it is the one an accessibility API
already draws: accesskit has `ListBox`/`ListBoxOption` on one side and
`Menu`/`MenuItem` on the other, with different keyboard expectations attached to
each. It is restated at the top of `src/elements/select.rs` and at the top of
`src/elements/context_menu.rs` so that it is re-checkable from inside either
file rather than only from here.

## The two families

<!-- family-table -->

| Module | Family | What it is |
| --- | --- | --- |
| `select` | Listbox | A trigger showing the current value, and a popup of the values it could have. The selection persists and is marked in the popup |
| `context_menu` | Menu | A menu of actions at the pointer. Items invoke and dismiss; nothing is marked afterwards |

Every row names a real `pub mod` in `src/elements.rs` and one of the two
families, and **no module in one family may `use` a module in the other** —
that is the mechanical form of the mistake this decision undid, and
`family_coverage` fails the build on it.

## 1. Why `Select` is the name that survived

`Select` and `Dropdown` were one component under two names. The same bordered
trigger with a chevron, the same popup one gap below it, the same `ControlSize`
— and `select.rs` imported `DropdownMenu`, `DropdownOption` and `MENU_GAP`
*from* `dropdown.rs` to get there, so one of them was literally built on the
other's internals. The only behavioural difference was that a `Dropdown`'s
selection could not be absent: `dropdown(id, options, value)` took a value,
`select(id, options)` did not. That is a constructor argument, not a component.

Of the two names, `Select` is the one that describes what the thing is for.
"Dropdown" describes a *presentation* — a popup that comes down — which is a
property this component shares with `ContextMenu`, `Popover` and a future
`DropdownMenu`, and therefore distinguishes nothing. "Select" describes a
*purpose*: choose one of these values. It is also the name every other toolkit
uses for exactly this control, and the name the crate's own triage already used
when it rejected "Native Select" on the grounds that "`Select` already covers
it".

So `Dropdown` is deleted, and `Select` takes the union of the two APIs.
`.selected(v)` is what makes a `Select` into the old `Dropdown`; leaving it off
is what the old `Select` was. `selected` is an `Option<T>` for everyone.

*If this is the wrong way round, this is the section to argue with.* The
mechanical cost of reversing it is low — the family is one module. The part
that should not be reversed without a replacement argument is the next section.

## 2. Why the popup is private

`Listbox` (the popup, formerly `pub struct DropdownMenu`) is **private to
`src/elements/select.rs`**. So is `LISTBOX_GAP` (formerly `pub(crate)
MENU_GAP`). `DropdownOption` is gone outright — it was a newtype over
`SharedString`, and the popup now takes `Vec<SharedString>`.

This is the part of the decision that enforces itself. A public popup type is
an invitation, and #154 exists because that invitation was accepted once
already: the cheapest way to build the next chooser in this neighbourhood is to
reach for the popup the last one used, and the result is two components that
cannot be told apart and one that cannot be changed without changing the other.
Privacy makes the cheap path unavailable, so the next component is built
*beside* this one rather than *on* it, and the sharing question gets asked
deliberately rather than answered by an `import`.

**When a second caller genuinely appears** — `docs/issues/combobox.md` is the
likely one — the move is *not* to make `Listbox` `pub`. It is to lift it into a
`pub(crate)` module of its own, named by both callers rather than by whichever
one existed first, at which point there are two implementations to read and the
shared thing can be described honestly. Until there are two, there is nothing
to name.

## 3. What the two families share

Placement, and only placement. Both follow [`docs/overlays.md`](overlays.md)
and call `gpui::anchored()` themselves; neither calls the other. Even that is a
convention rather than a shared implementation, and deliberately so: the two
want *different* fit modes. A popup under a trigger flips to the other side of
the trigger when it would leave the window (`SwitchAnchor`, the default); a
menu opened at the pointer snaps instead (`snap_to_window_with_margin`), because
a menu that jumps away from the click is worse than one that hugs an edge.

Nothing else is shared. A menu has separators, submenus, destructive items,
keyboard shortcuts and typeahead; a listbox has a persistent selection and
neither of the first four. Rows that look alike are not the same row.

## 4. The migration

`gpuikit::elements::dropdown` is gone in full — `Dropdown`, `DropdownState`,
`DropdownChanged`, `DropdownMenu`, `DropdownOption` and `dropdown()`.

| Was | Is |
| --- | --- |
| `dropdown(id, options, value)` | `select(id, options).selected(value)` |
| `DropdownState::new(…)` | `SelectState::new(…)` |
| `DropdownChanged` | `SelectChanged` |
| `state.selected` (a `T`) | `state.selected` (an `Option<T>`) |
| `state.set_selected(value, cx)` | `state.set_selected(Some(value), cx)` |
| `DropdownMenu`, `DropdownOption` | private; no replacement is offered |
| `MENU_GAP` | private `LISTBOX_GAP` |

`.on_change`, `.full_width`, `.disabled`, `.control_size`, `is_open`,
`is_disabled` and `set_disabled` are unchanged. `Select` additionally has
`.placeholder(…)` and `clear()`, which is what an absent selection needs and a
`Dropdown` could not express.

The element ids and debug selectors moved with the type: `dropdown-menu` and
`dropdown-option` are now `select-listbox` and `select-option`, and the test
selectors are `gpuikit-select-trigger` and `gpuikit-select-popup`.

## 5. The freed name is reserved

"Dropdown menu" is a real pattern this crate does not ship: a menu of *actions*
opened from a button rather than from a right-click. That is the name now free,
and it is reserved for exactly that. When it is built it is
`src/elements/dropdown_menu.rs`, it belongs to the **Menu** family, and it is
built on `context_menu.rs`'s items — `MenuItem`, its separators and its
keyboard model — not on `Select`. It gets a row in the family table above, and
the layering test then holds it to that.

`src/icons.rs::dropdown_menu()` is unrelated: it is an asset name from the icon
set, and it stays.

## What keeps this honest

`src/elements.rs` carries a `family_coverage` test module that parses the
family table above — anchored by the `<!-- family-table -->` comment — and
fails the build when this document stops describing the crate:

- every row names a `pub mod` that is really declared in `src/elements.rs`, and
  a family that is really one of the two;
- both families are represented, so the table cannot decay into a list;
- no module in one family `use`s a module in the other;
- `src/elements/dropdown.rs` has not come back.

The last one is the same shape as `overlay_coverage`'s check that
`src/traits/portal.rs` has not come back, and for the same reason: a deletion
that nothing guards is a deletion that gets undone by whoever next needs
something a bit like it.

## What would reopen this

- **A third chooser.** `docs/issues/combobox.md` is a text field that filters a
  list of values; it is a listbox by the sentence above, and building it is
  what makes the "two callers" question in §2 real. It does not reopen the
  naming — it inherits it.
- **A menu whose items stay selected.** A checkable or radio menu item is a
  menu row that persists a choice, which is the one place the sentence at the
  top blurs. The answer is that it is still a menu — the persistence belongs to
  the application state the item toggles, not to the popup — but if that stops
  being convincing, this is where to say so.
- **Accessibility roles.** `Listbox` is named for the accesskit role it will
  report, and it reports no role yet.
  `docs/issues/element-roles-convention.md` is still open and decides how, once,
  for every element. If that convention finds the two families need to share a
  mechanism, §3 gets a second sentence — but sharing a *role vocabulary* is not
  sharing an implementation.
