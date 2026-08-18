# Keyboard focus is part of the announcement, and `Select` declares its roles

`Button` announced `Role::Button` and could not take keyboard focus, so it
promised a screen reader an actionable control that a keyboard could not reach.
This closes that gap by making focus part of the announcement rather than a
second thing an element has to remember: `A11y` gains `focusable()`,
`focus_handle(handle)` and `not_focusable(why)`, and `Announce::announce`
*applies* the answer — `focusable().tab_stop(true)`, or
`track_focus(&handle.tab_stop(true))` for a caller's handle — in the same call
that reports the role, so the two cannot drift apart. For the roles in the new
`role_requires_keyboard_focus`, saying nothing trips a `debug_assert!`, the
counterpart of the existing missing-name one; declining is a decision and takes
a reason, which is what distinguishes it from a call someone made to silence the
assertion. `Button`, `SidebarTrigger` and `Select`'s trigger take focus when
enabled and decline it, in writing, when disabled. Enter and Space activation
comes free from gpui; Tab needed a keybinding, which `gpuikit::init` now
installs — before `input::bind_input_keys`, which is what keeps Tab inside a
focused text input. The issue's premise that a caller would have to supply a
`FocusHandle` turned out not to hold: gpui mints one for a focusable element and
keeps it in that element's own element state, so `focus_handle` is an optional
builder and no existing `button(…)` call site changed. `theme::focus_ring` is
one definition of the ring, drawn as a spread `BoxShadow` through gpui's
`focus_visible` so that arriving focus neither reflows neighbours nor lingers
after a click.

Alongside it, `Select` implements the role convention — the first element
adopted after it. The trigger announces `Role::ComboBox` with its name, its
`expanded` state and, as its *value*, the label of the chosen option; the popup
announces `Role::ListBox` named after the control it dropped out of; and each row
announces `Role::ListBoxOption` with `selected`, its position and the size of the
set (`A11y::size_of_set` is new — gpui had `aria_size_of_set` and this module had
not modelled it, so `position_in_set` announced "3" with no "of N"). That needed
one breaking change: `select()` and `Select::new()` take the accessible name as
their second argument, because `Role::ComboBox` is in `role_requires_a_name` and
every naming source the convention allows was unavailable — a select's visible
text is its value, the placeholder disappears on first choice, and gpui has no
`labelled_by` builder. `Cargo.toml` is at 0.8.0, cut but unpublished, so today is
the cheap moment for it. Finally, the convention's *negative* scan gains a
*positive* one: `a11y::ELEMENTS_WITHOUT_A_ROLE` names every element module that
still declares nothing and why, and
`a11y::tests::every_element_module_declares_a_role` checks it in both directions
so the list can only shrink. It starts at 36 entries, which is the finding rather
than the bloat — only `Button`, `Sidebar`, `Splitter` and now `Select` are
adopted, and the gap was previously invisible.

**What the keyboard can and cannot do with a `Select`.** Tab reaches the trigger,
and Enter or Space opens the popup, which is what `.focusable()` plus gpui's
keyboard activation give it; Shift-Tab walks back; a disabled select is not a tab
stop. The popup still has **no keyboard model at all** — no arrow keys between
options, no Escape to dismiss, no roving focus — and reports no
`aria-activedescendant`, because that property names the row keyboard focus is
virtually on and there is no such row yet. Both belong to the same follow-up and
are deliberately not in this change.

## Review feedback

- **Add `Role::DefaultButton` to `role_requires_keyboard_focus`** — done. It is
  on the list directly below `Role::Button`, with a comment saying why (a
  dialog's Enter key resolves to it), and `the_focus_rule_covers_the_roles_a_keyboard_operates`
  asserts it.
- **Decline `Role::Splitter` in writing rather than adding the arm** — done, and
  worth flagging: `src/elements/splitter.rs` is *already on this base* (#158
  landed in commit `1ff05f0`), announcing `Role::Splitter` with a raw
  `tab_index(0)` and no focus decision. So adding the arm would have reddened
  this branch, not only a later merge. The decline is written into
  `role_requires_keyboard_focus`'s docs beside the composite-item roles, naming
  gpuikit#181 as the adoption issue, and a test asserts the role is not on the
  list.
- **Delete `src/traits/visual_focus.rs` in this change** — done. The file and its
  `pub mod visual_focus;` line are gone, with a Breaking bullet in `CHANGELOG.md`
  and the `docs/component-triage.md` blocker row that cited it rewritten to point
  at `src/a11y.rs` §4 and `theme::focus_ring` instead.
- **Mutation-check the textarea test** — done, and it needed a real fix. Written
  as the spec described, the test would have been vacuous: gpui offers the
  dispatcher every matching binding *in turn*, so with no `FocusNext` listener in
  the path the swapped `init` order still ends up dispatching `input::Tab` and
  the tab character still lands. The harness therefore now has three things at
  once — a `FocusNext` listener above the input (a root `div`, standing in for an
  app's root), somewhere else for focus to go (a `button`, which `announce` makes
  a tab stop), and the input itself. With those in place, swapping the two calls
  in `crate::init` fails it: `Tab moved focus out of a focused text input`,
  `left: FocusId(2v1)`, `right: FocusId(1v1)`. The doc comment on the test says
  which three, and that removing any one makes the mutation pass.
- **`Select`'s `ComboBox` trigger must declare `.focusable()`, not
  `.not_focusable()`** — done. The one nuance: a *disabled* select declines, with
  a reason, exactly as a disabled `Button` does. That is not the case the
  feedback was about (a live combo box declining focus); it is the same
  weaker-of-two-ARIA-answers position gpui's missing `aria_disabled` forces on
  every control here, and `the_trigger_takes_keyboard_focus` pins both halves.
- **Do not build the listbox's keyboard model** — followed. Nothing here adds
  arrow keys, Escape or roving focus; what the trigger can and cannot do is
  stated in the paragraph above, in the module's `# Accessibility` section, and
  in `CHANGELOG.md`.
- **Drop `A11y::active_descendant`** — done, in the sense that it was never
  added: no field, no builder, no apply, no row in `option_a11y`, and no
  `an_active_descendant_is_invisible_to_a_test`. `size_of_set` is kept, applied
  and asserted in both `every_state_field_reaches_the_node` and
  `a_row_announces_its_place_in_the_set`.

## Directions for this implementation

- **The interlock between the two specs** — handled as described: `Role::ComboBox`
  is on `role_requires_keyboard_focus` and `SelectState`'s `A11y` declares
  `.focusable()`, so the assertion the two specs would otherwise have collided on
  is satisfied by construction rather than by luck.
- **Do not add `Role::Splitter`** — followed; see the review-feedback item above,
  including the correction that the splitter is already in the tree here.
- **Delete `visual_focus.rs`; drop `active_descendant`; keep `size_of_set`** —
  all three done.
- **`CHANGELOG.md`: bullets under `## [Unreleased]` only** — followed. No
  versioned heading was added; `release_version_guard`'s existing
  `the_script_agrees_with_this_file_about_the_top_heading` still passes.
- **Build serially and read the output, not the exit code** — followed. Every
  build and test run used `-j 1` and wrote to a file rather than a pipe, and the
  results were read from that file. No `ld` kill occurred in this session.
  `cargo check --examples --all-features` was not run, on the direction's own
  advice about its cost; `cargo check --examples` (default features) was.
- **Do not end the turn waiting on a background command** — followed; every
  command was waited for and its output read before finishing.

## Notes on the spec

Two places where what shipped differs from the spec text, both recorded above
rather than silently: the textarea test's harness (the spec's shape would have
been vacuous, which is what the reviewer suspected), and the button keyboard
tests' harness, which focuses an app-shaped root before pressing Tab — with
nothing focused, gpui dispatches above the root element and the very first Tab
reaches no listener at all. That gotcha is now itself a test
(`tab_does_nothing_before_anything_is_focused`), paired with
`a_root_that_tracks_a_handle_answers_the_first_tab`, which is the spec's "two
tests state both halves". The spec's claim that `Role::Splitter` is not yet in
the tree is wrong on this base; the decision it asked for is unchanged and is
now better justified.

Verification: PASSED — `cargo test --lib -j 1` (442 passed, 0 failed), `cargo test --doc -j 1` (2 passed, 51 ignored), `cargo fmt -- --check` (clean), `cargo check --examples -j 1` (clean but for the pre-existing `unused_mut` in `src/input/bindings.rs`). Mutation-verified: deleting `.tab_stop(true)` from either path in `Announce::announce` fails the keyboard tests, and swapping the two binding calls in `gpuikit::init` fails `tab_stays_inside_a_focused_text_input`.
