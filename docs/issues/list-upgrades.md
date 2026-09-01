# List: a selection model, scroll-to-item, and an opt-in non-uniform mode

## What it is

Three upgrades to `src/elements/list.rs`, which today is a `uniform_list` with
fixed-height rows, header entries, and a per-entry `selected` flag:

- **A selection model.** Single and multiple selection as state the caller
  owns, with the element interpreting the gestures — click replaces,
  cmd/ctrl-click toggles, shift-click extends, arrows move a cursor,
  shift-arrows extend from an anchor, cmd/ctrl-A selects all. Today `List` has
  only `ListEntry::selected(bool)` — a drawing instruction, not a model — and
  the showcase's sidebar navigation re-derives it by hand every frame.
- **Scroll-to-item.** A `List`-level API that scrolls an *item* into view.
  `track_scroll` exists but hands the caller a raw `UniformListScrollHandle`
  whose indices count headers as rows, and nothing inside the element ever
  scrolls — which is fine only because there is no keyboard model to need it.
- **An opt-in non-uniform mode.** Variable row heights are not a gap — a list
  with uniform rows is just the way lists work, and `uniform_list` is the right
  default. But gpui ships a second backing, `list()` + `ListState`, that
  measures each row, and `List` can offer it behind one opt-in without changing
  anything else about its API.

## Why it survives triage

`List` is not a row on the #59 roster — it shipped early, as the "Scrollable/
virtualised lists" blocker in `docs/component-triage.md`'s corrected blocker
table — so this is an upgrade issue for a shipped element.

**The selection state lives outside the element.** This is the house pattern,
stated twice already: `src/elements/sidebar.rs`'s module docs ("State is the
caller's — the element stores nothing across frames") and
`src/elements/table.rs`'s ("nothing moves until the caller moves it" — rows
arrive already sorted, the element reports `SortRequest` / `SelectRequest` /
`SelectAllRequest` back). The list selection model is the same shape: the
caller holds the selection, hands it in, and the element reports what the user
asked for. What the element *owns* is gesture interpretation — which click is a
replace, which is a toggle, which is an extension — because that contract is
the component, exactly as `SortRequest::toggle` owns the conventional
sort-toggle so every caller does not re-derive it.

**The non-uniform mode is an API extension, not a rewrite.** The entry model,
the header treatment, the selection model and the scroll API are all
independent of which gpui element draws the rows. One builder method swaps the
backing; everything above it is unchanged.

What is NOT built, said plainly: no data ownership (entries arrive from the
caller each frame, as they do today), no filtering, no sorting, no
drag-to-reorder (see Non-goals), no type-ahead (a chooser affordance;
`Listbox` has it, a file list does not need it in v1), and no tree — an
indented list is not a `TreeView` and this issue does not make it one.

## Prior art

- **WAI-ARIA APG, Listbox pattern** — the keyboard contract, and this issue
  adopts its *recommended* multi-select model (the one that does not require
  modifiers to move): Up/Down move focus, Home/End jump, Space toggles the
  focused option, Shift+Up/Down move-and-toggle, Shift+Space selects from the
  most recently selected item to the focused one, Ctrl/Cmd+A selects all. The
  APG names focus and selection as "functionally distinct" and warns that
  selection-following-focus is a choice, not a default — `List` keeps them
  distinct: plain arrows move the cursor without touching a multi-selection.
- **SwiftUI `List(selection:)`** — the ownership split to copy: the binding is
  the caller's (`Item.ID?` for single, `Set<Item.ID>` for multi) and the
  framework owns cmd-click, shift-click and arrow-key extension. Note what
  SwiftUI does *not* make the caller do: interpret a modifier. Its `onMove` is
  the reorder callback shape for the deferred issue, not this one.
- **`src/elements/table.rs`** — in-house prior art and the consistency
  constraint; see the next section.
- Re-open all of these before implementing.

## What it has to close in this crate

- **The List/Listbox relationship — reuse the helpers, not the component.**
  `src/elements/listbox.rs` is `pub(crate)`, a chooser popup with a *transient
  highlight* separate from one *chosen* row, two focus arrangements
  (`ListboxFocus`), and a dismissal on commit. `List` is a persistent surface:
  selection persists, may be plural, and choosing does not close anything —
  they are different state machines and merging them would rebuild the
  `Select`-on-`Dropdown` entanglement that `docs/menus-and-listboxes.md` was
  written to end. What *is* shared, and already deliberately factored for
  sharing: `listbox::wrapped_index` (already used by `command`), the
  free-function row-a11y shape (`listbox::option_a11y` — a test can read it
  without laying anything out), the "one place the highlight moves, so
  `scroll_to_item` cannot be forgotten" discipline (`Listbox::highlight`), and
  the container-focus + `active_descendant` arrangement. One behavioural
  difference to get right: `wrapped_index` wraps at both ends, which is
  correct for a chooser popup and wrong for shift-extension — a selection
  extended past the last row must clamp, not wrap to row 0 — so `List`'s
  cursor movement clamps and the shared helper is the arithmetic, not the
  policy.
- **Consistency with Table's selection.** `Table` already answered the
  ownership question: `Row::selected(bool)` draws, `SelectRequest { index,
  selected }` and `SelectAllRequest` report, the caller applies. `List` keeps
  that ownership split exactly and differs only where the interaction model
  differs: `Table`'s gestures are checkboxes (every click is a toggle, select-
  all is a header checkbox with the indeterminate middle), `List`'s are
  pointer modifiers and arrows. So `List` reports a request that carries the
  *interpreted* gesture — replace / toggle / extend-to / all — plus a caller-
  owned selection value type with an `apply(request)` convenience, the same
  courtesy `SortRequest::toggle` extends. Selection is addressed by item
  index within the entries handed in this frame, as `Table`'s `SelectRequest.
  index` is; a caller with stable ids owns that mapping, because the caller
  owns the data view — SwiftUI selects by id precisely because its `List`
  owns the data, and this one does not.
- **A keyboard, which `List` today has none of.** No focus handle, no key
  context, no bindings. Following `listbox.rs`: an `actions!` set, a
  `LIST_CONTEXT`, and a `bind_list_keys` that `crate::init` calls — an app
  assembling its own keymap calls it itself. Unlike the popup (a transient
  overlay that must not be a tab stop, so it declines `A11y::focusable` and
  calls `track_focus` by hand), a persistent `List` is a tab stop, so
  `A11y::focusable` applies cleanly through the convention.
- **Scroll-to-item as a `List` API, in item coordinates.** The keyboard cursor
  must scroll itself into view (the `Listbox::highlight` lesson), and the
  caller needs the same verb. Indices must be *item* indices — today a caller
  reaching through `track_scroll` scrolls to entry ix where headers count,
  which is wrong the moment a header is inserted. The two backings differ:
  `UniformListScrollHandle::scroll_to_item(ix, ScrollStrategy)` offers
  Top/Center/Bottom/Nearest (verified against gpui-unofficial 1.14.2), while
  the non-uniform `ListState` offers `scroll_to_reveal_item(ix)` — Nearest
  only — plus `scroll_to(ListOffset)`. Expose the intersection honestly: a
  reveal-item API everywhere, strategy where the uniform backing can honour
  it, not a strategy enum the non-uniform mode silently ignores.
- **The non-uniform opt-in, priced.** gpui's `list()` measures each row and
  caches heights in a SumTree; `ListState::new(item_count, ListAlignment,
  overdraw)` is *retained state the caller must hold across frames* — it is
  also the scroll handle — and it must be told via `splice`/`reset` when
  entries change, where `uniform_list` needs nothing retained. So the opt-in
  takes a caller-held state handle (the `track_scroll` precedent), and the
  element owes the caller the splice bookkeeping story in its docs. The
  default stays `uniform_list`; fixed-height rows remain just the way lists
  work.
- **`Sidebar`'s prescription becomes real.** `sidebar.rs` names `List` as its
  content, and the showcase's own navigation is the worked example of what is
  missing: `examples/showcase.rs` re-clones every `ListEntry` per frame just
  to set `.selected(...)`, and arrow-key navigation of the nav list does not
  exist. Single-selection with a caller-owned current item is the sidebar
  case; it must fall out of this model without the multi-select machinery
  tagging along.
- **`ListEntry::selected` stays.** It is the drawing input the model feeds; a
  second way to say "this row draws selected" would be the two-sources
  problem.

## Accessibility

Through `src/a11y.rs`'s mechanism — `Accessible` + one `.announce(a11y)` —
and nothing else; `a11y::tests::no_element_calls_gpuis_a11y_builders_directly`
enforces it.

- **Container:** `Role::ListBox`, named. `Role::ListBox` is deliberately
  absent from `role_requires_a_name` (that absence is argued in
  `src/elements/select.rs` — the popup names itself after its trigger), so a
  persistent `List` takes a `.label(...)` the way `Sidebar` does, as its own
  decision.
- **Rows:** `Role::ListBoxOption` with `selected`, `position_in_set` /
  `size_of_set` counted over items — headers are not options and must not
  inflate either number — following `listbox::option_a11y`'s shape.
- **Cursor:** `src/a11y.rs`'s `role_requires_keyboard_focus` docs name the
  missing roving-focus convention and list `List` among the elements that
  want it. Do not invent it here: use the arrangement the crate already has —
  the container holds the one tab stop and the cursor row claims
  `A11y::active_descendant`, which gpui honours under a focused ancestor.
  That is exactly the `ListboxFocus::Popup` arrangement, and it is why a
  focused `List` can announce its cursor while `Combobox`'s popup cannot.
- **Upstream gap, named the way the triage doc names `aria_sort`:** gpui has
  no `aria_multiselectable`. `accesskit::Node::set_multiselectable` exists;
  `div`'s builders stop short of it, so a multi-select `List` cannot declare
  itself one. Same policy as `aria_disabled` / `aria_sort` in `src/a11y.rs`
  §3: an upstream ask, not a silent field on `A11y`. When gpui grows the
  builder, `A11y` grows the field and this element uses it.

## Sizing

`list.rs` predates the shared scale and names its own constants —
`DEFAULT_ITEM_HEIGHT: f32 = 27.0`, `DEFAULT_FONT_SIZE: f32 = 13.0`. This issue
migrates it: `ControlSized`, row height and text size resolved from a
`ControlSize` rung through `Themeable::control`, with `item_height(px)`
retained as the explicit override. Anything list-specific (the header's
bottom-aligned inset) stays in this file, keyed off the rung, per the "What
belongs here" note atop `src/theme/control.rs`.

## Showcase

A showcase page is a build requirement, not a convention — `showcase_coverage`
in `src/elements.rs` fails the build without one. `List` already has a page;
this issue rebuilds it to demonstrate: multi-select with click / cmd-click /
shift-click and the full keyboard contract; a single-select variant; a
scroll-to-item control (a button that jumps to a named item in a long list,
proving item-vs-entry indices); and a non-uniform section with rows of
genuinely different heights. The showcase's own sidebar navigation adopts
single selection, replacing its per-frame `.selected(...)` re-derivation —
the same acceptance-test role its hand-rolled sidebar played for `Sidebar`.

## Non-goals

- **Reordering.** Drag-to-reorder (SwiftUI's `onMove`) is deferred to its own
  issue, deliberately: it needs gpui's drag primitives, a keyboard equivalent,
  a drop-indicator treatment, and an answer to "does dragging one selected row
  drag the whole selection" — each a decision this issue would otherwise take
  half of. It *depends on* this issue (a reorder callback wants the selection
  model's coordinates) and must not be folded into it. `onMove`'s
  indices-plus-offset callback is the shape to start from when it is written.
- **Type-ahead.** `Listbox` has it for choosers; a persistent list can grow it
  later without API breakage.
- **Data ownership, filtering, sorting.** The caller's, per the house pattern.
- **Tree semantics.** Disclosure, nesting and `TreeItem` are a different
  component.
- **Replacing `uniform_list` as the default.** The non-uniform mode is the
  opt-in; fixed-height rows stay the default and the fast path.
