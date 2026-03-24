//! Popover component for positioned overlay content.
//!
//! A general-purpose popover that positions content relative to a trigger element.
//! Supports configurable anchor edges, auto-flip on viewport bounds, and close behaviors.
//!
//! # Example
//!
//! ```ignore
//! use gpuikit::elements::popover::{popover, PopoverState};
//! use gpuikit::elements::button::button;
//! use gpuikit::layout::v_stack;
//! use gpuikit::traits::portal::AnchorEdge;
//!
//! let popover_state = cx.new(|_cx| PopoverState::new(
//!     popover("my-popover")
//!         .trigger(|_window, _cx| button("trigger", "Open Popover").into_any_element())
//!         .content(|_window, _cx| {
//!             v_stack()
//!                 .p_4()
//!                 .child("Popover content goes here")
//!                 .into_any_element()
//!         })
//!         .preferred_edge(AnchorEdge::Bottom)
//! ));
//! ```

use crate::theme::{ActiveTheme, Themeable};
use crate::traits::portal::{AnchorEdge, Portal, PortalPosition};
use gpui::{
    actions, anchored, deferred, div, point, prelude::*, px, AnyElement, App, Context,
    DismissEvent, ElementId, Entity, EventEmitter, FocusHandle, Focusable, Hsla, IntoElement,
    KeyBinding, ParentElement, Pixels, Render, Styled, Window,
};
use std::rc::Rc;

actions!(popover, [Close]);

/// The key context used for popover keybindings.
pub const POPOVER_CONTEXT: &str = "Popover";

/// Event emitted when the popover is opened.
pub struct PopoverOpened;

/// Event emitted when the popover is closed.
pub struct PopoverClosed;

/// Builder for creating a popover component.
///
/// Use the [`popover`] function to create an instance.
pub struct Popover {
    id: ElementId,
    trigger: Option<Rc<dyn Fn(&mut Window, &mut App) -> AnyElement>>,
    content: Option<Rc<dyn Fn(&mut Window, &mut App) -> AnyElement>>,
    position: PortalPosition,
    close_on_escape: bool,
    close_on_click_outside: bool,
    show_arrow: bool,
}

/// Creates a new popover builder.
///
/// # Arguments
///
/// * `id` - Unique identifier for the popover
///
/// # Example
///
/// ```ignore
/// popover("my-popover")
///     .trigger(|_, _| button("btn", "Click me").into_any_element())
///     .content(|_, _| div().child("Content").into_any_element())
/// ```
pub fn popover(id: impl Into<ElementId>) -> Popover {
    Popover::new(id)
}

impl Popover {
    /// Create a new popover builder.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            trigger: None,
            content: None,
            position: PortalPosition::new()
                .preferred_edge(AnchorEdge::Bottom)
                .offset(point(px(0.), px(4.))),
            close_on_escape: true,
            close_on_click_outside: true,
            show_arrow: false,
        }
    }

    /// Set the trigger element that opens/closes the popover on click.
    pub fn trigger(
        mut self,
        trigger: impl Fn(&mut Window, &mut App) -> AnyElement + 'static,
    ) -> Self {
        self.trigger = Some(Rc::new(trigger));
        self
    }

    /// Set the content to display in the popover.
    pub fn content(
        mut self,
        content: impl Fn(&mut Window, &mut App) -> AnyElement + 'static,
    ) -> Self {
        self.content = Some(Rc::new(content));
        self
    }

    /// Configure whether pressing Escape closes the popover.
    pub fn close_on_escape(mut self, close: bool) -> Self {
        self.close_on_escape = close;
        self
    }

    /// Configure whether clicking outside closes the popover.
    pub fn close_on_click_outside(mut self, close: bool) -> Self {
        self.close_on_click_outside = close;
        self
    }

    /// Show an arrow pointing toward the trigger element.
    pub fn show_arrow(mut self, show: bool) -> Self {
        self.show_arrow = show;
        self
    }
}

impl Portal for Popover {
    fn position(&self) -> &PortalPosition {
        &self.position
    }

    fn with_position(mut self, position: PortalPosition) -> Self {
        self.position = position;
        self
    }
}

/// The popover content panel that handles focus and dismissal.
pub struct PopoverPanel {
    content: Rc<dyn Fn(&mut Window, &mut App) -> AnyElement>,
    focus_handle: FocusHandle,
    close_on_escape: bool,
    show_arrow: bool,
    preferred_edge: AnchorEdge,
}

impl EventEmitter<DismissEvent> for PopoverPanel {}

impl Focusable for PopoverPanel {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl PopoverPanel {
    pub fn build(
        content: Rc<dyn Fn(&mut Window, &mut App) -> AnyElement>,
        close_on_escape: bool,
        show_arrow: bool,
        preferred_edge: AnchorEdge,
        window: &mut Window,
        cx: &mut App,
    ) -> Entity<Self> {
        cx.new(|cx| {
            let focus_handle = cx.focus_handle();
            window.focus(&focus_handle, cx);
            Self {
                content,
                focus_handle,
                close_on_escape,
                show_arrow,
                preferred_edge,
            }
        })
    }

    fn dismiss(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        cx.emit(DismissEvent);
    }

    fn handle_close(&mut self, _: &Close, _window: &mut Window, cx: &mut Context<Self>) {
        if self.close_on_escape {
            cx.emit(DismissEvent);
        }
    }
}

impl Render for PopoverPanel {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Extract theme colors before borrowing cx mutably
        let theme = cx.theme();
        let surface_color = theme.surface();
        let border_color = theme.border();

        let focus_handle = self.focus_handle.clone();
        let show_arrow = self.show_arrow;
        let preferred_edge = self.preferred_edge;

        // Now we can call content which borrows cx mutably
        let content = (self.content)(window, cx);

        let arrow_size = px(8.);

        div()
            .id("popover-panel")
            .key_context(POPOVER_CONTEXT)
            .track_focus(&focus_handle)
            .on_action(cx.listener(Self::handle_close))
            .on_mouse_down_out(cx.listener(|this, _, window, cx| {
                this.dismiss(window, cx);
            }))
            .flex()
            .flex_col()
            // Position arrow based on edge
            .when(show_arrow && preferred_edge == AnchorEdge::Bottom, |this| {
                this.child(render_arrow(arrow_size, preferred_edge, surface_color, border_color))
            })
            .child(
                div()
                    .bg(surface_color)
                    .border_1()
                    .border_color(border_color)
                    .rounded_md()
                    .shadow_lg()
                    .child(content),
            )
            .when(show_arrow && preferred_edge == AnchorEdge::Top, |this| {
                this.child(render_arrow(arrow_size, preferred_edge, surface_color, border_color))
            })
    }
}

/// Render an arrow pointing toward the trigger.
///
/// The arrow is rendered as a small rotated square that appears to point
/// toward the trigger element.
fn render_arrow(size: Pixels, edge: AnchorEdge, bg: Hsla, border: Hsla) -> impl IntoElement {
    // For simplicity, render a small triangular indicator using CSS borders
    // This creates a simple arrow effect without complex transforms
    div()
        .flex()
        .justify_center()
        .child(
            div()
                .w(size)
                .h(size / 2.)
                .bg(bg)
                .border_l_1()
                .border_r_1()
                .border_color(border)
                .when(edge == AnchorEdge::Bottom, |this| {
                    // Arrow pointing up (popover below trigger)
                    this.border_t_1()
                })
                .when(edge == AnchorEdge::Top, |this| {
                    // Arrow pointing down (popover above trigger)
                    this.border_b_1()
                }),
        )
}

/// Stateful popover component that manages open/close state.
///
/// Create using [`Popover`] and wrap in an Entity:
///
/// ```ignore
/// let state = cx.new(|_cx| PopoverState::new(popover("my-popover").trigger(...).content(...)));
/// ```
pub struct PopoverState {
    id: ElementId,
    trigger: Option<Rc<dyn Fn(&mut Window, &mut App) -> AnyElement>>,
    content: Option<Rc<dyn Fn(&mut Window, &mut App) -> AnyElement>>,
    position: PortalPosition,
    close_on_escape: bool,
    #[allow(dead_code)]
    close_on_click_outside: bool,
    show_arrow: bool,
    panel: Option<Entity<PopoverPanel>>,
}

impl EventEmitter<PopoverOpened> for PopoverState {}
impl EventEmitter<PopoverClosed> for PopoverState {}
impl EventEmitter<DismissEvent> for PopoverState {}

impl PopoverState {
    /// Create a new popover state from a Popover builder.
    pub fn new(popover: Popover) -> Self {
        Self {
            id: popover.id,
            trigger: popover.trigger,
            content: popover.content,
            position: popover.position,
            close_on_escape: popover.close_on_escape,
            close_on_click_outside: popover.close_on_click_outside,
            show_arrow: popover.show_arrow,
            panel: None,
        }
    }

    /// Check if the popover is currently open.
    pub fn is_open(&self) -> bool {
        self.panel.is_some()
    }

    /// Open the popover.
    pub fn open(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.panel.is_some() {
            return;
        }

        let Some(content) = self.content.clone() else {
            return;
        };

        let panel = PopoverPanel::build(
            content,
            self.close_on_escape,
            self.show_arrow,
            self.position.preferred_edge,
            window,
            cx,
        );

        cx.subscribe_in(
            &panel,
            window,
            |this, _, _event: &DismissEvent, _window, cx| {
                this.panel = None;
                cx.emit(PopoverClosed);
                cx.emit(DismissEvent);
                cx.notify();
            },
        )
        .detach();

        self.panel = Some(panel);
        cx.emit(PopoverOpened);
        cx.notify();
    }

    /// Close the popover.
    pub fn close(&mut self, cx: &mut Context<Self>) {
        if self.panel.is_none() {
            return;
        }

        self.panel = None;
        cx.emit(PopoverClosed);
        cx.emit(DismissEvent);
        cx.notify();
    }

    /// Toggle the popover open/closed state.
    pub fn toggle(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.is_open() {
            self.close(cx);
        } else {
            self.open(window, cx);
        }
    }

    /// Get the current portal position configuration.
    pub fn position(&self) -> &PortalPosition {
        &self.position
    }

    /// Set the preferred edge for the popover.
    pub fn set_preferred_edge(&mut self, edge: AnchorEdge, cx: &mut Context<Self>) {
        self.position = self.position.clone().preferred_edge(edge);
        cx.notify();
    }
}

impl Render for PopoverState {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let trigger = self.trigger.clone();
        let preferred_edge = self.position.preferred_edge;

        div()
            .id(self.id.clone())
            .relative()
            .child(
                div()
                    .id("popover-trigger")
                    .cursor_pointer()
                    .on_click(cx.listener(|this, _, window, cx| {
                        this.toggle(window, cx);
                    }))
                    .when_some(trigger, |this, trigger| this.child(trigger(window, cx))),
            )
            .when_some(self.panel.clone(), |this, panel| {
                // Use margins to offset based on preferred edge, similar to Dropdown
                let anchored_content = match preferred_edge {
                    AnchorEdge::Bottom => div().occlude().mt_1().child(panel),
                    AnchorEdge::Top => div().occlude().mb_1().child(panel),
                    AnchorEdge::Left => div().occlude().mr_1().child(panel),
                    AnchorEdge::Right => div().occlude().ml_1().child(panel),
                };

                this.child(
                    deferred(anchored().snap_to_window().child(anchored_content)).with_priority(1),
                )
            })
    }
}

/// Binds the popover keybindings to the application.
///
/// Call this in your application's initialization to enable escape-to-close functionality.
///
/// # Example
///
/// ```ignore
/// use gpuikit::elements::popover::bind_popover_keys;
///
/// fn main() {
///     Application::new().run(|cx| {
///         bind_popover_keys(cx);
///         // ... rest of app initialization
///     });
/// }
/// ```
pub fn bind_popover_keys(cx: &mut App) {
    cx.bind_keys([KeyBinding::new("escape", Close, Some(POPOVER_CONTEXT))]);
}
