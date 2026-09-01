# SecureField: a masked mode of TextField, not a second field

## What it is

Password entry: `text_field(&state, cx).secure(true)` — every character drawn
as a bullet, the value never leaving the process through the places a field
leaks it. A mode of `TextField`, not a new element and not a new `pub mod`.

Traits: whatever `TextField` already has — `Disableable`, `ControlSized`,
`Focusable`. Not `Labelable` (`TextField` is not; labels are `Field`'s job).

## Why it survives triage

**The house precedent decides the shape before anything else does.** The Input
OTP rejection reads: "one field with a mask, not six fields" — a visual
convention layered over a single value must not become a second component,
because the second component breaks paste, selection, screen readers and
password managers. A password field is the same sentence with a different
mask. shadcn, notably, has no SecureField entry at all, because on the web
this is one attribute on one input — `type="password"` — and that is exactly
the altitude it should have here: one builder method.

**What is genuinely new is small and all of it is leak-plugging.** The mask
itself is a display substitution. The work is every other channel the real
text currently flows through: the IME handler, the clipboard, and the
disabled-rendering path. Each is named below with its file.

Deliberately not built: a reveal (eye) toggle inside the mode. SwiftUI's
`SecureField` ships none — a reveal button is app-level UI, and `TextField`
already has the extension point for it: a suffix `Adornment::element` holding
an icon button (`src/icons.rs` has `eye_open`/`eye_closed`) that flips
`.secure(…)`. The showcase demonstrates that composition; the mode does not
own it. Also not built: strength meters, "caps lock is on" warnings, and any
keychain/password-manager integration — the last is a platform service gpui
does not expose, the same verdict as Native Select: not gpui's to give.

## Prior art

- **SwiftUI, `SecureField`** — the API shape: the secure counterpart to
  `TextField` with the same binding surface, masking as it types, no built-in
  reveal toggle. Re-open before implementing to confirm the no-reveal
  decision still matches.
- **macOS `NSSecureTextField`** (the behavior SwiftUI wraps) — the behavioral
  contract worth copying: copy and cut disabled, paste allowed, IME
  composition not displayed. Re-open for the edge cases, especially
  composition.

## What it has to close in this crate

- **Where the mask goes: `InputState`, once.** Display text is produced in two
  places that must agree — `InputState::update_line_layouts`
  (`src/input/state.rs`) shapes `self.content` into `line_layouts`, and
  `SingleLinePaintState::from_input` (`src/elements/input.rs`) walks
  `content()`'s chars against that layout for cursor/selection positions. A
  `display_text()` on `InputState` that substitutes one bullet per char in
  secure mode, read by both sites, keeps glyph positions, hit-testing and the
  caret consistent by construction. Masking anywhere else desynchronizes the
  caret from the glyphs. `TextField::secure(bool)` writes through to the state
  at the top of `render`, exactly the mechanism `TextField::read_only` already
  uses and documents.
- **The IME leak.** `EntityInputHandler::text_for_range`
  (`src/input/handler.rs`) hands the field's real text to the platform text
  system. In secure mode it must return the masked string. Composition
  (`replace_and_mark_text_in_range`) should commit text directly and never
  hold a marked range — the marked-underline painters in
  `src/elements/input.rs` would otherwise draw composition state over bullets,
  and NSSecureTextField's answer is to not compose at all. Dead keys ride the
  same path and get the same answer: the composed character lands, the
  in-progress state never shows.
- **The clipboard leak.** `InputState::copy` and `cut` (`src/input/state.rs`)
  write the selection to the clipboard; both are no-ops in secure mode. Paste
  stays: password managers paste.
- **The disabled-rendering leak.** `TextField`'s disabled arm
  (`src/elements/text_field.rs`) bypasses the `Input` element entirely and
  renders `state.read(cx).content()` as static text — which in secure mode
  prints the password in the clear. It must render the masked string.
- **Bidi.** `update_line_layouts` runs `detect_base_direction`
  (`src/input/bidi.rs`) per line on the text it shapes. Masking at the
  display-string level makes this a non-issue by construction: a string of
  bullets has no strong directional character and lays out LTR, which is the
  correct and platform-standard presentation for a masked value. This is one
  more reason the substitution happens before shaping, not during painting.

## Accessibility

`Role::PasswordInput` — accesskit's own variant, re-exported by gpui and
reachable through the existing `A11y`/`Announce` machinery; in current
accesskit the role itself carries the protected semantics, so no flag and no
upstream ask. `src/a11y.rs` already lists `PasswordInput` in both
`role_requires_a_name` and `role_requires_keyboard_focus`, with test pins. The
one hard rule: the node reports **no value** — no `A11yValue::Text` of the
secret, ever. Since `text_field` is still on `ELEMENTS_WITHOUT_A_ROLE`, the
clean sequence is: adopt `text_field`/`textarea` per that list, with secure
mode switching the role to `PasswordInput` and suppressing the value in the
same change.

## Sizing

Nothing to do: `TextField` already implements `ControlSized`, and a mode adds
no dimension. That is the point of it being a mode.

## Showcase

No new `pub mod`, so `showcase_coverage` demands no new row — the text-field
page grows a secure section: a masked field, the reveal-toggle composition via
`Adornment::element`, and a disabled masked field (pinning the
disabled-rendering fix).

## Non-goals

A `SecureField` element or module; a reveal toggle inside the mode; strength
meters; keychain integration; masking for `Textarea` (no platform has a
multiline password field); a segmented OTP mask — that is the Input OTP
revisit trigger and a separate masking feature.
