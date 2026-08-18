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
use crate::theme::{ActiveTheme, Themeable};
use crate::traits::disableable::Disableable;
use crate::traits::labelable::Labelable;
use crate::utils::element_manager::ElementManagerExt;
use gpui::{
    canvas, div, prelude::*, px, rems, App, Bounds, Context, DispatchPhase, ElementId,
    EventEmitter, IntoElement, MouseButton, MouseDownEvent, MouseMoveEvent, MouseUpEvent,
    ParentElement, Pixels, Point, Render, SharedString, Styled, Window,
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

    fn value_from_position(&self, position: Point<Pixels>) -> f32 {
        let Some(bounds) = self.track_bounds else {
            return self.value;
        };

        let thumb_radius = px(6.);
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
        _window: &mut Window,
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
        let new_value = self.value_from_position(event.position);
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
    fn on_drag_move(&mut self, event: &MouseMoveEvent, cx: &mut Context<Self>) {
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
        let new_value = self.value_from_position(event.position);
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
        let percentage = self.percentage();
        let label = self.label.clone();
        let display_value = self.display_value();
        let is_dragging = self.is_dragging;
        let show_value = self.show_value;
        let disabled = self.disabled;

        let track_height = rems(0.25);
        let thumb_size = rems(0.75);

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
            .gap(rems(0.25))
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
                    .h(thumb_size)
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
                                    move |event: &MouseMoveEvent, phase, _window, cx| {
                                        if phase != DispatchPhase::Bubble {
                                            return;
                                        }
                                        entity.update(cx, |this, cx| this.on_drag_move(event, cx));
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
                            .top(px(0.))
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

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::{size, Entity, Modifiers, TestAppContext, VisualTestContext};

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
    const THUMB_RADIUS: f32 = 6.;
    /// The track is `rems(0.75)` tall and is the slider's only row, since these
    /// sliders show no label and no value.
    const TRACK_Y: f32 = 6.;

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
}
