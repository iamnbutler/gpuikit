//! Toast notification component for temporary messages
//!
//! Provides popup notifications that appear and auto-dismiss after a configurable duration.
//! Supports multiple toasts stacking vertically, manual dismissal, and optional action buttons.
//!
//! # Example
//!
//! ```ignore
//! use gpuikit::elements::toast::{ToastExt, ToastPosition};
//! use std::time::Duration;
//!
//! // In init:
//! toast::init(cx);
//!
//! // Usage anywhere:
//! cx.toast("Changes saved").success().duration(Duration::from_secs(3)).show(window, cx);
//!
//! // With action:
//! cx.toast("File deleted")
//!     .action("Undo", |window, cx| { /* ... */ })
//!     .duration(Duration::from_secs(5))
//!     .show(window, cx);
//! ```

use crate::icons::Icons;
use crate::theme::{ActiveTheme, Themeable};
use gpui::{
    deferred, div, prelude::*, px, rems, AnyElement, App, ClickEvent, Context, ElementId,
    Entity, Global, Hsla, InteractiveElement, IntoElement, MouseButton, ParentElement, Render,
    SharedString, StatefulInteractiveElement, Styled, Svg, Window,
};
use smol::Timer;
use std::rc::Rc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;

/// Default auto-dismiss duration for toasts
const DEFAULT_DURATION: Duration = Duration::from_secs(5);

/// Toast position on the screen
#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
pub enum ToastPosition {
    /// Top-left corner
    TopLeft,
    /// Top-center
    TopCenter,
    /// Top-right corner (default)
    #[default]
    TopRight,
    /// Bottom-left corner
    BottomLeft,
    /// Bottom-center
    BottomCenter,
    /// Bottom-right corner
    BottomRight,
}

/// Toast variant determining styling
#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
pub enum ToastVariant {
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

impl ToastVariant {
    /// Returns the default icon for this variant
    fn default_icon(&self) -> Svg {
        match self {
            ToastVariant::Default => Icons::info_circled(),
            ToastVariant::Info => Icons::info_circled(),
            ToastVariant::Success => Icons::check_circled(),
            ToastVariant::Warning => Icons::exclamation_triangle(),
            ToastVariant::Destructive => Icons::cross_circled(),
        }
    }

    /// Returns the icon/accent color for this variant from the theme
    fn color(&self, theme: &dyn Themeable) -> Hsla {
        match self {
            ToastVariant::Default => theme.fg_muted(),
            ToastVariant::Info => theme.info(),
            ToastVariant::Success => theme.success(),
            ToastVariant::Warning => theme.warning(),
            ToastVariant::Destructive => theme.danger(),
        }
    }

    /// Returns the background color for this variant
    #[allow(dead_code)]
    fn bg_color(&self, theme: &dyn Themeable) -> Hsla {
        self.color(theme).opacity(0.1)
    }

    /// Returns the border color for this variant
    fn border_color(&self, theme: &dyn Themeable) -> Hsla {
        self.color(theme).opacity(0.3)
    }
}

/// Icon selection for a toast
#[derive(Clone, Copy, Default)]
enum ToastIcon {
    /// Use the default icon for the variant
    #[default]
    Default,
    /// Hide the icon
    Hidden,
}

/// Internal state for a single toast
struct ToastState {
    id: ElementId,
    title: Option<SharedString>,
    description: Option<SharedString>,
    variant: ToastVariant,
    icon: ToastIcon,
    action: Option<(SharedString, Rc<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>)>,
    on_dismiss: Option<Rc<dyn Fn(&mut Window, &mut App) + 'static>>,
    generation: usize,
}

/// A builder for creating and showing toast notifications
pub struct Toast {
    title: Option<SharedString>,
    description: Option<SharedString>,
    variant: ToastVariant,
    icon: ToastIcon,
    duration: Duration,
    action: Option<(SharedString, Rc<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>)>,
    on_dismiss: Option<Rc<dyn Fn(&mut Window, &mut App) + 'static>>,
}

impl Toast {
    /// Create a new toast builder
    pub fn new() -> Self {
        Self {
            title: None,
            description: None,
            variant: ToastVariant::Default,
            icon: ToastIcon::Default,
            duration: DEFAULT_DURATION,
            action: None,
            on_dismiss: None,
        }
    }

    /// Set the toast title
    pub fn title(mut self, title: impl Into<SharedString>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Set the toast description/message
    pub fn description(mut self, description: impl Into<SharedString>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Set the toast variant
    pub fn variant(mut self, variant: ToastVariant) -> Self {
        self.variant = variant;
        self
    }

    /// Convenience method to set Info variant
    pub fn info(mut self) -> Self {
        self.variant = ToastVariant::Info;
        self
    }

    /// Convenience method to set Success variant
    pub fn success(mut self) -> Self {
        self.variant = ToastVariant::Success;
        self
    }

    /// Convenience method to set Warning variant
    pub fn warning(mut self) -> Self {
        self.variant = ToastVariant::Warning;
        self
    }

    /// Convenience method to set Destructive variant
    pub fn destructive(mut self) -> Self {
        self.variant = ToastVariant::Destructive;
        self
    }

    /// Hide the icon
    pub fn no_icon(mut self) -> Self {
        self.icon = ToastIcon::Hidden;
        self
    }

    /// Set the auto-dismiss duration
    pub fn duration(mut self, duration: Duration) -> Self {
        self.duration = duration;
        self
    }

    /// Add an action button to the toast
    pub fn action(
        mut self,
        label: impl Into<SharedString>,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.action = Some((label.into(), Rc::new(handler)));
        self
    }

    /// Set a callback for when the toast is dismissed
    pub fn on_dismiss(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_dismiss = Some(Rc::new(handler));
        self
    }

    /// Show the toast
    pub fn show(self, window: &mut Window, cx: &mut App) {
        let manager = cx.global::<GlobalToastManager>().0.clone();
        manager.update(cx, |manager, cx| {
            manager.add_toast(self, window, cx);
        });
    }
}

impl Default for Toast {
    fn default() -> Self {
        Self::new()
    }
}

/// Manages the queue of active toasts
pub struct ToastManager {
    toasts: Vec<Entity<ToastState>>,
    position: ToastPosition,
    next_id: Arc<AtomicUsize>,
}

impl ToastManager {
    /// Create a new toast manager
    pub fn new() -> Self {
        Self {
            toasts: Vec::new(),
            position: ToastPosition::default(),
            next_id: Arc::new(AtomicUsize::new(0)),
        }
    }

    /// Set the position for toasts
    pub fn set_position(&mut self, position: ToastPosition) {
        self.position = position;
    }

    /// Get the current position
    pub fn position(&self) -> ToastPosition {
        self.position
    }

    /// Add a toast to the queue
    fn add_toast(&mut self, toast: Toast, window: &mut Window, cx: &mut Context<Self>) {
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        let element_id = ElementId::named_usize("toast", id);
        let generation = id;

        let toast_state = cx.new(|_cx| ToastState {
            id: element_id,
            title: toast.title,
            description: toast.description,
            variant: toast.variant,
            icon: toast.icon,
            action: toast.action,
            on_dismiss: toast.on_dismiss,
            generation,
        });

        self.toasts.push(toast_state.clone());

        // Schedule auto-dismiss
        let duration = toast.duration;
        cx.spawn(async move |this, cx| {
            Timer::after(duration).await;
            if let Some(this) = this.upgrade() {
                this.update(cx, |manager, cx| {
                    manager.dismiss_toast_by_generation(generation, cx);
                });
            }
        })
        .detach();

        cx.notify();
        window.refresh();
    }

    /// Dismiss a toast by its generation (used for auto-dismiss, no window access)
    fn dismiss_toast_by_generation(&mut self, generation: usize, cx: &mut Context<Self>) {
        if let Some(index) = self
            .toasts
            .iter()
            .position(|t| t.read(cx).generation == generation)
        {
            self.toasts.remove(index);
            cx.notify();
        }
    }

    /// Dismiss a toast with window access (used for manual dismiss)
    fn dismiss_toast(
        &mut self,
        toast: &Entity<ToastState>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if let Some(index) = self.toasts.iter().position(|t| t == toast) {
            let toast = self.toasts.remove(index);
            // Clone the callback before calling to avoid borrow conflict
            let on_dismiss = toast.read(cx).on_dismiss.clone();
            if let Some(callback) = on_dismiss {
                callback(window, cx);
            }
            cx.notify();
        }
    }

    /// Get the number of active toasts
    pub fn count(&self) -> usize {
        self.toasts.len()
    }

    /// Clear all toasts
    pub fn clear(&mut self, cx: &mut Context<Self>) {
        self.toasts.clear();
        cx.notify();
    }
}

impl Default for ToastManager {
    fn default() -> Self {
        Self::new()
    }
}

impl Render for ToastManager {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if self.toasts.is_empty() {
            return div().into_any_element();
        }

        let position = self.position;
        let toasts: Vec<_> = self.toasts.clone();

        // Build the toast container with proper positioning
        let container = div()
            .id("toast-container")
            .absolute()
            .top_0()
            .left_0()
            .right_0()
            .bottom_0()
            .size_full()
            .flex()
            .p_4()
            // Set alignment based on position
            .when(matches!(position, ToastPosition::TopLeft), |d| {
                d.items_start().justify_start()
            })
            .when(matches!(position, ToastPosition::TopCenter), |d| {
                d.items_start().justify_center()
            })
            .when(matches!(position, ToastPosition::TopRight), |d| {
                d.items_start().justify_end()
            })
            .when(matches!(position, ToastPosition::BottomLeft), |d| {
                d.items_end().justify_start()
            })
            .when(matches!(position, ToastPosition::BottomCenter), |d| {
                d.items_end().justify_center()
            })
            .when(matches!(position, ToastPosition::BottomRight), |d| {
                d.items_end().justify_end()
            })
            // Inner container for toasts
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .w(px(360.))
                    // Reverse order for bottom positions so newest appears closest to corner
                    .when(
                        matches!(
                            position,
                            ToastPosition::BottomLeft
                                | ToastPosition::BottomCenter
                                | ToastPosition::BottomRight
                        ),
                        |d| d.flex_col_reverse(),
                    )
                    .children(toasts.into_iter().map(|toast_entity| {
                        self.render_toast(&toast_entity, window, cx)
                    })),
            );

        deferred(container)
            .with_priority(15) // Higher than dialogs (10)
            .into_any_element()
    }
}

impl ToastManager {
    fn render_toast(
        &self,
        toast_entity: &Entity<ToastState>,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let toast = toast_entity.read(cx);

        // Extract theme colors
        let (fg_color, fg_muted_color, surface_color, border_theme_color, variant_color, border_color) = {
            let theme = cx.theme();
            let variant = toast.variant;
            (
                theme.fg(),
                theme.fg_muted(),
                theme.surface(),
                theme.border(),
                variant.color(theme.as_ref()),
                variant.border_color(theme.as_ref()),
            )
        };

        let variant = toast.variant;
        let icon_mode = toast.icon;

        let has_title = toast.title.is_some();
        let title = toast.title.clone();
        let description = toast.description.clone();
        let action = toast.action.clone();
        let toast_entity_for_action = toast_entity.clone();
        let toast_entity_for_dismiss = toast_entity.clone();
        let hover_bg: Hsla = border_theme_color.opacity(0.5);

        div()
            .id(toast.id.clone())
            .w_full()
            .flex()
            .gap(rems(0.75))
            .p(rems(0.75))
            .bg(surface_color)
            .border_1()
            .border_color(border_color)
            .rounded(rems(0.375))
            .shadow_lg()
            // Icon
            .when(matches!(icon_mode, ToastIcon::Default), |toast_div| {
                toast_div.child(
                    div()
                        .flex_none()
                        .pt(px(2.0))
                        .child(variant.default_icon().size(px(16.0)).text_color(variant_color)),
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
                    .when_some(title, |content, title| {
                        content.child(
                            div()
                                .text_sm()
                                .font_weight(gpui::FontWeight::SEMIBOLD)
                                .text_color(fg_color)
                                .child(title),
                        )
                    })
                    // Description
                    .when_some(description, |content, description| {
                        content.child(
                            div()
                                .text_sm()
                                .text_color(if has_title { fg_muted_color } else { fg_color })
                                .child(description),
                        )
                    })
                    // Action button
                    .when_some(action, |content, (label, handler)| {
                        content.child(
                            div()
                                .mt_1()
                                .child(
                                    div()
                                        .id("toast-action")
                                        .text_sm()
                                        .font_weight(gpui::FontWeight::MEDIUM)
                                        .text_color(variant_color)
                                        .cursor_pointer()
                                        .hover(|style| style.underline())
                                        .on_mouse_down(MouseButton::Left, |_, window, _| {
                                            window.prevent_default()
                                        })
                                        .on_click({
                                            let handler = handler.clone();
                                            let toast_entity = toast_entity_for_action.clone();
                                            cx.listener(move |this, event, window, cx| {
                                                handler(event, window, cx);
                                                this.dismiss_toast(&toast_entity, window, cx);
                                            })
                                        })
                                        .child(label),
                                ),
                        )
                    }),
            )
            // Close button
            .child(
                div()
                    .id("toast-dismiss")
                    .flex_none()
                    .size(px(20.0))
                    .flex()
                    .items_center()
                    .justify_center()
                    .rounded(px(4.0))
                    .cursor_pointer()
                    .hover(move |div| div.bg(hover_bg))
                    .on_mouse_down(MouseButton::Left, |_, window, _| window.prevent_default())
                    .on_click(cx.listener(move |this, _event, window, cx| {
                        this.dismiss_toast(&toast_entity_for_dismiss, window, cx);
                    }))
                    .child(Icons::cross_1().size(px(12.0)).text_color(fg_muted_color)),
            )
            .into_any_element()
    }
}

/// Global wrapper for ToastManager
pub struct GlobalToastManager(pub Entity<ToastManager>);

impl Global for GlobalToastManager {}

/// Extension trait for App to create toasts
pub trait ToastExt {
    /// Create a new toast builder with a description
    fn toast(&self, description: impl Into<SharedString>) -> Toast;

    /// Get a reference to the toast manager
    fn toast_manager(&self) -> &Entity<ToastManager>;
}

impl ToastExt for App {
    fn toast(&self, description: impl Into<SharedString>) -> Toast {
        Toast::new().description(description)
    }

    fn toast_manager(&self) -> &Entity<ToastManager> {
        &self.global::<GlobalToastManager>().0
    }
}

/// Set the toast position (requires mutable context)
pub fn set_toast_position(cx: &mut App, position: ToastPosition) {
    let manager = cx.global::<GlobalToastManager>().0.clone();
    manager.update(cx, |manager, _cx| {
        manager.set_position(position);
    });
}

/// Initialize the toast system
///
/// Call this in your application's initialization along with `gpuikit::init()`.
///
/// # Example
///
/// ```ignore
/// use gpuikit::elements::toast;
///
/// fn main() {
///     Application::new().run(|cx| {
///         gpuikit::init(cx);
///         toast::init(cx);
///         // ... rest of app initialization
///     });
/// }
/// ```
pub fn init(cx: &mut App) {
    let manager = cx.new(|_cx| ToastManager::new());
    cx.set_global(GlobalToastManager(manager));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_toast_variant_colors() {
        // Test that each variant has non-zero opacity colors using default theme
        let theme = crate::theme::Theme::default();
        for variant in [
            ToastVariant::Default,
            ToastVariant::Info,
            ToastVariant::Success,
            ToastVariant::Warning,
            ToastVariant::Destructive,
        ] {
            let color = variant.color(&theme);
            assert!(color.a > 0.0, "Variant {:?} should have non-zero alpha", variant);

            let bg = variant.bg_color(&theme);
            assert!(bg.a > 0.0 && bg.a < 1.0, "Variant {:?} bg should be translucent", variant);

            let border = variant.border_color(&theme);
            assert!(border.a > 0.0 && border.a < 1.0, "Variant {:?} border should be translucent", variant);
        }
    }

    #[test]
    fn test_toast_builder() {
        let toast = Toast::new()
            .title("Test Title")
            .description("Test description")
            .success()
            .duration(Duration::from_secs(3));

        assert_eq!(toast.title, Some("Test Title".into()));
        assert_eq!(toast.description, Some("Test description".into()));
        assert_eq!(toast.variant, ToastVariant::Success);
        assert_eq!(toast.duration, Duration::from_secs(3));
        assert!(matches!(toast.icon, ToastIcon::Default));
    }

    #[test]
    fn test_toast_no_icon() {
        let toast = Toast::new().no_icon();
        assert!(matches!(toast.icon, ToastIcon::Hidden));
    }
}
