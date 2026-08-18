//! A draggable divider between two panes.
//!
//! # What this is, and what it deliberately is not
//!
//! shadcn's Resizable is a wrapper over a panel-group library: nested groups,
//! persisted layouts, collapsible panels. Zed has a real pane tree
//! (`workspace::pane_group`), and that is a workspace structure with splitting,
//! joining and serialisation in it. Neither is an element.
//!
//! The part that is genuinely a toolkit element is the **splitter**: one
//! divider, two neighbours, a drag that moves the boundary, a floor under each
//! side, and a keyboard equivalent. That is what ships here.
//!
//! There is no pane tree, no persisted layout and no collapse gesture. Three
//! panes is two splitters, nested by the caller — which is the point
//! `docs/issues/resizable.md` makes about the tree.
//!
//! # The ratio is the caller's
//!
//! [`Splitter`] stores nothing about where the boundary sits. It takes the
//! current ratio and emits the new one through
//! [`on_resize`](Splitter::on_resize), which is what lets a layout be
//! persisted, restored or reset from outside the element.
//!
//! ```ignore
//! splitter("editor-split", "Editor and preview", self.ratio)
//!     .start(self.editor(cx))
//!     .end(self.preview(cx))
//!     .min_start(px(160.))
//!     .min_end(px(120.))
//!     .on_resize(cx.listener(|this, ratio: &f32, _window, cx| {
//!         this.ratio = *ratio;
//!         cx.notify();
//!     }))
//! ```
//!
//! # The drawn line and the band you can hit
//!
//! The line is [`Separator`](crate::elements::separator)'s 1px hairline, not a
//! second one. The *interactive* band around it is 6/8/12px depending on the
//! rung — that difference is what decides whether a divider feels good or
//! whether nobody can hit it, and `docs/issues/resizable.md` names it as the
//! detail to get right.
//!
//! # Accessibility
//!
//! The band reports [`Role::Splitter`] with its position, both floors and one
//! arrow key's worth expressed as percentages, and follows the WAI-ARIA window
//! splitter keyboard contract: the arrow keys on the split axis move the
//! divider by one step, `home` and `end` go to the floors. The pattern's
//! optional enter-collapses-the-pane is deliberately absent — a collapsed pane
//! is state, and this element holds none.
//!
//! A divider has no visible text to borrow a name from, so the name is a
//! constructor argument rather than a builder, exactly as
//! [`icon_button`](crate::elements::icon_button) does it. `Role::Splitter` is
//! in [`crate::a11y::role_requires_a_name`] for that reason.

use std::rc::Rc;

use gpui::{
    canvas, div, prelude::*, px, AnyElement, App, Bounds, Context, CursorStyle, DispatchPhase, Div,
    ElementId, Entity, FocusHandle, KeyDownEvent, MouseButton, MouseDownEvent, MouseMoveEvent,
    MouseUpEvent, Pixels, Rems, Role, SharedString, Window,
};

use crate::a11y::{A11y, Announce};
use crate::element_id::scoped;
use crate::elements::separator::Separator;
use crate::layout::{h_stack, v_stack};
use crate::theme::{ActiveTheme, ControlMetrics, ControlSize, Themeable};
use crate::traits::accessible::Accessible;
use crate::traits::control_sized::ControlSized;
use crate::traits::orientable::{Orientable, Orientation};

/// What the caller is handed when the divider moves: the new ratio.
type ResizeHandler = Rc<dyn Fn(&f32, &mut Window, &mut App) + 'static>;

/// Every dimension this element needs, derived from a rung of the shared
/// control size scale rather than named.
///
/// It lives here rather than in `src/theme/control.rs` because of that file's
/// "What belongs here" note: a shape specific to one control stays in that
/// control's file, keyed off its rung. Nothing else in this crate draws a drag
/// band.
///
/// `pub(crate)` on purpose. It is a separate type because the maths is easier
/// to check without a window, not because a caller needs it; promoting it to
/// `pub` later is additive, and un-publishing it would not be.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct SplitterMetrics {
    /// The interactive band's thickness across the split axis. Twice the
    /// rung's `gap`, which is 6/8/12px — wide enough to hit, narrow enough not
    /// to swallow a click meant for a pane.
    pub(crate) band: Rems,
    /// The thickness of the accent line drawn while the divider is being
    /// dragged. One `gap`.
    pub(crate) highlight: Rems,
    /// How far one arrow key moves the boundary. One rung height.
    pub(crate) arrow_step: Rems,
    /// The floor under each pane when the caller names none. Three rung
    /// heights — enough to keep a pane's content addressable rather than a
    /// number picked to look right.
    pub(crate) default_floor: Rems,
}

impl SplitterMetrics {
    /// Derive every dimension from `control`.
    pub(crate) fn for_rung(control: ControlMetrics) -> Self {
        Self {
            band: control.gap * 2.0,
            highlight: control.gap,
            arrow_step: control.height,
            default_floor: control.height * 3.0,
        }
    }
}

/// The split maths, with no window in it.
///
/// Everything that decides *where the boundary goes* is here: the space the
/// ratio actually divides, the range the floors leave, and the two conversions
/// between a position in pixels and a ratio. It is a separate type because
/// this is the part that is easy to get wrong, and a pure type can be checked
/// exhaustively without drawing anything.
///
/// `pub(crate)` for the same reason as [`SplitterMetrics`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct SplitterGeometry {
    length: Pixels,
    band: Pixels,
    min_start: Pixels,
    min_end: Pixels,
}

impl SplitterGeometry {
    /// A container `length` long across the split axis, with a `band`-thick
    /// divider and a floor under each pane. Negative inputs are treated as
    /// zero; a resize really does hand out negative sizes in passing.
    pub(crate) fn new(length: Pixels, band: Pixels, min_start: Pixels, min_end: Pixels) -> Self {
        Self {
            length: length.max(px(0.)),
            band: band.max(px(0.)),
            min_start: min_start.max(px(0.)),
            min_end: min_end.max(px(0.)),
        }
    }

    /// The space the ratio divides.
    ///
    /// The band is a real flex child rather than an overlay, so it is *not*
    /// part of what gets divided — which is exactly why a ratio of `0.5` puts
    /// the same number of pixels on each side.
    pub(crate) fn usable(&self) -> Pixels {
        (self.length - self.band).max(px(0.))
    }

    /// The lowest and highest ratio the two floors leave.
    ///
    /// When they cannot both be satisfied — a container too small for
    /// `min_start + min_end` — the range collapses to the single ratio that
    /// splits the usable space *in proportion to the floors*. That is a
    /// decision rather than a fallback: honouring one floor outright would
    /// strand whichever pane was named second, and a proportional split at
    /// least degrades the two together.
    pub(crate) fn range(&self) -> (f32, f32) {
        let usable = f32::from(self.usable());

        let (low, high) = if usable > 0. {
            (
                (f32::from(self.min_start) / usable).clamp(0., 1.),
                (1. - f32::from(self.min_end) / usable).clamp(0., 1.),
            )
        } else {
            // No space at all is the extreme case of unsatisfiable, and
            // falls through to the proportional answer below.
            (1., 0.)
        };

        if low <= high {
            (low, high)
        } else {
            let point = self.proportional();
            (point, point)
        }
    }

    /// The ratio that divides the usable space in proportion to the two
    /// floors. With no floors at all there is nothing to be proportional to,
    /// so it is the middle.
    fn proportional(&self) -> f32 {
        let total = f32::from(self.min_start) + f32::from(self.min_end);
        if total > 0. {
            f32::from(self.min_start) / total
        } else {
            0.5
        }
    }

    /// `ratio`, held inside the range the floors leave.
    ///
    /// A non-finite ratio — a caller dividing by a zero it did not expect —
    /// resolves to the low end rather than propagating a `NaN` into layout.
    pub(crate) fn clamp(&self, ratio: f32) -> f32 {
        let (low, high) = self.range();
        if ratio.is_nan() {
            return low;
        }
        ratio.clamp(low, high)
    }

    /// The ratio a boundary `offset` from the container's leading edge means,
    /// clamped. Overshooting either end stops at the floor rather than
    /// snapping back.
    pub(crate) fn ratio_at(&self, offset: Pixels) -> f32 {
        let usable = f32::from(self.usable());
        if usable <= 0. {
            return self.clamp(self.proportional());
        }
        self.clamp(f32::from(offset) / usable)
    }

    /// One press of an arrow key, as a fraction of the usable space.
    pub(crate) fn step_ratio(&self, step: Pixels) -> f32 {
        let usable = f32::from(self.usable());
        if usable <= 0. {
            return 0.;
        }
        f32::from(step) / usable
    }
}

/// What the element keeps between frames, which is only the things that are
/// not layout: where the container was measured, and — while a drag is live —
/// how far the boundary sat from the pointer when it was grabbed.
struct SplitterState {
    /// Filled in by the canvas during paint, so it lands on the frame *after*
    /// the first. Until then the element draws the caller's ratio unclamped
    /// and announces a nominal range; see [`Splitter::a11y`].
    container: Option<Bounds<Pixels>>,
    /// The distance from the pointer to the boundary at mouse-down. Without
    /// it the divider jumps to centre itself under the pointer on the first
    /// pixel of movement.
    grab: Option<Pixels>,
    focus_handle: FocusHandle,
}

impl SplitterState {
    fn new(cx: &mut Context<Self>) -> Self {
        Self {
            container: None,
            grab: None,
            focus_handle: cx.focus_handle(),
        }
    }
}

/// A divider between two panes that a pointer can drag and a keyboard can
/// move.
///
/// See the [module docs](self) for what this deliberately is not, and for
/// where the ratio lives.
#[derive(IntoElement)]
pub struct Splitter {
    id: ElementId,
    name: SharedString,
    ratio: f32,
    orientation: Orientation,
    size: ControlSize,
    start: Option<AnyElement>,
    end: Option<AnyElement>,
    min_start: Option<Pixels>,
    min_end: Option<Pixels>,
    on_resize: Option<ResizeHandler>,
}

/// A divider between two panes.
///
/// `name` is required rather than a builder: a divider has no visible text to
/// borrow an accessible name from. `ratio` is the fraction of the divisible
/// space the *start* pane takes, and it stays the caller's — see the
/// [module docs](self).
pub fn splitter(id: impl Into<ElementId>, name: impl Into<SharedString>, ratio: f32) -> Splitter {
    Splitter::new(id, name, ratio)
}

impl Splitter {
    /// A splitter with a vertical divider — two panes side by side.
    pub fn new(id: impl Into<ElementId>, name: impl Into<SharedString>, ratio: f32) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            ratio,
            orientation: Orientation::Vertical,
            size: ControlSize::default(),
            start: None,
            end: None,
            min_start: None,
            min_end: None,
            on_resize: None,
        }
    }

    /// The pane before the divider — left of a vertical one, above a
    /// horizontal one.
    pub fn start(mut self, pane: impl IntoElement) -> Self {
        self.start = Some(pane.into_any_element());
        self
    }

    /// The pane after the divider.
    pub fn end(mut self, pane: impl IntoElement) -> Self {
        self.end = Some(pane.into_any_element());
        self
    }

    /// The floor under the start pane. Defaults to three rung heights.
    pub fn min_start(mut self, min: impl Into<Pixels>) -> Self {
        self.min_start = Some(min.into());
        self
    }

    /// The floor under the end pane. Defaults to three rung heights.
    pub fn min_end(mut self, min: impl Into<Pixels>) -> Self {
        self.min_end = Some(min.into());
        self
    }

    /// Called with the new ratio whenever the divider moves. The element does
    /// not store it — a splitter with no handler is a fixed divider.
    pub fn on_resize(mut self, handler: impl Fn(&f32, &mut Window, &mut App) + 'static) -> Self {
        self.on_resize = Some(Rc::new(handler));
        self
    }

    /// Whether the divider itself runs top-to-bottom, i.e. whether the panes
    /// sit side by side.
    fn is_vertical(&self) -> bool {
        matches!(self.orientation, Orientation::Vertical)
    }

    /// The announcement, given whatever geometry is known.
    ///
    /// With none — the first frame, before the canvas has measured anything —
    /// the answer is 0–100 with a nominal 1% step. That is deliberately a
    /// usable answer rather than no value at all: a splitter that announces
    /// nothing for a frame reads to assistive technology as a splitter with no
    /// position, which is worse than one whose step is briefly approximate.
    fn announcement(&self, geometry: Option<SplitterGeometry>, step: Pixels) -> A11y {
        let (position, low, high, step) = match geometry {
            Some(geometry) => {
                let (low, high) = geometry.range();
                (
                    geometry.clamp(self.ratio),
                    low,
                    high,
                    geometry.step_ratio(step),
                )
            }
            None => (self.ratio.clamp(0., 1.), 0., 1., 0.01),
        };

        A11y::new(Role::Splitter)
            .name(self.name.clone())
            .orientation(match self.orientation {
                Orientation::Vertical => gpui::Orientation::Vertical,
                Orientation::Horizontal => gpui::Orientation::Horizontal,
            })
            .number_value(
                f64::from(position) * 100.,
                f64::from(low) * 100.,
                f64::from(high) * 100.,
                f64::from(step) * 100.,
            )
    }
}

impl Accessible for Splitter {
    fn a11y(&self) -> A11y {
        self.announcement(None, px(0.))
    }
}

impl Orientable for Splitter {
    fn orientation(mut self, orientation: Orientation) -> Self {
        self.orientation = orientation;
        self
    }
}

impl ControlSized for Splitter {
    fn control_size(mut self, size: ControlSize) -> Self {
        self.size = size;
        self
    }
}

/// Which key, if any, moves the divider, and where to.
///
/// Split out of the element so the WAI-ARIA contract can be checked without a
/// window, a focus handle or a dispatch tree. `None` means "not ours" — the
/// cross-axis arrows are left alone so a splitter inside a scrolling pane does
/// not eat the scroll keys.
fn ratio_for_key(
    key: &str,
    orientation: Orientation,
    current: f32,
    geometry: SplitterGeometry,
    step: Pixels,
) -> Option<f32> {
    let (back, forward) = match orientation {
        Orientation::Vertical => ("left", "right"),
        Orientation::Horizontal => ("up", "down"),
    };
    let (low, high) = geometry.range();
    let step = geometry.step_ratio(step);

    let next = if key == back {
        geometry.clamp(current - step)
    } else if key == forward {
        geometry.clamp(current + step)
    } else if key == "home" {
        low
    } else if key == "end" {
        high
    } else {
        return None;
    };

    Some(next)
}

/// Hand the caller a ratio, but only when it is really a different one.
fn emit(
    handler: &Option<ResizeHandler>,
    next: f32,
    current: f32,
    window: &mut Window,
    cx: &mut App,
) {
    if (next - current).abs() <= f32::EPSILON {
        return;
    }
    if let Some(handler) = handler {
        handler(&next, window, cx);
    }
}

/// A pane: it takes `grow` of the divisible space and no more, whatever its
/// own content would rather be.
///
/// `flex_basis(0)` plus a zero minimum is what makes that true — without them
/// a pane's intrinsic width outvotes the ratio, and `overflow_hidden` is what
/// stops content that does not fit from pushing the boundary back.
fn pane(child: Option<AnyElement>, grow: f32) -> Div {
    div()
        .flex_grow(grow)
        .flex_basis(px(0.))
        .min_w(px(0.))
        .min_h(px(0.))
        .overflow_hidden()
        .children(child)
}

impl RenderOnce for Splitter {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let state: Entity<SplitterState> =
            window.use_keyed_state(scoped(&self.id, "state"), cx, |_window, cx| {
                SplitterState::new(cx)
            });

        let rem_size = window.rem_size();
        let control = cx.theme().control(self.size);
        let metrics = SplitterMetrics::for_rung(control);
        let band_px = metrics.band.to_pixels(rem_size);
        let step_px = metrics.arrow_step.to_pixels(rem_size);
        let floor = metrics.default_floor.to_pixels(rem_size);
        let vertical = self.is_vertical();

        let (container_bounds, dragging, focus_handle) = {
            let state = state.read(cx);
            (
                state.container,
                state.grab.is_some(),
                state.focus_handle.clone(),
            )
        };

        // The geometry of the frame being drawn, which exists only once the
        // canvas below has measured a frame. Everything downstream has an
        // answer for `None`.
        let geometry = container_bounds.map(|bounds| {
            SplitterGeometry::new(
                if vertical {
                    bounds.size.width
                } else {
                    bounds.size.height
                },
                band_px,
                self.min_start.unwrap_or(floor),
                self.min_end.unwrap_or(floor),
            )
        });

        // Clamped for drawing as well as for emitting — but never written
        // back. The caller's ratio stays the caller's even in a window
        // currently too small to honour it, so widening the window restores
        // the layout instead of having quietly destroyed it.
        let drawn = match geometry {
            Some(geometry) => geometry.clamp(self.ratio),
            None => self.ratio.clamp(0., 1.),
        };

        // Where the band's leading edge sits inside the container, which is
        // what turns a pointer position into a grab offset.
        let boundary = geometry.map(|geometry| geometry.usable() * drawn);

        let a11y = self.announcement(geometry, step_px);
        let theme = cx.theme();
        let (line_color, hover_color) = (theme.border_subtle(), theme.accent());

        let Splitter {
            id,
            orientation,
            start,
            end,
            on_resize,
            ..
        } = self;

        let band = div()
            .id(scoped(&id, "band"))
            .announce(a11y)
            .tab_index(0)
            .track_focus(&focus_handle)
            .flex_none()
            .flex()
            .items_center()
            .justify_center()
            .map(|band| {
                if vertical {
                    band.w(metrics.band)
                        .h_full()
                        .cursor(CursorStyle::ResizeLeftRight)
                } else {
                    band.h(metrics.band)
                        .w_full()
                        .cursor(CursorStyle::ResizeUpDown)
                }
            })
            .child(if dragging {
                div()
                    .bg(hover_color)
                    .map(|line| {
                        if vertical {
                            line.w(metrics.highlight).h_full()
                        } else {
                            line.h(metrics.highlight).w_full()
                        }
                    })
                    .into_any_element()
            } else {
                // The resting line is `Separator`'s hairline, not a second
                // one — `docs/issues/resizable.md` asks for exactly that.
                Separator::new().orientation(orientation).into_any_element()
            })
            .hover(|band| band.bg(line_color))
            .on_mouse_down(MouseButton::Left, {
                let state = state.clone();
                let focus_handle = focus_handle.clone();
                move |event: &MouseDownEvent, window, cx| {
                    let (Some(bounds), Some(boundary)) = (container_bounds, boundary) else {
                        // Nothing measured yet, so there is no boundary to
                        // grab. One frame, and only before anything is drawn.
                        return;
                    };
                    let along = if vertical {
                        bounds.origin.x
                    } else {
                        bounds.origin.y
                    };
                    let pointer = if vertical {
                        event.position.x
                    } else {
                        event.position.y
                    };
                    state.update(cx, |state, cx| {
                        state.grab = Some(along + boundary - pointer);
                        cx.notify();
                    });
                    focus_handle.focus(window, cx);
                    cx.stop_propagation();
                }
            })
            .on_key_down({
                let on_resize = on_resize.clone();
                move |event: &KeyDownEvent, window, cx| {
                    let Some(geometry) = geometry else {
                        return;
                    };
                    let Some(next) = ratio_for_key(
                        event.keystroke.key.as_str(),
                        orientation,
                        drawn,
                        geometry,
                        step_px,
                    ) else {
                        return;
                    };
                    cx.stop_propagation();
                    emit(&on_resize, next, drawn, window, cx);
                }
            });

        // The drag lives on the *window*, not on the band.
        //
        // `div().on_mouse_move` only fires while its hitbox is hovered, so a
        // drag built on it dies the moment the pointer leaves the band — which
        // is precisely when a drag is interesting. `Window::on_mouse_event`
        // has no such limit, and it fills the frame currently being painted,
        // so it cannot be called from `render`. A `canvas` paint closure is
        // the hook, the same one `src/elements/input.rs` uses. The canvas also
        // measures the container, since it is already the thing that sees real
        // bounds.
        let measure = canvas(move |bounds, _window, _cx| bounds, {
            let state = state.clone();
            move |bounds, _, window, cx| {
                if state.read(cx).container != Some(bounds) {
                    state.update(cx, |state, cx| {
                        state.container = Some(bounds);
                        cx.notify();
                    });
                }

                let Some(geometry) = geometry else {
                    // No geometry to drag against until the next frame.
                    return;
                };
                let origin = if vertical {
                    bounds.origin.x
                } else {
                    bounds.origin.y
                };

                window.on_mouse_event({
                    let state = state.clone();
                    let on_resize = on_resize.clone();
                    move |event: &MouseMoveEvent, phase, window, cx| {
                        if phase != DispatchPhase::Bubble {
                            return;
                        }
                        let Some(grab) = state.read(cx).grab else {
                            return;
                        };
                        if !event.dragging() {
                            // A release the window never saw — the pointer
                            // left the window with the button down, or
                            // another handler swallowed the mouse-up.
                            state.update(cx, |state, cx| {
                                state.grab = None;
                                cx.notify();
                            });
                            return;
                        }

                        let pointer = if vertical {
                            event.position.x
                        } else {
                            event.position.y
                        };
                        let next = geometry.ratio_at(pointer + grab - origin);
                        emit(&on_resize, next, drawn, window, cx);
                    }
                });

                window.on_mouse_event({
                    let state = state.clone();
                    move |event: &MouseUpEvent, phase, _window, cx| {
                        if phase != DispatchPhase::Bubble
                            || event.button != MouseButton::Left
                            || state.read(cx).grab.is_none()
                        {
                            return;
                        }
                        state.update(cx, |state, cx| {
                            state.grab = None;
                            cx.notify();
                        });
                    }
                });
            }
        })
        .absolute()
        .size_full();

        let base = if vertical { h_stack() } else { v_stack() };

        base.relative()
            .size_full()
            .child(pane(start, drawn))
            .child(band)
            .child(pane(end, 1. - drawn))
            .child(measure)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::a11y::role_requires_a_name;
    use crate::theme::ControlScale;
    use gpui::{size, Modifiers, Point, Render, Size, TestAppContext, VisualTestContext};
    use std::cell::RefCell;

    // --- the split maths, with no window ---

    fn geometry(length: f32, band: f32, min_start: f32, min_end: f32) -> SplitterGeometry {
        SplitterGeometry::new(px(length), px(band), px(min_start), px(min_end))
    }

    /// The band is a real child rather than an overlay, so it is not part of
    /// what the ratio divides. That is the whole reason `0.5` is exactly half
    /// and half.
    #[test]
    fn the_band_is_taken_out_before_the_ratio_divides_anything() {
        let split = geometry(408., 8., 0., 0.);

        assert_eq!(split.usable(), px(400.));
        assert_eq!(split.usable() * split.clamp(0.5), px(200.));
    }

    #[test]
    fn the_floors_bound_the_range_at_both_ends() {
        let split = geometry(408., 8., 100., 50.);
        let (low, high) = split.range();

        assert_eq!(low, 0.25, "a 100px floor in 400px of usable space");
        assert_eq!(high, 0.875, "a 50px floor in 400px of usable space");
    }

    #[test]
    fn clamping_holds_a_ratio_inside_the_floors() {
        let split = geometry(408., 8., 100., 50.);

        assert_eq!(split.clamp(0.5), 0.5);
        assert_eq!(split.clamp(0.0), 0.25);
        assert_eq!(split.clamp(1.0), 0.875);
        assert_eq!(split.clamp(-4.0), 0.25);
        assert_eq!(split.clamp(f32::NAN), 0.25);
    }

    #[test]
    fn a_position_reads_back_as_the_ratio_it_means() {
        let split = geometry(408., 8., 0., 0.);

        assert_eq!(split.ratio_at(px(100.)), 0.25);
        assert_eq!(split.ratio_at(px(300.)), 0.75);
    }

    /// A drag that runs past the end of the container stops at the floor
    /// rather than overshooting and snapping back on release.
    #[test]
    fn a_drag_past_either_end_stops_at_the_floor() {
        let split = geometry(408., 8., 100., 50.);

        assert_eq!(split.ratio_at(px(-500.)), 0.25);
        assert_eq!(split.ratio_at(px(5_000.)), 0.875);
    }

    /// Both floors cannot always be honoured, and the policy is a decision:
    /// split what is left in proportion to the floors, so neither pane is
    /// stranded outright.
    #[test]
    fn unsatisfiable_floors_collapse_to_a_proportional_split() {
        // 100px of usable space, and 300px of floors asked for.
        let split = geometry(108., 8., 200., 100.);
        let (low, high) = split.range();

        assert_eq!(low, high, "the range has to collapse to a single ratio");
        assert!(
            (low - 2. / 3.).abs() < 1e-6,
            "a 200/100 pair of floors splits two thirds to the start pane, not {low}"
        );
        assert_eq!(split.clamp(0.1), low);
        assert_eq!(split.clamp(0.9), low);
    }

    /// A window narrower than the band is a real state during a resize, and
    /// `f32::clamp` panics when its bounds cross.
    #[test]
    fn a_container_smaller_than_the_band_does_not_panic() {
        for split in [geometry(0., 8., 40., 40.), geometry(4., 8., 40., 10.)] {
            assert_eq!(split.usable(), px(0.));
            let (low, high) = split.range();
            assert_eq!(low, high);
            assert!(split.clamp(0.5).is_finite());
            assert!(split.ratio_at(px(20.)).is_finite());
            assert_eq!(split.step_ratio(px(20.)), 0.);
        }
    }

    #[test]
    fn with_no_floors_the_whole_range_is_available() {
        let split = geometry(408., 8., 0., 0.);

        assert_eq!(split.range(), (0., 1.));
        assert_eq!(split.clamp(0.), 0.);
        assert_eq!(split.clamp(1.), 1.);
    }

    #[test]
    fn one_arrow_press_is_a_fraction_of_the_usable_space() {
        let split = geometry(408., 8., 0., 0.);

        assert_eq!(split.step_ratio(px(20.)), 0.05);
    }

    /// A negative size is what a container hands out mid-resize, and it must
    /// not become a negative floor.
    #[test]
    fn negative_inputs_are_treated_as_zero() {
        let split = SplitterGeometry::new(px(-10.), px(-2.), px(-30.), px(-30.));

        assert_eq!(split.usable(), px(0.));
        assert_eq!(
            split.clamp(0.5),
            0.5,
            "no floors left to be proportional to"
        );
    }

    // --- the metrics, off the rung ---

    #[test]
    fn every_dimension_comes_off_the_rung() {
        for size in ControlSize::ALL {
            let control = ControlScale::default().metrics(size);
            let metrics = SplitterMetrics::for_rung(control);

            assert_eq!(metrics.band.0, control.gap.0 * 2.0, "{}", size.name());
            assert_eq!(metrics.highlight.0, control.gap.0, "{}", size.name());
            assert_eq!(metrics.arrow_step.0, control.height.0, "{}", size.name());
            assert_eq!(
                metrics.default_floor.0,
                control.height.0 * 3.0,
                "{}",
                size.name()
            );
        }
    }

    /// The point of the band: a 1px divider is unhittable, so the interactive
    /// area has to be several times the line it draws. 6/8/12px at a 16px
    /// root.
    #[test]
    fn the_band_is_much_wider_than_the_hairline_it_draws() {
        let bands: Vec<f32> = ControlSize::ALL
            .into_iter()
            .map(|size| {
                SplitterMetrics::for_rung(ControlScale::default().metrics(size))
                    .band
                    .0
                    * 16.
            })
            .collect();

        assert_eq!(bands, vec![6., 8., 12.]);
    }

    #[test]
    fn the_metrics_grow_with_the_rung() {
        let metrics: Vec<SplitterMetrics> = ControlSize::ALL
            .into_iter()
            .map(|size| SplitterMetrics::for_rung(ControlScale::default().metrics(size)))
            .collect();

        for pair in metrics.windows(2) {
            assert!(pair[0].band.0 < pair[1].band.0);
            assert!(pair[0].arrow_step.0 < pair[1].arrow_step.0);
            assert!(pair[0].default_floor.0 < pair[1].default_floor.0);
        }
    }

    /// The floor has to leave a pane worth looking at, or a splitter is a
    /// divider between two slivers.
    #[test]
    fn the_default_floor_is_bigger_than_the_band() {
        for size in ControlSize::ALL {
            let metrics = SplitterMetrics::for_rung(ControlScale::default().metrics(size));
            assert!(
                metrics.default_floor.0 > metrics.band.0,
                "{}: a floor no bigger than the divider is not a floor",
                size.name(),
            );
        }
    }

    // --- the keyboard contract, with no window ---

    fn keyed(key: &str, orientation: Orientation, current: f32) -> Option<f32> {
        ratio_for_key(
            key,
            orientation,
            current,
            geometry(408., 8., 40., 40.),
            px(20.),
        )
    }

    #[test]
    fn the_arrows_on_the_split_axis_move_the_divider_one_step() {
        assert_eq!(keyed("right", Orientation::Vertical, 0.5), Some(0.55));
        assert_eq!(keyed("left", Orientation::Vertical, 0.5), Some(0.45));
        assert_eq!(keyed("down", Orientation::Horizontal, 0.5), Some(0.55));
        assert_eq!(keyed("up", Orientation::Horizontal, 0.5), Some(0.45));
    }

    /// The cross-axis arrows are left alone deliberately: a splitter inside a
    /// scrolling pane must not eat the keys that scroll it.
    #[test]
    fn the_cross_axis_arrows_are_not_this_elements() {
        assert_eq!(keyed("up", Orientation::Vertical, 0.5), None);
        assert_eq!(keyed("down", Orientation::Vertical, 0.5), None);
        assert_eq!(keyed("left", Orientation::Horizontal, 0.5), None);
        assert_eq!(keyed("right", Orientation::Horizontal, 0.5), None);
        assert_eq!(keyed("enter", Orientation::Vertical, 0.5), None);
        assert_eq!(keyed("escape", Orientation::Vertical, 0.5), None);
    }

    #[test]
    fn home_and_end_go_to_the_two_floors() {
        assert_eq!(keyed("home", Orientation::Vertical, 0.5), Some(0.1));
        assert_eq!(keyed("end", Orientation::Vertical, 0.5), Some(0.9));
    }

    #[test]
    fn the_arrows_stop_at_the_floors_too() {
        assert_eq!(keyed("left", Orientation::Vertical, 0.1), Some(0.1));
        assert_eq!(keyed("right", Orientation::Vertical, 0.9), Some(0.9));
    }

    // --- what it announces ---
    //
    // The role sits on the *band*, which is a child, and
    // `a11y::test_support::announced` reads the root element of a `render`. So
    // these build the node by hand from the same `A11y` value the element
    // applies, which is what gpui's own walk would have seen.

    fn node_for(a11y: A11y) -> gpui::accesskit::Node {
        crate::a11y::test_support::announced_element(div().id("band").announce(a11y))
            .node
            .expect("a splitter with an id and a role is a node")
    }

    /// Two `A11y` values cannot be compared field-for-field here: the
    /// percentages come out of an f32 division, and 60% is not exactly `60.0`.
    fn close(actual: Option<f64>, expected: f64, what: &str) {
        let actual = actual.unwrap_or_else(|| panic!("the node reports no {what}"));
        assert!(
            (actual - expected).abs() < 1e-4,
            "{what} was {actual}, expected about {expected}"
        );
    }

    #[test]
    fn a_splitter_announces_a_named_splitter_with_its_position() {
        let split = splitter("panes", "Editor and preview", 0.6);
        let node = node_for(split.announcement(Some(geometry(408., 8., 40., 40.)), px(20.)));

        assert_eq!(node.label(), Some("Editor and preview"));
        close(node.numeric_value(), 60., "position");
        close(node.min_numeric_value(), 10., "minimum");
        close(node.max_numeric_value(), 90., "maximum");
        close(node.numeric_value_step(), 5., "step");
    }

    /// The canvas measures during paint, so the first frame has no geometry.
    /// The documented answer for it is a usable range rather than silence: a
    /// splitter that announces no value at all reads as one with no position.
    #[test]
    fn an_unmeasured_splitter_still_reports_a_range() {
        let split = splitter("panes", "Panes", 0.4);
        assert_eq!(split.a11y().role(), Role::Splitter);

        let node = node_for(split.a11y());
        close(node.numeric_value(), 40., "position");
        close(node.min_numeric_value(), 0., "minimum");
        close(node.max_numeric_value(), 100., "maximum");
        close(node.numeric_value_step(), 1., "step");
    }

    #[test]
    fn the_announced_position_is_clamped_like_the_drawn_one() {
        let split = splitter("panes", "Panes", 5.0);
        let node = node_for(split.announcement(Some(geometry(408., 8., 40., 40.)), px(20.)));

        close(node.numeric_value(), 90., "position");
    }

    #[test]
    fn the_divider_announces_which_way_it_runs() {
        assert_eq!(
            node_for(splitter("panes", "Panes", 0.5).a11y()).orientation(),
            Some(gpui::Orientation::Vertical),
        );
        assert_eq!(
            node_for(splitter("panes", "Panes", 0.5).horizontal().a11y()).orientation(),
            Some(gpui::Orientation::Horizontal),
        );
    }

    /// A divider has no visible text to borrow a name from, so the convention
    /// has to require one — which is why `name` is a constructor argument
    /// rather than a builder.
    #[test]
    fn a_splitter_is_one_of_the_roles_that_must_be_named() {
        assert!(role_requires_a_name(Role::Splitter));
        assert!(A11y::new(Role::Splitter).is_missing_a_required_name());
        assert!(!splitter("panes", "Panes", 0.5)
            .a11y()
            .is_missing_a_required_name());
    }

    // --- drawing, and the drag ---
    //
    // An element with *both* a role and a mouse listener cannot be drawn with
    // `VisualTestContext::draw`: registering a mouse listener reads
    // `Window::current_view`, which is only set while a *view* renders. Hence
    // a real view, copied from `src/elements/sidebar.rs`.

    type Build = Box<dyn Fn(&mut Window, &mut App) -> AnyElement>;

    struct Harness {
        build: Build,
        drawn: Entity<usize>,
    }

    impl Render for Harness {
        fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            self.drawn.update(cx, |count, _| *count += 1);
            (self.build)(window, cx)
        }
    }

    fn draw(
        cx: &mut TestAppContext,
        window_size: Size<Pixels>,
        build: impl Fn(&mut Window, &mut App) -> AnyElement + 'static,
    ) -> &mut VisualTestContext {
        cx.update(crate::theme::init);

        let drawn = cx.update(|cx| cx.new(|_| 0usize));
        let counter = drawn.clone();
        let window = cx.open_window(window_size, move |_window, _cx| Harness {
            build: Box::new(build),
            drawn: counter,
        });

        let cx = VisualTestContext::from_window(*std::ops::Deref::deref(&window), cx).into_mut();
        cx.run_until_parked();

        assert!(
            drawn.read_with(cx, |count, _| *count) > 0,
            "the harness never drew, so this test is checking nothing"
        );
        cx
    }

    /// Every ratio the element emitted, in order.
    type Emitted = Rc<RefCell<Vec<f32>>>;

    /// A splitter filling the window, whose ratio the caller keeps — exactly
    /// the arrangement the module docs describe.
    fn panes(
        ratio: Rc<RefCell<f32>>,
        emitted: Emitted,
        orientation: Orientation,
    ) -> impl Fn(&mut Window, &mut App) -> AnyElement + 'static {
        move |_window, _cx| {
            let ratio_cell = ratio.clone();
            let emitted = emitted.clone();
            splitter("test-split", "Panes", *ratio.borrow())
                .orientation(orientation)
                .min_start(px(40.))
                .min_end(px(40.))
                .start(div().child("start"))
                .end(div().child("end"))
                .on_resize(move |next: &f32, _window, _cx| {
                    *ratio_cell.borrow_mut() = *next;
                    emitted.borrow_mut().push(*next);
                })
                .into_any_element()
        }
    }

    fn scenario(
        cx: &mut TestAppContext,
        orientation: Orientation,
    ) -> (&mut VisualTestContext, Rc<RefCell<f32>>, Emitted) {
        let ratio = Rc::new(RefCell::new(0.5));
        let emitted: Emitted = Rc::new(RefCell::new(Vec::new()));
        let cx = draw(
            cx,
            size(px(408.), px(408.)),
            panes(ratio.clone(), emitted.clone(), orientation),
        );
        // The canvas measures during paint and notifies; the geometry is on
        // the frame after. Without this there is nothing to drag against.
        cx.run_until_parked();
        (cx, ratio, emitted)
    }

    #[gpui::test]
    fn a_side_by_side_splitter_draws(cx: &mut TestAppContext) {
        scenario(cx, Orientation::Vertical);
    }

    #[gpui::test]
    fn a_stacked_splitter_draws(cx: &mut TestAppContext) {
        scenario(cx, Orientation::Horizontal);
    }

    #[gpui::test]
    fn dragging_the_band_moves_the_boundary(cx: &mut TestAppContext) {
        let (cx, ratio, emitted) = scenario(cx, Orientation::Vertical);

        // The window is 408px wide with an 8px band, so 400px is divisible and
        // the band sits at 200..208.
        cx.simulate_mouse_move(Point::new(px(204.), px(50.)), None, Modifiers::none());
        cx.run_until_parked();
        cx.simulate_mouse_down(
            Point::new(px(204.), px(50.)),
            MouseButton::Left,
            Modifiers::none(),
        );
        cx.run_until_parked();
        cx.simulate_mouse_move(
            Point::new(px(304.), px(50.)),
            MouseButton::Left,
            Modifiers::none(),
        );
        cx.run_until_parked();

        assert!(
            !emitted.borrow().is_empty(),
            "the drag emitted nothing at all"
        );
        let moved = *ratio.borrow();
        assert!(
            (moved - 0.75).abs() < 1e-4,
            "a 100px drag across 400px of usable space should land on 0.75, not {moved}"
        );
    }

    /// The grab offset, in the only form a test can see it: a mouse-down
    /// somewhere other than the exact boundary must not move the divider, and
    /// the following drag must be relative to where it was grabbed.
    #[gpui::test]
    fn grabbing_the_band_off_centre_does_not_jump_the_divider(cx: &mut TestAppContext) {
        let (cx, ratio, emitted) = scenario(cx, Orientation::Vertical);

        // 207px is inside the 200..208 band but 7px past its leading edge.
        cx.simulate_mouse_move(Point::new(px(207.), px(50.)), None, Modifiers::none());
        cx.run_until_parked();
        cx.simulate_mouse_down(
            Point::new(px(207.), px(50.)),
            MouseButton::Left,
            Modifiers::none(),
        );
        cx.run_until_parked();

        assert!(
            emitted.borrow().is_empty(),
            "pressing on the band moved the divider before anything was dragged"
        );

        cx.simulate_mouse_move(
            Point::new(px(247.), px(50.)),
            MouseButton::Left,
            Modifiers::none(),
        );
        cx.run_until_parked();

        let moved = *ratio.borrow();
        assert!(
            (moved - 0.6).abs() < 1e-4,
            "a 40px drag from a grab 7px off the boundary lands on exactly 0.6, not {moved} — \
             a tolerance loose enough to swallow those 7px would not see the bug this is for"
        );
    }

    /// The release the window never saw. Without this the divider follows the
    /// pointer forever after the mouse comes back into the window.
    #[gpui::test]
    fn a_move_with_no_button_held_ends_the_drag(cx: &mut TestAppContext) {
        let (cx, ratio, _emitted) = scenario(cx, Orientation::Vertical);

        cx.simulate_mouse_move(Point::new(px(204.), px(50.)), None, Modifiers::none());
        cx.run_until_parked();
        cx.simulate_mouse_down(
            Point::new(px(204.), px(50.)),
            MouseButton::Left,
            Modifiers::none(),
        );
        cx.run_until_parked();

        // No button held: the drag ends here.
        cx.simulate_mouse_move(Point::new(px(304.), px(50.)), None, Modifiers::none());
        cx.run_until_parked();
        let after_release = *ratio.borrow();

        // And a later move, button down, must not resume it.
        cx.simulate_mouse_move(
            Point::new(px(360.), px(50.)),
            MouseButton::Left,
            Modifiers::none(),
        );
        cx.run_until_parked();

        assert_eq!(
            *ratio.borrow(),
            after_release,
            "the drag survived a move with no button held"
        );
    }

    /// A stacked splitter drags on the other axis, which is the half of the
    /// orientation branch the side-by-side tests never touch.
    #[gpui::test]
    fn a_stacked_splitter_drags_vertically(cx: &mut TestAppContext) {
        let (cx, ratio, _emitted) = scenario(cx, Orientation::Horizontal);

        cx.simulate_mouse_move(Point::new(px(50.), px(204.)), None, Modifiers::none());
        cx.run_until_parked();
        cx.simulate_mouse_down(
            Point::new(px(50.), px(204.)),
            MouseButton::Left,
            Modifiers::none(),
        );
        cx.run_until_parked();
        cx.simulate_mouse_move(
            Point::new(px(50.), px(104.)),
            MouseButton::Left,
            Modifiers::none(),
        );
        cx.run_until_parked();

        let moved = *ratio.borrow();
        assert!(
            (moved - 0.25).abs() < 1e-4,
            "a 100px upward drag across 400px should land on 0.25, not {moved}"
        );
    }

    /// The floors are enforced against a real drag, not only in the maths.
    #[gpui::test]
    fn a_drag_past_the_end_stops_at_the_floor(cx: &mut TestAppContext) {
        let (cx, ratio, _emitted) = scenario(cx, Orientation::Vertical);

        cx.simulate_mouse_move(Point::new(px(204.), px(50.)), None, Modifiers::none());
        cx.run_until_parked();
        cx.simulate_mouse_down(
            Point::new(px(204.), px(50.)),
            MouseButton::Left,
            Modifiers::none(),
        );
        cx.run_until_parked();
        cx.simulate_mouse_move(
            Point::new(px(5_000.), px(50.)),
            MouseButton::Left,
            Modifiers::none(),
        );
        cx.run_until_parked();

        let moved = *ratio.borrow();
        assert!(
            (moved - 0.9).abs() < 1e-4,
            "a 40px floor in 400px leaves 0.9 as the maximum, not {moved}"
        );
    }

    /// A splitter with no handler is a fixed divider — the element stores no
    /// ratio, so there is nothing for a drag to change.
    #[gpui::test]
    fn a_splitter_with_no_handler_does_not_move(cx: &mut TestAppContext) {
        let cx = draw(cx, size(px(408.), px(408.)), |_window, _cx| {
            splitter("fixed-split", "Panes", 0.5)
                .start(div().child("start"))
                .end(div().child("end"))
                .into_any_element()
        });
        cx.run_until_parked();

        cx.simulate_mouse_move(Point::new(px(204.), px(50.)), None, Modifiers::none());
        cx.run_until_parked();
        cx.simulate_mouse_down(
            Point::new(px(204.), px(50.)),
            MouseButton::Left,
            Modifiers::none(),
        );
        cx.simulate_mouse_move(
            Point::new(px(304.), px(50.)),
            MouseButton::Left,
            Modifiers::none(),
        );
        cx.run_until_parked();
    }
}
