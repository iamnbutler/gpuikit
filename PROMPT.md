You are a Builder in the Double Diamond architecture.

You are implementing 4 approved spec(s). Verify a spec's claims against the code in front of you; where a spec has a Scout behind it, trust its pitfalls.

## Spec 1 of 4: Combobox: a text field that filters a list of choices (#160)

*A Scout wrote this spec after exploring the work by implementing it once in a throwaway branch you cannot see, and a reviewer approved it. The spec is the distilled result — trust its pitfalls.*

## Spec: Combobox — a text field that filters a list of choices

### Summary

A `Combobox` is a `TextField` that owns a listbox popup of values: the typed
text filters the options, arrow keys move a highlight through what survived the
filter, and Enter commits the highlighted option to the control's **value**.
The state holds the value and the text separately, because they diverge the
moment the user types; what happens to unmatched text on blur is a documented
three-way choice with `Revert` as the default. It ships as
`src/elements/combobox.rs` plus a new `pub(crate) mod listbox` lifted out of
`select.rs` — which is the move `docs/menus-and-listboxes.md` §2 already
prescribed for the second caller — so the crate gains a chooser without gaining
a fourth popup. All three of the issue's stated blockers are already closed in
`main`; the naming block in particular was settled by #154, and this change is
the first exercise of the escape clause that decision wrote for it.

### Implementation Approach

**1. Lift the popup: `src/elements/listbox.rs`, `pub(crate)`.**

- Move out of `select.rs`, unchanged in behaviour: `struct Listbox` and its
  `Accessible` / `Focusable` / `Render` / `EventEmitter<DismissEvent>` impls,
  `fn option_a11y`, the `actions!` block (`HighlightNext`, `HighlightPrevious`,
  `HighlightFirst`, `HighlightLast`, `ChooseHighlighted`, `DismissListbox`),
  `LISTBOX_CONTEXT`, `LISTBOX_GAP`, and the binding function.
- Declared `pub(crate) mod listbox;` in `src/elements.rs`, **not** `pub mod`.
  Three tests key off the literal string `"pub mod "`
  (`showcase_coverage::element_modules`, `family_coverage`,
  `triage_coverage`), so `pub(crate)` keeps the shared popup out of the
  showcase and out of the family table — which is right: it is not a component,
  it is the thing two components are made of. This is exactly
  `menus-and-listboxes.md` §2's "lift `Listbox` into a `pub(crate)` module
  *named by both callers*, do not make it `pub` where it sits".
- `select.rs` keeps `pub fn bind_select_keys` and `pub const LISTBOX_CONTEXT`
  as re-exports of the new module's items, because both are public API today
  and `crate::init` calls the first. Add `pub fn bind_listbox_keys` as the new
  name and make `bind_select_keys` a one-line delegate with a doc note.
- `Listbox::build` grows one argument, `restore_focus_on_open: bool` — a
  `Select` moves focus into its popup, a `Combobox` must **not** (the user is
  typing). See the keyboard section below. Everything else about the popup —
  rows, check-versus-fill, type-ahead, scroll-into-view, `DismissEvent` — is
  used by both callers as-is.

**2. `src/elements/combobox.rs`.**

- `Combobox<T: Clone + PartialEq>` builder + `ComboboxState<T>` entity, the
  same two-type shape as `Select`/`SelectState`, because a combobox has to
  outlive a frame (it owns an `Entity<InputState>` and a popup entity).
- `combobox(id, name, options)` — `name` is a required argument for the same
  reason `select()`'s is: `Role::ComboBox` is in `a11y::role_requires_a_name`,
  the visible text is the *value*, and gpui has no `labelled_by`.
- Builder methods, all mirroring `Select` where they exist: `.selected(T)`,
  `.placeholder(…)`, `.on_change(…)`, `.full_width(bool)`, `.filter(…)`,
  `.keep_unmatched_text()`, `.on_create(…)`, plus `Disableable` and
  `ControlSized`.
- State fields: `options: Vec<(T, SharedString)>`, `selected: Option<T>`,
  `input: Entity<InputState>` (the text), `listbox: Option<Entity<Listbox>>`,
  `visible: Vec<usize>` (indices into `options` that survived the filter, so
  the popup's row index and the option index are never confused), `size`,
  `disabled`, `unmatched: UnmatchedText`, `filter: Rc<dyn Fn(&str, &T,
  &SharedString) -> bool>`.
- `render` is `text_field(&self.input, cx)` with
  `.suffix(Adornment::icon(Icons::chevron_down()))`, `.control_size(self.size)`,
  `.disabled(self.disabled)`, wrapped in the `div().relative()` +
  `deferred(anchored().offset(point(px(0.), gap)).child(div().occlude()
  .child(listbox))).with_priority(1)` recipe from `docs/overlays.md` — rung 1,
  the same rung select/popover/context-menu already use, so the priority ladder
  needs no new row.

**3. Value versus text — the state holds both.**

- `selected: Option<T>` is the value. The text lives in `InputState`, which
  already owns editing, IME and selection; duplicating it into a `String` field
  would be two sources of truth for one string.
- Committing an option (click, or Enter on the highlight) sets `selected` and
  writes that option's label into the input with `set_content`, so text and
  value agree the moment they can.
- Typing (`InputStateEvent::TextChanged`) re-runs the filter, opens the popup
  if it is closed, resets the highlight to row 0, and — this is the decision —
  **clears `selected` to `None`**. A value the text no longer names is a lie
  the control would keep reporting through `on_change` and through its a11y
  value. `SelectChanged`'s counterpart, `ComboboxChanged`, is emitted for that
  clear too.
- On `InputStateEvent::Blur`, if the text does not exactly match the label of
  `selected`, `UnmatchedText` decides:
  - `Revert` (**default**) — put back the label of `selected`, or empty when
    there is none. The combobox is a chooser; its value is the thing that
    persists, and a field left showing text that is not the value is the bug.
  - `Keep` — leave the text, leave `selected` at `None`. For a field whose list
    is a suggestion rather than a constraint.
  - `Create` — call the `on_create` handler with the text. Set by calling
    `.on_create(handler)`, so the mode cannot be selected without the handler
    that makes it mean anything. The handler decides whether to push an option;
    the combobox does not mutate its own option list behind the caller's back.
  `UnmatchedText` is a public enum with a `.unmatched_text()` reader, so the
  two non-default answers are explicit options rather than forks of the file.

**4. Filtering is the caller's.**

`.filter(impl Fn(&str, &T, &SharedString) -> bool)` — query, value, label. The
default is a case-insensitive substring match on the label, which is the
behaviour nine callers in ten want and the tenth replaces in one line. A
predicate rather than "hand me the filtered list" because the state already
owns the option list and the selection; handing it a second, shorter list every
frame would make the row index and the option index two different numbers the
caller has to keep straight, which is the bug `visible: Vec<usize>` exists to
prevent. `command.md`'s argument is satisfied either way — the crate does not
own the matching algorithm.

**5. Accessibility.**

- The field announces `Role::ComboBox`, the constructor `name`, `.expanded(is_open)`,
  and `.text_value(current text)`. It is announced on a wrapper `div` around
  the `TextField`, because `TextField` reports no role of its own and teaching
  it one would give a role to every plain text field in the crate — a separate
  decision, and a wrong one to make by side effect.
- The popup announces `Role::ListBox` named after the control; each row
  announces `Role::ListBoxOption` with `selected`, `position_in_set`,
  `size_of_set`. All three come free with the lifted module.
- **`active_descendant` is deliberately not claimed** — see pitfalls.

**6. Showcase and docs, which are build requirements here.**

- `src/elements.rs`: `pub mod combobox;` (and `pub(crate) mod listbox;`).
- `examples/showcase.rs`: `("combobox", "combobox")` in `ELEMENT_COVERAGE`,
  `("combobox", "Combobox")` in `NAV_SECTIONS`' Input section next to Select,
  and a `"combobox" => …` arm in `Showcase::render`'s match with a page
  showing: default, pre-selected, all three `ControlSize` rungs, disabled, a
  `Keep` field and a `Create` field.
- `docs/menus-and-listboxes.md`: a `combobox` row in the `<!-- family-table -->`
  (family `Listbox`), and §"What would reopen this" updated to say the
  candidate arrived and took the §2 escape clause. The table test parses that
  comment anchor and asserts every row names a real `pub mod`.
- `docs/component-triage.md` and `docs/issues/combobox.md` — check whether
  `triage_coverage` requires the issue file to move or be marked done.

### Discovered Pitfalls

1. **All three "blocked on" issues are already closed, under different names.**
   `docs/issues/menu-vs-listbox-naming.md` → `docs/menus-and-listboxes.md`
   (#154, `dropdown.rs` deleted, `mod family_coverage` enforcing it).
   `docs/issues/portal-adopt-or-delete.md` → `docs/overlays.md` (#155,
   `src/traits/portal.rs` deleted). `element-roles-convention.md` still sits in
   `docs/issues/` but `src/a11y.rs` has landed and `select.rs` already reports
   `ComboBox`/`ListBox`/`ListBoxOption`. **The issue body is stale on this
   point and nothing here is blocked.**

2. **The keyboard is the hard part, and it is a binding-context collision.**
   Every key a combobox needs — `up`, `down`, `home`, `end`, `enter`, `escape`
   — is already bound by `src/input/bindings.rs` in `INPUT_CONTEXT`, and the
   input keeps real focus while the popup is open. `Select`'s model does not
   transfer: it *moves focus into the popup* and binds in the deeper
   `LISTBOX_CONTEXT`, which a combobox cannot do without breaking typing.
   The arrangement that works, and the one to build:
   - focus stays on the input; the popup is created with
     `restore_focus_on_open: false` and never calls `window.focus`;
   - the combobox's wrapper `div` carries `.key_context(COMBOBOX_CONTEXT)`
     *and* the bindings, in a context **outside** `INPUT_CONTEXT`. gpui
     dispatches from the focused node upward, so an `INPUT_CONTEXT` binding on
     the focused input wins any key both want. That is correct for `home`,
     `end`, `left`, `right` (text navigation) and **wrong** for `up`, `down`,
     `enter` and `escape`, which the combobox must take.
   - So those four cannot be won by a binding in an ancestor context. Two ways
     out, and the Builder should try them in this order:
     (a) put the combobox's bindings in `INPUT_CONTEXT`-qualified form —
     `KeyBinding::new("down", HighlightNext, Some("Combobox > Input"))` or
     whatever gpui's context-predicate syntax spells — so they are *deeper*
     than the plain-`Input` binding and win by depth;
     (b) failing that, subscribe to the input's actions: the combobox wrapper
     adds `on_action` listeners for the input's own `Up`/`Down`/`Enter`/
     `Escape` actions. Bubble-phase action listeners run deepest-first and the
     input's handler runs first, so this needs the input to leave the action
     un-stopped — check `src/input/handler.rs` before relying on it.
     Do not reach for a raw `on_key_down`: `select.rs`'s module docs and
     `menus-and-listboxes.md` §3 both record why `context_menu.rs`'s raw
     handler is a defect (an enclosing `Dialog` takes Escape first), and a new
     element repeating it would be a regression the docs already forbid.
   The `LISTBOX_CONTEXT` bindings must *not* be active for a combobox popup,
   since that popup never holds focus.

3. **`active_descendant` cannot be expressed for a combobox, and claiming it
   would fail in silence.** `src/a11y.rs`'s `A11y::active_descendant` says so
   explicitly: gpui puts the property on the *item*, honours it only under a
   focused **ancestor**, and "the APG arrangement where focus stays on a combo
   box trigger and points into a popup beside it **cannot be expressed**".
   That is precisely this component. The popup is a sibling subtree under
   `div().relative()`, not a child of the focused input. So the highlighted row
   gets its fill and no claim, and the decline is written into the module docs
   with the reason — the crate's convention is that a decline in writing is the
   difference between a gap and an oversight (`role_requires_keyboard_focus`
   does the same for `Role::Splitter`). Do **not** hand `active_descendant`
   through from the lifted `Listbox` unconditionally: it must become a
   constructor flag, off for the combobox, or the row will claim something no
   focused ancestor can honour and `select.rs`'s only-one-claim invariant stops
   being the only claim in the frame.

4. **The popup does not match the field's width, and must not try to.**
   `docs/overlays.md`'s closing section names this component as the first that
   will want it, and says the thing to build is a small custom `Element` that
   measures a trigger — *when a second element wants it*. This is the first, so
   the popup keeps `Listbox`'s existing `min_w(px(120.))` and the Builder
   should not build the measuring element. Note it and move on.

5. **Row index is not option index.** Once a filter runs, the popup's rows are
   a subset. `Listbox` reports the row index to `on_select`. `visible:
   Vec<usize>` maps back. Getting this wrong selects the wrong value only when
   a filter is active, which is exactly the case no smoke test covers.

6. **`Listbox::build` reads `window.focused(cx)` and then focuses itself.**
   Both halves have to become conditional for the combobox, not just the
   focus call — a stored `restore_focus` that is never used is harmless, but
   the `window.focus` is not.

7. **Element ids.** `Combobox` must key its ids on an entity id via
   `crate::element_id::for_entity`, as `text_field.rs` and `select.rs` do.
   gpui hashes the whole id path into the accessibility node id and
   `debug_assert!`s on a duplicate; the field, the popup and each row are three
   nodes. `TextField` already derives its own id from the `InputState` entity,
   so two comboboxes never collide as long as each has its own `InputState`.

8. **`showcase_coverage` fails the build before the component works.** The
   `pub mod combobox;` line and the showcase page have to land in the same
   commit. Same for the family table row.

9. **Multiple selection is out of scope for this change.** Headless UI has it
   and the issue mentions it; it changes `selected: Option<T>` into a set, the
   text into a query that is not the value, and the blur rule into something
   else entirely. It is a second issue, and the state shape above does not
   prevent it.

### Blockers & Dependencies

- `docs/issues/menu-vs-listbox-naming.md` — **closed**, by #154. Became
  `docs/menus-and-listboxes.md`. This change takes the escape clause its §2 and
  "What would reopen this" wrote for exactly this component.
- `docs/issues/portal-adopt-or-delete.md` — **closed**, by #155. Became
  `docs/overlays.md`. Follow its recipe; do not build the width-matching
  element yet.
- `docs/issues/element-roles-convention.md` — **effectively closed**:
  `src/a11y.rs` ships the convention and `select.rs` consumes it. The file is
  still in `docs/issues/`, so confirm `triage_coverage` does not require it to
  be reconciled in this change.
- Nothing else. The one real dependency is internal: the `Listbox` lift has to
  land with (or before) the combobox, and `select.rs`'s existing tests are the
  regression net for it.

### Complexity

Complex.

### What is already in the working tree

This scout left a working draft of the change, **uncompiled** (see below):

- `src/elements/listbox.rs` — new, `pub(crate)`. The lift, done as a near-pure
  move of `Listbox`, `option_a11y`, the `actions!` block, `LISTBOX_CONTEXT`,
  `LISTBOX_GAP` and the bindings out of `select.rs`, plus one new thing:
  `enum ListboxFocus { Popup, Caller }`, threaded through `build`, `row_a11y`
  (the active-descendant claim) and `render` (`track_focus` + `key_context`).
  The driving methods are `pub(crate)`, and `set_options` is new — a combobox's
  rows change under an open popup, a select's do not.
- `src/elements/select.rs` — the moved code removed; `Listbox::build` gains
  `ListboxFocus::Popup`; `bind_select_keys` and `LISTBOX_CONTEXT` kept as a
  delegate and a re-export, because both are public API.
- `src/elements/combobox.rs` — new. Builder + state as described above.
- `src/elements.rs` — `pub mod combobox;` and `pub(crate) mod listbox;`.
- `docs/menus-and-listboxes.md` — the `combobox` family row, and a paragraph in
  "What would reopen this" recording that the clause was taken.
- `examples/showcase.rs` — `("combobox", "select")` in `ELEMENT_COVERAGE`.

**Unfinished, and the Builder should expect to do it:**

- The showcase page. The coverage row currently points `combobox` at the
  existing `select` page, which satisfies `showcase_coverage` but does not
  actually draw a combobox. Either add a `combobox` section to the select page
  (defensible — same family, and the two are worth comparing side by side) or
  give it a page and a `NAV_SECTIONS` entry of its own.
- The keyboard. `combobox.rs` documents the arrangement and does not implement
  it: there are no `on_action` listeners for the input's `Up`/`Down`/`Enter`/
  `Escape` yet. This is pitfall 2 and it is the real work left.
- Tests. `select.rs`'s suite is the regression net for the lift; nothing new
  was written for `combobox.rs`. The three that matter most: the row-index /
  option-index mapping under an active filter, each of the three
  `UnmatchedText` modes on blur, and that typing clears the value.
- `docs/menus-and-listboxes.md` §2 still opens "`Listbox` … is a private type
  in `src/elements/select.rs`", which this change makes untrue. §2's *rule* is
  unchanged and was followed; its first sentence needs rewording to describe
  where the type lives now.
- Imports in `select.rs` are almost certainly now partly unused
  (`KeyDownEvent`, `ScrollHandle`, `KeyBinding`, possibly `Role`,
  `for_entity`). `missing_docs` is `warn` and `rust_2018_idioms` is on; expect
  a first compile to be noisy.

### Notes

- **Nothing was compiled.** There is no `target/` and no populated cargo registry
  in this environment, and a cold gpui build does not fit the run. Treat every
  API call in the implementation as needing a compiler's opinion.
- The lift is the risky half and it is also the half with a test suite already
  pointed at it: `select.rs`'s ~700 lines of tests exercise the popup's
  keyboard, its a11y nodes and its focus restoration. Do the lift as a pure
  move first, run `cargo test --lib`, and only then add the combobox. A green
  `select.rs` after the move is worth more than any test written for the new
  file.
- `cargo test --lib`, not `--all-targets` and not `--features examples`:
  Cargo.toml's own comment explains that the examples feature exists to keep
  eight links of gpui out of the ordinary commands.
- Read `src/elements/select.rs`'s module docs top to bottom before starting.
  Its "# The keyboard" section already contains the two-states argument
  (choice versus highlight) that a combobox needs a third time, and its
  Tab/type-ahead exceptions are the reasons a new element should not invent
  its own key handling.

## Spec 2 of 4: Command: a filterable action list in an overlay (#159)

*A Scout wrote this spec after exploring the work by implementing it once in a throwaway branch you cannot see, and a reviewer approved it. The spec is the distilled result — trust its pitfalls.*

## Spec: Command — a filterable action list in an overlay (#159)

### Summary
`src/elements/command.rs` adds `CommandState`, a view that owns a query field
(`InputState` + `TextField`), a list of `CommandItem`s, a caller-supplied
matcher, and the selected row — and draws them as a scrimmed panel near the top
of the window. Matching is deliberately not the crate's business: the caller
installs a closure (`Fn(&str, &[CommandItem]) -> Vec<usize>`, returning item
indices in the order they should be shown), or matches elsewhere and calls
`set_matches` in answer to `CommandEvent::QueryChanged`. The part that could not
be assembled from `Dialog` + `TextField` + `List` is what this owns: the
selection model, and a keyboard contract where focus never leaves the query
field while up/down move the selection, enter runs it and escape dismisses.
Both of the issue's blockers are settled and were followed rather than
re-decided: `src/a11y.rs` for roles, `docs/overlays.md` for placement.

### Implementation Approach
- **`src/elements/command.rs` (new, ~700 lines).** `CommandItem`
  (label/subtitle/keywords/shortcut/icon/disabled/`on_run`, plus `haystack()`
  so a caller's matcher has one place to read searchable text from);
  `CommandState` (query entity, items, `matches: Vec<usize>`, optional matcher,
  `selected` as an index *into `matches`*, open flag, `ControlSize`);
  `CommandEvent::{QueryChanged, Run, Dismissed}`; `bind_command_keys`.
- **The keyboard is the design problem, and the fix is a child predicate.**
  `src/input/bindings.rs` binds `up`, `down`, `enter` and `escape` in the
  `Input` context *on the focused field itself*, and gpui resolves a keystroke
  by key-context depth — so a palette that key-contexts an ancestor loses all
  four keys. `bind_command_keys` registers each key twice: under
  `"CommandPalette > Input"`, which matches at the field's own depth and so
  *ties* with the field's binding, and under bare `"CommandPalette"`. gpui
  breaks a depth tie by registration order, later wins, so `crate::init` calls
  `bind_command_keys` **after** `input::bind_input_keys` (added at the end of
  `src/lib.rs`'s `init`, with the reasoning in a comment beside the existing
  Tab and Escape ordering notes). This is Zed's own `"Picker > Editor"` shape.
- **Selection model** copied in shape from `context_menu.rs`: `runnable_rows()`
  is `selectable_indices` over matched rows, `next_selection()` is `next_focus`
  (wraps at both ends; a selection that is no longer runnable re-enters from
  the edge the movement came from). Six unit tests cover it, plus one for
  `haystack()`. Re-matching re-selects the first runnable row, which is what
  makes type-then-enter work with no arrow press.
- **Overlay**: `deferred(...).with_priority(10)`, `anchored()`-free, exactly
  like `dialog` — there is no trigger to hang off, so the distance from the top
  is padding on the full-window scrim, not an `offset`. `.occlude()` on the
  panel; a scrim click dismisses.
- **A11y** through `src/a11y.rs` only: the results list announces
  `Role::ListBox` named by a required constructor argument (a palette has no
  visible text of its own — the field's text is the *query* — which is section 2
  of `a11y.rs` and the answer `Select` had to take), each row announces
  `Role::ListBoxOption` with `selected`, `position_in_set`/`size_of_set`
  together, and `active_descendant` on the selected row: the field owns the
  selection.
- **`ControlSized`** implemented; every dimension comes from
  `theme.control(size)`. The three things the rung cannot express — the panel's
  distance from the top, its width, the results' max height — are named
  constants in this file, as `src/theme/control.rs`'s "what belongs here" note
  prescribes.
- **Docs the build checks**: a row in `docs/overlays.md`'s overlay table (and
  the priority-10 rung reworded to cover the palette, and the "two of these are
  not anchored" sentence corrected to three); a row in
  `docs/menus-and-listboxes.md`'s family table as **Menu** — its rows are
  actions and nothing stays selected once one has run — with a paragraph on why
  a menu-family component reports listbox *roles*.
- **Showcase**: `("command", "command")` in `ELEMENT_COVERAGE`, a nav entry, a
  `"command" =>` arm and `render_command_page`, with the palette entity built in
  `Showcase::new`. Its matcher is the two-line case-insensitive substring filter
  a consumer writes — deliberately in the example rather than in the crate.

### Discovered Pitfalls
- **The binding-depth trap above is the whole component.** Bound in a plain
  ancestor context, every key silently goes to the text field instead: the
  palette looks wired and does nothing. Registration order in `init` is
  load-bearing in the same way the file already documents for Tab.
- `gpui::Svg` is not `Clone`, so `CommandItem::icon` takes `impl Fn() -> Svg`
  (`context_menu.rs` has the same signature for the same reason).
- `gpui::Pixels`/`Rems` tuple constructors are private outside gpui — `Rems(6.0)`
  compiles as a const, `Pixels(560.0)` does not; those two constants are `f32`
  wrapped in `px()` at the call site.
- `window.focus` takes `(&handle, cx)` in this gpui version.
- `ScrollHandle`, not `UniformListScrollHandle`, is what `track_scroll` wants.
- `selected` indexes `matches`, never `items`; `selected_item()` is the
  translation. Keeping one field and translating at the edges is what stops the
  "selection survived a re-match onto a different row" bug.
- Roles: `ListBox` requires a *name* (hence the constructor argument) but is
  deliberately not in `role_requires_keyboard_focus`; focus is declared with
  `track_focus` rather than through `A11y`, following the comment in
  `select.rs` about a transient overlay that must not be a tab stop.
- Four test modules in `src/elements.rs` gate the build on documentation:
  `overlay_coverage` (any module containing `deferred(` needs a table row; any
  `with_priority(n)` literal must be an existing rung), `family_coverage`,
  `showcase_coverage` (both directions), `triage_coverage`.

### Blockers & Dependencies
Both blockers named in the issue are **already settled**, and this follows them
rather than reopening either: `docs/issues/element-roles-convention.md` landed
as `src/a11y.rs` (its banner says so), and the overlay question is answered by
`docs/overlays.md` — `docs/issues/portal-adopt-or-delete.md` no longer exists,
`src/traits/portal.rs` was deleted in #155.

One thing left deliberately undone, for the Builder to decide rather than for
me to decide silently: `docs/component-triage.md` still lists Command as
`Issue`. Moving it to `Shipped` is not a one-line edit — `triage_coverage`
enforces the 12/6/11 counts in `EXPECTED`, the same counts restated in the
document's prose *and* in its attribution section, and
`every_written_issue_is_reachable_from_the_triage` requires
`docs/issues/command.md` to still be pointed at from the triage after the row
stops naming it. All four move together or the build fails.

### Complexity
Medium.

### Notes
- **Verification status, stated plainly.** `cargo check --lib` passes clean (no
  warnings from the new module). `cargo test --lib` was started but the run's
  wall clock ended during the test binary's link, so the unit tests in
  `command.rs` and the four documentation-coverage test modules are **not
  observed green** — the doc edits were made against those tests' parsers by
  reading them, which is not the same thing. Run `cargo test --lib` first.
  `examples/showcase.rs` is behind the `examples` feature and was **not
  compiled at all** (`--features examples` links gpui eight times); the page is
  the least-verified part of this change.
- Not built, and worth building next: `CommandState` has no
  `CommandGroup`/section vocabulary (shadcn's `cmdk` groups) and no loading
  state — `CommandEvent::QueryChanged` plus `set_matches` is the async hook, but
  nothing draws a spinner. Both are additive.
- The keyboard is currently only unit-tested at the index-list level.
  `select.rs`'s test module has the harness for the real thing — a view opened
  with `cx.open_window`, `press(cx, "down")` — and a palette test needs it,
  because the binding-depth argument above is exactly the kind of claim that a
  unit test over `Vec<usize>` cannot check.

## Spec 3 of 4: Calendar: a month grid of selectable days (#157)

*A Scout wrote this spec after exploring the work by implementing it once in a throwaway branch you cannot see, and a reviewer approved it. The spec is the distilled result — trust its pitfalls.*

## Spec: Calendar — a month grid of selectable days (#157)

### Summary
The issue's real content is the date-type decision, and it is settled as the issue recommended:
a minimal `Date { year, month, day }` in a new `src/date.rs`, not a `chrono` / `time` / `jiff`
dependency in the public API. The arithmetic that buys — leap years, month lengths,
weekday-of-date, add-days, add-months-with-clamping — is Howard Hinnant's `days_from_civil` /
`civil_from_days` plus one leap-year rule, and it is verified over every day from 1800 to 2200
rather than sampled. On top of it, `src/elements/calendar.rs` is a six-by-seven month grid with
weekday headers, muted leading/trailing days so the grid never changes height, single selection,
a `today` marker passed in rather than read from the clock, a disabled-day *predicate*, month
navigation, and the WAI-ARIA date-grid keyboard contract. Roles go through the crate's existing
`src/a11y.rs` convention — `Grid` / `Row` / `ColumnHeader` / `GridCell`, with the grid as the one
tab stop and `active_descendant` on the focused day. Range selection is deliberately not built:
the issue says to ship single first, and the module documents what range would change.

### Implementation Approach
- **`src/date.rs` (new, `pub mod date` in `src/lib.rs`)** — pure `std`, no gpui import, which is
  also what let it be tested standalone with `rustc --test` before the crate compiled. `Date` has
  private fields and a validating `Date::new`, so a date that exists is a day that exists;
  ordering is derived and chronological because the field order is. Everything else —
  `weekday`, `add_days`, `add_months`, `is_same_month` — is defined through `to_days`/`from_days`,
  so the leap-year rule appears once. `Weekday` carries `days_from(start)` and `week_from()`,
  which is the whole of "which column does a date sit in": the grid needs no first-day-of-week
  table. English `month_name` / `Weekday::{name, short_name, min_name}` are the defaults
  localisation parameters fall back to.
- **`src/elements/calendar.rs` (new)** — an entity (`Render` + `EventEmitter<CalendarEvent>`)
  rather than a `RenderOnce` struct, because the grid holds three things across frames a caller
  should not have to: the visible month, the keyboard-focused day, and the `FocusHandle`.
  Builders: `.selected(Option<Date>)`, `.today(Date)`, `.disabled_days(impl Fn(Date) -> bool)`,
  `.first_day_of_week(Weekday)`, `.month_labels([_; 12])`, `.weekday_labels([_; 7])`, plus
  `ControlSized` and `Disableable`. Events are `Selected(Date)` and `MonthChanged(Date)`.
- **Sizing** — `ControlSized` is implemented and every dimension comes from
  `Themeable::control(size)`. The one number that is this component's own shape, the square day
  cell, is `CELL_RATIO * metrics.height` in this file, keyed off the rung, per the "What belongs
  here" note at the top of `src/theme/control.rs`.
- **Accessibility** — `impl Accessible for Calendar` returns `A11y::new(Role::Grid).name(title)
  .focus_handle(handle)`; rows announce `Role::Row`, headings `Role::ColumnHeader`, days
  `Role::GridCell` with `.selected(..)` and `.active_descendant(is_focused)`. Every id is
  `element_id::scoped(&self.id, ..)` — a day's part is its day number, which is unique by
  construction — because a cell with a role and a duplicate id is a `debug_assert!`.
- **Keyboard** — one `on_key_down` on the grid: arrows by day/week, `home`/`end` to the ends of
  that week, `pageup`/`pagedown` by month, with shift by year, `enter`/`space` selects. Moving off
  the visible month brings that month into view rather than refusing.
- **Showcase** — `examples/showcase.rs` gains the import, an `ELEMENT_COVERAGE` row, a nav entry
  under Input, a `calendar: Entity<Calendar>` field built on a fixed month (2026-08, today
  2026-08-20, Sundays disabled), a `render_calendar_page`, and the `"calendar" => ` match arm.
- **`docs/component-triage.md`** — Calendar moves Issue → Shipped naming `src/elements/calendar.rs`,
  and the "Prerequisites" entry that had it hard-blocking `date-picker.md` is marked discharged.

### Discovered Pitfalls
- **The roles convention has already landed.** The issue calls
  `docs/issues/element-roles-convention.md` a prerequisite; it is settled, as `src/a11y.rs`. There
  is nothing to wait for, but there is a mechanism to obey — and
  `a11y::tests::every_element_module_declares_a_role` fails the build for a new element module
  that neither implements `Accessible` nor is excused in `ELEMENTS_WITHOUT_A_ROLE`.
- **There is still no roving-focus convention**, and `src/a11y.rs` says so explicitly: the
  composite-item roles are off `role_requires_keyboard_focus` because `Tabs`, `List`,
  `ContextMenu` and `Select`'s popup all want one and none has it. The calendar therefore copies
  `Select`'s popup — container focus plus `active_descendant` — rather than inventing a fifth
  mechanism. It is a fourth caller for whoever takes that decision.
- **`A11y` has no grid-index fields.** gpui's `div` has `aria_row_index` / `aria_column_index` /
  `aria_row_count` / `aria_column_count`, but `crate::a11y::A11y` models none of them and
  `a11y::tests::no_element_calls_gpuis_a11y_builders_directly` fails the build if an element calls
  gpui's builders directly. Left unreported, with each cell carrying its full date as its name.
  Adding the four fields to `A11y` is the follow-up.
- **`cx.theme()` borrows the app.** `cx.theme()` hands back a borrow that conflicts with
  `cx.listener(..)`, and a `.hover(..)` closure that captures the theme keeps it alive. Every
  colour is copied into a local at the top of `render` and of `month_button`. This is the only
  thing that failed to compile twice.
- **`window.focus` takes two arguments** in gpui 1.16 (`&FocusHandle, &mut App`).
- **The month buttons are `div`s, not this crate's `Button`.** `Button`'s label *is* its
  accessible name (`src/a11y.rs` §2) and a chevron is not a name, so they are ids + `announce`
  with `Role::Button`, a spelled-out name, and an explicit `not_focusable("…")` — silence there
  is a `debug_assert!`.
- **`Date::add_months` clamps and does not round-trip** (Jan 31 → Feb 28 → Jan 28). That is the
  right behaviour for a month button and the wrong one for re-deriving an original day, so the
  grid keeps the focused date and re-derives rather than paging it back and forth.
- **Negative years need `div_euclid`/`rem_euclid`**, not `/` and `%`; there is a test for
  year 1 minus 13 months.
- **Bookkeeping is enforced by the test suite**, not by convention: `showcase_coverage` needs both
  an `ELEMENT_COVERAGE` row and a live match arm, and `triage_coverage` needs the verdict counts
  changed in *three* places at once — `EXPECTED` in `src/elements.rs`, the prose split, and the
  `<!-- ratification -->` section's `**N Shipped**` strings.

### Blockers & Dependencies
- **Nothing blocks this.** `docs/issues/element-roles-convention.md`, the one hard prerequisite
  the issue names, is already settled as `src/a11y.rs`, and the `ControlSize` scale from #141
  exists.
- **This unblocks `docs/issues/date-picker.md`** — both halves of its hard block, the grid and the
  date-type decision, are now answered.
- Wanted but not required: a roving-focus convention (this is its fourth caller); grid-index
  fields on `A11y`; an `aria_disabled` in gpui, without which a disabled day says "unavailable"
  in its name and offers no `Click` action.

### Complexity
Medium. The date arithmetic is small and completely specified; the grid is a nested loop; what
makes it more than Simple is the number of crate-wide contracts a new element has to satisfy at
once (roles, ids, sizing, showcase, triage) and the date-type decision at the front.

### Notes
- **Verified**: `cargo test --lib` — 512 passed, 0 failed — including the 11 new `date::tests`
  (the four-century round-trip walks ~146,000 days and runs in 30ms), `showcase_coverage`,
  `triage_coverage` and `a11y::tests::every_element_module_declares_a_role`.
  `cargo check --lib` and `cargo check --example showcase --features examples` are both clean;
  the only warning in the tree is a pre-existing `unused_mut` in `src/input/bindings.rs:461`.
- **Not verified**: nothing draws the grid. There are no rendering or keyboard tests for
  `Calendar` — `handle_key`, `days()` and `day_a11y` are all reachable from a `TestAppContext`
  entity and `a11y::test_support::announced` is the crate's tool for the announcement half, but
  building that harness did not fit the run. It is the first thing a Builder should add, and the
  three most valuable assertions are: the grid is always 42 days starting on
  `first_day_of_week`; `pagedown` from the 31st lands on the clamped day and emits exactly one
  `MonthChanged`; and a day cell announces `Role::GridCell` with `selected` and
  `active_descendant` set on the right two cells.
- Range selection is the known gap, and the module documents why it is not a stub: `selected`
  would become an enum, so it is a breaking change to make deliberately rather than a field to
  bolt on.
- `Weekday::min_name()` is two letters ("Su", "Mo") rather than one on purpose — one letter makes
  Tuesday/Thursday and Saturday/Sunday indistinguishable.

## Spec 4 of 4: Form: grouping and label association, not form state (#164)

*A Scout wrote this spec after exploring the work by implementing it once in a throwaway branch you cannot see, and a reviewer approved it. The spec is the distilled result — trust its pitfalls.*

## Spec: Form — grouping and label association, not form state

### Summary
Adds `src/elements/form.rs`: a `Fieldset` with a legend, a group-level error, and a
`disabled` that cascades to everything inside it through an **ambient** `FormContext`
rather than a prop threaded by hand; and adopts `Field` onto that context so a field
names the control beside it, publishes the focus handle its label clicks, and inherits
an enclosing fieldset's disabled. `Field` now takes a required id and announces a named
`Role::Group`, which removes it from `a11y::ELEMENTS_WITHOUT_A_ROLE`. No submission, no
validation, no dirty tracking — the line Headless UI draws, and the one #164 argued for.
`cargo test --lib`: **511 passed, 0 failed.**

### Implementation Approach
- **`src/elements/form.rs` (new).**
  - `FormContext { disabled, name, focus_handle }` — what a group tells the controls
    inside it. `over(outer)` defines nesting: `disabled` is OR'd (an inner scope cannot
    re-enable), `name`/`focus_handle` are inherited when unset.
  - `WithFormContext` — the one hand-written `gpui::Element` under `src/elements/`. It
    wraps a single child and pushes the context around `request_layout`, `prepaint` and
    `paint`. It reports no id and no role, so it adds nothing to the a11y tree or the
    element-id path.
  - The stack is a `thread_local!` `Vec<FormContext>` with a `Drop` guard, so a
    panicking child cannot leave it one deep. A `gpui::Global` was rejected: reads would
    need `&mut App`, which puts a `cx` argument on `disabled_here` and takes the ambient
    value straight back to being threaded by hand.
  - Read API: `current()`, `disabled_here(own) -> bool`, `name_here()`,
    `focus_handle_here()`. A control's entire adoption cost is one line in `render`:
    `let disabled = form::disabled_here(self.disabled);`.
  - `Fieldset` / `fieldset(id)`: `legend`, `description`, `error` (the group-level
    summary that had nowhere to go), `Disableable`, `ControlSized`, `ParentElement`.
    Announces `A11y::new(Role::Group).name(legend)`. Children are wrapped in
    `WithFormContext`; legend, description and error sit outside it.
  - `field_focus_handle(&ElementId, &mut App)` — a `thread_local` registry of one
    `FocusHandle` per field id.
- **`src/elements/field.rs`.** `field(id)` / `Field::new(id)` now require an id (breaking;
  `Default` dropped). `impl Accessible` → named `Role::Group`. `render` reads
  `form::disabled_here(self.disabled)`, gives the label its own scoped id
  (`element_id::scoped(&self.id, "label")`), a `cursor_pointer` and an `on_click` that
  calls `window.focus(&handle, cx)`, and wraps the child in a `WithFormContext` carrying
  `disabled` + the label as `name` + that handle.
- **`src/a11y.rs`.** Deleted the `"field"` row from `ELEMENTS_WITHOUT_A_ROLE` — required,
  because `every_element_module_declares_a_role` fails in *both* directions.
- **`src/elements.rs`.** `pub mod form;`.
- **`examples/showcase.rs`.** `("form", "form")` in `ELEMENT_COVERAGE`, nav entry
  `("form", "Form")`, `"form" =>` arm, `render_form_page` (a labelled fieldset with a
  group error; a second fieldset disabled at the group where neither field says
  `disabled`), and the six existing `field()` calls given ids.

### Discovered Pitfalls
- **gpui has no `labelled_by` builder.** grep across gpui 1.16.1 for
  labelled/labeled returns one unrelated doc comment; accesskit has the relation,
  `AriaProperties` has no field. So association is expressed as *the result* of the
  relation — the `Field` publishes its label as the ambient accessible name and the
  control announces it. This is exactly the reason `ELEMENTS_WITHOUT_A_ROLE` recorded
  for `field`, now discharged by a different route than that note assumed.
- **Why an ambient value works at all:** `ViewElement::request_layout` calls
  `RenderOnce::render` and then lays out the result, and `Div` does the same per child —
  so a whole descendant subtree renders *inside* an ancestor's `request_layout`. The
  scope must also be opened in `prepaint`/`paint`, which walk the subtree again.
- **A deferred draw escapes the scope.** `Window::defer_draw` restores the element-id
  stack, not this one. A popover deferred out of a disabled `Fieldset` reads no ambient
  disabled and needs `disabled(true)` by hand. Documented in the module docs.
- **`Window::use_keyed_state` is not usable from `RenderOnce::render`.** It goes through
  `with_element_state`, which `debug_assert!`s paint-or-prepaint; `render` runs during
  `request_layout`. That is why the field focus handle is a registry keyed on the
  (stable, per-`element_id`-rule) field id. It never evicts — a bounded leak, named in
  the code rather than hidden.
- **`overlay_coverage::every_overlay_is_written_down` matches source text, including
  doc comments.** Writing `deferred()` in prose in `form.rs` failed the build until the
  sentence was rephrased. Real, and easy to hit again.
- **No `Cargo.lock` in the repo**, so `^1.14.2` resolves to gpui-unofficial **1.16.1**.
  `#[derive(IntoElement)]` expands to `ViewElement<Self>` there; `a11y.rs`'s docs still
  say `Component<C>` (1.14's name). Docs only, nothing broken.
- **gpui still has no `aria_disabled`.** A disabled control is distinguishable only by
  the `Click` action its node does not offer. Cascading disabled is therefore a *visual
  and behavioural* cascade, not an announced one — unchanged by this work.

### Blockers & Dependencies
- `docs/issues/element-roles-convention.md` — **cleared.** `src/a11y.rs` already is that
  decision (`A11y` + `Announce` + `Accessible`), so this component used it rather than
  inventing a mechanism.
- Upstream gpui: `aria_labelled_by` and `aria_disabled`. Both would be local changes to
  `A11y`/`Announce` when they land.
- Not blocking, but left undone on purpose: `docs/component-triage.md` still lists
  `Form | Issue | docs/issues/form.md`. Flipping it to Shipped means editing the row,
  `EXPECTED` in `src/elements.rs` (12/6/11 → 13/5/11), the two prose restatements of
  those counts, and adding a prose reference to `docs/issues/form.md` so
  `every_written_issue_is_reachable_from_the_triage` keeps passing. Nothing fails today.

### Complexity
Medium

### Notes
- **The remaining half of "cascading disabled" is adoption.** The mechanism is complete
  and tested, and `Fieldset` and `Field` use it, but no leaf control reads it yet:
  `Checkbox`, `Switch`, `Toggle`, `Button`, `TextField`, `Textarea`, `Select`, `Slider`
  each still need the one line `let disabled = form::disabled_here(self.disabled);` in
  `render`. Same for `form::name_here()` (announce it when the control has no name of its
  own) and `form::focus_handle_here()` (use `A11y::focus_handle(h)` instead of
  `focusable()`, which is what makes label-click-to-focus actually land). Until a control
  adopts the handle, a label click focuses a handle nothing tracks — harmless, and inert.
  That adoption is a per-control change and interacts with the a11y rollout order in
  `a11y.rs` §6, which is why it is not bundled here.
- **`field()` is a breaking change** (`field(id)`). Every call site in the repo is
  updated; downstream consumers are not.
- **Verification.** `cargo check --lib` clean apart from a pre-existing warning in
  `src/input/bindings.rs:461`. `cargo test --lib` → 511 passed / 0 failed, including the
  seven new unit tests in `form.rs` (empty stack, scope visibility, nesting cannot
  re-enable, name inheritance, guard-through-panic, fieldset announcement, derived
  legend id) and three in `field.rs`, plus every pre-existing coverage test
  (`showcase_coverage`, `every_element_module_declares_a_role`,
  `no_element_calls_gpuis_a11y_builders_directly`, `no_element_mints_a_constant_id`,
  `overlay_coverage`, `triage_coverage`).
- **The showcase example was not compiled.** It is behind `--features examples` and a
  full link of gpui did not fit the run's budget. `showcase_coverage` proves the table,
  the nav entry and the match arm are consistent; it does not prove `render_form_page`
  type-checks. That is the one thing a Builder should compile first:
  `cargo check --example showcase --features examples`.

## Review feedback on these specs

A reviewer read the spec(s) above and approved them **with** the following. It is part of what was approved: the spec says what to build, this says what the reviewer required of it. It is not part of any spec text, so nothing above repeats it.

Treat every item as a requirement, not a suggestion. Where one genuinely conflicts with the spec it was written about, the feedback wins — it is the later word, written by the person who approved that spec — but **say so in `SUMMARY.md`**.

Account for every item in `SUMMARY.md` under a `## Review feedback` heading: one line per item saying you did it, or that you decided against it and why. Declines are fine and are expected to be written down; an item you silently dropped is indistinguishable from one you never read, and the reviewer reads the spec rather than this section.

### On spec 1 of 4: Combobox: a text field that filters a list of choices (#160)

Approved. The design is right and the pitfalls are the best set I have read today — pitfall 3 (`active_descendant` cannot be expressed for this arrangement, so decline it in writing and make it a constructor flag rather than inheriting it from the lift) and pitfall 5 (row index is not option index) are both real and both the kind of thing that ships broken. Five items, each accounted for in SUMMARY.md.

1. THE DRAFT DESCRIBED IN "WHAT IS ALREADY IN THE WORKING TREE" DOES NOT REACH YOU, AND YOU MUST NOT GO LOOKING FOR IT. That section describes files a Scout wrote inside a VM that no longer exists. Scout branches are never pushed and you never see Scout code — the spec is the whole deliverable. So `src/elements/listbox.rs`, the `ListboxFocus` enum, `set_options`, the `select.rs` edits, the docs row and the showcase entry are **descriptions of work to do**, not work you have inherited. Read that section as the last paragraph of the design and nothing more, and read "Unfinished, and the Builder should expect to do it" as "this part is not even designed yet". You are implementing all of it from `main`. If any instruction in this spec only makes sense as a diff against a tree you cannot see, say so in SUMMARY.md rather than guessing at it.

2. PITFALL 2 IS ANSWERABLE FROM THE SOURCE, AND ROUTE (b) IS ALREADY REFUTED. The spec offers two ways to win `up`/`down`/`enter`/`escape` from the focused input and asks you to try (b) — an ancestor `on_action` listener — after checking `src/input/handler.rs`. Check `src/input/state.rs:998` instead. `InputState::copy` is the crate's only `cx.propagate()` call, and the comment on it states the rule: *"gpui clears `propagate_event` before every bubble-phase listener, so returning without this call would silently swallow the action and stop anything further out on the focus path from handling it."* The input consumes those four actions and propagates none of them, so an ancestor listener receives nothing. Route (b) as written does not work.

   What that same comment shows is the shape that does: an action *the input has no reason to consume* is handed outward deliberately. `Up`/`Down` in a single-line input are the exact analogue of `Copy` with an empty selection. So settle the keyboard **first, before you write a line of the combobox**, and pick between: making the input propagate the four when it cannot act on them (matching the `copy` precedent, and a change to shared code that needs its own argument in the module docs and its own tests); or a `KeyBinding` context predicate that outranks the plain `Input` binding, which is route (a) and which the spec could not confirm gpui's predicate syntax supports — establish that it does before building on it. Do **not** reach for a raw `on_key_down`: `select.rs`'s module docs and `menus-and-listboxes.md` §3 both record why `context_menu.rs`'s raw handler is a defect. If neither route works, stop and say so in SUMMARY.md — a combobox whose arrow keys move the text cursor is not a partial success, and I would rather have that sentence than a shipped one.

3. THE COVERAGE ENTRY MUST NOT POINT AT ANOTHER ELEMENT'S PAGE. `("combobox", "select")` satisfies `showcase_coverage` while drawing no combobox — that is a build gate answered rather than met, and it is worse than a red build because it is green. Ship a real page (or a real combobox section on the select page, which is defensible — same family, worth comparing) with the six states the spec lists: default, pre-selected, all three `ControlSize` rungs, disabled, a `Keep` field and a `Create` field.

4. NOTHING IN THIS SPEC WAS COMPILED, AND IN THIS REPOSITORY IT COMPILES. The spec says a cold gpui build does not fit the run. A scout on #195, in this same repository this same day, measured a **cold build at 7m11s on 4 cores** and then ran `cargo test --lib` to 501 passed. So treat every API call here as unverified — the spec says so itself — and follow its own sequencing, which is the right one: do the `Listbox` lift as a pure move, run `cargo test --lib` and get `select.rs`'s existing suite green **before** adding the combobox. A green `select.rs` after the move is the only regression net either half of this change has. Then write the three tests the spec names — row index versus option index under an active filter, the three `UnmatchedText` modes on blur, and that typing clears the value — and report the run.

5. `examples/showcase.rs` IS SHARED WITH THE #165 CONFIRMATION-DIALOG SPEC, which is approved and which I intend to put in the same batch. Both add an `ELEMENT_COVERAGE` entry and a `Showcase::render` arm. Keep your edits to those lists additive and local; do not reformat or reorder either table.

Checked so you do not have to: `Listbox`, `option_a11y`, the `actions!` block, `LISTBOX_CONTEXT` and `bind_select_keys` are all where the spec says they are in `src/elements/select.rs` (lines 140–451), with `LISTBOX_CONTEXT` and `bind_select_keys` both `pub` — so the delegate-and-re-export plan is required, not optional. `src/a11y.rs:458` really does say the APG arrangement this component needs *cannot be expressed*. And `src/elements.rs` is a flat list of `pub mod` lines, so `pub(crate) mod listbox;` will sit in it visibly as the one exception; put the reason on the line.

### On spec 2 of 4: Command: a filterable action list in an overlay (#159)

Approved. This is the first spec today that solved its hardest problem instead of listing options for it, and the selection model borrowed wholesale from `context_menu.rs` is the right instinct. Four items, each accounted for in SUMMARY.md.

1. DROP THE `active_descendant` CLAIM. IT WILL BE SILENTLY IGNORED, AND NO TEST WILL TELL YOU. The spec puts it on the selected row with the reason "the field owns the selection". `src/a11y.rs` (the doc comment on `A11y::active_descendant`, around line 452) states the opposite as a property of gpui rather than of this crate: the builder goes on the *item*, takes no argument, and "is honoured only while a focused **ancestor** of that item is on the node stack" — then names your arrangement outright as the one that "**cannot be expressed**. A sibling is not an ancestor, so the claim would be dropped in silence." In a command palette focus is on the query field and the results rows are its siblings under the panel, so this is exactly that case. Worse, the same doc comment records that this is the one state field `every_state_field_reaches_the_node` excludes, because the property applies at paint time behind `window.a11y.is_active()` and no test platform here switches accessibility on — so nothing in the suite can read it back and a wrong claim ships green forever.

   Do what `role_requires_keyboard_focus` does for `Role::Splitter` and what the combobox spec does for the same arrangement: declare the decline in the module docs with the reason, and let the selected row carry its fill and its `selected` state and nothing more. Two components independently needing this arrangement and both being unable to express it is worth a sentence in `docs/menus-and-listboxes.md` too — that is a gap in gpui, not in either component.

2. YOUR BINDING-DEPTH ARGUMENT IS CORRECT, AND HERE IS THE CITATION SO THE COMMENT CAN CARRY IT. I checked it in gpui 1.14.2 rather than take it: `KeyBindingContextPredicate::Descendant` exists (`src/keymap/context.rs:181`, parsed from `>` at `:361`, rendered back as `"{parent} > {child}"` at `:205`), so `"CommandPalette > Input"` is real syntax and not a hopeful guess. And `Keymap::bindings_for_input` (`src/keymap.rs:173`) collects every enabled binding and sorts them `depth_b.cmp(depth_a).then(ix_b.cmp(ix_a))` — descending depth, then **descending registration index**. So a later-registered binding wins a depth tie, which is precisely the property you are relying on. Put those two file:line references in the comment beside the `bind_command_keys` call in `init`, because "registration order is load-bearing" without the reason is a line the next person reorders.

3. NOTHING WAS RUN, AND THE ONE CLAIM YOU MOST NEED CHECKED IS THE ONE UNIT TESTS CANNOT CHECK. `cargo test --lib` was started and died in the link; the showcase page was **not compiled at all**. Three runs, in this order, and report each: `cargo test --lib`; then the real-window keyboard test the spec's own closing note asks for — `select.rs`'s harness, a view opened with `cx.open_window`, `press(cx, "down")` — because a unit test over `Vec<usize>` cannot tell a wired palette from one whose four keys all went to the text field; then `cargo check --example showcase --features examples`. A scout measured a cold build in this repository at 7m11s and then ran the full 501-test lib suite, so the budget is there.

4. THE TRIAGE MOVE IS FOUR EDITS IN ONE COMMIT OR NONE. You are right that `docs/component-triage.md`'s `Issue` row, the `EXPECTED` counts in `triage_coverage`, the counts restated in the document's prose and in its attribution section, and `every_written_issue_is_reachable_from_the_triage`'s requirement about `docs/issues/command.md` all move together. Make the call and do it in one commit; a half-move is a red build with four separate-looking failures.

You are likely carrying the combobox (#160) on the same branch — the two share `docs/menus-and-listboxes.md`, `examples/showcase.rs` and `src/elements.rs`, and both are chooser-family components. If so, item 2 above is the answer to a question that spec left open with two guesses, and item 1 is a decision the two of you must make the same way.

### On spec 3 of 4: Calendar: a month grid of selectable days (#157)

Approved. This is the second spec today that ran what it wrote — `cargo test --lib` to 512 passed *and* `cargo check --example showcase --features examples` clean — and it settled the date-type question the issue said had to be settled before any grid is drawn, with arithmetic verified over every day from 1800 to 2200 rather than sampled. Three items, each accounted for in SUMMARY.md.

1. THE KEYBOARD MUST BE ACTIONS, NOT ONE `on_key_down`. The spec says "one `on_key_down` on the grid" for arrows, `home`/`end`, `pageup`/`pagedown` and `enter`/`space`. `docs/menus-and-listboxes.md` §3 (lines 120–127) rules on exactly this: *"gpui dispatches bound actions **before** key-down listeners, so the raw handler loses any key an enclosing element has bound — a context menu inside a `Dialog` gives up Escape to the dialog and closes the wrong thing. The listbox is the one that is right; the menu has the same defect and its own fix to come. Nothing here says the two must converge on one mechanism, only that **a new popup in either family should copy the listbox**."* A calendar in a date-picker popup is a new popup, and every key you are taking is a named key.

   The narrow exception the crate does allow is in `select.rs`'s module docs and does not cover you: type-ahead stays an `on_key_down` because "a binding per letter is not a keymap". Arrows and `enter` are not letters. Copy the listbox: an `actions!` block, a `Calendar` key context, a `bind_calendar_keys` called from `crate::init` next to `bind_select_keys`. `select.rs` is the worked example and its module docs carry the argument. Retrofitting this later means adding a public `bind_*` function that `init` has to call, so it is cheaper now than at any later point.

2. THE GRID NEEDS AN IN-DIRECTION FOR THE VISIBLE MONTH, NOT ONLY AN OUT. You have `CalendarEvent::MonthChanged(Date)` and no way for an owner to *set* the visible month. The date-picker spec (#162) identified this as the single requirement it imposes on you, and it is right: a calendar that keeps its visible month privately cannot be told to follow a date the user typed into the field, which is the first of that component's agreement rules. Add the setter — a `set_visible_month` on the entity, paired with the event you already emit, so the two directions are named symmetrically. This is cheap now and awkward later, because `Calendar` is an entity and this is its public surface.

3. THE GRID IS UNTESTED AS A GRID, AND ITEM 1 CHANGES THE PART THAT IS. You name the gap plainly and you name the right three assertions — 42 days always, starting on `first_day_of_week`; `pagedown` from the 31st landing on the clamped day and emitting exactly one `MonthChanged`; a day cell announcing `Role::GridCell` with `selected` and `active_descendant` on the right two cells. Build that harness and write those three, plus one for the keyboard after you have moved it to actions, since a `TestAppContext` entity plus `a11y::test_support::announced` is the crate's own tooling for it. The date arithmetic is the part least likely to be wrong and it is the only part currently proven.

Two things I checked rather than assumed. Your `active_descendant` arrangement is the **valid** one — the grid takes focus and the day cell inside it makes the claim, which is a focused *ancestor* and exactly what `a11y.rs` says is honoured. Another spec in flight claims it from a focused field onto a sibling popup, which `a11y.rs` says is dropped in silence; you got it right and it is worth a sentence in your module docs saying which of the two arrangements this is, because the distinction is invisible in a diff. And the decision to leave `A11y`'s missing grid-index fields unreported rather than reaching for gpui's builders directly is correct — `no_element_calls_gpuis_a11y_builders_directly` would fail the build, and a name carrying the full date is a reasonable stand-in until `A11y` grows the four fields.

This unblocks #162. That task is held in the backlog until your work is in the trunk, not merely approved.

### On spec 4 of 4: Form: grouping and label association, not form state (#164)

Approved. `511 passed, 0 failed` with seven new unit tests including the guard-through-panic one, and the `Global` alternative rejected for a stated reason rather than by default — this is careful work. Four items, each accounted for in SUMMARY.md.

1. NOTHING THE ISSUE ASKED FOR WORKS UNTIL ONE CONTROL ADOPTS THE CONTEXT — ADOPT ONE HERE. Your own note is the finding: no leaf control reads `disabled_here`, `name_here` or `focus_handle_here`, so after this lands a fieldset's `disabled` cascades to nothing, a label click focuses a handle nothing tracks, and the label association in the issue's title does not exist. What ships is a mechanism with no consumer, which is the state that looks finished and is not.

   I am not asking for all eight — that is a separate change and you are right that it interacts with the rollout order in `a11y.rs` §6. Adopt **one**, whichever is cheapest (`Checkbox` is the obvious candidate: it has a `Disableable`, a name, and a focus handle), and write the end-to-end test that a mechanism deserves — a disabled `Fieldset` containing a `Checkbox` that says nothing about `disabled` renders it disabled, and a click on its `Field` label lands focus on it. One working path proves the three read APIs at once; seven unit tests over the stack prove the stack.

2. YOUR DEFERRED-DRAW HAZARD IS REAL AND NARROWER THAN YOU STATED — SAY WHICH. "A popover deferred out of a disabled `Fieldset` reads no ambient disabled" is true of a read performed *inside* the deferred closure. It is not true of a read in `render`, which runs during `request_layout`, inside the scope — and `render` is exactly where your adoption instruction puts it. So the rule is not "deferred elements are broken", it is **read the ambient value in `render` and pass it into whatever you defer; never call `disabled_here()` inside a deferred closure.** Write it that way in the module docs. The version in the spec reads as an unavoidable hole and would teach the next author to hand-thread `disabled(true)` into every popup, which is the threading the whole design exists to remove. If there is a case where the deferred closure genuinely must read it, name that case.

3. "A BOUNDED LEAK" IS AN ASSUMPTION ABOUT CALLERS, NOT A PROPERTY. The focus-handle registry never evicts, so it is bounded by the number of distinct field ids *ever rendered*, not by the number of fields on screen. That is fine for a form with fixed ids and unbounded for any caller that derives a field id from a row, a record or a task — which is the ordinary way an id gets made in a list-driven app. Keep the registry; change the comment to say what the bound actually is and what makes it grow, so nobody reads "bounded" and stops thinking. If there is a cheap eviction (dropping an entry whose only remaining `FocusHandle` reference is the registry's), say whether you considered it and why not.

4. COMPILE THE SHOWCASE. `cargo check --example showcase --features examples` — `showcase_coverage` proves the table, the nav entry and the match arm agree with each other, and proves nothing about whether `render_form_page` type-checks. You identified this as the first thing to do; do it. #157's scout ran `cargo test --lib` to 512 *and* this command clean in the same run, so it fits.

Two things about the tree you will build on. `src/a11y.rs` is also edited by PR #201, which is open and unmerged — #165 deletes the `"dialog"` row from `ELEMENTS_WITHOUT_A_ROLE` and you delete the `"field"` row from the same list. Expect to resolve that, and do not take the opportunity to reorder or reformat the list. And `docs/component-triage.md`: you are right that nothing fails today, but if you flip Form to Shipped, the row, `EXPECTED` in `src/elements.rs`, and both prose restatements move in one commit or the build fails three ways at once.

## Directions for this implementation

The orchestrator agent added the following when requesting this build. It is **not** part of any spec above, and no reviewer has seen it — it is addressed to you.

Treat it as a requirement, not a suggestion. The specs are still what is being implemented; these directions say how to go about it. Where one genuinely conflicts with a spec, the direction wins — it was written after the spec was approved, with this build in view — but **say so in `SUMMARY.md`**, because the reviewer reads the spec and cannot see this section.

Account for every direction in `SUMMARY.md` — including any you decided against, and why. A direction you silently dropped is indistinguishable from one you never read.

Four component specs — Calendar (#157), Command (#159), Combobox (#160), Form (#164) — in one build because all four touch `examples/showcase.rs` and `src/elements.rs`, and #157/#159/#160 also touch `src/lib.rs`. Keep them as four separable commits and integrate with each other rather than around each other: the second one to add a showcase section should read what the first added, not append a parallel structure beside it.

Your base is another build's branch (#190, #198), not main. That build declares `.tasks/verify` for this repository for the first time, so **your suite runs and a red suite fails this build before it opens a pull request** — which has never been true for a gpuikit build before. Read the script in your base rather than assuming what it runs.

Measured facts about this repository, so you do not spend budget rediscovering them: `cargo test --lib` on the current trunk is 514 tests in 0.19s, and effectively all the wall clock is the cold `gpui` compile. `cargo clippy --all-targets` is 30 warnings and zero errors on the trunk — all pre-existing, mostly `type_complexity` on callback fields — so do not treat clippy noise as yours and do not clean it up here; #203 covers the repository's missing CI and that decision belongs with it.

One ordering note you cannot see from the specs: #162 (Date Picker) was sent back to the backlog because it authored the same files this Calendar spec owns. #157 owns them. If you find yourself wanting a date-picker field to hang the calendar on, do not build one — leave the seam and say in SUMMARY.md what shape you left, because #162 will be re-scouted against whatever you land.

The strongest thing in this batch's review feedback is the keyboard rule, and it applies to more than the spec it is attached to: gpui dispatches bound actions **before** key-down listeners, so a raw `on_key_down` handler silently loses any key an enclosing element has bound. `docs/menus-and-listboxes.md` states this and `select.rs`'s Listbox is the worked example — copy it for Command and Combobox rather than reaching for a raw handler, and if you genuinely need a raw one (Listbox has exactly one, for type-ahead), say in SUMMARY.md why the action route would not carry it.

## Your job

1. Implement every spec above, in order, as one coherent change in the cloned repo (cwd). You are on the right branch already.
2. Run the project's tests / lint / typecheck — get them green.
3. Commit your work with clear messages (a git identity is configured).
4. Write `SUMMARY.md` in the repo root: one or two paragraphs describing the change, suitable as a pull request body. Do not use GitHub closing keywords (`Closes #N`, `Fixes #N`) — the server links the issues itself.
5. Do NOT push and do NOT open a PR — the server does both.

**You have 60 minutes, once.** That is the whole run — the clone before you started, this turn, the supervisor's own test run and the packaging after it — measured on the wall clock from dispatch. There is no later: when you end your turn the run is over. A backgrounded command buys you nothing — its child is killed with the turn — so anything whose result you need must be awaited inline, and a poll loop over a file another process will write can only report to a turn that has already ended. Nor should you start what cannot finish: a cold build in a large workspace can run forty minutes, so weigh what a command will cost against what is left.

On step 2: when this project declares a test suite at `.tasks/verify`, the supervisor runs it itself after you finish, against the committed tree your branch carries. If it fails you get one chance to fix it and then the build fails with no pull request, so getting there first is entirely in your interest. It reads that script out of the build's BASE commit, so editing it changes nothing about what runs.
