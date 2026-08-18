# Menus and listboxes

This is both a decision record — why `src/elements/dropdown.rs` was deleted
rather than kept beside `src/elements/select.rs` (#154) — and the convention
that replaces it. It is held to the crate by `mod family_coverage` in
`src/elements.rs`, because a naming rule nothing checks is how two names for
one component survived long enough for one of them to be built on the other's
internals.

The sentence the whole document exists to make re-checkable, and the one that
opens `src/elements/select.rs`:

> A **listbox** offers *values* to choose between, and the choice persists. A
> **menu** offers *actions* to invoke, and nothing stays selected once one has
> been.

Everything else here follows from it.

## The two families

<!-- family-table -->

| Module | Family | What it is |
| --- | --- | --- |
| `select` | Listbox | The chooser: a bordered trigger, a chevron, and a popup of values one of which is marked. Its popup is `Listbox`, private to the module |
| `context_menu` | Menu | A menu of actions at the pointer, with its own row vocabulary (items, separators, disabled items) and its own keyboard model |

Two rows is the honest size of this table today, and it is the size the test
enforces against the crate rather than a target. A third chooser or a third
menu adds a row; anything that would need a row in *both* columns is the shape
this document exists to catch.

Elements that choose a value without a popup — `radio_group`, `toggle_group`,
`tabs`, `list` — are not in the table. They are not listboxes in the sense that
matters here: none of them places an overlay, so none of them could be tempted
to reach for another element's popup.

## 1. Why `Select` is the name that survived

`Select` and `Dropdown` were one component under two names. Both drew the same
bordered trigger with a chevron, both dropped the same popup one gap below it,
both took the same `ControlSize` — and `select.rs` imported `DropdownMenu`,
`DropdownOption` and `MENU_GAP` *from* `dropdown.rs` to do it. The only
behavioural difference was that a `Dropdown`'s selection could not be absent,
which is a constructor argument, not a component.

`Select` is the name that survived for three reasons:

- **It names a purpose, not a presentation.** "Dropdown" describes where the
  popup goes, which is a fact about placement that `docs/overlays.md` already
  owns. Every other element here is named for what it is for.
- **It is the accessibility vocabulary.** accesskit's roles are `ListBox` and
  `ListBoxOption`; the platform's own name for this control is a select.
  `docs/issues/element-roles-convention.md`'s answer has since reached this
  element, and the roles it reports and the name of the file it lives in agree:
  the trigger is a `ComboBox` (which is how ARIA maps a select-only combobox),
  the popup a `ListBox`, and each row a `ListBoxOption`. Not Chromium's native
  `MenuListPopup` / `MenuListOption` mapping — this document had already stated
  the crate's vocabulary, and `docs/issues/combobox.md` asks for the same three
  roles for the editable combobox that does not exist yet.
- **It already had the larger API.** `Select` had `placeholder`, `clear()` and
  an optional selection; `Dropdown` had none of them and could not express
  "nothing chosen yet". Merging in the other direction would have meant adding
  three features to `Dropdown` and deleting nothing.

To disagree, attack this section. The mechanical cost of reversing it is one
module and one migration table — low. The part that should not be reversed
without a replacement argument is the next section.

## 2. Why the popup is private

`Listbox` — the popup — is a private type in `src/elements/select.rs`. It is
not `pub`, not `pub(crate)`, and not in a module of its own.

This is the part of the decision that enforces itself. The mistake #154 undid
was not that two components had confusable names; it was that one of them was
*built on the other's internals*, which is what made them impossible to
distinguish in the first place. A public popup type invites exactly that again:
the next component in this neighbourhood reaches for it, imports it, and there
are two elements sharing a row type that neither of them owns.

With the popup private, the next chooser has two honest options — grow
`select.rs`, or write its own popup — and both of them are visible in review.
Sharing becomes a decision someone makes, rather than an import someone adds.

When a second caller genuinely needs it, the move is to lift `Listbox` into a
`pub(crate)` module *named by both callers* — not to make it `pub` where it
sits. A shared type in one caller's file is the state this whole document is
about.

## 3. What the two families share

Placement, and nothing else.

Both follow `docs/overlays.md`: `deferred(anchored()…).with_priority(1)`,
`.occlude()` on the panel, the gap as an `offset` rather than a margin. That is
a convention, not an implementation — and it has to be, because the two want
*different* fit modes. A popup under a trigger flips to the other side of the
trigger when it would leave the window; a menu opened at the pointer snaps
instead, because a menu that jumped away from the click would be worse than one
that overlapped. `docs/overlays.md` says so at the point the choice is made.

They do not share a row type, a popup type, a keyboard model, or a state
struct, and they should not:

- a listbox row is a **value**, is marked when it is the chosen one, and its
  keyboard model is "move the selection";
- a menu row is an **action**, is never marked afterwards, and its keyboard
  model is "move the highlight, then invoke" — with separators, disabled rows
  and eventually submenus, none of which mean anything in a listbox.

## 4. The migration

`gpuikit::elements::dropdown` is gone in full — `Dropdown`, `DropdownState`,
`DropdownChanged`, `DropdownMenu`, `DropdownOption` and `dropdown()`.

<!-- migration-table -->

| Was | Is |
| --- | --- |
| `dropdown(id, options, value)` | `select(id, name, options).selected(value)` |
| `DropdownState::new(…)` | `SelectState::new(…)` |
| `DropdownChanged` | `SelectChanged` |
| `state.selected` (a `T`) | `state.selected` (an `Option<T>`) |
| `state.set_selected(value, cx)` | `state.set_selected(Some(value), cx)` |
| `DropdownMenu`, `DropdownOption` | no replacement — the popup is private |

Every other method carried over unchanged: `on_change`, `full_width`,
`disabled`, `control_size`, `is_open`, `is_disabled`, `set_disabled`. `Select`
adds `placeholder` and `clear`, which a `Dropdown` could not express.

`select()` and `Select::new()` have since gained a second argument, the
accessible **name**: `select(id, name, options)`. It is required rather than
optional because `Role::ComboBox` is in `a11y::role_requires_a_name` and every
naming source that convention allows was unavailable here. A select's visible
text is its *value*, so naming the control after it would rename the control
every time the user changed it; the placeholder disappears the moment a choice
is made, and defaults to "Select…"; and gpui has no `labelled_by` builder, so a
`Field` or `Label` beside the control cannot name it either. A required
constructor argument is what section 2 of `src/a11y.rs` prescribes for exactly
this case — it was written for `IconButton`, and `Select` got there first.

The one behaviour that changed shape rather than name is the selection itself.
A `Dropdown` always had a value, so its popup always marked a row; a `Select`
may have none, and `select.rs` used to say so by passing `usize::MAX` as the
selected index and relying on no list being that long. The popup now takes an
`Option<usize>`, and `an_unselected_select_marks_no_row_and_a_selected_one_marks_its_own`
is the test for the state the sentinel stood for.

## 5. The freed name

`DropdownMenu` is **reserved**, not retired. A menu of *actions* opened from a
trigger — the thing shadcn calls a dropdown menu and Primer calls an action
menu — is a real component this crate does not have, and it is the correct use
of the name: it is a menu, it drops down, and it is not a chooser.

If it is built, it belongs to the **menu** family. It is built on
`context_menu.rs`'s items, separators and keyboard model, with a trigger in
place of a pointer position, and it gets a row in the table above saying so. It
is not built on `Select`, and it does not reach for `Listbox` — a component
whose rows are actions has no selected row to mark.

## What keeps this honest

`src/elements.rs` carries a `family_coverage` test module that parses the
family table above — anchored by the `<!-- family-table -->` comment — and
fails the build when this document stops describing the crate:

- every row names a `pub mod` that is really declared in `src/elements.rs`, and
  a family that is one of the two;
- **no module in one family `use`s a module in the other.** That is the
  mechanical form of the mistake this decision undid, and it is checked as
  source text rather than trusted;
- `src/elements/dropdown.rs` has not come back — the same shape as
  `overlay_coverage::the_deleted_portal_trait_has_not_come_back`, and for the
  same reason: a deletion that nothing checks is a deletion that gets undone by
  a merge.

The privacy of `Listbox` is checked by the compiler, which is stronger than any
test here could be.

## What would reopen this

**A second element that genuinely needs a listbox popup.**
`docs/issues/combobox.md` is the candidate: a text field that filters a list of
choices is a listbox with typing, and its popup rows are values. That issue
inherits this decision rather than re-taking it, and it inherits the
instruction in §2 with it — lift `Listbox` into a `pub(crate)` module named by
both callers, do not make it `pub` where it sits, and do not build a fourth
popup beside it.

**A menu that wants a trigger.** See §5. That reopens nothing here; it fills a
row in the table.

What would *not* reopen it is a component finding `Select`'s API too small.
`Select` is one module, and growing it is cheaper than the second name this
document deleted.
