//! Toast notification component for temporary messages
//!
//! Toasts are brief notifications that appear and auto-dismiss. They support
//! multiple variants, configurable positions, and optional action buttons.
//!
//! # Example
//!
//! ```ignore
//! use gpuikit::elements::toast;
//!
//! // In init:
//! toast::init(cx);
//!
//! // Usage anywhere:
//! cx.toast("Changes saved").success().duration(Duration::from_secs(3)).show();
//!
//! // With action:
//! cx.toast("File deleted")
//!     .action("Undo", |window, cx| { /* ... */ })
//!     .duration(Duration::from_secs(5))
//!     .show();
//! ```

use crate::elements::alert::AlertVariant;
use crate::icons::Icons;
use crate::theme::{ActiveTheme, Themeable};
use gpui::{
    div, hsla, prelude::FluentBuilder, px, rems, App, AnyElement, AppContext, Context, Div,
    Entity, Global, Hsla, InteractiveElement, IntoElement, MouseButton, ParentElement, Render,
    SharedString, StatefulInteractiveElement, Styled, Svg, Window,
};
use smol::Timer;
use std::rc::Rc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;

/// Default duration for toast auto-dismiss (5 seconds)
const DEFAULT_DURATION: Duration = Duration::from_secs(5);

/// Position for toast container
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ToastPosition {
    /// Top-right corner (default)
    #[default]
    TopRight,
    /// Top-left corner
    TopLeft,
    /// Bottom-right corner
    BottomRight,
    /// Bottom-left corner
    BottomLeft,
    /// Top center
    TopCenter,
    /// Bottom center
    BottomCenter,
}

/// Internal state for a single toast notification
#[derive(Clone)]
pub struct ToastData {
    /// Unique ID for this toast
    pub id: usize,
    /// Toast title (optional)
    pub title: Option<SharedString>,
    /// Toast message/description
    pub message: SharedString,
    /// Visual variant
    pub variant: AlertVariant,
    /// Duration before auto-dismiss (None = no auto-dismiss)
    pub duration: Option<Duration>,
    /// Optional action button
    pub action: Option<ToastAction>,
}

/// Action button for a toast
#[derive(Clone)]
pub struct ToastAction {
    /// Button label
    pub label: SharedString,
    /// Click handler
    pub handler: Rc<dyn Fn(&mut Window, &mut App)>,
}

impl ToastAction {
    /// Create a new toast action
    pub fn new(
        label: impl Into<SharedString>,
        handler: impl Fn(&mut Window, &mut App) + 'static,
    ) -> Self {
        Self {
            label: label.into(),
            handler: Rc::new(handler),
        }
    }
}

/// Builder for creating toast notifications
pub struct ToastBuilder<'a> {
    cx: &'a mut App,
    title: Option<SharedString>,
    message: SharedString,
    variant: AlertVariant,
    duration: Option<Duration>,
    action: Option<ToastAction>,
}

impl<'a> ToastBuilder<'a> {
    /// Create a new toast builder
    pub fn new(cx: &'a mut App, message: impl Into<SharedString>) -> Self {
        Self {
            cx,
            title: None,
            message: message.into(),
            variant: AlertVariant::Default,
            duration: Some(DEFAULT_DURATION),
            action: None,
        }
    }

    /// Set the toast title
    pub fn title(mut self, title: impl Into<SharedString>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Set the toast variant
    pub fn variant(mut self, variant: AlertVariant) -> Self {
        self.variant = variant;
        self
    }

    /// Set the variant to Info
    pub fn info(mut self) -> Self {
        self.variant = AlertVariant::Info;
        self
    }

    /// Set the variant to Success
    pub fn success(mut self) -> Self {
        self.variant = AlertVariant::Success;
        self
    }

    /// Set the variant to Warning
    pub fn warning(mut self) -> Self {
        self.variant = AlertVariant::Warning;
        self
    }

    /// Set the variant to Destructive/Error
    pub fn error(mut self) -> Self {
        self.variant = AlertVariant::Destructive;
        self
    }

    /// Set auto-dismiss duration
    pub fn duration(mut self, duration: Duration) -> Self {
        self.duration = Some(duration);
        self
    }

    /// Disable auto-dismiss (toast must be manually dismissed)
    pub fn persistent(mut self) -> Self {
        self.duration = None;
        self
    }

    /// Add an action button to the toast
    pub fn action(
        mut self,
        label: impl Into<SharedString>,
        handler: impl Fn(&mut Window, &mut App) + 'static,
    ) -> Self {
        self.action = Some(ToastAction::new(label, handler));
        self
    }

    /// Show the toast (consumes the builder)
    pub fn show(self) {
        let manager = self.cx.global::<GlobalToastManager>().0.clone();
        let id = manager.next_id();

        let toast_data = ToastData {
            id,
            title: self.title,
            message: self.message,
            variant: self.variant,
            duration: self.duration,
            action: self.action,
        };

        manager.state.update(self.cx, |state, cx| {
            state.add_toast(toast_data, cx);
        });
    }
}

/// Global toast manager that holds the notification queue
pub struct ToastManager {
    next_id: Arc<AtomicUsize>,
    /// The state entity for reactive updates
    pub state: Entity<ToastState>,
}

impl ToastManager {
    /// Create a new toast manager
    pub fn new(state: Entity<ToastState>) -> Self {
        Self {
            next_id: Arc::new(AtomicUsize::new(0)),
            state,
        }
    }

    /// Get the next toast ID
    fn next_id(&self) -> usize {
        self.next_id.fetch_add(1, Ordering::SeqCst)
    }
}

/// Reactive state for the toast system
pub struct ToastState {
    /// Active toasts
    toasts: Vec<ToastData>,
    /// Position for toast container
    position: ToastPosition,
    /// Maximum number of visible toasts
    max_visible: usize,
}

impl ToastState {
    /// Create a new toast state
    pub fn new() -> Self {
        Self {
            toasts: Vec::new(),
            position: ToastPosition::default(),
            max_visible: 5,
        }
    }

    /// Set the toast position
    pub fn set_position(&mut self, position: ToastPosition, cx: &mut Context<Self>) {
        self.position = position;
        cx.notify();
    }

    /// Set maximum visible toasts
    pub fn set_max_visible(&mut self, max: usize, cx: &mut Context<Self>) {
        self.max_visible = max;
        cx.notify();
    }

    /// Add a toast to the queue
    pub fn add_toast(&mut self, toast: ToastData, cx: &mut Context<Self>) {
        let duration = toast.duration;
        let toast_id = toast.id;

        self.toasts.push(toast);
        cx.notify();

        // Set up auto-dismiss timer if duration is specified
        if let Some(duration) = duration {
            cx.spawn(async move |this, cx| {
                Timer::after(duration).await;
                if let Some(this) = this.upgrade() {
                    this.update(cx, |state, cx| {
                        state.dismiss_toast(toast_id, cx);
                    });
                }
            })
            .detach();
        }
    }

    /// Dismiss a toast by ID
    pub fn dismiss_toast(&mut self, id: usize, cx: &mut Context<Self>) {
        self.toasts.retain(|t| t.id != id);
        cx.notify();
    }

    /// Clear all toasts
    pub fn clear(&mut self, cx: &mut Context<Self>) {
        self.toasts.clear();
        cx.notify();
    }

    /// Get visible toasts (limited by max_visible)
    pub fn visible_toasts(&self) -> impl Iterator<Item = &ToastData> {
        self.toasts.iter().take(self.max_visible)
    }

    /// Get the current position
    pub fn position(&self) -> ToastPosition {
        self.position
    }
}

impl Default for ToastState {
    fn default() -> Self {
        Self::new()
    }
}

impl Render for ToastState {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let toasts: Vec<_> = self.visible_toasts().cloned().collect();
        let position = self.position;

        if toasts.is_empty() {
            return div().into_any_element();
        }

        // Create the base container with position-based styling
        let container: Div = match position {
            ToastPosition::TopRight => div()
                .absolute()
                .top_4()
                .right_4()
                .flex()
                .flex_col()
                .items_end(),
            ToastPosition::TopLeft => div()
                .absolute()
                .top_4()
                .left_4()
                .flex()
                .flex_col()
                .items_start(),
            ToastPosition::BottomRight => div()
                .absolute()
                .bottom_4()
                .right_4()
                .flex()
                .flex_col_reverse()
                .items_end(),
            ToastPosition::BottomLeft => div()
                .absolute()
                .bottom_4()
                .left_4()
                .flex()
                .flex_col_reverse()
                .items_start(),
            ToastPosition::TopCenter => div()
                .absolute()
                .top_4()
                .left_0()
                .right_0()
                .flex()
                .flex_col()
                .items_center(),
            ToastPosition::BottomCenter => div()
                .absolute()
                .bottom_4()
                .left_0()
                .right_0()
                .flex()
                .flex_col_reverse()
                .items_center(),
        };

        let toast_elements: Vec<AnyElement> = toasts
            .into_iter()
            .map(|toast| render_toast(toast, cx).into_any_element())
            .collect();

        container
            .gap_2()
            .w(px(360.0))
            .max_h_full()
            .overflow_hidden()
            .children(toast_elements)
            .into_any_element()
    }
}

/// Render a single toast notification
fn render_toast(toast: ToastData, cx: &mut Context<ToastState>) -> impl IntoElement {
    let theme = cx.theme();
    let toast_id = toast.id;

    // Get variant colors
    let variant_color = get_variant_color(&toast.variant);
    let bg_color = variant_color.opacity(0.1);
    let border_color = variant_color.opacity(0.3);
    let icon = get_variant_icon(&toast.variant);

    let fg_color = theme.fg();
    let fg_muted_color = theme.fg_muted();
    let surface_color = theme.surface();
    let surface_secondary = theme.surface_secondary();

    let has_title = toast.title.is_some();
    let action = toast.action.clone();

    div()
        .id(("toast", toast_id))
        .w_full()
        .flex()
        .gap(rems(0.75))
        .p(rems(0.75))
        .bg(surface_color)
        .border_1()
        .border_color(border_color)
        .rounded(rems(0.5))
        .shadow_lg()
        // Left color bar for visual distinction
        .child(
            div()
                .flex_none()
                .w(px(4.0))
                .h_full()
                .min_h(px(32.0))
                .rounded(px(2.0))
                .bg(variant_color),
        )
        // Icon
        .child(
            div()
                .flex_none()
                .pt(px(2.0))
                .child(icon.size(px(16.0)).text_color(variant_color)),
        )
        // Content
        .child(
            div()
                .flex_1()
                .flex()
                .flex_col()
                .gap(rems(0.25))
                .min_w_0()
                // Title
                .when_some(toast.title.clone(), |content, title| {
                    content.child(
                        div()
                            .text_sm()
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .text_color(fg_color)
                            .child(title),
                    )
                })
                // Message
                .child(
                    div()
                        .text_sm()
                        .text_color(if has_title { fg_muted_color } else { fg_color })
                        .child(toast.message.clone()),
                )
                // Action button
                .when_some(action, |content, action| {
                    let handler = action.handler.clone();
                    content.child(
                        div().mt_1().child(
                            div()
                                .id(("toast-action", toast_id))
                                .flex()
                                .px_2()
                                .py(px(4.0))
                                .text_xs()
                                .font_weight(gpui::FontWeight::MEDIUM)
                                .text_color(variant_color)
                                .rounded(px(4.0))
                                .cursor_pointer()
                                .hover(|style: gpui::StyleRefinement| style.bg(bg_color))
                                .on_mouse_down(MouseButton::Left, |_, window, _| {
                                    window.prevent_default()
                                })
                                .on_click(cx.listener(move |state, _, window, cx| {
                                    (handler)(window, cx);
                                    state.dismiss_toast(toast_id, cx);
                                }))
                                .child(action.label.clone()),
                        ),
                    )
                }),
        )
        // Dismiss button
        .child(
            div()
                .id(("toast-dismiss", toast_id))
                .flex_none()
                .size(px(20.0))
                .flex()
                .items_center()
                .justify_center()
                .rounded(px(4.0))
                .cursor_pointer()
                .hover(|d: gpui::StyleRefinement| d.bg(surface_secondary.opacity(0.5)))
                .on_mouse_down(MouseButton::Left, |_, window, _| window.prevent_default())
                .on_click(cx.listener(move |state, _, _window, cx| {
                    state.dismiss_toast(toast_id, cx);
                }))
                .child(Icons::cross_1().size(px(12.0)).text_color(fg_muted_color)),
        )
}

/// Get the color for a variant
fn get_variant_color(variant: &AlertVariant) -> Hsla {
    match variant {
        AlertVariant::Default => hsla(0.0, 0.0, 0.5, 1.0),          // gray
        AlertVariant::Info => hsla(210.0 / 360.0, 0.7, 0.5, 1.0),   // blue
        AlertVariant::Success => hsla(142.0 / 360.0, 0.7, 0.4, 1.0), // green
        AlertVariant::Warning => hsla(38.0 / 360.0, 0.9, 0.5, 1.0), // orange/yellow
        AlertVariant::Destructive => hsla(0.0, 0.7, 0.5, 1.0),      // red
    }
}

/// Get the icon for a variant
fn get_variant_icon(variant: &AlertVariant) -> Svg {
    match variant {
        AlertVariant::Default => Icons::info_circled(),
        AlertVariant::Info => Icons::info_circled(),
        AlertVariant::Success => Icons::check_circled(),
        AlertVariant::Warning => Icons::exclamation_triangle(),
        AlertVariant::Destructive => Icons::cross_circled(),
    }
}

/// Global wrapper for ToastManager
#[derive(Clone)]
pub struct GlobalToastManager(pub Arc<ToastManager>);

impl Global for GlobalToastManager {}

/// Extension trait for App to create toasts
pub trait ToastExt {
    /// Create a new toast notification
    ///
    /// # Example
    /// ```ignore
    /// cx.toast("Message saved").success().show();
    /// ```
    fn toast(&mut self, message: impl Into<SharedString>) -> ToastBuilder<'_>;

    /// Get the toast state entity for rendering
    fn toast_state(&self) -> Entity<ToastState>;
}

impl ToastExt for App {
    fn toast(&mut self, message: impl Into<SharedString>) -> ToastBuilder<'_> {
        ToastBuilder::new(self, message)
    }

    fn toast_state(&self) -> Entity<ToastState> {
        self.global::<GlobalToastManager>().0.state.clone()
    }
}

/// Initialize the toast system. Call this in your app initialization.
///
/// # Example
/// ```ignore
/// fn main() {
///     Application::new()
///         .with_assets(gpuikit::assets())
///         .run(|cx| {
///             gpuikit::init(cx);
///             // Toast is now initialized automatically
///             // ...
///         });
/// }
/// ```
pub fn init(cx: &mut App) {
    let state = cx.new(|_| ToastState::new());
    let manager = ToastManager::new(state);
    cx.set_global(GlobalToastManager(Arc::new(manager)));
}

/// Create a toast container element to render in your app's root view.
///
/// This should be rendered at the root level of your application to ensure
/// toasts appear above all other content.
///
/// # Example
/// ```ignore
/// impl Render for MyApp {
///     fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
///         div()
///             .size_full()
///             .relative()  // Required for toast positioning
///             .child(/* your app content */)
///             .child(toast_container(cx))
///     }
/// }
/// ```
pub fn toast_container(cx: &App) -> Entity<ToastState> {
    cx.toast_state()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_toast_position_default() {
        assert_eq!(ToastPosition::default(), ToastPosition::TopRight);
    }

    #[test]
    fn test_variant_colors() {
        // Ensure all variants have defined colors
        let variants = [
            AlertVariant::Default,
            AlertVariant::Info,
            AlertVariant::Success,
            AlertVariant::Warning,
            AlertVariant::Destructive,
        ];

        for variant in variants {
            let color = get_variant_color(&variant);
            assert!(color.a > 0.0, "Variant {:?} should have visible color", variant);
        }
    }

    #[test]
    fn test_toast_action_creation() {
        let action = ToastAction::new("Undo", |_, _| {});
        assert_eq!(action.label, SharedString::from("Undo"));
    }
}
