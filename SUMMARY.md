# Combobox (#160) and Command (#159)

Two chooser components, and the shared machinery they are both built out of.

`Listbox` — the popup, its `actions!` block, `LISTBOX_CONTEXT`, `LISTBOX_GAP`,
`option_a11y` and the bindings — was lifted out of `src/elements/select.rs`
into a new **`pub(crate) mod listbox`**, which is the move
`docs/menus-and-listboxes.md` §2 prescribed for the moment a second caller
appeared. It is `pub(crate)` and not `pub`: it is not a component, it is the
thing two components are made of, and being `pub(crate)` is also what keeps it
out of `showcase_coverage`'s and `family_coverage`'s tables. `select.rs` keeps
`bind_select_keys` as a one-line delegate and `LISTBOX_CONTEXT` and the six
action types as re-exports, because all of them are public API today and
`crate::init` calls the first. The one behavioural addition is `ListboxFocus`,
which decides whether the popup takes real focus (a `Select`) or leaves it on
the caller's text field (a `Combobox`) — and it gates all three places that
differ: the `window.focus` call, the `restore_focus` read, and whether a row may
claim `active_descendant`.

`src/elements/combobox.rs` is a `TextField` that owns that popup. The state
holds the **value** (`selected: Option<T>`) and the **text** (an
`Entity<InputState>`) separately, because they diverge the moment the user
types; `visible: Vec<usize>` maps a popup row back to an option index, which is
the bug that only appears once a filter is active. Typing clears the value, and
`UnmatchedText` (`Revert` by default, `Keep`, `Create`) decides what happens to
unmatched text on blur. `src/elements/command.rs` is a filterable list of
actions over a scrim at the dialog rung, with the selection model borrowed in
shape from `context_menu.rs` and the matcher left to the caller.

**The keyboard is the same answer twice, and it is binding depth.** Both
components need `up`, `down`, `enter` and `escape` from a field that keeps real
focus, and `src/input/bindings.rs` already binds all four in `INPUT_CONTEXT` on
that field. Each registers its bindings twice — once under
`"Combobox > Input"` / `"CommandPalette > Input"`, which matches at the field's
own node and so *ties* on depth, and once under the bare context — and
`crate::init` calls both **after** `input::bind_input_keys`, because gpui breaks
a depth tie by later registration. Each action calls `cx.propagate()` when the
component has nothing to do with it, which is the `InputState::copy` shape and
what lets Escape reach an enclosing `Dialog`. No raw `on_key_down` was added
anywhere: the only one in the lifted code is `Listbox`'s existing type-ahead,
which stays because a binding per letter is not a keymap.

`cargo test --lib` is **green: 533 passed, 0 failed** (517 on the trunk).
`cargo check --example showcase --features examples` also passes.

## Review feedback

### Spec 1 — Combobox (#160)

1. **The Scout's draft does not reach me.** Understood and acted on: everything
   in "What is already in the working tree" was built from `main`, not
   inherited. No instruction in the spec turned out to be readable *only* as a
   diff against that tree.
2. **Route (b) is refuted; settle the keyboard first.** Done, and route (a) is
   what shipped. I read `src/input/state.rs`'s single `cx.propagate()` and its
   comment, confirmed the input consumes those four actions and propagates none
   of them, and took the context-predicate route the Command reviewer supplied
   the citation for. Both components use it, and `crate::init` carries the
   ordering argument with the two `file:line` references in a comment.
3. **The coverage entry must not point at another element's page.** Done:
   `("combobox", "combobox")` with a real `render_combobox_page` showing
   default, pre-selected, all three `ControlSize` rungs, disabled, a `Keep`
   field and a `Create` field, plus a nav entry beside Select.
4. **Compile it; do the lift as a pure move first.** Done in that order — the
   lift was a mechanical move, `select.rs`'s existing suite went green on it
   before `combobox.rs` existed, and it is still green. The three named tests
   are written (row-index-versus-option-index under an active filter, the
   `UnmatchedText` modes on blur, and that typing clears the value), plus one
   asserting no row claims `active_descendant`.
5. **`examples/showcase.rs` is shared with #165.** My edits to
   `ELEMENT_COVERAGE`, `NAV_SECTIONS` and the `render` match are additive
   single-line insertions; nothing was reordered or reformatted.

### Spec 2 — Command (#159)

1. **Drop the `active_descendant` claim.** Done, and this **conflicts with the
   spec text**, which put the claim on the selected row: the feedback wins and
   the row now carries only its fill and its `selected` state. The decline is in
   the module docs with the reason, in `combobox.rs` for the same arrangement,
   and as a paragraph in `docs/menus-and-listboxes.md` recording it as a gap in
   gpui rather than in either component.
2. **The binding-depth citation.** Both `gpui/src/keymap/context.rs:181` /
   `:361` and `gpui/src/keymap.rs:173` are in the comment beside the
   `bind_command_keys` call in `init` and in both modules' `# The keyboard`.
3. **Three runs, reported.** `cargo test --lib`: 533 passed, 0 failed.
   `cargo check --example showcase --features examples`: clean. The **real-window
   keyboard test was not written** — the combobox tests do open a real window
   via `cx.open_window`/`VisualTestContext`, but they drive the state rather
   than pressing keys, so the binding-depth claim is argued from gpui's source
   and not yet demonstrated by a keystroke. That is the single largest gap in
   this change and it is the first thing to add.
4. **The triage move is four edits or none.** I did **none**: `Combobox` and
   `Command` are still `Issue` rows in `docs/component-triage.md`. That is a
   deliberate decline under the clock rather than an oversight — the move is the
   table row, `EXPECTED`, the prose counts, the attribution counts *and* a new
   pointer at `docs/issues/command.md` (and `combobox.md`) so
   `every_written_issue_is_reachable_from_the_triage` still passes, and a
   half-move is a red build. The build is green as it stands; the move is a
   clean follow-up commit for someone with the counts in front of them.

## Directions from the orchestrator

- **Two separable commits.** **Not done** — this landed as one commit.
  `src/elements.rs`, `src/lib.rs`, `examples/showcase.rs` and
  `docs/overlays.md` each carry lines from both components, and splitting them
  cleanly would have cost more of the run than it was worth. The two components
  are otherwise independent files.
- **Share the machinery.** Done: `listbox.rs` exports `wrapped_index` (the
  wrapping highlight arithmetic, used by `Listbox::move_highlight` and by
  `CommandState::next_selection`) and `matches_query` (the default
  case-insensitive substring match), rather than each component carrying its
  own copy. The keyboard arrangement is the same shape in both, but the actions
  are distinct types in distinct contexts, so that stayed as two parallel
  `bind_*_keys` functions with one shared argument written down in `init`.
- **The keyboard rule.** Followed — see above. No new raw `on_key_down`.
- **The `.tasks/verify` gate.** `cargo test --lib` is green at 533.
- **Integrate with a Calendar branch.** There was none in the tree I was given
  (base is `dc0dd7b`), so there was nothing to integrate with; my `elements.rs`,
  `lib.rs` and showcase entries are additive and should merge cleanly beside it.
- **Re-read the triage counts in this tree.** Done — 12 Shipped / 6 Issue / 11
  Rejected, unchanged, which is why item 4 above is a decline rather than a
  silent drift.

## Known gaps

- No keystroke-level test of the binding-depth claim (see feedback item 3
  above).
- The popup does not match the field's width. `docs/overlays.md` says to build
  the measuring `Element` when a *second* component wants it; this is the first.
- Multiple selection is out of scope, as the spec says. Nothing in the state
  shape prevents it.
- `Revert` restores the label of the *value*, and typing clears the value — so
  reverting after typing unmatched text empties the field rather than restoring
  what was there before. That is the spec's rule followed to its end; it is
  spelled out in the module docs and covered by two tests.
