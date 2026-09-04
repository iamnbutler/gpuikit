//! Slider component for gpuikit
//!
//! # The drag lives on the window, not on the track
//!
//! `div().on_mouse_move` and `div().on_mouse_up` only fire while their hitbox
//! is hovered, so a drag built on them dies the moment the pointer leaves the
//! track — which is precisely when a drag is interesting. The movement and the
//! release are registered on the *window* instead, via
//! `Window::on_mouse_event`; only the press stays on the track, because a
//! press always starts there.
//!
//! `Window::on_mouse_event` fills the frame *currently being painted* and
//! asserts the paint phase, so it cannot be called from `render`. The hook is
//! the `canvas` paint closure that already measures the track — the same one
//! `src/elements/splitter.rs` and `src/elements/input.rs` use. The handlers
//! are registered on every paint rather than only while a drag is live: the
//! frame that has to carry a drag was painted *before* the mouse went down, so
//! a conditional registration would arrive one frame late. They guard on
//! `is_dragging` internally instead.
//!
//! `Window::capture_pointer` would be the textbook answer and is not usable
//! here: it takes a `HitboxId`, which only exists inside a custom `Element`'s
//! prepaint, and nothing `InteractiveElement` exposes can reach one.

use crate::element_id::scoped;
use crate::layout::{h_stack, v_stack};
use crate::theme::{ActiveTheme, ControlSize, Themeable};
use crate::traits::control_sized::ControlSized;
use crate::traits::disableable::Disableable;
use crate::traits::labelable::Labelable;
use crate::utils::element_manager::ElementManagerExt;
use gpui::{
    App, Bounds, Context, DispatchPhase, ElementId, EventEmitter, IntoElement, MouseButton,
    MouseDownEvent, MouseMoveEvent, MouseUpEvent, ParentElement, Pixels, Point, Rems, Render,
    SharedString, Styled, Window, canvas, div, prelude::*, px,
};
use std::ops::RangeInclusive;

/// Event emitted when the slider value changes
pub struct SliderChanged {
    pub value: f32,
}

/// A slider component for selecting numeric values within a range
pub struct Slider {
    id: ElementId,
    label: Option<SharedString>,
    value: f32,
    range: RangeInclusive<f32>,
    step: Option<f32>,
    is_dragging: bool,
    track_bounds: Option<Bounds<Pixels>>,
    show_value: bool,
    disabled: bool,
    size: ControlSize,
}

impl EventEmitter<SliderChanged> for Slider {}

impl Slider {
    pub fn new(id: impl Into<ElementId>, value: f32, range: RangeInclusive<f32>) -> Self {
        Self {
            id: id.into(),
            label: None,
            value: value.clamp(*range.start(), *range.end()),
            range,
            step: None,
            is_dragging: false,
            track_bounds: None,
            show_value: true,
            disabled: false,
            size: ControlSize::default(),
        }
    }

    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn step(mut self, step: f32) -> Self {
        self.step = Some(step);
        self
    }

    pub fn show_value(mut self, show: bool) -> Self {
        self.show_value = show;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn value(&self) -> f32 {
        self.value
    }

    pub fn set_value(&mut self, value: f32, cx: &mut Context<Self>) {
        let clamped = value.clamp(*self.range.start(), *self.range.end());
        let new_value = if let Some(step) = self.step {
            let steps = ((clamped - self.range.start()) / step).round();
            (self.range.start() + steps * step).clamp(*self.range.start(), *self.range.end())
        } else {
            clamped
        };

        if (new_value - self.value).abs() > f32::EPSILON {
            self.value = new_value;
            cx.emit(SliderChanged { value: self.value });
            cx.notify();
        }
    }

    /// The thumb's diameter — the one place it is read.
    ///
    /// It is the rung's ink, the same quantity a checkbox's box and a switch's
    /// track are: the thumb is what the slider *fills* its row with, not the
    /// row. `render` draws the thumb at this and `value_from_position` insets
    /// the usable track by half of it at each end, so the drawn thumb and the
    /// mapping are the same length by construction rather than by two literals
    /// agreeing. They only agreed at gpui's default 16px rem before: the inset
    /// was a hardcoded `px(6.)` while the thumb was `rems(0.75)`, which skewed
    /// the mapping at any other rem size.
    ///
    /// **Both `render` and `value_from_position` must keep reading this.** If
    /// one of them goes back to a literal the two can disagree again, and
    /// nothing in the test module catches it — the tests exercise the mapping
    /// only, never the painted thumb's bounds.
    fn thumb_size(&self, cx: &App) -> Rems {
        cx.theme().control(self.size).ink
    }

    fn value_from_position(
        &self,
        position: Point<Pixels>,
        rem_size: Pixels,
        thumb_size: Rems,
    ) -> f32 {
        let Some(bounds) = self.track_bounds else {
            return self.value;
        };

        let thumb_radius = thumb_size.to_pixels(rem_size) / 2.;
        let usable_width = bounds.size.width - thumb_radius * 2.;
        let relative_x = (position.x - bounds.origin.x - thumb_radius).max(px(0.));
        let percentage = (relative_x / usable_width).clamp(0., 1.);

        let range_size = self.range.end() - self.range.start();
        self.range.start() + percentage * range_size
    }

    fn percentage(&self) -> f32 {
        let range_size = self.range.end() - self.range.start();
        if range_size == 0. {
            return 0.;
        }
        (self.value - self.range.start()) / range_size
    }

    fn on_mouse_down(
        &mut self,
        event: &MouseDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.disabled {
            return;
        }
        self.is_dragging = true;
        // `set_value` is silent when the value does not change, which is
        // exactly what pressing on the thumb does, so this is what puts the
        // dragging border on screen for such a press. Pinned by
        // `a_press_that_does_not_move_the_value_still_repaints`, which has to
        // call this handler directly: gpui's `div` refreshes the window on any
        // mouse-down over an id'd hitbox, so a simulated press repaints either
        // way and cannot tell the two apart.
        cx.notify();
        let thumb_size = self.thumb_size(cx);
        let new_value = self.value_from_position(event.position, window.rem_size(), thumb_size);
        self.set_value(new_value, cx);
    }

    /// The release. Called from the window handler, so it also runs for a
    /// mouse-up the track never saw.
    fn end_drag(&mut self, cx: &mut Context<Self>) {
        if !self.is_dragging {
            return;
        }
        self.is_dragging = false;
        cx.notify();
    }

    /// The movement, from the window rather than the track.
    fn on_drag_move(
        &mut self,
        event: &MouseMoveEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if !self.is_dragging {
            return;
        }
        // `dragging()` is `pressed_button == Some(Left)`, so this also ends the
        // drag when a second button goes down mid-drag — the same behaviour as
        // `Splitter`. Otherwise it is the release the window never saw: the
        // pointer left the window with the button held, or another handler
        // swallowed the mouse-up. `disabled` is checked here too, because the
        // window handlers are registered whether or not the slider is enabled.
        if !event.dragging() || self.disabled {
            self.end_drag(cx);
            return;
        }
        let thumb_size = self.thumb_size(cx);
        let new_value = self.value_from_position(event.position, window.rem_size(), thumb_size);
        self.set_value(new_value, cx);
    }

    fn display_value(&self) -> String {
        if self.step.is_some_and(|s| s >= 1.) {
            format!("{}", self.value.round() as i32)
        } else {
            format!("{:.1}", self.value)
        }
    }
}

impl Render for Slider {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let metrics = theme.control(self.size);
        let percentage = self.percentage();
        let label = self.label.clone();
        let display_value = self.display_value();
        let is_dragging = self.is_dragging;
        let show_value = self.show_value;
        let disabled = self.disabled;

        // Shapes specific to this control, derived from the rung rather than
        // named here: the thumb is the rung's ink, and the rail it slides on
        // is a quarter of that. Both used to be literals, so a slider matched
        // the button beside it on no rung and moved with the scale on none.
        let thumb_size = metrics.ink;
        let track_height = metrics.ink / 4.;
        // The thumb is centred in the rung by hand: it is absolutely
        // positioned, so `items_center` on the row does not reach it.
        let thumb_top = (metrics.height - metrics.ink) / 2.;

        let track_color = theme.surface_secondary();
        let fill_color = if disabled {
            theme.fg_disabled()
        } else {
            theme.accent()
        };
        let thumb_color = theme.fg();
        let thumb_border = if is_dragging {
            theme.accent()
        } else {
            theme.border_secondary()
        };

        v_stack()
            .id(self.id.clone())
            .w_full()
            .gap(metrics.gap)
            .when(label.is_some() || show_value, |this| {
                this.child(
                    h_stack()
                        .justify_between()
                        .text_xs()
                        .when_some(label, |this, label| {
                            this.child(
                                div()
                                    .text_color(if disabled {
                                        theme.fg_disabled()
                                    } else {
                                        theme.fg_muted()
                                    })
                                    .child(label),
                            )
                        })
                        .when(show_value, |this| {
                            this.child(
                                div()
                                    .text_color(if disabled {
                                        theme.fg_disabled()
                                    } else {
                                        theme.fg()
                                    })
                                    .child(display_value),
                            )
                        }),
                )
            })
            .child(
                div()
                    // Was unique only because it sits under the slider's own
                    // `.id(self.id)`; it derives from that id directly now.
                    .id(scoped(&self.id, "track"))
                    .relative()
                    .h(metrics.height)
                    .w_full()
                    .flex()
                    .items_center()
                    .when(!disabled, |this| {
                        // Only the press: a press always starts on the track.
                        // The movement and the release are on the window, from
                        // the canvas below.
                        this.cursor_pointer()
                            .on_mouse_down(MouseButton::Left, cx.listener(Self::on_mouse_down))
                    })
                    .when(disabled, |this| this.cursor_not_allowed().opacity(0.65))
                    .child(
                        div()
                            .absolute()
                            .left(thumb_size / 2.)
                            .right(thumb_size / 2.)
                            .h(track_height)
                            .bg(track_color)
                            .rounded(track_height / 2.)
                            .child(
                                div()
                                    .h_full()
                                    .w(gpui::relative(percentage))
                                    .bg(fill_color)
                                    .rounded(track_height / 2.),
                            ),
                    )
                    .child(
                        canvas(move |bounds, _, _cx| bounds, {
                            let entity = cx.entity().clone();
                            move |bounds, _, window, cx| {
                                entity.update(cx, |this, _cx| {
                                    this.track_bounds = Some(bounds);
                                });

                                // Registered on every paint, live drag or not —
                                // see the module docs.
                                window.on_mouse_event({
                                    let entity = entity.clone();
                                    move |event: &MouseMoveEvent, phase, window, cx| {
                                        if phase != DispatchPhase::Bubble {
                                            return;
                                        }
                                        entity.update(cx, |this, cx| {
                                            this.on_drag_move(event, window, cx)
                                        });
                                    }
                                });

                                window.on_mouse_event({
                                    let entity = entity.clone();
                                    move |event: &MouseUpEvent, phase, _window, cx| {
                                        if phase != DispatchPhase::Bubble
                                            || event.button != MouseButton::Left
                                        {
                                            return;
                                        }
                                        entity.update(cx, |this, cx| this.end_drag(cx));
                                    }
                                });
                            }
                        })
                        .absolute()
                        .size_full(),
                    )
                    .child(
                        div()
                            .absolute()
                            .left(gpui::relative(percentage))
                            .top(thumb_top)
                            .size(thumb_size)
                            .bg(thumb_color)
                            .rounded_full()
                            .border_1()
                            .border_color(thumb_border)
                            .shadow_sm(),
                    ),
            )
    }
}

/// Convenience function to create a slider builder
pub fn slider(id: impl Into<ElementId>, value: f32, range: RangeInclusive<f32>) -> Slider {
    Slider::new(id, value, range)
}

/// Convenience function to create a slider with auto-generated ID
pub fn slider_auto(cx: &App, value: f32, range: RangeInclusive<f32>) -> Slider {
    Slider::new(cx.next_id_named("slider"), value, range)
}

impl Disableable for Slider {
    fn is_disabled(&self) -> bool {
        self.disabled
    }

    fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

impl Labelable for Slider {
    fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }
}

impl ControlSized for Slider {
    fn control_size(mut self, size: ControlSize) -> Self {
        self.size = size;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::{Entity, Modifiers, TestAppContext, VisualTestContext, size};

    // A `Slider` cannot be drawn with `VisualTestContext::draw`: registering a
    // mouse listener reads `Window::current_view`, which is only set while a
    // *view* renders. Hence a real view, of the kind `src/elements/sidebar.rs`
    // uses. It also supplies the narrow wrapper these tests need — the wrapper
    // has to be narrower than the window, or "outside the track" and "outside
    // the window" are the same point and the tests prove less.

    /// The track runs from x=100 to x=300 inside a 400px window.
    const TRACK_LEFT: f32 = 100.;
    const TRACK_WIDTH: f32 = 200.;
    /// `value_from_position` insets the track by the thumb radius at each end.
    /// The thumb is the `Medium` rung's ink, `rems(0.875)`.
    const THUMB_RADIUS: f32 = 7.;
    /// The track row is the `Medium` rung, `rems(1.25)`, and is the slider's
    /// only row, since these sliders show no label and no value.
    const TRACK_Y: f32 = 10.;
    /// The rem size `the_mapping_holds_at_a_non_default_rem_size` draws at, and
    /// the thumb radius that follows from it: `rems(0.875) * 32 / 2`.
    const BIG_REM_SIZE: f32 = 32.;
    const BIG_THUMB_RADIUS: f32 = 14.;
    // These three restate the geometry by hand on purpose. A test that derived
    // its expected x from `Slider::thumb_size` would agree with any value the
    // rung took, including a wrong one, and so would pin nothing.

    struct Harness {
        slider: Entity<Slider>,
        drawn: Entity<usize>,
    }

    impl Render for Harness {
        fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            self.drawn.update(cx, |count, _| *count += 1);
            div().relative().size_full().child(
                div()
                    .absolute()
                    .left(px(TRACK_LEFT))
                    .top(px(0.))
                    .w(px(TRACK_WIDTH))
                    .child(self.slider.clone()),
            )
        }
    }

    /// The x that maps to `value` on the 0..=100 slider below.
    fn x_for(value: f32) -> Pixels {
        px(TRACK_LEFT + THUMB_RADIUS + (TRACK_WIDTH - THUMB_RADIUS * 2.) * value / 100.)
    }

    /// The same, for a slider drawn at `BIG_REM_SIZE`.
    fn big_x_for(value: f32) -> Pixels {
        px(TRACK_LEFT + BIG_THUMB_RADIUS + (TRACK_WIDTH - BIG_THUMB_RADIUS * 2.) * value / 100.)
    }

    fn at(x: Pixels) -> Point<Pixels> {
        Point::new(x, px(TRACK_Y))
    }

    fn value_of(cx: &mut VisualTestContext, slider: &Entity<Slider>) -> f32 {
        slider.read_with(cx, |slider, _| slider.value)
    }

    fn is_dragging(cx: &mut VisualTestContext, slider: &Entity<Slider>) -> bool {
        slider.read_with(cx, |slider, _| slider.is_dragging)
    }

    /// A slider at 50 in 0..=100, drawn in the wrapper above.
    fn scenario(
        cx: &mut TestAppContext,
        disabled: bool,
    ) -> (&mut VisualTestContext, Entity<Slider>, Entity<usize>) {
        cx.update(crate::theme::init);

        let slider = cx.update(|cx| {
            cx.new(|_| {
                Slider::new("test-slider", 50., 0.0..=100.)
                    .show_value(false)
                    .disabled(disabled)
            })
        });
        let drawn = cx.update(|cx| cx.new(|_| 0usize));
        let window = {
            let slider = slider.clone();
            let counter = drawn.clone();
            cx.open_window(size(px(400.), px(300.)), move |_window, _cx| Harness {
                slider,
                drawn: counter,
            })
        };

        let cx = VisualTestContext::from_window(*std::ops::Deref::deref(&window), cx).into_mut();
        // The canvas measures during paint, so there are no track bounds — and
        // no window handlers — until a frame has been drawn.
        cx.run_until_parked();

        assert!(
            drawn.read_with(cx, |count, _| *count) > 0,
            "the harness never drew, so this test is checking nothing"
        );
        (cx, slider, drawn)
    }

    /// The plain move first, as the splitter tests do — the div's hit test
    /// wants the hitbox hovered.
    fn press(cx: &mut VisualTestContext, x: Pixels) {
        cx.simulate_mouse_move(at(x), None, Modifiers::none());
        cx.run_until_parked();
        cx.simulate_mouse_down(at(x), MouseButton::Left, Modifiers::none());
        cx.run_until_parked();
    }

    fn drag_to(cx: &mut VisualTestContext, x: Pixels) {
        cx.simulate_mouse_move(at(x), MouseButton::Left, Modifiers::none());
        cx.run_until_parked();
    }

    // --- controls: these pass before the fix as well as after ---

    #[gpui::test]
    fn a_press_moves_the_value_and_starts_the_drag(cx: &mut TestAppContext) {
        let (cx, slider, _drawn) = scenario(cx, false);

        press(cx, x_for(75.));

        let value = value_of(cx, &slider);
        assert!(
            (value - 75.).abs() < 1e-3,
            "a press three quarters along the track should land on 75, not {value}"
        );
        assert!(is_dragging(cx, &slider), "the press did not start a drag");
    }

    #[gpui::test]
    fn a_drag_inside_the_track_tracks_the_pointer(cx: &mut TestAppContext) {
        let (cx, slider, _drawn) = scenario(cx, false);

        press(cx, x_for(50.));
        drag_to(cx, x_for(75.));

        let value = value_of(cx, &slider);
        assert!(
            (value - 75.).abs() < 1e-3,
            "an in-track drag should follow the pointer to 75, not {value}"
        );
    }

    #[gpui::test]
    fn a_disabled_slider_ignores_the_pointer(cx: &mut TestAppContext) {
        let (cx, slider, _drawn) = scenario(cx, true);

        press(cx, x_for(75.));
        drag_to(cx, px(380.));

        assert_eq!(
            value_of(cx, &slider),
            50.,
            "a disabled slider moved — the window handlers are registered \
             whether or not it is enabled, so they have to check"
        );
        assert!(
            !is_dragging(cx, &slider),
            "a disabled slider started a drag"
        );
    }

    // --- the regressions: these fail before the fix ---

    #[gpui::test]
    fn a_drag_past_the_end_pins_to_the_maximum(cx: &mut TestAppContext) {
        let (cx, slider, _drawn) = scenario(cx, false);

        press(cx, x_for(50.));
        // 380 is outside the track and still inside the window.
        drag_to(cx, px(380.));

        let value = value_of(cx, &slider);
        assert!(
            (value - 100.).abs() < 1e-3,
            "a drag past the end of the track should pin to 100, not stop at {value}"
        );
    }

    #[gpui::test]
    fn a_drag_past_the_start_pins_to_the_minimum(cx: &mut TestAppContext) {
        let (cx, slider, _drawn) = scenario(cx, false);

        press(cx, x_for(50.));
        drag_to(cx, px(5.));

        let value = value_of(cx, &slider);
        assert!(
            value.abs() < 1e-3,
            "a drag past the start of the track should pin to 0, not stop at {value}"
        );
    }

    #[gpui::test]
    fn a_release_outside_the_track_ends_the_drag(cx: &mut TestAppContext) {
        let (cx, slider, _drawn) = scenario(cx, false);

        press(cx, x_for(50.));
        drag_to(cx, px(380.));
        cx.simulate_mouse_up(at(px(380.)), MouseButton::Left, Modifiers::none());
        cx.run_until_parked();

        assert!(
            !is_dragging(cx, &slider),
            "a release outside the track left the slider dragging, so the \
             thumb keeps its dragging border"
        );

        // And the released slider does not follow the pointer back in.
        drag_to(cx, x_for(25.));
        let value = value_of(cx, &slider);
        assert!(
            (value - 100.).abs() < 1e-3,
            "the slider kept tracking after the release, moving to {value}"
        );
    }

    /// The release the window never saw at all: the pointer left the window
    /// with the button held, or another handler swallowed the mouse-up.
    #[gpui::test]
    fn a_move_with_no_button_held_ends_the_drag(cx: &mut TestAppContext) {
        let (cx, slider, _drawn) = scenario(cx, false);

        press(cx, x_for(50.));
        cx.simulate_mouse_move(at(px(380.)), None, Modifiers::none());
        cx.run_until_parked();

        assert!(
            !is_dragging(cx, &slider),
            "a move with no button held left the slider dragging"
        );
        let after_release = value_of(cx, &slider);

        // And a later move with the button down must not resume it.
        drag_to(cx, x_for(25.));
        assert_eq!(
            value_of(cx, &slider),
            after_release,
            "the drag resumed after it had ended"
        );
    }

    /// The `cx.notify()` in `on_mouse_down`. A press on the thumb leaves the
    /// value where it is, and `set_value` is silent when the value does not
    /// change, so that notify is the only thing here that puts the dragging
    /// border on screen.
    ///
    /// The handler is called directly rather than through a simulated press,
    /// because that is what isolates the line. gpui's `div` binds its
    /// active-state handlers unconditionally and refreshes the window on any
    /// mouse-down over an id'd hitbox, so a simulated press draws a new frame
    /// whether or not the notify is there — a test built on one passes with the
    /// line deleted, which is no pin at all.
    #[gpui::test]
    fn a_press_that_does_not_move_the_value_still_repaints(cx: &mut TestAppContext) {
        let (cx, slider, drawn) = scenario(cx, false);

        let before = drawn.read_with(cx, |count, _| *count);
        let press = MouseDownEvent {
            button: MouseButton::Left,
            position: at(x_for(50.)),
            modifiers: Modifiers::none(),
            click_count: 1,
            first_mouse: false,
        };
        cx.update(|window, cx| {
            slider.update(cx, |slider, cx| slider.on_mouse_down(&press, window, cx));
        });
        cx.run_until_parked();

        assert_eq!(
            value_of(cx, &slider),
            50.,
            "this test only means anything if the press leaves the value alone"
        );
        assert!(is_dragging(cx, &slider), "the press did not start a drag");
        assert!(
            drawn.read_with(cx, |count, _| *count) > before,
            "a press that does not move the value drew no new frame, so the \
             thumb keeps its idle border while it is being dragged"
        );
    }

    /// The thumb is sized in rems, so the inset `value_from_position` takes off
    /// each end of the track has to be read at the rem size the thumb is drawn
    /// at. It used to be a hardcoded `px(6.)`, which is only right at gpui's
    /// default 16px rem.
    ///
    /// Note the 75 and the 25: the centre of the track maps to the centre of
    /// the range under *any* thumb radius, so a test that pressed at 50 would
    /// pass with the defect intact. Under the old literal these two x's read
    /// 73.40 and 26.60.
    #[gpui::test]
    fn the_mapping_holds_at_a_non_default_rem_size(cx: &mut TestAppContext) {
        let (cx, slider, _drawn) = scenario(cx, false);

        // `set_rem_size` only assigns the field — unlike `set_scale_factor` it
        // does not invalidate the window — so the refresh is what makes the
        // thumb genuinely 24px on the next frame. The assertions below would
        // hold without it, because the fix reads the rem size when the event is
        // handled and the track's bounds are set in `px` and do not move; but
        // then they would be checking a frame that was never drawn.
        cx.update(|window, _cx| {
            window.set_rem_size(px(BIG_REM_SIZE));
            window.refresh();
        });
        cx.run_until_parked();

        press(cx, big_x_for(75.));
        let value = value_of(cx, &slider);
        assert!(
            (value - 75.).abs() < 1e-3,
            "at a {BIG_REM_SIZE}px rem, a press three quarters along the track \
             gave {value}, not 75"
        );

        drag_to(cx, big_x_for(25.));
        let value = value_of(cx, &slider);
        assert!(
            (value - 25.).abs() < 1e-3,
            "at a {BIG_REM_SIZE}px rem, a drag to one quarter along the track \
             gave {value}, not 25"
        );
    }
}
