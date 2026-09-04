//! Toggle button component for gpuikit

use crate::layout::h_stack;
use crate::theme::{ActiveTheme, ControlSize, Themeable};
use crate::traits::control_sized::ControlSized;
use crate::traits::disableable::Disableable;
use crate::traits::labelable::Labelable;
use crate::traits::selectable::Selectable;
use crate::utils::element_manager::ElementManagerExt;
use gpui::{
    App, Context, ElementId, EventEmitter, InteractiveElement, IntoElement, MouseButton,
    ParentElement, Render, SharedString, StatefulInteractiveElement, Styled, Window, div,
    prelude::*,
};

/// Event emitted when the toggle state changes
pub struct ToggleChanged {
    pub enabled: bool,
}

/// A toggle button component for switching between on/off states
pub struct Toggle {
    id: ElementId,
    label: Option<SharedString>,
    enabled: bool,
    disabled: bool,
    size: ControlSize,
}

impl EventEmitter<ToggleChanged> for Toggle {}

impl Toggle {
    pub fn new(id: impl Into<ElementId>, enabled: bool) -> Self {
        Self {
            id: id.into(),
            label: None,
            enabled,
            disabled: false,
            size: ControlSize::default(),
        }
    }

    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn set_enabled(&mut self, enabled: bool, cx: &mut Context<Self>) {
        if self.enabled != enabled {
            self.enabled = enabled;
            cx.emit(ToggleChanged {
                enabled: self.enabled,
            });
            cx.notify();
        }
    }

    pub fn toggle(&mut self, cx: &mut Context<Self>) {
        self.set_enabled(!self.enabled, cx);
    }

    fn on_click(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        if !self.disabled {
            self.toggle(cx);
        }
    }
}

impl Render for Toggle {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let metrics = theme.control(self.size);
        let enabled = self.enabled;
        let disabled = self.disabled;
        let label = self.label.clone();

        // Shared with `Switch` — see `ControlMetrics::track`. What still tells
        // the two apart is the foreground-coloured thumb and lighter shadow
        // below.
        let track = metrics.track();

        let track_bg = if disabled {
            theme.surface_tertiary()
        } else if enabled {
            theme.accent()
        } else {
            theme.surface_secondary()
        };

        let thumb_bg = if disabled {
            theme.fg_disabled()
        } else {
            theme.fg()
        };

        let track_border = if disabled {
            theme.border_subtle()
        } else if enabled {
            theme.accent()
        } else {
            theme.border()
        };

        h_stack()
            .id(self.id.clone())
            .h(metrics.height)
            // A label outside the control's own box wants more room than the
            // gap between an icon and a label inside one.
            .gap(metrics.gap * 2.0)
            .items_center()
            .when(!disabled, |this| {
                this.cursor_pointer()
                    .on_mouse_down(MouseButton::Left, |_, window, _| window.prevent_default())
                    .on_click(cx.listener(|this, _, window, cx| {
                        this.on_click(window, cx);
                    }))
            })
            .when(disabled, |this| this.cursor_not_allowed().opacity(0.65))
            .child(
                div()
                    .relative()
                    .w(track.width)
                    .h(track.height)
                    .bg(track_bg)
                    .border_1()
                    .border_color(track_border)
                    .rounded(track.height / 2.)
                    .when(!disabled, |this| {
                        this.hover(|style| {
                            style.border_color(if enabled {
                                theme.accent()
                            } else {
                                theme.border_secondary()
                            })
                        })
                    })
                    .child(
                        div()
                            .absolute()
                            .top(track.thumb_margin)
                            .when(enabled, |this| this.right(track.thumb_margin))
                            .when(!enabled, |this| this.left(track.thumb_margin))
                            .size(track.thumb)
                            .bg(thumb_bg)
                            .rounded_full()
                            .shadow_sm(),
                    ),
            )
            .when_some(label, |this, label| {
                this.child(
                    div()
                        .text_size(metrics.text_size)
                        .line_height(metrics.line_height)
                        .text_color(if disabled {
                            theme.fg_disabled()
                        } else {
                            theme.fg()
                        })
                        .child(label),
                )
            })
    }
}

/// Convenience function to create a toggle builder
pub fn toggle(id: impl Into<ElementId>, enabled: bool) -> Toggle {
    Toggle::new(id, enabled)
}

/// Convenience function to create a toggle with auto-generated ID
pub fn toggle_auto(cx: &App, enabled: bool) -> Toggle {
    Toggle::new(cx.next_id_named("toggle"), enabled)
}

impl Disableable for Toggle {
    fn is_disabled(&self) -> bool {
        self.disabled
    }

    fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

impl Selectable for Toggle {
    fn is_selected(&self) -> bool {
        self.enabled
    }

    fn selected(mut self, selected: bool) -> Self {
        self.enabled = selected;
        self
    }
}

impl ControlSized for Toggle {
    fn control_size(mut self, size: ControlSize) -> Self {
        self.size = size;
        self
    }
}

impl Labelable for Toggle {
    fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }
}
