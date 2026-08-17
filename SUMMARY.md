# Merge `Dropdown` into `Select`, and write down what a listbox is

`Select` and `Dropdown` were one component under two names: the same bordered
trigger with a chevron, the same popup one gap below it, the same `ControlSize`
— and `select.rs` importing `DropdownMenu`, `DropdownOption` and `MENU_GAP`
*from* `dropdown.rs` to get there. The only behavioural difference was that a
`Dropdown`'s selection could not be absent, which is a constructor argument
rather than a component. So `src/elements/dropdown.rs` is deleted and `Select`
takes the union of the two APIs: `dropdown(id, options, v)` becomes
`select(id, options).selected(v)`, and `selected` is an `Option<T>` for
everyone. The shared popup is renamed `Listbox` and made **private** to
`src/elements/select.rs` — that privacy is the part of the decision that
enforces itself, because a public popup type is what let one element get built
on another's internals in the first place. Its selected row is now an
`Option<usize>` rather than the `usize::MAX` sentinel `select.rs` used to pass
for "nothing chosen", and there is a test for the state that sentinel stood
for. The menu family (`ContextMenu`) keeps its own popup, row vocabulary and
keyboard model, and shares only `docs/overlays.md`.

The decision, the sentence that separates the two families, and the migration
table live in a new `docs/menus-and-listboxes.md`, held to the crate by a new
`family_coverage` test module in `src/elements.rs` in the same shape as the
`triage_coverage` and `overlay_coverage` modules beside it: every row of the
family table names a real `pub mod` and one of the two families, no module in
one family names a module in the other in a Rust path, and `dropdown.rs` has
not come back. `docs/issues/menu-vs-listbox-naming.md` is deleted following the
precedent of `portal-adopt-or-delete.md` (a settled question becomes a document
in `docs/`), and `docs/issues/combobox.md`'s hard block is rewritten as an
inherited answer — including the instruction to lift `Listbox` into a
`pub(crate)` module named by both callers rather than making it `pub` where it
sits. `docs/overlays.md`, `docs/component-triage.md`, `todo.md` and `README.md`
are brought in line, the showcase has one page for one chooser (showing both
shapes a select comes in, with and without a value), and the CHANGELOG carries
the migration table. Two small consumer follow-ons: `examples/input/sandbox.rs`
now builds three selects, and `control_size_tests.rs` measures one fewer
control on its row.

The layering test was fault-injected before being trusted: adding
`use crate::elements::context_menu::menu_item;` to `select.rs` makes
`neither_family_is_built_on_the_other` fail with the intended message, and the
import was then removed. Its floor is `checked >= 1` cross-family pairs rather
than a per-module one on purpose — an element that imports nothing from a
sibling is the state this decision wants, so a per-module floor would punish
success.

One deviation from the spec worth flagging: the spec asked for the new
CHANGELOG entry to *replace* the `DropdownMenu::build` bullet "from earlier in
the same unreleased cycle". That bullet is actually inside the dated, already
cut `## [0.8.0]` section, where it is a true record of what 0.8.0 shipped, so
it is left in place and the new Unreleased entry says explicitly that it is now
moot. Everything else follows the spec as written.

Verification: PASSED — `cargo test --lib` (379 passed / 0 failed), with `cargo fmt --check`, `cargo check --examples --all-features` and `cargo doc --no-deps` clean, and `cargo clippy --all-targets --all-features` adding no warning (three `type_complexity` warnings fewer than baseline, from the deleted module)
