# Layout: `zstack()` and `spacer()`, not parameters

## What it is

`src/layout.rs` is four functions: `h_stack()`, `v_stack()`, `centered()`, `justified_row()`. Measured against SwiftUI's stack vocabulary — HStack, VStack, ZStack, Spacer — it is missing the two *names* and none of the *parameters*. This issue adds `zstack()` and `spacer()`, deletes the two helpers nobody calls, and declines to give `h_stack()`/`v_stack()` the spacing and alignment arguments SwiftUI's stacks take, because gpui's `Styled` trait already is those arguments and the crate's own usage proves the chain form won.

## Why it survives triage

This is crate infrastructure, not a #59 roster entry, so it earns its place on usage evidence rather than on a verdict table row. The evidence is one grep deep:

- **The bare stacks are the most-called functions in the crate.** ~190 `h_stack()`/`v_stack()` call sites across `src/` and `examples/`, in 19 element modules and every substantial example.
- **The composed helpers are dead.** `centered()` and `justified_row()` have **zero callers** — every `centered(` hit in the tree is gpui's `Bounds::centered`. Meanwhile `.justify_between()` is written inline 12 times: callers had the exact problem `justified_row` solves and chose the chain anyway, because the helper bakes in `.flex_1().w_full()` they did not ask for. A helper that names a flex *direction* thrives; a helper that bakes in an *arrangement* dies. That asymmetry is the design instruction for everything below, and the house has deleted better-established code on thinner evidence (`src/traits/portal.rs`, `Dropdown`).
- **The two missing names are being hand-written today.** Layering via `.relative()` + `.absolute()` is spelled out by hand in twelve modules (`switch.rs`, `toggle.rs`, `slider.rs`, `aspect_ratio.rs`, `splitter.rs`, `dialog.rs`, `combobox.rs`, `select.rs`, `popover.rs`, `toast.rs`, `command.rs`, `src/editor/meta_line.rs` — 29 sites). Flexible space has no name at all: gpui's `Styled` has **no auto-margin helpers** (checked against the styled macro in gpui-unofficial 1.14/1.17), so the only way to push one child to the far end of a row is an anonymous `div().flex_grow(1.)`, and `justify_between` only covers the symmetric case.

**Why NOT spacing parameters.** 151 of the ~190 stack calls — 79% — chain `.gap_*()` as the literally next call. Read naively that says "spacing parameter wanted"; read against the dead helpers it says the opposite. The chain *is* the crate's spacing API, it costs one call, and Rust has no default arguments, so `h_stack(spacing)` would either force an argument on the 21% that want none or split into `h_stack()`/`h_stack_with_gap()` — a second name for something `Styled` already names. Alignment is the same argument with worse numbers: only 29 of 78 `h_stack()` sites chain `.items_center()`, so baking it in (Zed's `h_flex` does) would change the majority to serve the third. Every parameter SwiftUI's initializers carry exists here as a `Styled` method on the very `Div` the helper returns. Adding them as parameters would be adding a second way.

**Why not theme-scale gaps.** `src/theme/mod.rs` has no spacing tokens — the theme's contract is colors plus the control size scale, and `ControlMetrics::gap` (`src/theme/control.rs`) is a control's *internal* icon-to-label gap, not page rhythm. The usage already has a de facto scale: `gap_1/2/3/4/6/8` account for 145 of 151 chained gaps (gap_2 ×69, gap_4 ×38, gap_1 ×18), with only 6 ad-hoc `gap(px(…))`/`gap(rems(…))` escapes. gpui's Tailwind rem scale is the spacing scale, it is serving, and inventing a parallel theme token set is a separate decision this issue does not take.

## Prior art

- **SwiftUI `HStack(alignment:spacing:)` / `VStack(alignment:spacing:)`** — the parameter shapes deliberately not copied, for the reasons above. `spacing: nil` means "platform default", a concept gpui has no metric for.
- **SwiftUI `ZStack(alignment:)`** — layers children, aligned, sized to the union. The alignment parameter *does* partially map: flex alignment on an `.absolute().inset_0()` layer reproduces the nine positions.
- **SwiftUI `Spacer(minLength:)`** — flexible space along the stack's axis. Maps cleanly: flex-grow is axis-aware for free, and `minLength` is a chained `.min_w()`/`.min_h()`.
- **SwiftUI alignment guides** (`alignmentGuide`, custom `AlignmentID`, `firstTextBaseline`) — **do not map and are out of scope forever.** They are a cross-child measurement protocol; gpui is Taffy flexbox, which aligns by box edges and cannot see one child's text baseline from another child.
- **Zed's `ui::h_flex`/`v_flex`** — same two bare functions, confirming the shape; Zed bakes `items_center` into `h_flex` and this crate's 29-of-78 says not to follow it.
- Re-open all of these before implementing.

## What it has to close in this crate

- **`zstack()` in `src/layout.rs`** — `div().relative()`, with module docs stating the layer idiom: the first child lays out normally and sizes the stack; each further layer is `.absolute().inset_0()`, aligned with its own flex properties. Migrate one hand-rolled site (`src/elements/aspect_ratio.rs` is the cleanest) as the worked example.
- **The zstack/overlay boundary, in writing.** `zstack()` is for layers that live *inside* an element's bounds — a thumb on a track, a badge on an avatar. Anything that escapes its container's bounds or clips is an overlay and belongs to `gpui::anchored()`/`deferred()` under `docs/overlays.md`'s priority ladder; gpui has no z-index (checked — nothing in the crate), so paint order and `deferred` priority are the only stacking controls, and a `zstack()` doc that does not say so will grow popovers.
- **`spacer()` in `src/layout.rs`** — `div().flex_grow(1.)` with a doc note that `.min_w()`/`.min_h()` is SwiftUI's `minLength`. It fills the auto-margin hole gpui's `Styled` has.
- **Delete `centered()` and `justified_row()`.** Zero callers, and each is two chained calls under a name that hides them. The crate has precedent for deleting unadopted abstractions rather than documenting around them.
- **Do not add spacing, alignment, or distribution parameters to `h_stack()`/`v_stack()`, and do not change their defaults.** If this issue grows an `items_center` in `h_stack`, it has silently restyled ~78 call sites and become the thing its own evidence argues against.

## Accessibility

None. These are layout wrappers with no semantics — the same answer `aspect_ratio` already has on record in `src/a11y.rs`'s `ELEMENTS_WITHOUT_A_ROLE` ("a layout wrapper with no semantics of its own"). `zstack()` and `spacer()` announce nothing, and `spacer()` must never take focus.

## Sizing

n/a. `ControlSized` is for controls with a height; a stack's size is its children's. The one adjacent decision — whether gaps come from a theme scale — is answered above: they stay on `Styled`'s rem scale.

## Showcase

`showcase_coverage` keys off `pub mod` in `src/elements.rs`, so `src/layout.rs` owes no page. The showcase already exercises the vocabulary on every page; migrating one visible layering (the switch or slider thumb) onto `zstack()` is demonstration enough.

## Non-goals

- Spacing/alignment parameters, a `Stack` type, or builder structs — argued above.
- Theme spacing tokens — a separate decision with its own issue if anyone wants it.
- Alignment guides or baseline alignment — not expressible in Taffy; upstream if ever.
- Any overlay behavior — `docs/overlays.md` owns that.
- Grid helpers — different vocabulary, different issue.
