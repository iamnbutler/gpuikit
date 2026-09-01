# SplitView: column coordination, not a pane tree

## What it is

A coordinated two- or three-column layout — sidebar, optional content column, detail — in the spirit of SwiftUI's `NavigationSplitView`: each column has a preferred and a minimum width, the set of visible columns is a value the caller owns, columns give way in a defined order as the window narrows, and one trigger toggles the sidebar knowing which of those states it is actually toggling between.

This issue is **partly an element, partly a recipe**, weighted toward the recipe. What ships is small: a `SplitViewState` value naming which columns the caller wants visible, a pure `SplitViewLayout::resolve` that turns available width plus column preferences into per-column widths and a presentation per column, and a thin `RenderOnce` that applies that resolution to three slots by composing the elements that already exist. Everything with an opinion about *content* — what goes in the sidebar, where selection lives, what the detail shows when nothing is selected — is module-doc prose, in the tradition of `Table`'s filtering answer ("a `TextField` above the table, said in the module docs").

## Why it survives triage

`docs/component-triage.md` already rejected the pane tree, and this issue does not reopen that: "three panes is two splitters, nested by the caller" stands. A pane tree is arbitrary nesting with splitting, joining and serialisation — a workspace structure. A split view is a *fixed* arrangement of at most three named columns with navigation roles, which is a different and much smaller thing; SwiftUI, which has both `HSplitView` and `NavigationSplitView`, ships them as different components for exactly this reason.

What Sidebar and Splitter leave on the table is the coordination, and it is real:

- **`SidebarLayout::resolve` keys off the window, on purpose and with a documented apology** — `src/elements/sidebar.rs` says a `RenderOnce` "gets nothing about its parent's box", so a sidebar inside a 400px pane on a 1400px window still pushes. A split view that draws the row *is* the parent, can measure its own box the way `src/elements/splitter.rs`'s canvas does, and can finally make the collapse decision against the width that actually exists.
- **Nothing decides which column gives way first.** Sidebar knows expanded/rail/drawer for one column; Splitter clamps one boundary between two floors. Three columns under a shrinking window need an ordering — and both SwiftUI and libadwaita treat that ordering as the component's whole job.
- **`SidebarTrigger` toggles a `SidebarState`, but in a coordinated layout the toggle's meaning depends on the layout**: above the breakpoint it means rail/expanded; below it, open/close the drawer. Today the caller writes that branch by hand or gets it wrong.

Deliberately **not** built: a pane tree (rejected, stays rejected), persisted layout, a container that owns child state, drag-to-collapse gestures, and the detail column's own drill-in navigation — that is NavigationStack's issue, and SwiftUI splits it the same way ("the split view doesn't provide a stack for the detail closure on purpose").

## Prior art

- **SwiftUI `NavigationSplitView`** ([developer.apple.com/documentation/swiftui/navigationsplitview](https://developer.apple.com/documentation/swiftui/navigationsplitview)). Take: `columnVisibility` as a **binding the caller owns** (`NavigationSplitViewVisibility`: `.all` / `.doubleColumn` / `.detailOnly` / `.automatic`) — the exact shape of `SidebarState`, widened to three columns; `navigationSplitViewColumnWidth(min:ideal:max:)` as per-column width preferences the layout "does its best to accommodate"; and the collapse rule that the stack "shows the last column that displays useful information" — the detail keeps priority. Also take its restraint: the visibility binding is *ignored* while collapsed. Leave: `NavigationSplitViewStyle` prominence styling, and the compact-width collapse-into-a-stack — that is a phone behavior; this crate's narrow-window answer is Sidebar's drawer, which already exists.
- **libadwaita `AdwNavigationSplitView`** ([class.NavigationSplitView.html](https://gnome.pages.gitlab.gnome.org/libadwaita/doc/main/class.NavigationSplitView.html)). Take: the sidebar width model — `sidebar-width-fraction` (default 25%) clamped by `min-sidebar-width` (180sp) and `max-sidebar-width` (280sp) — fraction-with-floors is precisely the marriage of Splitter's ratio and SwiftUI's points that "What it has to close" below needs; and the breakpoint model — `collapsed` is a plain property "typically used together with an `AdwBreakpoint`", i.e. the *decision* to collapse is external and declarative, the *choreography* is the component's. Leave: the collapsed state becoming an `AdwNavigationView` — that is the phone stack again.
- **In-house: `SidebarLayout::resolve` and `SplitterGeometry`** (`src/elements/sidebar.rs`, `src/elements/splitter.rs`). Take the shape whole: behavior as a pure function over widths, unit-tested with no window, state as a caller-owned value. `SplitViewLayout::resolve` is the third of the family.

Re-open every one of these before implementing; the API names above are as of August 2026 and both toolkits move.

## What it has to close in this crate

- **The container-width measurement.** `resolve` must run against the split view's own measured bounds, not the window. The mechanism exists: the `canvas`-measures-then-notifies pattern in `src/elements/splitter.rs` (`SplitterState::container`), including its one-frame-behind caveat and the documented first-frame answer. This is the item that makes SplitView an element rather than pure prose — a recipe cannot measure anything.
- **Feeding Sidebar a decision instead of letting it take one.** `Sidebar` decides push-versus-overlay from `window.viewport_size()`. Composed under SplitView, that decision must be SplitView's. Either `Sidebar` grows a way to accept a resolved `SidebarPresentation` (additive, and `SidebarLayout::resolve` was written "so there is one place to argue with if push-versus-overlay should key off something else" — this is that argument arriving), or SplitView drives `never_overlay` plus its own drawer. The first is right; the second duplicates the drawer. Name the change in `src/elements/sidebar.rs` and keep it additive.
- **The collapse order, as one pure function.** Three columns, shrinking width: (1) all fit at preferred widths, boundaries resizable; (2) squeeze columns toward minimums, detail last; (3) sidebar to rail (Sidebar's `Collapsed`); (4) content column yields to detail — detail is "the last column that displays useful information"; (5) below Sidebar's `overlay_below`, the expanded sidebar is a drawer. Every threshold derives from the columns' declared minimums plus the rail width from `SidebarMetrics` — no new named breakpoints beyond the one Sidebar already has.
- **Widths versus ratios.** Splitter speaks ratio; column preferences are lengths (`Rems`, like `Sidebar::width`). The adwaita answer — preferred as fraction-or-length, clamped by min/max lengths — resolves to pixels inside `resolve`, and the sidebar|content boundary being user-draggable is `Splitter` with `min_start`/`min_end` set from the same column minimums, the resulting width emitted through the same caller-owned channel as `Splitter::on_resize`. One source of truth for floors, stated in the module docs.
- **The trigger.** `sidebar_trigger` already carries `aria-expanded` and the panel-glyph pair. What it needs is to be handed the *resolved* state so its toggle targets the right transition. `src/elements/sidebar.rs` says "Resist adding a second" sub-component — resist it here too: this is a parameter, not a new element.
- **`SplitViewState`.** A value like `SidebarState`: which columns the caller wants visible (`All` / `SidebarAndDetail` / `DetailOnly`), with `From<bool>`-style conveniences, owned by the caller, ignored where the width forbids it — SwiftUI's rule, adopted verbatim.
- **What stays prose.** Selection flowing sidebar→content→detail, the detail placeholder when nothing is selected, a `List` as sidebar content: all recipes in the module docs, demonstrated on the showcase page, exactly as `Table` answered filtering.

## Accessibility

Against `src/a11y.rs`'s mechanism (`Accessible` + one `.announce(a11y)`):

- **The split view itself announces nothing.** It is a layout wrapper with no semantics of its own — the same argument `ELEMENTS_WITHOUT_A_ROLE` already records for `aspect_ratio`, and gpui rejects a `GenericContainer` escape anyway. If it ships silent it must be listed there with this reason; better, it ships with nothing *to* list because its columns carry the roles.
- **The columns keep their landmarks.** The sidebar column is Sidebar's `Role::Complementary` with the caller's label; the boundaries are Splitter's named `Role::Splitter` bands, tab stops by declaration per a11y section 4. SplitView must not wrap either in a second role — the sidebar drawer's no-role gutter in `src/elements/sidebar.rs` is the worked example of *not* duplicating a landmark.
- **`aria-expanded` stays on the trigger** — a11y section 3's rule, "state goes on the element that changes it", already implemented by `SidebarTrigger`; the coordinated state must keep reporting through it.
- **Focus must survive a collapse.** When the column containing focus leaves the layout (content column yields, sidebar goes to rail), focus must land on the trigger rather than silently falling to the window — a keyboard user whose focus evaporates has no way back. This is the one new focus decision this component takes, and it goes through `A11y`'s machinery, not ad-hoc `track_focus`.

## Sizing

`ControlSized`, off the shared scale: the rail width is already `SidebarMetrics::from_control`, the band and floors already `SplitterMetrics::for_rung`, and SplitView adds no named dimension of its own — default column minimums derive from the rung the way `SplitterMetrics::default_floor` does (per the "What belongs here" note atop `src/theme/control.rs`, anything specific to this shape stays in its file, keyed off the rung).

## Showcase

A build requirement, not a convention: `showcase_coverage` in `src/elements.rs` fails the build for a `pub mod` with no `ELEMENT_COVERAGE` row. The page shows the three-column composition live — resize the demo boundary and watch the collapse order fire — plus the two-column form, the toggle in all three regimes, and the detail placeholder recipe. The showcase's own chrome (nav sidebar + page) is the crate's first consumer; adopting SplitView there is the proof the coordination earns its keep.

## Non-goals

- A pane tree, nested groups, splitting/joining, persisted layout — rejected in `docs/component-triage.md` and `docs/issues/resizable.md`; nothing here reopens it.
- Owning navigation or selection state. Every stateful thing is a caller-owned value.
- Drill-in inside the detail column — `NavigationStack`'s issue (`docs/issues/navigation-stack.md`).
- Phone-style collapse into a single stack; the narrow answer is Sidebar's drawer.

## Blocked on

- The additive change to `src/elements/sidebar.rs` letting a parent supply the resolved presentation — small, named above, and worth landing as its own PR since `docs/component-triage.md` already notes Sidebar API changes are "its own decision".
