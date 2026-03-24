//! Toast notification component for temporary messages
//!
//! Provides toast notifications that appear temporarily and auto-dismiss.
//! Toasts stack vertically and can be positioned in different corners.
//!
//! # Example
//!
//! ```ignore
//! use gpuikit::elements::toast::ToastExt;
//!
//! // Simple toast
//! cx.toast("Changes saved").success();
//!
//! // Toast with title and custom duration
//! cx.toast("Operation complete")
//!     .title("Success")
//!     .success()
//!     .duration(Duration::from_secs(5));
//!
//! // Toast with action button
//! cx.toast("File deleted")
//!     .warning()
//!     .action("Undo", |window, cx| {
//!         // Handle undo
//!     });
//! ```

use crate::icons::Icons;
use crate::theme::{ActiveTheme, Themeable};
use gpui::{
    deferred, div, hsla, prelude::FluentBuilder, px, rems, AnyElement, App, AppContext, ClickEvent,
    Context, ElementId, Entity, EventEmitter, Global, Hsla, InteractiveElement, IntoElement,
    MouseButton, ParentElement, Render, SharedString, StatefulInteractiveElement, Styled, Svg,
    Window,
};
use smol::Timer;
use std::rc::Rc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

/// Default duration for toast auto-dismiss (4 seconds)
const DEFAULT_DURATION: Duration = Duration::from_secs(4);

/// Toast variant determining severity/styling
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
    Error,
}

impl ToastVariant {
    /// Returns the default icon for this variant
    fn default_icon(&self) -> Svg {
        match self {
            ToastVariant::Default => Icons::info_circled(),
            ToastVariant::Info => Icons::info_circled(),
            ToastVariant::Success => Icons::check_circled(),
            ToastVariant::Warning => Icons::exclamation_triangle(),
            ToastVariant::Error => Icons::cross_circled(),
        }
    }

    /// Returns the icon/accent color for this variant
    fn color(&self) -> Hsla {
        match self {
            ToastVariant::Default => hsla(0.0, 0.0, 0.5, 1.0),          // gray
            ToastVariant::Info => hsla(210.0 / 360.0, 0.7, 0.5, 1.0),   // blue
            ToastVariant::Success => hsla(142.0 / 360.0, 0.7, 0.4, 1.0), // green
            ToastVariant::Warning => hsla(38.0 / 360.0, 0.9, 0.5, 1.0), // orange/yellow
            ToastVariant::Error => hsla(0.0, 0.7, 0.5, 1.0),            // red
        }
    }

    /// Returns the background color for this variant
    fn bg_color(&self) -> Hsla {
        self.color().opacity(0.1)
    }

    /// Returns the border color for this variant
    fn border_color(&self) -> Hsla {
        self.color().opacity(0.3)
    }
}

/// Position for toast container on screen
#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
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
    /// Top-center
    TopCenter,
    /// Bottom-center
    BottomCenter,
}

/// Internal toast entry in the queue
#[derive(Clone)]
struct ToastEntry {
    id: usize,
    title: Option<SharedString>,
    description: SharedString,
    variant: ToastVariant,
    show_icon: bool,
    duration: Option<Duration>,
    action: Option<ToastAction>,
    generation: usize,
}

/// Toast action button configuration
#[derive(Clone)]
struct ToastAction {
    label: SharedString,
    handler: Rc<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>,
}

/// Global toast manager that holds the notification queue
pub struct ToastManager {
    toasts: Vec<ToastEntry>,
    position: ToastPosition,
    next_id: AtomicUsize,
}

impl ToastManager {
    /// Create a new toast manager
    pub fn new() -> Self {
        Self {
            toasts: Vec::new(),
            position: ToastPosition::TopRight,
            next_id: AtomicUsize::new(0),
        }
    }

    /// Set the position for all toasts
    pub fn set_position(&mut self, position: ToastPosition) {
        self.position = position;
    }

    /// Get the current toast position
    pub fn position(&self) -> ToastPosition {
        self.position
    }

    /// Add a toast to the queue
    fn add_toast(&mut self, entry: ToastEntry) {
        self.toasts.push(entry);
    }

    /// Remove a toast by ID
    fn remove_toast(&mut self, id: usize) {
        self.toasts.retain(|t| t.id != id);
    }

    /// Get the next unique toast ID
    fn next_id(&self) -> usize {
        self.next_id.fetch_add(1, Ordering::SeqCst)
    }

    /// Check if there are any active toasts
    pub fn has_toasts(&self) -> bool {
        !self.toasts.is_empty()
    }

    /// Get the current toasts
    fn toasts(&self) -> &[ToastEntry] {
        &self.toasts
    }
}

impl Default for ToastManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Event emitted when a toast is dismissed
pub struct ToastDismissed {
    pub id: usize,
}

/// Stateful toast manager entity that handles rendering
pub struct ToastManagerState {
    manager: ToastManager,
}

impl EventEmitter<ToastDismissed> for ToastManagerState {}

impl ToastManagerState {
    /// Create a new toast manager state
    pub fn new() -> Self {
        Self {
            manager: ToastManager::new(),
        }
    }

    /// Set the position for all toasts
    pub fn set_position(&mut self, position: ToastPosition, cx: &mut Context<Self>) {
        self.manager.set_position(position);
        cx.notify();
    }

    /// Add a toast and schedule auto-dismiss
    fn add_toast(&mut self, entry: ToastEntry, cx: &mut Context<Self>) {
        let id = entry.id;
        let generation = entry.generation;
        let duration = entry.duration.unwrap_or(DEFAULT_DURATION);

        self.manager.add_toast(entry);
        cx.notify();

        // Schedule auto-dismiss
        cx.spawn(async move |this, cx| {
            Timer::after(duration).await;
            if let Some(this) = this.upgrade() {
                this.update(cx, |this, cx| {
                    // Only dismiss if the toast still exists with same generation
                    if let Some(toast) = this.manager.toasts.iter().find(|t| t.id == id) {
                        if toast.generation == generation {
                            this.dismiss_toast(id, cx);
                        }
                    }
                });
            }
        })
        .detach();
    }

    /// Dismiss a toast by ID
    pub fn dismiss_toast(&mut self, id: usize, cx: &mut Context<Self>) {
        self.manager.remove_toast(id);
        cx.emit(ToastDismissed { id });
        cx.notify();
    }

    /// Check if there are any active toasts
    pub fn has_toasts(&self) -> bool {
        self.manager.has_toasts()
    }
}

impl Default for ToastManagerState {
    fn default() -> Self {
        Self::new()
    }
}

impl Render for ToastManagerState {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if self.manager.toasts.is_empty() {
            return div().into_any_element();
        }

        let theme = cx.theme();
        let surface_color = theme.surface();
        let position = self.manager.position;

        // Build position styling
        let container = div()
            .id("toast-container")
            .absolute()
            .flex()
            .flex_col()
            .gap_2()
            .p_4()
            .max_w(px(420.));

        // Apply position styling
        let container = match position {
            ToastPosition::TopRight => container.top_0().right_0(),
            ToastPosition::TopLeft => container.top_0().left_0(),
            ToastPosition::BottomRight => container.bottom_0().right_0(),
            ToastPosition::BottomLeft => container.bottom_0().left_0(),
            ToastPosition::TopCenter => container.top_0().left_0().right_0().items_center(),
            ToastPosition::BottomCenter => container.bottom_0().left_0().right_0().items_center(),
        };

        // Render toasts
        let toasts: Vec<AnyElement> = self
            .manager
            .toasts()
            .iter()
            .map(|toast| {
                self.render_toast(toast.clone(), surface_color, cx)
            })
            .collect();

        deferred(container.children(toasts))
            .with_priority(9) // Below dialogs (10), above dropdowns (1)
            .into_any_element()
    }
}

impl ToastManagerState {
    fn render_toast(
        &self,
        toast: ToastEntry,
        surface_color: Hsla,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let theme = cx.theme();
        let fg_color = theme.fg();
        let fg_muted_color = theme.fg_muted();

        let variant_color = toast.variant.color();
        let bg_color = toast.variant.bg_color();
        let border_color = toast.variant.border_color();

        let icon = if toast.show_icon {
            Some(toast.variant.default_icon())
        } else {
            None
        };

        let has_title = toast.title.is_some();
        let toast_id = toast.id;

        div()
            .id(ElementId::NamedInteger("toast".into(), toast_id as u64))
            .min_w(px(300.))
            .max_w(px(400.))
            .flex()
            .gap(rems(0.75))
            .p(rems(0.75))
            .bg(surface_color)
            .border_1()
            .border_color(border_color)
            .rounded(rems(0.375))
            .shadow_lg()
            // Left accent bar
            .child(
                div()
                    .absolute()
                    .left_0()
                    .top_0()
                    .bottom_0()
                    .w(px(3.0))
                    .bg(variant_color)
                    .rounded_l(rems(0.375)),
            )
            .pl(rems(1.0)) // Extra padding for accent bar
            // Icon
            .when_some(icon, |container, icon: Svg| {
                container.child(
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
                    .when_some(toast.title.clone(), |content, title| {
                        content.child(
                            div()
                                .text_sm()
                                .font_weight(gpui::FontWeight::SEMIBOLD)
                                .text_color(fg_color)
                                .child(title),
                        )
                    })
                    // Description
                    .child(
                        div()
                            .text_sm()
                            .text_color(if has_title { fg_muted_color } else { fg_color })
                            .child(toast.description.clone()),
                    )
                    // Action button
                    .when_some(toast.action.clone(), |content, action| {
                        content.child(
                            div()
                                .id(ElementId::NamedInteger("toast-action".into(), toast_id as u64))
                                .mt_1()
                                .text_sm()
                                .font_weight(gpui::FontWeight::MEDIUM)
                                .text_color(variant_color)
                                .cursor_pointer()
                                .hover(|style| style.underline())
                                .on_click(move |event, window, cx| {
                                    (action.handler)(event, window, cx);
                                })
                                .child(action.label.clone()),
                        )
                    }),
            )
            // Close button
            .child(
                div()
                    .id(ElementId::NamedInteger("toast-close".into(), toast_id as u64))
                    .flex_none()
                    .size(px(20.0))
                    .flex()
                    .items_center()
                    .justify_center()
                    .rounded(px(4.0))
                    .cursor_pointer()
                    .hover(|div| div.bg(bg_color))
                    .on_mouse_down(MouseButton::Left, |_, window, _| window.prevent_default())
                    .on_click(cx.listener(move |this, _, _window, cx| {
                        this.dismiss_toast(toast_id, cx);
                    }))
                    .child(Icons::cross_1().size(px(12.0)).text_color(fg_muted_color)),
            )
            .into_any_element()
    }
}

/// Global wrapper for ToastManagerState entity
pub struct GlobalToastManager(pub Entity<ToastManagerState>);

impl Global for GlobalToastManager {}

/// Toast builder for creating notifications
pub struct Toast {
    title: Option<SharedString>,
    description: SharedString,
    variant: ToastVariant,
    show_icon: bool,
    duration: Option<Duration>,
    action: Option<ToastAction>,
}

impl Toast {
    /// Create a new toast with a description
    pub fn new(description: impl Into<SharedString>) -> Self {
        Self {
            title: None,
            description: description.into(),
            variant: ToastVariant::Default,
            show_icon: true,
            duration: None,
            action: None,
        }
    }

    /// Set the toast title
    pub fn title(mut self, title: impl Into<SharedString>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Set the toast variant
    pub fn variant(mut self, variant: ToastVariant) -> Self {
        self.variant = variant;
        self
    }

    /// Set to Info variant
    pub fn info(mut self) -> Self {
        self.variant = ToastVariant::Info;
        self
    }

    /// Set to Success variant
    pub fn success(mut self) -> Self {
        self.variant = ToastVariant::Success;
        self
    }

    /// Set to Warning variant
    pub fn warning(mut self) -> Self {
        self.variant = ToastVariant::Warning;
        self
    }

    /// Set to Error variant
    pub fn error(mut self) -> Self {
        self.variant = ToastVariant::Error;
        self
    }

    /// Hide the icon
    pub fn no_icon(mut self) -> Self {
        self.show_icon = false;
        self
    }

    /// Set the auto-dismiss duration
    pub fn duration(mut self, duration: Duration) -> Self {
        self.duration = Some(duration);
        self
    }

    /// Add an action button
    pub fn action(
        mut self,
        label: impl Into<SharedString>,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.action = Some(ToastAction {
            label: label.into(),
            handler: Rc::new(handler),
        });
        self
    }

    /// Show this toast (consumes the builder)
    pub fn show(self, cx: &mut App) {
        let manager = cx.global::<GlobalToastManager>().0.clone();
        manager.update(cx, |state, cx| {
            let id = state.manager.next_id();
            let entry = ToastEntry {
                id,
                title: self.title,
                description: self.description,
                variant: self.variant,
                show_icon: self.show_icon,
                duration: self.duration,
                action: self.action,
                generation: 0,
            };
            state.add_toast(entry, cx);
        });
    }
}

/// Extension trait for App to create toasts
pub trait ToastExt {
    /// Create a toast builder with the given description
    ///
    /// The toast is shown when the builder is dropped or when `.show()` is called.
    ///
    /// # Example
    ///
    /// ```ignore
    /// cx.toast("Changes saved").success();
    /// ```
    fn toast(&mut self, description: impl Into<SharedString>) -> ToastBuilder<'_>;

    /// Get the toast manager entity
    fn toast_manager(&self) -> Entity<ToastManagerState>;

    /// Set the toast position
    fn set_toast_position(&mut self, position: ToastPosition);
}

impl ToastExt for App {
    fn toast(&mut self, description: impl Into<SharedString>) -> ToastBuilder<'_> {
        ToastBuilder::new(self, description)
    }

    fn toast_manager(&self) -> Entity<ToastManagerState> {
        self.global::<GlobalToastManager>().0.clone()
    }

    fn set_toast_position(&mut self, position: ToastPosition) {
        let manager = self.global::<GlobalToastManager>().0.clone();
        manager.update(self, |state, cx| {
            state.set_position(position, cx);
        });
    }
}

/// Builder for creating and showing toasts
///
/// Call `.show()` to display the toast.
pub struct ToastBuilder<'a> {
    cx: &'a mut App,
    title: Option<SharedString>,
    description: SharedString,
    variant: ToastVariant,
    show_icon: bool,
    duration: Option<Duration>,
    action: Option<ToastAction>,
}

impl<'a> ToastBuilder<'a> {
    fn new(cx: &'a mut App, description: impl Into<SharedString>) -> Self {
        Self {
            cx,
            title: None,
            description: description.into(),
            variant: ToastVariant::Default,
            show_icon: true,
            duration: None,
            action: None,
        }
    }

    /// Set the toast title
    pub fn title(mut self, title: impl Into<SharedString>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Set the toast variant
    pub fn variant(mut self, variant: ToastVariant) -> Self {
        self.variant = variant;
        self
    }

    /// Set to Info variant
    pub fn info(mut self) -> Self {
        self.variant = ToastVariant::Info;
        self
    }

    /// Set to Success variant
    pub fn success(mut self) -> Self {
        self.variant = ToastVariant::Success;
        self
    }

    /// Set to Warning variant
    pub fn warning(mut self) -> Self {
        self.variant = ToastVariant::Warning;
        self
    }

    /// Set to Error variant
    pub fn error(mut self) -> Self {
        self.variant = ToastVariant::Error;
        self
    }

    /// Hide the icon
    pub fn no_icon(mut self) -> Self {
        self.show_icon = false;
        self
    }

    /// Set the auto-dismiss duration
    pub fn duration(mut self, duration: Duration) -> Self {
        self.duration = Some(duration);
        self
    }

    /// Add an action button
    pub fn action(
        mut self,
        label: impl Into<SharedString>,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.action = Some(ToastAction {
            label: label.into(),
            handler: Rc::new(handler),
        });
        self
    }

    /// Show the toast immediately (consumes the builder)
    pub fn show(self) {
        let toast = Toast {
            title: self.title,
            description: self.description,
            variant: self.variant,
            show_icon: self.show_icon,
            duration: self.duration,
            action: self.action,
        };
        toast.show(self.cx);
    }
}

/// Initialize the toast system
///
/// This must be called in your application's initialization.
///
/// # Example
///
/// ```ignore
/// use gpuikit::elements::toast;
///
/// fn main() {
///     Application::new().run(|cx| {
///         toast::init(cx);
///         // ... rest of app initialization
///     });
/// }
/// ```
pub fn init(cx: &mut App) {
    let manager = cx.new(|_cx| ToastManagerState::new());
    cx.set_global(GlobalToastManager(manager));
}

/// Renders the toast container
///
/// Add this to your root element to display toasts.
///
/// # Example
///
/// ```ignore
/// div()
///     .size_full()
///     .child(your_app_content)
///     .child(toast::toast_container(cx))
/// ```
pub fn toast_container(cx: &App) -> AnyElement {
    let manager = cx.global::<GlobalToastManager>().0.clone();
    manager.into_any_element()
}
