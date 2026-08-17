# Merge `Dropdown` into `Select`, and write down what a listbox is

`Select` and `Dropdown` were one component under two names — the same bordered
trigger with a chevron, the same popup one gap below it, the same `ControlSize`,
and `select.rs` importing `DropdownMenu`, `DropdownOption` and `MENU_GAP` *out
of* `dropdown.rs` to get there. The only behavioural difference was that a
`Dropdown`'s selection could not be absent, which is a constructor argument
rather than a component. So `src/elements/dropdown.rs` is deleted and `Select`
takes the union of the two APIs: `dropdown(id, options, v)` becomes
`select(id, options).selected(v)`, and `selected` is an `Option<T>` for
everyone. The shared popup is renamed `Listbox` and made **private** to
`src/elements/select.rs` — that is the part of the decision that enforces itself,
since a public popup type is exactly how the next component in this
neighbourhood gets built *on* this one rather than beside it. Its selected row
is now an `Option<usize>` instead of the `usize::MAX` sentinel `select.rs` used
to pass for "no row". The menu family (`ContextMenu`) keeps its own popup, row
vocabulary and keyboard model, and shares only `docs/overlays.md`.

The decision, the sentence that distinguishes the two families, the migration
table and the reservation of the freed `DropdownMenu` name (a future
menu-of-actions-from-a-button, built on `context_menu.rs`'s items) live in a new
`docs/menus-and-listboxes.md`, held to the crate by a new `family_coverage` test
module in `src/elements.rs`: every row of its family table has to name a real
`pub mod` and one of the two families, neither family may `use` the other, and
`src/elements/dropdown.rs` may not come back. That layering test was
fault-injected — adding `use crate::elements::context_menu::menu_item;` to
`select.rs` makes it fail with the intended message — and then restored.
`docs/issues/menu-vs-listbox-naming.md` is deleted following the precedent of
`portal-adopt-or-delete.md`, and `docs/issues/combobox.md`'s hard block becomes
an inherited answer, including the instruction to lift `Listbox` into a
`pub(crate)` module named by both callers rather than making it public. The two
overlay-placement tests moved from `dropdown.rs` to `select.rs` — they are the
crate's only test of `docs/overlays.md` — joined by a new one covering the state
the sentinel stood for. The showcase now has one page for one chooser, showing
both shapes (a select that starts with a value and one that starts with a
placeholder); `examples/input/sandbox.rs` and the control-size row follow.
`CHANGELOG.md` carries the full migration table under Unreleased. Breaking
changes are sanctioned by `README.md`; what disappears is
`gpuikit::elements::dropdown` in full.

Verification: PASSED — `cargo test --lib` (363 passed / 0 failed), `cargo fmt --check` (clean), `cargo check --examples`, `cargo check --all-features` and `cargo clippy --all-targets` (only pre-existing warnings; the three in `select.rs` are the `Rc<dyn Fn…>` fields the lint already flagged in both merged modules), plus `cargo doc --no-deps` (the same 8 pre-existing warnings, none about `Listbox` or `Select`).
