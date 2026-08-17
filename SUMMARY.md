# Table, and a `Copy` that propagates when an input has nothing selected

Two changes, both self-contained.

**`InputState::copy` no longer swallows the copy action when its selection is
empty.** gpui clears `propagate_event` before every bubble-phase listener, so
an empty-selection `copy` that simply returned was indistinguishable from one
that handled the action: it stopped at the input and never reached a handler
further out on the focus path. A focused composer therefore ate ⌘C, and a
markdown selection visible elsewhere in the same window could not be copied.
The empty branch now calls `cx.propagate()`; an input that does have a
selection consumes the action exactly as before, and `cut` is deliberately
untouched because an empty selection means something there (it cuts the current
line). The existing clipboard tests could not have caught this — they call
`copy` directly, outside any dispatch, on a view that renders a bare `div()` —
so the regression test renders a real `text_area` under a `div` that handles
`Copy` and dispatches at the focused input. It is mutation-checked: with the
`cx.propagate()` removed, that test fails and the other three still pass.

**`Table` ships as one element rather than two.** #161 asked for the
interactions that turn a grid into a data view, and `docs/issues/data-table.md`
recommended folding them into `Table`; `Table` did not exist, so
`src/elements/table.rs` builds both. A `Column<R>` carries a header, a width, an
optional minimum width, an alignment and a per-cell render closure returning any
element; the header is a sibling of the body rather than its first child, which
is how it stays put while `max_h` caps and scrolls the body; cells wrap. The
data-view state stays with the caller — the table is handed rows that are
already filtered and already sorted plus the `SortDescriptor` describing how,
and reports `SortRequest` / `SelectRequest` / `SelectAllRequest` back — so
nothing moves until the caller moves it, and a `sortable()` column with no
`on_sort` is inert on purpose. The header checkbox selects all, with the
indeterminate middle state, but only for a caller that asked for it with
`on_select_all`, because "all" is only meaningful where the caller's table has
all the rows. Filtering is a `TextField` above the table, said in the module
docs and demonstrated that way on the new showcase page, which owns the sort
comparator and a `HashSet` of selected ids and re-derives its rows in the
handlers — that round trip is the demonstration. `ColumnWidth` has `Flex` and
`Fixed` arms and no content-sized one: with the header outside the scrolled body
and every row its own flex container, a cell sized to its own content is
measured per row and no two agree, which wants a hand-written `Element`;
`Column::min_width` recovers most of the use and the doc comment argues the
rest. `CheckState` and `checkbox_box()` are extracted from `Checkbox` (whose API
and rendering are unchanged) because a table cannot mint an entity per row and
would otherwise have drawn its own approximation of the box. No accessibility
roles are reported — `docs/issues/element-roles-convention.md` is still open —
but the roles the element will need are recorded in its module docs along with
two findings that decision has to cover: gpui has no `aria_sort`, and `role()`
needs an id, which turns body cells into id-minting sites.
`docs/component-triage.md`, its verdict counts, `todo.md` and `CHANGELOG.md`
follow, including the note `data-table.md` asked for: half of Pagination's
revisit trigger has now fired (`Table` exists) and half has not (still no
paginated data source), so that rejection stands on the surviving half.

Verification: PASSED — `cargo test -j 1` (all targets: 316 lib tests + 2
doc-tests, examples built), `cargo test --lib --features editor,schema -j 2`
(463 tests), `cargo fmt --check` clean, and `cargo clippy --all-targets` reports
no errors and no warnings from the touched files (the crate's other warnings are
pre-existing).
