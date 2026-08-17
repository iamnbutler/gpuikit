# Overlays

This is both a decision record — why `src/traits/portal.rs` was deleted rather
than adopted (#155) — and the convention that replaces it. It is held to the
crate by `mod overlay_coverage` in `src/elements.rs`, because a description
nothing checks is exactly how `portal.rs` survived a year with no callers.

## The decision: `gpui::anchored()`, not a `Portal` trait

`src/traits/portal.rs` shipped `Portal`, `PortalPosition`, `AnchorCorner`,
`AnchorEdge` and `FitMode` — 486 lines of positioning math with zero callers,
zero implementors and zero tests. It was read against all six overlay call
sites in this crate. The answer to "what would it have saved here" was
*nothing*, six times out of six, against #155's stated threshold of four.

| | `portal.rs` | `gpui::anchored()` |
| --- | --- | --- |
| Which corner of the overlay attaches | `AnchorCorner` (4 corners) | `Anchor` (4 corners **and** 4 edge centres) |
| Where it attaches | trigger corner or edge centre, inconsistently | the element's own position, or `.position(point)` in window coordinates |
| Offset from that point | `.offset(Point<Pixels>)` | `.offset(Point<Pixels>)` |
| Behaviour on overflow | `FitMode::{SwitchAnchor, SnapToViewport, None}` | `SwitchAnchor` (default), `snap_to_window()`, `snap_to_window_with_margin(edges)`, plus an unconditional clamp into the window |
| Matching the trigger's width | not modelled at all — `calculate_position` returns a `Point`, never a `Size` | not modelled either |
| **Who supplies the overlay's measured size and the viewport size** | **the caller, as arguments** | **gpui, in `prepaint`** |

The last row is what makes this a delete rather than a redundancy.
`PortalPosition::calculate_position` demands `portal_size` and `viewport_size`
from its caller. No `render()` body in this crate has either: an element's
measured size does not exist until layout has run, and a `RenderOnce` gets no
viewport. Adopting `Portal` anywhere therefore meant first writing a custom
`Element` with a `prepaint` to obtain them — at which point that element *is*
`Anchored`.

It was also wrong where it was not redundant. `preferred_edge()` picked a
corner via `to_anchor_corner(true)` while `get_anchor_point_on_trigger()`
anchored at the trigger's edge *centre*, so `PortalPosition::tooltip()` put the
overlay's bottom-left corner on the trigger's top-centre — off by half the
trigger width, never centred. Its flip path re-derived the anchor from trigger
*corners*, inconsistently with the initial placement, and flipped without
checking that the flipped box fits (gpui checks). None of it was caught, because
nothing called it.

## The recipe

```rust
this.child(
    deferred(
        anchored()
            .offset(point(px(0.), gap))   // never a margin on the child
            .child(div().occlude().child(panel)),
    )
    .with_priority(1),
)
```

- **`deferred()` over `anchored()`.** `deferred` paints the subtree after the
  rest of the frame, so the overlay draws over its siblings; `anchored` places
  it and keeps it inside the window. `deferred` keeps the ambient element-id
  stack (see `src/element_id.rs`), so a popup's contents stay scoped under the
  caller's id.
- **`.occlude()` on the panel**, so a click on the overlay does not fall
  through to whatever is underneath it.
- **`.offset(…)`, never a margin on the anchored child.** `Anchored::prepaint`
  measures the *union of its children's layout bounds*, fits that to the
  window, and applies the result with `with_element_offset`. A margin is
  outside that union, so it is added *after* the fit: a popup near the window
  edge gets clamped correctly and then pushed straight back out by its own
  margin. gpui says so in `anchored()`'s own doc comment ("Its children should
  have no margin to avoid measurement issues"), and three elements here were
  doing exactly that until #155.
- **Scope ids before roles.** See `src/element_id.rs`: gpui hashes an element's
  whole id path into an accessibility node id and refuses duplicates.

### Choosing a fit mode

`snap_to_window()` and `snap_to_window_with_margin(edges)` **replace**
`SwitchAnchor`; they do not add to it. In `Anchored::prepaint` the flip to the
other side of the trigger is guarded by `fit_mode == SwitchAnchor`, so a popup
that asks for a snap margin loses its flip. This is not obvious from the API.

- **Default (`SwitchAnchor`)** — right for anything anchored to a *trigger*:
  when the popup would leave the window, it flips to the other side of the
  trigger, which keeps the trigger visible. Every menu and panel here uses it.
- **`snap_to_window*`** — right when there is no trigger to flip around.
  `context_menu` is the only site that snaps, and correctly: a menu opened at
  the pointer should not jump away from the click.

The clamp-into-window pass runs for *every* fit mode, so no `anchored()`
overlay can leave the window whatever it is configured with.

## The draw-priority ladder

`deferred(…).with_priority(n)` — higher paints later, i.e. on top. These
literals used to exist only scattered through `src/elements/`, discoverable by
grep. Every `with_priority` in `src/elements/` must be a rung named here, and
`every_layer_is_on_the_ladder` checks it.

<!-- priority-ladder -->

| Priority | Layer |
| --- | --- |
| 1 | Popups anchored to a trigger — dropdown, select, popover, context menu |
| 10 | Dialog, and its scrim |
| 15 | Toast — above a dialog, because a toast reports the result of what the dialog did |

## The overlays this crate places

<!-- overlay-table -->

| Module | What it places |
| --- | --- |
| `dialog` | A centred modal over a full-window scrim. Anchored to nothing — it is `anchored()`-free by design |
| `dropdown` | A menu hanging one `MENU_GAP` below its trigger |
| `select` | The same `DropdownMenu`, below the same trigger shape |
| `popover` | A caller-built panel, offset from its trigger by a caller-supplied `Point<Pixels>` |
| `context_menu` | A menu at the pointer, `snap_to_window_with_margin(8px)` |
| `toast` | A stack in a window corner. Anchored to nothing |

Two of the six are not anchored to anything, and should not be made to look as
though they are. `src/elements/tooltip.rs` is deliberately absent: a `Tooltip`
is a view handed to gpui's `.tooltip()`, and gpui positions it — which is why
`portal.rs`'s one named use case, `PortalPosition::tooltip()`, was the one
overlay it could never have been used for.

## What would reopen the abstraction question

**A popup that has to match its trigger's width.** That needs the trigger's
measured size, which is the one thing neither `anchored()` nor `portal.rs`
supplies, and it needs it in more than one element before it is worth building
once. `docs/issues/combobox.md` will want it first. When a second element wants
it, the thing to build is a small custom `Element` that measures a trigger and
hands its size to the overlay — not a positioning trait, which is the part
gpui already does better.
