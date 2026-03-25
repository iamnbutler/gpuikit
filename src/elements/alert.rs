//! Alert component for displaying important messages
//!
//! Supports different severity levels and optional dismiss functionality.

use crate::icons::Icons;
use crate::theme::{ActiveTheme, Themeable};
use gpui::{
    div, prelude::FluentBuilder, px, rems, App, ClickEvent, Context, ElementId, Entity,
    Hsla, InteractiveElement, IntoElement, MouseButton, ParentElement, RenderOnce, SharedString,
    StatefulInteractiveElement, Styled, Svg, Window,
};

/// Creates a new alert with an optional message
pub fn alert(message: impl Into<SharedString>) -> Alert {
    Alert::new().description(message)
}

/// Alert variant determining severity/styling
#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub enum AlertVariant {
    /// Neutral informational (default styling)
    #[default]
    Default,
    /// Informational (blue)
    Info,
    /// Positive/success (green)
    Success,
    /// Caution (yellow/orange)
    Warning,
    /// Error/danger (red)
    Destructive,
}

impl AlertVariant {
    /// Returns the default icon for this variant
    fn default_icon(&self) -> Svg {
        match self {
            AlertVariant::Default => Icons::info_circled(),
            AlertVariant::Info => Icons::info_circled(),
            AlertVariant::Success => Icons::check_circled(),
            AlertVariant::Warning => Icons::exclamation_triangle(),
            AlertVariant::Destructive => Icons::cross_circled(),
        }
    }

    /// Returns the icon/accent color for this variant from the theme
    fn color(&self, theme: &dyn Themeable) -> Hsla {
        match self {
            AlertVariant::Default => theme.fg_muted(),
            AlertVariant::Info => theme.info(),
            AlertVariant::Success => theme.success(),
            AlertVariant::Warning => theme.warning(),
            AlertVariant::Destructive => theme.danger(),
        }
    }

    /// Returns the background color for this variant
    fn bg_color(&self, theme: &dyn Themeable) -> Hsla {
        self.color(theme).opacity(0.1)
    }

    /// Returns the border color for this variant
    fn border_color(&self, theme: &dyn Themeable) -> Hsla {
        self.color(theme).opacity(0.3)
    }
}

/// Internal state for dismiss behavior
struct DismissState {
    dismissed: bool,
}

/// Alert component for displaying important messages
#[derive(IntoElement)]
pub struct Alert {
    id: Option<ElementId>,
    title: Option<SharedString>,
    description: Option<SharedString>,
    variant: AlertVariant,
    icon: Option<Svg>,
    show_icon: bool,
    dismissible: bool,
    on_dismiss: Option<Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>>,
}

impl Default for Alert {
    fn default() -> Self {
        Self::new()
    }
}

impl Alert {
    /// Create a new empty alert
    pub fn new() -> Self {
        Alert {
            id: None,
            title: None,
            description: None,
            variant: AlertVariant::Default,
            icon: None,
            show_icon: true,
            dismissible: false,
            on_dismiss: None,
        }
    }

    /// Set the alert ID (required for dismissible alerts)
    pub fn id(mut self, id: impl Into<ElementId>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// Set the alert title
    pub fn title(mut self, title: impl Into<SharedString>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Set the alert description/message
    pub fn description(mut self, description: impl Into<SharedString>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Set the alert variant
    pub fn variant(mut self, variant: AlertVariant) -> Self {
        self.variant = variant;
        self
    }

    /// Convenience method to set Info variant
    pub fn info(mut self) -> Self {
        self.variant = AlertVariant::Info;
        self
    }

    /// Convenience method to set Success variant
    pub fn success(mut self) -> Self {
        self.variant = AlertVariant::Success;
        self
    }

    /// Convenience method to set Warning variant
    pub fn warning(mut self) -> Self {
        self.variant = AlertVariant::Warning;
        self
    }

    /// Convenience method to set Destructive variant
    pub fn destructive(mut self) -> Self {
        self.variant = AlertVariant::Destructive;
        self
    }

    /// Override the default icon for the variant
    pub fn icon(mut self, icon: Svg) -> Self {
        self.icon = Some(icon);
        self
    }

    /// Hide the icon
    pub fn no_icon(mut self) -> Self {
        self.show_icon = false;
        self
    }

    /// Make the alert dismissible
    /// Note: Requires setting an ID via `.id()` for state management
    pub fn dismissible(mut self, dismissible: bool) -> Self {
        self.dismissible = dismissible;
        self
    }

    /// Set a callback for when the alert is dismissed
    pub fn on_dismiss(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_dismiss = Some(Box::new(handler));
        self
    }
}

impl RenderOnce for Alert {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        // Extract theme colors first to avoid borrow conflicts
        let (fg_color, fg_muted_color, surface_secondary_color, variant_color, bg_color, border_color) = {
            let theme = cx.theme();
            (
                theme.fg(),
                theme.fg_muted(),
                theme.surface_secondary(),
                self.variant.color(theme.as_ref()),
                self.variant.bg_color(theme.as_ref()),
                self.variant.border_color(theme.as_ref()),
            )
        };

        // Handle dismiss state if dismissible
        let is_dismissed = if self.dismissible {
            if let Some(ref id) = self.id {
                let state: Entity<DismissState> = window.use_keyed_state(
                    id.clone(),
                    cx,
                    |_window, _cx: &mut Context<DismissState>| DismissState { dismissed: false },
                );
                state.read(cx).dismissed
            } else {
                false
            }
        } else {
            false
        };

        // Don't render if dismissed
        if is_dismissed {
            return div().into_any_element();
        }

        let icon = if self.show_icon {
            Some(self.icon.unwrap_or_else(|| self.variant.default_icon()))
        } else {
            None
        };

        let has_title = self.title.is_some();

        let id = self.id.clone();
        let on_dismiss = self.on_dismiss;

        div()
            .w_full()
            .flex()
            .gap(rems(0.75))
            .p(rems(0.75))
            .bg(bg_color)
            .border_1()
            .border_color(border_color)
            .rounded(rems(0.375))
            // Icon
            .when_some(icon, |alert, icon| {
                alert.child(
                    div()
                        .flex_none()
                        .pt(px(2.0))
                        .child(icon.size(px(16.0)).text_color(variant_color)),
                )
            })
            // Content
            .child(
                div()
                    .flex_1()
                    .flex()
                    .flex_col()
                    .gap(rems(0.25))
                    // Title
                    .when_some(self.title, |content, title| {
                        content.child(
                            div()
                                .text_sm()
                                .font_weight(gpui::FontWeight::SEMIBOLD)
                                .text_color(fg_color)
                                .child(title),
                        )
                    })
                    // Description
                    .when_some(self.description, |content, description| {
                        content.child(
                            div()
                                .text_sm()
                                .text_color(if has_title {
                                    fg_muted_color
                                } else {
                                    fg_color
                                })
                                .child(description),
                        )
                    }),
            )
            // Dismiss button
            .when(self.dismissible && id.is_some(), |alert| {
                // Capture the dismiss state entity during render so the click handler can use it
                let dismiss_id = id.clone().unwrap();
                let dismiss_state: Entity<DismissState> = window.use_keyed_state(
                    dismiss_id,
                    cx,
                    |_window, _cx: &mut Context<DismissState>| DismissState { dismissed: false },
                );
                alert.child(
                    div()
                        .id("alert-dismiss")
                        .flex_none()
                        .size(px(20.0))
                        .flex()
                        .items_center()
                        .justify_center()
                        .rounded(px(4.0))
                        .cursor_pointer()
                        .hover(|div| div.bg(surface_secondary_color.opacity(0.5)))
                        .on_mouse_down(MouseButton::Left, |_, window, _| window.prevent_default())
                        .on_click(move |event, window, cx| {
                            cx.stop_propagation();
                            dismiss_state.update(cx, |s, _| {
                                s.dismissed = true;
                            });
                            // Call dismiss handler
                            if let Some(ref handler) = on_dismiss {
                                handler(event, window, cx);
                            }
                        })
                        .child(Icons::cross_1().size(px(12.0)).text_color(fg_muted_color)),
                )
            })
            .into_any_element()
    }
}
