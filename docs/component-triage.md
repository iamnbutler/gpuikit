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

## Provenance: why #59 stopped meaning anything

Three things are checkable, and all three hold:

- **Its entries shipped without it noticing.** Eight rows below are Shipped —
  #146 counted seven; re-reading the list against `src/elements/` found one
  more, which is itself the point. Nothing moved any of them off the deferred
  list, because nothing connected the list to the crate.
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
resolved. One row of that table does not hold.

| #59's blocker | Status | Evidence |
|---|---|---|
| Overlay/popup rendering | **Resolved in practice, not in one place** | Six elements (`dialog`, `dropdown`, `select`, `popover`, `context_menu`, `toast`) each hand-roll `anchored()`/`deferred()`. `src/traits/portal.rs` — 486 lines of positioning math — has **zero callers, zero implementors and zero tests**. See `docs/issues/portal-adopt-or-delete.md` |
| Focus management | Resolved | `FocusHandle` throughout; `src/traits/visual_focus.rs`; `Textarea`, `TextField`, `DropdownMenu` and `ContextMenu` all track focus |
| Keyboard dispatch | Resolved | `src/keymap/`, `src/input/bindings.rs`, and `ContextMenu`'s arrow-key navigation with its own tests |
| Scrollable/virtualised lists | Resolved | `src/elements/scroll_area.rs`, `src/elements/list.rs` |
| A shared control size scale | Resolved **by this change** | `src/theme/control.rs` and `src/traits/control_sized.rs`. #59-era components could not state their own metrics; they can now |
| Accessibility roles | **Not resolved** | `grep -rn '\.role(' src/elements/` returns nothing. The crate's only a11y is in `src/markdown/`. See `docs/issues/element-roles-convention.md` |

The two unresolved rows are why this triage produces thirteen issue files for
ten components: three of them are prerequisites, not components.

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
| Table | Issue | `docs/issues/table.md` |
| Data Table | Issue | `docs/issues/data-table.md` |
| Resizable | Issue | `docs/issues/resizable.md` |
| Sidebar | Issue | `docs/issues/sidebar.md` |
| Calendar | Issue | `docs/issues/calendar.md` |
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
paginated data source to drive it — which means `Table`, which is not built.
Shipping the buttons first produces a component whose only demonstration is a
showcase page that pages through nothing. *Revisit* once `docs/issues/table.md`
lands and something has more rows than fit.

## Prerequisites

Three issue bodies under `docs/issues/` are not components. They are decisions
that ten component issues would otherwise each have to invent an answer to.

- **`docs/issues/element-roles-convention.md`** — no element in `src/elements/` reports an
  accessibility role. #146 makes the a11y answer a precondition for every new
  component, so ten issues would each pick a different mechanism. Decide it
  once. Should land before the first component issue that reports a role.
- **`docs/issues/menu-vs-listbox-naming.md`** — `Select` is built on `Dropdown`'s
  internals (`select.rs` imports `DropdownMenu` and `DropdownOption` from
  `dropdown.rs`) and `ELEMENT_COVERAGE` maps both modules to one showcase page.
  A chooser and a command list are different things with different roles and
  different keyboard models. Hard-blocks Combobox.
- **`docs/issues/portal-adopt-or-delete.md`** — `src/traits/portal.rs` is unused
  positioning math and the six existing overlays each hand-roll their own.
  Three of the surviving components are anchored overlays. Settle this before
  a seventh hand-rolled placement.

## The dependency graph

- `docs/issues/element-roles-convention.md` shapes all ten component issues.
- `docs/issues/menu-vs-listbox-naming.md` hard-blocks `combobox.md`.
- `docs/issues/portal-adopt-or-delete.md` should be settled before `command.md`,
  `combobox.md` or `date-picker.md`.
- `docs/issues/table.md` hard-blocks `data-table.md`.
- `docs/issues/calendar.md`, and the date-type decision in it, hard-blocks `date-picker.md`.
- `docs/issues/resizable.md` blocks the resizable edge of `sidebar.md`.
- `docs/issues/confirmation-dialog.md` needs a destructive `ButtonVariant`, which does not
  exist — `src/elements/button.rs` has only `Filled`, with its own `// todo`.

## What keeps this honest

`src/elements.rs` carries a `triage_coverage` test module that parses the
verdict table above — anchored by the `<!-- verdict-table -->` comment, so the
document's other tables are not mistaken for it — and fails the build when the
document stops describing the crate:

- the table has one row per #59 entry, and the verdict split matches the
  8 Shipped / 10 Issue / 11 Rejected stated in prose here;
- every Shipped row names a `src/elements/<module>.rs` that is really declared;
- every Issue row names a file under `docs/issues/` that exists and is not a
  stub;
- every file under `docs/issues/` is reachable from this document;
- every Rejected row is argued in the section above, not just asserted in a
  cell.

The counts are stated in two places on purpose: editing the table without
editing the prose fails. If a verdict legitimately changes, both move together.
If a component ships, its row moves to Shipped and must then name its module —
the test will demand it.

## Closing note for #59

Ready to paste:

> Closing in favour of [`docs/component-triage.md`](../docs/component-triage.md),
> which gives every entry on this list a verdict: 8 had already shipped, 11 are
> rejected with a reason and a named revisit trigger, and 10 have a
> ready-to-file issue body under `docs/issues/` (plus three prerequisites the
> triage surfaced — an element role convention, a menu-vs-listbox naming
> decision, and adopt-or-delete for `src/traits/portal.rs`).
>
> This list was shadcn/ui's roster rather than a decision, and it went stale
> because nothing connected it to the crate. The replacement is checked by
> tests in `src/elements.rs`: a Shipped row has to name a module that exists,
> an Issue row has to name an issue body that exists, and a Rejected row has to
> be argued in prose. If one of the rejections is wrong, argue with the
> paragraph — that is what it is there for.
