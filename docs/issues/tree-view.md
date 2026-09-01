# TreeView: flatten the visible nodes into a uniform list, not a recursive element

## What it is

A hierarchical list — a file explorer, a symbol outline, nested settings — with
disclosure per parent node, one selection, and the WAI-ARIA tree keyboard
contract. The component is a flattening function plus a keyboard, layered on
the shape `src/elements/list.rs` already renders: the caller hands over a tree
of nodes, the element computes the *visible* rows (a node is visible when every
ancestor is expanded), and paints them as fixed-height rows in a `uniform_list`
with indentation per depth. There is no recursive element and no per-node
entity.

Expansion state is the caller's, the way `SidebarState` is: `src/elements/sidebar.rs`
opens with "The element stores nothing across frames," and this element makes
the same promise. The caller holds a set of expanded node ids and the selected
node id; the element renders from them and reports toggles through callbacks.
SwiftUI's `DisclosureGroup` draws the same line with its `isExpanded:
Binding<Bool>` — the component owns show/hide, the caller owns the state.

## Why it survives triage

Primer ships a TreeView because GitHub's product needed one, and
`docs/component-triage.md` already cites that roster as evidence of product
need. Any app with a navigable hierarchy — every editor, most inspectors —
needs this, and it is exactly the kind of component a caller cannot improvise:
the keyboard contract and the accessibility announcements are a specification,
not a styling choice.

The existing disclosure primitives do not cover it:

- **`src/elements/collapsible.rs`** is one trigger over one content region,
  rendered inline. Nesting Collapsibles gives nested layout, but no shared
  selection, no arrow-key contract across levels, no `aria-level`, and no
  virtualization — every expanded descendant is a live element.
- **`src/elements/accordion.rs`** is a flat set of Collapsibles with a
  single/multiple mode. Same story, plus its items are `SharedString` content,
  not rows.
- **`src/elements/list.rs`** has the right rendering shape — uniform rows,
  virtualized, selection styling, a scroll handle — and no hierarchy, no
  keyboard, and no focus.

The scope argument is the flattening. A recursive element cannot be
virtualized (a 10k-node tree becomes 10k elements the moment its roots are
expanded), cannot give `uniform_list` the flat index space its scroll handle
needs, and re-litigates layout per level. Zed's project panel — the canonical
gpui tree — settled this: it keeps an `expanded_dir_ids` set, walks the tree in
`update_visible_entries` skipping the children of collapsed directories,
produces a flat `visible_entries` vec where each row carries a `depth`, and
hands `uniform_list("entries", item_count, …)` a closure that indexes into it.
Expand, collapse, and selection movement mutate the state and recompute the
flat vec; revealing the selection is `scroll_to_item_strict` on the scroll
handle. This element is that technique, made reusable.

What is deliberately **not** built is listed under Non-goals; the headline cuts
are multi-select, async child loading, and drag between nodes.

## Prior art

Re-open every one of these before implementing; the paragraphs below are what
to take, not a substitute for the source.

- **WAI-ARIA Authoring Practices, Tree View pattern**
  (https://www.w3.org/WAI/ARIA/apg/patterns/treeview/) — the contract this
  element implements, keys and roles both. Take the single-select keyboard
  model verbatim: Down/Up move focus without changing expansion; Right opens a
  closed node, then moves to its first child; Left closes an open node, then
  moves to the parent; Home/End go to first and last visible node; Enter
  activates. Take the role structure — `tree` container, `treeitem` rows,
  `aria-expanded` only on parents, `aria-level`/`aria-setsize`/`aria-posinset`
  when the full sibling set is not in the tree (which a flattened, virtualized
  list guarantees it is not, so all three are mandatory here). Note which
  parts the APG marks optional — type-ahead, `*` expand-all-siblings, the two
  multi-select models — because the Non-goals below lean on that.
- **Primer React, TreeView** (https://primer.style/components/tree-view) —
  the API-shape evidence. Its `TreeView.Item` carries `current`,
  `expanded`/`onExpandedChange`, leading/trailing visual slots; its
  `TreeView.SubTree` carries a `state` prop (`initial`/`loading`/`done`/`error`)
  for async children, with skeleton rows and a completion announcement. Take
  the *item* anatomy — a disclosure triangle, a leading visual, a label, a
  trailing slot — and treat the async-loading state machine and `current`
  (auto-expanding the path to the active item) as the product features they
  are: evidence of need, not toolkit scope. The caller who owns the node data
  can render a loading row today.
- **SwiftUI, `List(_:children:)` and `DisclosureGroup`** (Apple developer
  documentation) — the ownership line. `List(data, children: \.children)`
  takes the tree as data plus a children keypath and owns none of it;
  `DisclosureGroup(isExpanded:)` binds expansion state outward. Take both
  decisions: the tree is the caller's data structure described to the element,
  and expansion is caller state with an uncontrolled default.
- **Zed, `project_panel`** (crates/project_panel/src/project_panel.rs in
  zed-industries/zed) — the gpui implementation technique described above:
  `expanded_dir_ids` + `update_visible_entries` + a `depth` per flattened row +
  `uniform_list` + `scroll_to_item_strict`. Also the honest catalogue of what a
  *product* tree accretes — auto-folding single-child directory chains, rename
  editing, drag, git status — none of which belongs in this element.

## What it has to close in this crate

- **The flatten.** A `TreeNode` (id, row content, children) and a visible-rows
  computation from it plus the expanded set. Each visible row carries its
  depth, its parent-relative position and sibling count (for
  `posinset`/`setsize` — these come from the *tree*, not from indices into the
  flat vec), and whether it is a parent. This is the whole novel data
  structure; everything after it is `List`'s existing shape.
- **Rendering: through `src/elements/list.rs` or beside it.** `ListEntry` has
  no depth, no disclosure triangle, and no expanded flag, and `List` hard-codes
  `DEFAULT_ITEM_HEIGHT: f32 = 27.0` — a pre-scale height of exactly the kind
  `src/theme/control.rs`'s docs argue against. The tree should own its own
  `uniform_list` call (rows need per-row a11y announcements `List` cannot
  make today) and take its row height from the control scale; if extending
  `List` instead, `ListEntry` grows a depth and a disclosure, and `List`'s
  height moves onto the scale in the same change. Indentation is
  `padding-left = depth × indent step` inside a fixed-height row — depth never
  fights the uniform height, which is why the flatten and `uniform_list` are
  compatible at all.
- **Keyboard and focus.** `List` has neither. The tree container holds one
  `FocusHandle` (the one tab stop) and key handlers for the APG contract —
  `src/elements/context_menu.rs`'s arrow-key navigation and its tests are the
  in-crate precedent for action-based key handling on a container.
- **Scroll-to-revealed-node.** Moving the selection with the keyboard must
  keep it visible: `UniformListScrollHandle::scroll_to_item` /
  `scroll_to_item_strict` with a `ScrollStrategy` exists in gpui and `List`
  already exposes `track_scroll`; the tree calls it on every keyboard move,
  the way Zed's panel does.
- **Selection.** Single selection, caller-owned, rendered with the
  accent-on-`accent_bg` treatment `List` already uses. Activation (Enter,
  double-click per platform convention if desired later) is a callback.
- **Traits** (`src/traits/`): the element implements
  `Accessible` (`src/traits/accessible.rs`) and `ControlSized`
  (`src/traits/control_sized.rs`). Its item builder carries `Selectable` and
  `Disableable` (`src/traits/selectable.rs`, `src/traits/disableable.rs`),
  with an `on_click`-shaped activation callback per `Clickable`'s contract. It
  does not implement `Orientable` — a tree is vertical and the APG's
  `aria-orientation` defaults to vertical — nor `Labelable` or the button
  traits.

## Accessibility

Stated against `src/a11y.rs`'s mechanism, which this element must use rather
than reinvent (`no_element_calls_gpuis_a11y_builders_directly` enforces it).

- **Container:** `Role::Tree`, named (`A11y::name` — "Files", "Outline"),
  `A11y::focus_handle(handle)` for the single tab stop the element owns.
- **Rows:** `Role::TreeItem` with a **required name** — `role_requires_a_name`
  already lists `TreeItem`, so a nameless row is a `debug_assert!`. Each row
  announces `.level(depth + 1)`, `.position_in_set(…)` and `.size_of_set(…)`
  *from its sibling set in the tree* (a virtualized list never has the full
  set in the node tree, so the APG makes these required), `.expanded(bool)` on
  parent rows only, and `.selected(bool)`. All of these are existing `A11y`
  fields that `every_state_field_reaches_the_node` already pins.
- **Focus model: container focus + active descendant, not roving tab stops.**
  `role_requires_keyboard_focus` deliberately excludes the composite-item
  roles — `TreeItem` among them — pending a roving-focus convention that does
  not exist, and names making each item a tab stop as exactly the APG's
  called-out mistake. The tree therefore does what `src/elements/select.rs`'s
  listbox does: real focus stays on the container, and the row the keyboard
  has reached claims `A11y::active_descendant(true)`. This makes the tree the
  *second* caller of `active_descendant`, and `A11y::active_descendant`'s docs
  say in so many words to read them before adding one — the
  one-claim-per-frame `debug_assert!` and the ancestor requirement both apply,
  and both are satisfied by a focused container announcing one row per frame.
- **Upstream gpui gaps, named the way `docs/component-triage.md` names the
  missing `aria_sort`:** gpui's `AriaProperties` has no
  **`aria_multiselectable`** builder (checked against gpui-unofficial 1.14.2's
  `div.rs` builder set), so a multi-select tree could not announce itself
  correctly today — one more reason multi-select is a non-goal rather than a
  variant. And gpui still has no `aria_disabled` (a11y.rs §3), so a disabled
  row is distinguishable only by the click action its node does not offer,
  which is the crate's existing convention.
- `src/a11y.rs`'s `ELEMENTS_WITHOUT_A_ROLE` list means a new `tree_view`
  module either implements `Accessible` or ships with a written excuse. Unlike
  `list` — whose entry is waiting on the roving-focus convention — this
  element has no excuse available, because the container-focus model above
  needs nothing that does not already exist. It implements `Accessible` from
  the first commit.

## Sizing

The element implements `ControlSized` and takes its row height, text size, and
paddings from the rung's `ControlMetrics`
(`src/theme/control.rs`, resolved through `Themeable::control`) — not a named
height, which is the mistake `List`'s hard-coded 27px preserves from before
the scale existed. The indent step per depth level is specific to this
component's shape, so per the "What belongs here" note at the top of
`src/theme/control.rs` it lives in the tree's own file, derived from the rung
(the natural choice is the rung's height, so the disclosure triangle of a
child aligns under its parent's label). Small/Medium/Large give the dense
file-explorer, the default, and the touch-friendlier row in one call.

## Showcase

`showcase_coverage` in `src/elements.rs` fails the build until `tree_view` has
a row in `examples/showcase.rs`'s `ELEMENT_COVERAGE` and a page a match arm
renders. The page demonstrates: a file-explorer-shaped tree several hundred
nodes deep enough to nest four or five levels; mouse expand/collapse on the
disclosure triangle and the row; the full keyboard contract (arrows, Home/End,
Enter) driven from the container's focus, with the selection scrolling into
view when moved off-screen; selection styling; a disabled row; and the three
`ControlSize` rungs side by side.

## Non-goals

- **Multi-select.** The APG documents two optional models; gpui cannot
  announce `aria-multiselectable` at all. Single-select ships; multi-select is
  an upstream ask plus a later issue, and building it now would mean a
  selection model a screen reader is told nothing about.
- **Async loading of children.** Primer's `SubTree` `state` machine is a
  product feature. The caller owns the node data here, so a "loading…" child
  row is already expressible without the element knowing.
- **Drag between nodes, rename editing, auto-folding single-child chains.**
  All present in Zed's project panel, all product behavior layered on top of
  the flatten, none of them toolkit scope.
- **Type-ahead and `*` expand-all-siblings.** Both optional in the APG.
  Revisit type-ahead when a consumer asks; the state machine (a timed prefix
  buffer) is self-contained and additive.
- **A `current` item that auto-expands its ancestor path** (Primer). The
  caller who owns the expanded set can add a path's ids to it in one line.

## Blocked on

Nothing hard. `src/a11y.rs` exists and covers every property this element
announces; the one soft dependency is care around being
`A11y::active_descendant`'s second caller, which that method's docs demand be
read first.
