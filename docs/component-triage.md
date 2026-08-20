# Component triage

A decision per component for every entry on the deferred list from
[#59](https://github.com/iamnbutler/gpuikit/issues/59), re-taken against the
crate as it stands rather than against the list's own reasoning.

This document is the deliverable of
[#146](https://github.com/iamnbutler/gpuikit/issues/146). #146's point is that
#59 has stopped describing anything: seven of its entries have shipped, the
infrastructure the rest were blocked on now exists, and the list itself was
never a decision — it was shadcn/ui's roster, copied. A roster is not a
backlog. Every entry below gets a verdict, and a verdict is a decision someone
can disagree with.

## The three verdicts

- **Shipped** — the component exists. The row names the module, and a test
  checks that module is really declared in `src/elements.rs`.
- **Issue** — worth building. The row names a file under `docs/issues/`
  holding a complete issue body: prior art, references, the gaps in this crate
  it would have to close, its accessibility answer, its sizing, and its
  showcase page. A test checks the file exists and is not a stub.
- **Rejected** — not worth building *now*, with a reason and a named revisit
  trigger. A test checks each rejection is argued in prose below, not merely
  asserted in a table cell.

There is deliberately no fourth verdict. "Deferred", "maybe", "someday" and
"blocked on infrastructure" are what #59 was made of, and a list of them
survives contact with reality by never being read. A component is either here,
worth writing an issue for, or turned down for a reason that could later stop
being true.

<!-- ratification -->

## Who took these verdicts, and which of them are binding

Not every claim in this document is the same *kind* of claim, and the table
below flattens three of them into one column. Read it this way:

- The **13 Shipped** rows are facts. Each names a module, and
  `triage_coverage` in `src/elements.rs` fails the build if that module is not
  really declared. Nobody has to take these on trust.
- The **5 Issue** rows are proposals, and cheap ones: each is a written issue
  body under `docs/issues/`, and writing one commits nobody to building it. The
  bodies were filed separately under the maintainer's authorisation.
- The **11 Rejected** rows are **one agent's reading of this crate, proposed
  and not ratified.** They were taken while writing #146's deliverable, against
  the crate as it stood, and no maintainer has signed off on a single one of
  them.

The reasons in "The rejections, argued" below are written flat and unhedged, on
purpose: a rejection that hedges is the "deferred / maybe / someday" this
document exists to replace, and a reason you cannot disagree with is not a
reason. What was missing was never doubt about the arguments — it was who made
them. This section is that, and it does not soften a single one.

**How to ratify.** Read a rejection, and either say so in the pull request that
adopts it or edit its paragraph here. When the rejections are ratified, replace
this section's third bullet with who ratified them and when. Until then, treat
a Rejected row as an argument to attack rather than as a decision that has been
taken.

## Provenance: why #59 stopped meaning anything

Three things are checkable, and all three hold:

- **Its entries shipped without it noticing.** Thirteen rows below are Shipped —
  eight when this triage was first taken (#146 counted seven; re-reading the
  list against `src/elements/` found one more, which is itself the point), plus
  Table and Data Table, built since as one module, plus Sidebar and Resizable,
  built since against their own issue bodies. Nothing moved any of the first
  eight off the deferred list, because nothing connected the list to the
  crate.
- **Its blockers expired.** #59 deferred most of its roster on missing
  infrastructure. Overlays, focus, keyboard dispatch, virtualised lists and —
  as of this change — a shared control size scale all exist. See the corrected
  blocker table below.
- **Its source moved.** #59's roster is shadcn/ui's component list as it stood
  when it was copied. That list has since changed on both sides — entries added,
  and at least one ("Sonner") folded away. Neither change reached #59, which is
  the clearest available proof that a copied menu has no relationship to the
  project that copied it.

The 29 entries are the union of #59's roster as this repository recorded it:
the "Deferred (see #59)" and "Future — Data & Complex" lists that used to sit in
`todo.md` (21 entries, of which two were *partially* covered — see below), plus
the eight that had already shipped into `src/elements/`. `todo.md` no longer
restates any of it; restating a roster in a second, uncheckable place is how the
first one stayed alive.

The two partial entries:

- **Native Select** — the crate has `Select`, which is a styled trigger and a
  popup. What #59 meant by "Native Select" is the platform control. Rejected
  below.
- **Alert Dialog** — the crate has `Dialog`, which is the right shape. What is
  missing is the *confirmation* affordance: a destructive action, a focused
  default, and a title/description pairing an assistive technology announces as
  one. That is `docs/issues/confirmation-dialog.md`.

## The blocker table, corrected

#146 lists the infrastructure #59's entries were blocked on and marks it
resolved. Every row of that table now holds. The last one that did not —
accessibility roles — held in the same sense the overlay row did: the
mechanism existed all along, and what was missing was a written convention.
Both now exist, in `src/a11y.rs` and `docs/overlays.md`.

| #59's blocker | Status | Evidence |
|---|---|---|
| Overlay/popup rendering | Resolved, and now written down | Six elements (`dialog`, `select`, `popover`, `context_menu`, `sidebar`, `toast`) each place their own `anchored()`/`deferred()`, which is the right amount of sharing — `gpui::anchored()` is the abstraction. `src/traits/portal.rs` (486 lines of positioning math, zero callers) was read against all six and **deleted**; the convention, the fit-mode choice and the draw-priority ladder are in `docs/overlays.md`, checked by `overlay_coverage` in `src/elements.rs` |
| Focus management | Resolved | `FocusHandle` throughout; `Textarea`, `TextField`, `Select`'s `Listbox` and `ContextMenu` all track focus; and since #173 keyboard focus is part of the role announcement — `src/a11y.rs` §4 applies `A11y::focusable` / `focus_handle` / `not_focusable` in the same call that reports the role, `theme::focus_ring` is the one ring, and `bind_focus_keys` supplies the Tab binding gpui ships without. The `VisualFocus` / `FocusStyle` pair that used to be cited here had no implementors in three years and was deleted as part of that change |
| Keyboard dispatch | Resolved | `src/keymap/`, `src/input/bindings.rs`, and `ContextMenu`'s arrow-key navigation with its own tests |
| Scrollable/virtualised lists | Resolved | `src/elements/scroll_area.rs`, `src/elements/list.rs` |
| A shared control size scale | Resolved **by this change** | `src/theme/control.rs` and `src/traits/control_sized.rs`. #59-era components could not state their own metrics; they can now |
| Accessibility roles | Resolved, and now written down | `src/a11y.rs`: an element implements `traits::accessible::Accessible` and applies the `A11y` it returns with one `.announce(a11y)`. `Button` is the worked example, `sidebar.rs` — which shipped ahead of the convention — has been migrated onto it, and `a11y::tests::no_element_calls_gpuis_a11y_builders_directly` fails the build if anything under `src/` calls gpui's builders directly. The decision record is that module's docs; `docs/issues/element-roles-convention.md` records it against the questions it was asked |

The rows that were unresolved when this triage was taken are why it produces
thirteen issue files for what is now six surviving components: three of them
are prerequisites rather than components, and four have since been built —
`table.md` and `data-table.md` by one module, `sidebar.md`, and `resizable.md`
by `src/elements/splitter.rs`. All three prerequisites are now settled:
`overlays.md` by deleting the trait it was about,
`element-roles-convention.md` by `src/a11y.rs`, and `menu-vs-listbox-naming.md`
by merging `Dropdown` into `Select`. Two of the three no longer have a body
under `docs/issues/` at all — a settled question becomes a document in `docs/`,
which is why the count of files there is smaller than the count of outputs
above. `Table` still reports no roles, but nothing blocks it now beyond the
derived cell ids it needs first.

## Roster comparison

For the entries that are neither shipped nor obviously ours, the question worth
asking is not "is it on shadcn's list" but "do the libraries that curate rather
than enumerate ship it". Two useful comparisons:

- **Headless UI** ships a deliberately short set — dialog, disclosure, listbox,
  combobox, menu, popover, radio group, switch, tabs, fieldset, and little
  else. It has no table, no calendar, no carousel, no chart, no OTP input, and
  its form story is `Fieldset` and label association rather than form state.
- **Primer** is a product design system rather than a headless kit, and ships
  what GitHub's own product needed: an action menu, a data table, a pagination
  control, a tree view. Its roster is evidence about *product* need, not about
  what a toolkit owes everyone.

Where the two disagree with shadcn, this triage sides with the shorter list and
says so in the issue body. Three issues argue for building materially less than
#59 imagined: **Form** becomes grouping and label association rather than form
state; **Sidebar** becomes a docked panel rather than a twenty-part API; and
**Resizable** becomes a two-pane splitter rather than a pane tree.

External citations here are by project and component name rather than by file
and line. Anyone implementing one of these issues should re-open the cited
source and read it, not trust a name in a table.

<!-- verdict-table -->

| Component | Verdict | Where |
|---|---|---|
| Accordion | Shipped | `src/elements/accordion.rs` |
| Aspect Ratio | Shipped | `src/elements/aspect_ratio.rs` |
| Breadcrumb | Shipped | `src/elements/breadcrumb.rs` |
| Collapsible | Shipped | `src/elements/collapsible.rs` |
| Context Menu | Shipped | `src/elements/context_menu.rs` |
| Progress | Shipped | `src/elements/progress.rs` |
| Scroll Area | Shipped | `src/elements/scroll_area.rs` |
| Toggle Group | Shipped | `src/elements/toggle_group.rs` |
| Command | Issue | `docs/issues/command.md` |
| Combobox | Issue | `docs/issues/combobox.md` |
| Table | Shipped | `src/elements/table.rs` |
| Data Table | Shipped | `src/elements/table.rs` |
| Resizable | Shipped | `src/elements/splitter.rs` |
| Sidebar | Shipped | `src/elements/sidebar.rs` |
| Calendar | Shipped | `src/elements/calendar.rs` |
| Date Picker | Issue | `docs/issues/date-picker.md` |
| Alert Dialog | Issue | `docs/issues/confirmation-dialog.md` |
| Form | Issue | `docs/issues/form.md` |
| Hover Card | Rejected | keyboard-inaccessible by construction |
| Sheet | Rejected | `Dialog` with an edge and a slide |
| Drawer | Rejected | a phone gesture on a desktop toolkit |
| Menubar | Rejected | an application shell concern, not an element |
| Navigation Menu | Rejected | a website pattern with no consumer here |
| Native Select | Rejected | `Select` already covers it; the platform one is not gpui's to give |
| Sonner | Rejected | `Toast` is this, and its source dropped the name |
| Carousel | Rejected | no consumer, and the accessible version is not the fun one |
| Chart | Rejected | a rendering library, not a component |
| Input OTP | Rejected | one field with a mask, not six fields |
| Pagination | Rejected | wants a paginated data source that does not exist here |

## Two rows, one module

`Table` and `Data Table` both name `src/elements/table.rs`, and that is the
decision `docs/issues/data-table.md` was written to record rather than an
accident of implementation. shadcn separates them because its `Table` is
unstyled markup and its `DataTable` is a TanStack recipe; that split is an
artifact of its ecosystem. Primer ships one `DataTable`. A second `pub mod`
here would have meant a second showcase page, a second `ELEMENT_COVERAGE` row
and a permanent question about which one to reach for — so sorting and
selection are properties of `Table`, off unless a caller opts in.

Both rows stay in the table because both were entries on #59, and a verdict
per entry is the whole point; deleting one to tidy the count would be the
deferral-with-better-manners this document exists to prevent.

What was built, against the two issue bodies:

- From `docs/issues/table.md`: the column model (header, width, alignment,
  per-cell render closure), a header that stays put over a scrolling body,
  wrapping cells, and `ControlSized` off the shared scale. **Not built: the
  content-sized column width.** The header sits outside the scrolled body and
  every row is its own flex container, so a cell sized to its own content is
  measured per row and no two of them agree. Sizing a column to the widest cell
  *in the column* needs a measurement pass across rows, which flex cannot do
  and which gpui's grid — uniform `repeat(n, minmax(_, 1fr))` tracks — cannot
  express either. It wants a hand-written `Element`. `ColumnWidth`'s doc
  comment says so at the point someone reaches for the missing arm, and
  `Column::min_width` recovers most of the use soundly.
- From `docs/issues/data-table.md`: sorting as a column property with the state
  outside the element, row selection, and the answer to its first design
  question — the header checkbox does select all, with the indeterminate middle
  state, but only for a caller that asked for it, because "all" is only
  meaningful where the caller's table has all the rows. Filtering is a
  `TextField` above the table, said in the module docs and demonstrated that way
  on the showcase page. Row virtualisation, column resizing, column visibility
  and multi-column sort are not built, deliberately.
- **Not built from either: the accessibility roles.** The convention that
  blocked them has since landed (`src/a11y.rs`), and both of the findings this
  element recorded for it have been answered: gpui still has no `aria_sort`, so
  a sorted `ColumnHeader` cannot report its direction and that is an upstream
  ask; and `role()` still needs an id, so the roles wait on derived cell ids.
  The roles the element needs are named in its module docs.

## One row, another name

The **Resizable** row is Shipped and names `src/elements/splitter.rs`. The row
keeps #59's name because the table's contract is one row per #59 entry, and
`Resizable` is what #59 called it; the *module* is called something else, and
that is deliberate.

`docs/issues/resizable.md` — kept, because the argument in it is the argument
for the shape that shipped — says the part that is genuinely a toolkit element
is the splitter: one divider, two panes, a drag, a floor under each side, a
keyboard equivalent. Everything above that is an application's layout model.
"Resizable" names a *property* almost anything can have; `Splitter` names the
thing that was built. Three more reasons the name is not a matter of taste:

- `Role::Splitter` is accesskit's own variant, and the element reports it.
- WAI-ARIA calls the pattern **Window Splitter**, and the element implements
  that pattern's keyboard contract.
- This crate has already paid once for a module whose name described something
  it was not — see [`docs/menus-and-listboxes.md`](menus-and-listboxes.md) and
  the deleted `Dropdown`.

The line above, "**Resizable** becomes a two-pane splitter rather than a pane
tree", already said this in prose before anything was built. The module now
agrees with it, with the role it reports, and with the pattern it implements.

Not built, and named here so the next person does not go looking: a pane tree,
nested groups, persisted layout, a collapse gesture, or any notion of more than
two panes. Three panes is two splitters, nested by the caller.

## The rejections, argued

**Hover Card.** A card that appears on hover and nowhere else has no keyboard
equivalent and no touch equivalent; the accessible version of it is a
`Popover` with a trigger, which the crate already has. Building it would mean
shipping a component whose primary interaction the crate's own accessibility
convention would have to make an exception for. *Revisit if* a consumer needs
hover-preview specifically and is willing to specify the focus and dismiss
behaviour that makes it usable without a mouse — at which point it is a
`Popover` variant, not a new element.

**Sheet.** A `Dialog` anchored to an edge of the window that slides in instead
of fading. Every hard part — the scrim, the focus trap, the dismiss handling —
is `Dialog`'s and is already solved. Shipping it as its own element duplicates
all of that to change two style properties. *Revisit if* `Dialog` grows an
`edge` / `presentation` option and someone wants a name for the common case;
that is a `Dialog` feature request, not this.

**Drawer.** The gesture-driven, drag-to-dismiss bottom sheet from mobile. It
depends on touch velocity and a rubber-banding scroll model that gpui does not
expose and this toolkit's consumers do not target. *Revisit if* gpuikit gains a
touch-first target.

**Menubar.** The strip of application menus along the top of a window is a
shell concern: on macOS it is the system menu bar, which is the platform's to
draw, and on other platforms it is a window-chrome decision an application
makes once. `ContextMenu` already covers the popup-menu mechanics. *Revisit if*
gpui grows a cross-platform application-menu API worth wrapping.

**Navigation Menu.** A multi-column mega-menu for website navigation. Nothing
that consumes this toolkit is a website. *Revisit if* a consumer appears with
hierarchical navigation that `List` plus `Popover` genuinely cannot express.

**Native Select.** `Select` covers the use. The distinct thing #59 meant — a
control drawn by the operating system — is not something gpui exposes, and
would not match the theme if it did. *Revisit if* gpui gains platform control
embedding, which would change far more than this.

**Sonner.** This is `Toast` with a different name. It was a specific
third-party library that shadcn's roster wrapped, and that roster has since
dropped the entry. Carrying it forward would be carrying forward a naming
accident. *Revisit* never as an element; if `Toast` lacks stacking, swipe
dismissal or promise states, those are `Toast` issues.

**Carousel.** No consumer has asked for one, and the version worth shipping —
keyboard-reachable slides, an announced position, respect for
`App::reduce_motion` — is a large component that nothing here would use. The
version that is quick to build is the one that should not ship. *Revisit if* a
consumer needs one and can say what its keyboard model is.

**Chart.** Charting is a rendering library — scales, axes, layout algorithms,
data binding — that happens to draw into a UI toolkit. It is out of proportion
with everything else in `src/elements/`, and a partial one is worse than none.
*Revisit if* gpuikit's scope explicitly widens to data visualisation, which
should be its own decision rather than a row on a roster.

**Input OTP.** The six-separate-boxes treatment is a visual convention layered
over a single value, and implementing it as six inputs breaks paste, breaks
selection, breaks screen readers and breaks password managers. The version
worth building is one `TextField` with a segmented mask. *Revisit if* a
consumer needs one-time-code entry, at which point it is a `TextField` masking
feature.

**Pagination.** A page-number strip is trivial to draw and useless without a
paginated data source to drive it. Shipping the buttons first produces a
component whose only demonstration is a showcase page that pages through
nothing. *Revisit* once something has more rows than fit — of which **half has
now happened**: `src/elements/table.rs` exists, so the "which means `Table`,
which is not built" half of the original argument has expired, but no consumer
has a paginated data source and the showcase's ten rows are ten rows. The
rejection stands on the surviving half. `docs/issues/data-table.md` asked for
exactly this to be recorded when the table landed.

## Prerequisites

Three of the triage's outputs are not components. They are decisions that the
component issues would otherwise each have to invent an answer to. All three
are now settled, and their answers are `src/a11y.rs`,
[`docs/menus-and-listboxes.md`](menus-and-listboxes.md) and
[`docs/overlays.md`](overlays.md).

- **`docs/issues/element-roles-convention.md`** — **settled.** #146 makes the
  a11y answer a precondition for every new component, so each issue would
  otherwise have picked a different mechanism. It is decided once, in
  `src/a11y.rs`: an element implements `traits::accessible::Accessible` and
  applies the `A11y` it returns with `.announce(a11y)`, which gpui only offers
  to an element that already has an id. `Button` is the worked example, and
  `src/elements/sidebar.rs` — which had shipped a role ahead of the decision,
  because its own issue's Accessibility section required a `Complementary`
  landmark — was migrated with it, as that issue said it would have to be.
  `src/elements/table.rs`, which shipped without roles rather than invent a
  mechanism, is unblocked; it still needs derived cell ids before it can use
  one.
- **`docs/menus-and-listboxes.md`** — **settled.** `Select` was built on
  `Dropdown`'s internals (`select.rs` imported `DropdownMenu` and
  `DropdownOption` from `dropdown.rs`) and `ELEMENT_COVERAGE` mapped both
  modules to one showcase page, because they were one component under two
  names. `dropdown.rs` is deleted, `Select` took the union of the two APIs, and
  the popup is now a private `Listbox` — private because a public one is what
  let the two grow into each other. A chooser and a menu of actions are
  different things with different roles and different keyboard models, and the
  document says so in a sentence the module doc repeats. The Combobox block is
  discharged.
- **`docs/overlays.md`** — **settled.** `src/traits/portal.rs` was unused
  positioning math, and the answer to "what would it have saved" was nothing at
  all six overlay call sites: `gpui::anchored()` already does every one of its
  jobs, and does them in `prepaint`, where the measured sizes it demanded from
  its callers actually exist. Deleted, and replaced by a written convention
  with tests. The three components below that were blocked on it now follow
  that document instead.

## The dependency graph

- `docs/issues/element-roles-convention.md` shapes every component issue, and
  is what the shipped `Table` is still waiting on for its roles.
- `docs/menus-and-listboxes.md` hard-blocked `combobox.md`. **Discharged** — it
  was `docs/issues/menu-vs-listbox-naming.md`, now settled by merging
  `Dropdown` into `Select`. `combobox.md` inherits the answer, including where
  its popup comes from.
- `docs/overlays.md` is the convention `command.md`, `combobox.md` and
  `date-picker.md` follow when they place an overlay. **Discharged** — it was
  `docs/issues/portal-adopt-or-delete.md`, now settled by deleting the trait.
- `docs/issues/table.md` hard-blocked `data-table.md`. **Discharged**: both are
  built, as one module — see "Two rows, one module" above.
- `docs/issues/calendar.md`, and the date-type decision in it, hard-blocks `date-picker.md`.
- `docs/issues/resizable.md` blocked the resizable edge of `Sidebar`.
  **Discharged** — `src/elements/splitter.rs` exists. `Sidebar` is not migrated
  onto it: its own issue said it could ship without one, and adopting a
  splitter changes that element's API. That is its own decision, not a
  consequence of this one.
- `docs/issues/confirmation-dialog.md` needs a destructive `ButtonVariant`, which does not
  exist — `src/elements/button.rs` has only `Filled`, with its own `// todo`.

## What keeps this honest

`src/elements.rs` carries a `triage_coverage` test module that parses the
verdict table above — anchored by the `<!-- verdict-table -->` comment, so the
document's other tables are not mistaken for it — and fails the build when the
document stops describing the crate:

- the table has one row per #59 entry, and the verdict split matches the
  13 Shipped / 5 Issue / 11 Rejected stated in prose here;
- every Shipped row names a `src/elements/<module>.rs` that is really declared;
- every Issue row names a file under `docs/issues/` that exists and is not a
  stub;
- every file under `docs/issues/` is reachable from this document;
- every Rejected row is argued in the section above, not just asserted in a
  cell;
- the attribution section above still says who took the verdicts, still says
  the rejections are proposed rather than ratified, and still states the same
  three counts the table has — because an unchecked claim in prose is exactly
  how this document's missing attribution went unnoticed in the first place.

A sibling `overlay_coverage` module does the same for
[`docs/overlays.md`](overlays.md): every `src/elements/` module that calls
`deferred(` has a row in its overlay table and vice versa, every
`with_priority(n)` literal is a rung of the ladder it states, and the deleted
`src/traits/portal.rs` has not come back.

A third, `family_coverage`, does the same for
[`docs/menus-and-listboxes.md`](menus-and-listboxes.md): every row of its family
table names a real `pub mod` and one of the two families, no module in one
family names a module in the other in a Rust path, and the deleted
`src/elements/dropdown.rs` has not come back.

The counts are stated in two places on purpose: editing the table without
editing the prose fails. If a verdict legitimately changes, both move together.
If a component ships, its row moves to Shipped and must then name its module —
the test will demand it.

## What became of #59

**#59 is closed, and this document is what replaced it.** It was closed
`COMPLETED` on 2026-03-25, five months before this file existed, so there is no
note to post and nothing left to gate: whoever closed it did so without a
replacement, which is exactly the failure "Provenance" above describes. The
substance is there — #59 was shadcn/ui's roster rather than a decision, and it
went stale because nothing connected it to the crate.

What is different now is that the replacement is checked. A Shipped row has to
name a module that exists, an Issue row has to name an issue body that exists,
and a Rejected row has to be argued in prose; "What keeps this honest" above
says which test does which. If one of the rejections is wrong, argue with the
paragraph — that is what it is there for, and per the attribution section none
of them has been ratified yet.

An earlier draft of this section carried a ready-to-paste closing comment with
its own counts in it. It is gone rather than corrected: it asserted a pending
action on an issue that had been closed for months, and its counts were the one
part of a machine-checked document that no test reached. Counts live in the
attribution section and in the verdict table, where `triage_coverage` can see
them.
