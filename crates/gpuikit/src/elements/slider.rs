//! Slider component for gpuikit

use crate::layout::{h_stack, v_stack};
use crate::utils::element_manager::ElementManagerExt;
use gpui::{
    canvas, div, prelude::*, px, rems, App, Bounds, Context, ElementId, EventEmitter, IntoElement,
    MouseButton, MouseDownEvent, MouseMoveEvent, MouseUpEvent, ParentElement, Pixels, Point,
    Render, SharedString, Styled, Window,
};
use gpuikit_theme::{ActiveTheme, Themeable};
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
        let new_value = self.value_from_position(event.position);
        self.set_value(new_value, cx);
    }

    fn on_mouse_up(&mut self, _event: &MouseUpEvent, _window: &mut Window, cx: &mut Context<Self>) {
        self.is_dragging = false;
        cx.notify();
    }

    fn on_mouse_move(
        &mut self,
        event: &MouseMoveEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.is_dragging && event.dragging() {
            let new_value = self.value_from_position(event.position);
            self.set_value(new_value, cx);
        }
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
                    .id("slider-track-container")
                    .relative()
                    .h(thumb_size)
                    .w_full()
                    .flex()
                    .items_center()
                    .when(!disabled, |this| {
                        this.cursor_pointer()
                            .on_mouse_down(MouseButton::Left, cx.listener(Self::on_mouse_down))
                            .on_mouse_up(MouseButton::Left, cx.listener(Self::on_mouse_up))
                            .on_mouse_move(cx.listener(Self::on_mouse_move))
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
                            move |bounds, _, _window, cx| {
                                entity.update(cx, |this, _cx| {
                                    this.track_bounds = Some(bounds);
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
