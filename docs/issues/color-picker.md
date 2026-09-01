# ColorPicker: a swatch, a popover, and sliders that a keyboard can drive

## What it is

A swatch button showing the current color that opens a popover containing: a
saturation/value plane, a hue slider, an alpha slider (on by default,
`supports_opacity(false)` to drop it, SwiftUI's flag by another spelling), and
a hex field. Value type is `gpui::Hsla` — the crate's theme API already speaks
`Hsla` in every signature, so nothing new enters the public surface.

Traits: `Disableable`, `ControlSized` (the swatch). Not `Labelable` — the
swatch has no visible text, so its name is a required constructor argument,
the `IconButton`/`Splitter`/`ComboBox` precedent. Not `Orientable`, not
`Selectable`.

## Why it survives triage

**The platform picker is not gpui's to give.** SwiftUI's `ColorPicker` is a
swatch that opens the *system* color panel. That is the Native Select verdict
verbatim: "the platform one is not gpui's to give" — gpui has no platform
control embedding, and the system panel would not match the theme if it did.
So a toolkit that wants a color picker must draw one, and the only real
question is how much.

**More than a swatch grid, less than Photoshop.** A palette-only picker (GTK's
default view) answers "pick one of ours," which a caller can already build
from buttons in a popover. The thing a caller cannot cheaply build is the
editor: continuous hue/SV/alpha with a hex escape hatch, hit-tested against a
gradient. That editor is the component. The palette is deliberately not built
— when a consumer wants preset swatches, that is a row of buttons above the
editor, composed by the caller.

Deliberately not built besides the palette: an eyedropper (needs screen
capture, a platform service gpui does not expose — same verdict again), RGB/
HSL numeric entry tabs (the hex field is the numeric path; tabs are a revisit
when a consumer asks), and recent-colors memory (application state, not
toolkit state).

## Prior art

- **SwiftUI, `ColorPicker`** — take the API: label, `selection` binding,
  `supportsOpacity` defaulting to true. Reject the delegation to the platform.
- **GTK 4, `ColorChooserWidget`** — the one non-platform editor worth
  reading: palette first, single-color editor behind it, alpha behind
  `set_use_alpha`. Note before copying anything structural: it is deprecated
  as of GTK 4.10 in favor of `ColorDialog`; re-open both and read what GTK
  decided the replacement should keep.
- **WAI-ARIA APG** — has **no color-picker pattern**. That absence is a fact
  to design against, not around: there is no blessed answer for a 2-D
  saturation/value plane, and every accessible implementation in the wild
  reduces it to one-dimensional controls. The pragmatic answer, taken here:
  the sliders and the hex field are the accessible path and can set every
  reachable color; the SV plane is a pointer-only enhancement that never
  holds the only route to a value. (react-colorful demonstrates an
  arrow-key-operable plane is possible; it is a revisit trigger, not a
  requirement.)

## What it has to close in this crate

- **The overlay follows `docs/overlays.md`**: `deferred(anchored(…))` at
  priority 1 (the "popups anchored to a trigger" rung), `.occlude()` on the
  panel, offset not margin, default `SwitchAnchor` fit. `src/elements/popover.rs`
  is the shell to reach for — a caller-built panel offset from its trigger is
  literally its row in the overlay table.
- **Color math.** gpui has `Hsla` natively, `Rgba: TryFrom<&str>` for hex
  parsing, and `Hsla`↔`Rgba` conversions — but the SV plane is HSV, not HSL,
  and the crate has no HSV conversion (`src/utils/` contains only
  `element_manager`). HSL↔HSV is a dozen lines of pure, testable math; per
  `src/theme/control.rs`'s "what belongs here" note, control-specific shape
  stays in the control's file, so it lives in the element module until a
  second consumer appears.
- **Rendering the gradients.** gpui has `linear_gradient`; the SV plane is
  two stacked linear gradients (white→hue, transparent→black) and the hue
  strip is six adjacent two-stop gradient segments. No `PathBuilder` needed.
- **The sliders cannot be `Slider` yet.** `src/elements/slider.rs` shows the
  drag mechanics to copy — window-registered mouse handlers via the canvas
  paint hook — but it is an `Entity` with a label/value row, no keyboard
  support and no role. The hue/alpha sliders here need the keyboard contract
  (arrows, Home/End) and `Role::Slider` with `A11yValue::Number` from day
  one. Either `Slider` grows those first and is embedded, or the picker draws
  its own tracks on `Slider`'s drag pattern; the first is better for the
  crate and this issue should say so out loud.
- **Hex entry** is a `TextField` (`src/elements/text_field.rs`) with a `#`
  prefix `Adornment::text`, validating on commit via `Rgba::try_from`.

## Accessibility

The swatch trigger announces accesskit's own `Role::ColorWell` with a
required name and `expanded` on open (the `Sidebar`/`SidebarTrigger`
precedent: state on the control the user operates). accesskit carries a
`color_value` property for `ColorWell`; **gpui has no builder for it** — the
same shape as the missing `aria_sort` that `docs/component-triage.md` records
for Table, and it is named here as the identical upstream ask. Until it lands,
the swatch reports its color as `aria_value` text (the hex string), which is
what a screen reader can actually read anyway. Inside the popover: sliders as
above, hex field per `text_field`'s adoption, and the SV plane reports nothing
— it is the redundant pointer path by design.

## Sizing

`ControlSized` on the swatch: the swatch is `ink` square inside a
rung-height trigger, every dimension off `ControlMetrics`. The popover panel's
internal dimensions are the control-specific shape that stays in the element's
file, keyed off the rung.

## Showcase

A `color_picker` row in `ELEMENT_COVERAGE` and a page: the three rungs, a
picker without alpha, hex round-tripping, and a caller-composed preset row
demonstrating that the palette is composition.

## Non-goals

The system color panel; an eyedropper; palette/recent-colors state; RGB/HSL
input tabs; a color *type* of the crate's own — `Hsla` in, `Hsla` out.
