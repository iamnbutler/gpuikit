# Stepper: a spinbutton, not a pair of buttons

## What it is

A numeric field with an increment and a decrement button: one value, a step, a
range, typed entry. The module is `stepper`, because the visible shape — the
paired buttons — is what the name describes; the contract it implements is
WAI-ARIA's **Spinbutton** pattern, and the role it reports is accesskit's own
`Role::SpinButton`. That is the same division `src/elements/splitter.rs` drew
between what a thing is called and which pattern it answers to.

Traits: `Disableable`, `Labelable`, `ControlSized`. Not `Clickable` (the value
changing is the event, not a click), not `Selectable`, not `Orientable`.

## Why it survives triage

**The shape question is SwiftUI versus ARIA, and ARIA wins.** SwiftUI's
`Stepper` is buttons only — no typed entry at all; `value:`, `in:`, `step:`
and nothing to type into. That shape is defensible on a phone, where the
numeric keyboard is one tap away in a paired `TextField`. On a desktop toolkit
it means sixty presses to get from 0 to 60. The APG Spinbutton pattern is the
other shape: a text field that holds the one tab stop, with increase/decrease
buttons **deliberately outside the tab order** — their function is reachable as
Up/Down on the field — and typed entry restricted to valid characters. Build
that.

**This is one field with two buttons, not a new text element.** The house
precedent is the Input OTP rejection: "one field with a mask, not six fields."
The field exists — `TextField` over `InputState` — and `TextField` already has
the extension point the buttons need: `Adornment::element`, whose doc comment
already anticipates a button-sized element sized against the rung. What is
genuinely new is the numeric contract on top: parsing, clamping, stepping, and
the keyboard meaning of Up/Down.

Deliberately not built: a formatted/currency/unit input (that is an
application's formatter driving `set_content`), long-press auto-repeat on the
buttons (revisit when a consumer asks; it is additive), and Page Up/Down large
steps (APG marks them optional; add them the day a consumer has a large step
to bind them to).

## Prior art

- **WAI-ARIA APG, Spinbutton pattern** — the keyboard contract this element
  commits to: Up/Down step, Home/End jump to min/max, focus stays on the text
  field, buttons out of the tab order, `aria-valuenow`/`min`/`max` on the
  field's node. Re-open it before implementing; the tab-order clause is the
  part everyone gets wrong.
- **SwiftUI, `Stepper`** — the value/step/range API surface worth copying
  (`value`, `in:`, `step:` defaulting to 1), and the shape worth rejecting
  (no typed entry). Re-open to check the API names, not the interaction.

## What it has to close in this crate

- **Up/Down mean the wrong thing inside an input.** `src/input/bindings.rs`
  binds `up`/`down` to cursor movement in the `Input` key context
  (`InputState::up`/`down` in `src/input/state.rs`). The stepper needs those
  keys to mean "step the value" on a single-line field. That is a key-context
  or binding-order decision of exactly the kind `src/a11y.rs` §4 documents for
  Tab, and it has to be taken deliberately, not by whichever binding happens
  to win.
- **Character restriction.** APG says typed input may be restricted to valid
  characters. `InputState` (`src/input/state.rs`, `insert_text`) has no input
  filter; either the stepper validates on commit (blur/Enter re-clamps and
  reformats) or `InputState` grows a character-filter hook. Commit-time
  validation is the smaller change and the recommended one — a filter hook is
  a new public API on the crate's most complicated state object.
- **The buttons.** Suffix `Adornment::element` housing two icon buttons
  (`src/icons.rs` has `plus()` and `minus()` / `chevron_up()`/`chevron_down()`),
  each stopping mouse-down propagation so a click does not focus-then-blur the
  field, and each **not** a tab stop — which for `IconButton` means declining
  focus the way `A11y::not_focusable` records, with the APG clause as the
  reason.
- **Slider is the wrong precedent to copy, and knows it.** `src/elements/slider.rs`
  is the crate's other bounded-numeric control and currently has no keyboard
  support and no role — `ELEMENTS_WITHOUT_A_ROLE` lists it as "the
  `A11yValue::Number` case section 3 was written for". The stepper should land
  the keyboard-numeric-value pattern properly, and slider's adoption then
  copies *it*.

## Accessibility

`Role::SpinButton`, fully expressible today: `A11yValue::Number { value, min,
max, step }` exists in `src/a11y.rs` and gpui has the builders
(`aria_numeric_value`, `aria_min_numeric_value`, `aria_max_numeric_value`,
`aria_numeric_value_step`). No upstream ask. `role_requires_a_name` and
`role_requires_keyboard_focus` already cover `SpinButton` — both pinned by
tests in `src/a11y.rs` — so a nameless or unfocusable stepper is a
`debug_assert!` on day one. The name comes from `Labelable`'s label when set
and is otherwise a constructor argument, the `ComboBox`/`Splitter` precedent.

## Sizing

`ControlSized` off `ControlMetrics` — it delegates to `TextField`'s existing
implementation, so the field, its adornment buttons (sized against `ink`) and
its text all move with the rung. Nothing in this element names a height.

## Showcase

`showcase_coverage` in `src/elements.rs` makes this a build requirement: a
`stepper` row in `examples/showcase.rs`'s `ELEMENT_COVERAGE` and a page
rendering all three rungs, a stepped range, a disabled stepper, and typed
entry that clamps on commit.

## Non-goals

Formatted numbers, units and currency; auto-repeat on press-and-hold;
Page Up/Down; a buttons-only SwiftUI-shaped variant; touching `Slider`'s
keyboard story (that is slider's own adoption, which should copy this one).
