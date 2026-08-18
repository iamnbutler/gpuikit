# A keyboard model for the select listbox, and the `A11y::active_descendant` that comes with it

0.8.0 shipped a `Select` that told a screen reader it was a combo box with a
list of *N* options and which one was chosen — `Role::ComboBox` on the trigger,
`Role::ListBox` on the popup, `Role::ListBoxOption` with `selected` and "n of N"
on every row — and then moved keyboard focus into that popup and offered nothing
to do with it. Every one of those statements was true; the defect was the
missing half of the pair. This change adds the interaction rather than narrowing
the roles: Up and Down move a highlight and wrap, Home and End jump to the ends,
Enter or Space chooses the highlighted option and closes, Escape closes without
choosing, Tab closes and moves focus on, and a printable character jumps to the
next option starting with it (press the same letter again to walk the options
that share it). Every close the keyboard asked for hands the trigger back its
focus; a click outside deliberately does not, because that click is already on
its way to focusing whatever it landed on. The **highlight** — where the
keyboard has got to — is a new state distinct from the **choice**, which is the
control's value: the popup opens with the two on the same row and they separate
on the first arrow key, hovering a row moves the highlight rather than drawing a
second one, and the highlighted row announces itself with `aria-activedescendant`
via a restored `A11y::active_descendant`. The keys are gpui *actions* bound in a
new `Listbox` key context by `elements::select::bind_select_keys` (called from
`gpuikit::init`, public for apps that assemble their own keymap) rather than a
raw `on_key_down`, because gpui dispatches bound actions before key listeners and
a raw Escape handler would lose the keystroke to `Dialog`'s escape binding
whenever a select sat inside a dialog. Tab is the one exception in the other
direction — `a11y`'s `tab` binding carries no key context and a context-less
binding outranks every scoped one, so the popup answers `FocusNext` /
`FocusPrevious` with a bubble-phase `on_action` listener, which runs before any
ancestor's `moves_focus_on_tab`.

The visible consequence is that **the chosen row is now a check rather than an
accent fill**, and the fill marks the highlight; the check sits in a fixed-width
slot every row reserves, which is the shape `context_menu.rs` already uses for
its toggled items, so this is consistency with the element next to it rather
than taste. `A11y` gains `active_descendant(bool)`, `is_active_descendant()` and
the apply in `Announce::announce`. It is the odd field on that struct in three
ways — a plain `bool` because gpui's builder takes no argument and `Some(false)`
would be a state the crate could hold and never report; set on the *descendant*
rather than the container, which is why the canonical APG arrangement (focus on
the trigger, pointing across at a sibling popup) cannot be expressed here at all;
and the one field no test in this crate can read back off a node, because gpui
applies it at paint time behind `window.a11y.is_active()`. All three are argued
in ~60 lines of doc comment at the field, including a paragraph on exactly what
its guard catches (a new field going unhandled, via the exhaustive `let A11y { … }`
destructure) and what it does not (a wrong or unconditional apply). Eleven new
tests in `select.rs` open a real window and press a real key, one new test in
`a11y.rs` holds the declaration, `CHANGELOG.md` and `docs/menus-and-listboxes.md`
§3 are updated, and the whole suite is green.

## Review feedback

- **Say what the `active_descendant` guard actually guards.** Done. The field's
  doc comment has a `# What guards this field, and what does not` section that
  names the two things the destructure and
  `an_active_descendant_is_declared_rather_than_read_back` *do* catch (a new
  field going unhandled; the builder or reader disappearing), states plainly
  that neither catches a wrong or unconditional apply, and says the escaping
  failure is gpui's two-nodes-in-one-frame panic in an app with accessibility
  on. `every_state_field_reaches_the_node`'s doc comment carries the same
  statement from the other side, and the module docs' section 3 points at the
  field. The apply is written as the single narrowest form — `if
  active_descendant { element = element.aria_active_descendant(); }` and nothing
  else — with a comment at that line saying why it is shaped that way.
- **Make the visual change legible to a human.** Done. `CHANGELOG.md` gets its
  own `### Changed` entry, written in terms of what a user sees ("the chosen
  option in an open select now shows a check mark instead of a filled row"),
  and it repeats the invariant: what has to survive any redesign is that the
  highlighted row and the chosen row stay distinguishable *when they are
  different rows*. It also follows `context_menu.rs`'s existing fixed-width
  check slot — the same `w(…)` + `flex_shrink_0()` + conditional `Icons::check()`
  shape it draws its toggled items with — which is the argument that makes this
  consistency rather than a taste call.
- **Type-ahead stays.** Kept, not cut, and tested — including the repeat-press
  walk and the wrap. It is the only affordance that makes a long option list
  usable by keyboard.
- **Enter with nothing highlighted does nothing.** Kept, with the test:
  `an_empty_listbox_answers_every_key_by_doing_nothing` presses every key
  including Enter and Space against a zero-option listbox and asserts the popup
  is still open and nothing was chosen, then presses Escape to prove there is
  still a way out.

## Directions from the orchestrator

- **Join the verification run before ending the turn.** Done. The build was
  started as the first action of the turn and the test run was waited on to
  completion; the line at the bottom reports what came back, not what was
  expected.
- **Never two cargo invocations at once.** Followed — the warm build, `cargo
  clippy`, and each `cargo test` ran strictly one at a time.
- **Read `src/lib.rs` and `CHANGELOG.md` as they are on the branch, not as the
  spec describes them.** Done. `src/lib.rs` already carried #187's two
  `#[cfg(test)] mod` declarations (`release_version_guard`,
  `release_input_validation`) and its `init` already called
  `elements::dialog::bind_dialog_keys`; `bind_select_keys` was added after it,
  and the ordering comment states both halves of the reason (it can never
  outrank `a11y`'s context-less Tab, and it should be registered after
  `Dialog`'s Escape). The changelog entries were added alongside the existing
  `## [Unreleased]` items — four under `### Added` before the existing
  `docs/menus-and-listboxes.md` entry, one at the top of `### Changed` before
  the existing `release.yml` entry — and nothing was replaced.

Nothing in the review feedback or the directions conflicted with the spec, so
there is nothing to report under that heading. Two places where I went beyond a
literal reading of the spec, both stated for the reviewer: `Listbox` gained a
small `row_a11y(index)` method so that `render` and
`exactly_one_row_claims_the_active_descendant` read the *same* declaration
rather than the test restating the render logic (the reviewer's point that the
select-side test reads the struct rather than the node still stands — that is
the limit of what is observable, and it is written down); and the Tab handlers
restore focus to the trigger *before* calling `window.focus_next`, so that "the
next tab stop" is the one after the control rather than after a popup that has
just stopped existing.

Adjacent work deliberately left alone, each its own change: `context_menu.rs`'s
raw `on_key_down` losing Escape to a surrounding dialog (same root cause,
different element — recorded in this module's docs and in
`docs/menus-and-listboxes.md` §3); a roving-focus convention for composite
widgets, which `Tabs`, `List` and `ContextMenu` all still want; and a third
focus answer in `A11y` ("takes focus, is not a tab stop"), which would let the
popup declare its focus through `A11y` like everything else instead of keeping
a bare `track_focus` — the standing reason for which is now stated at that call
site rather than the old "when the listbox grows a keyboard model".

Verification: PASSED — `cargo test --lib` (463 passed, 0 failed), plus `cargo fmt --check` clean and `cargo clippy --lib --tests --all-features` producing no new warnings in the changed files.
