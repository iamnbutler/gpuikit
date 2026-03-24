//! Popover component for positioning overlay content relative to a trigger element.
//!
//! # Example
//!
//! ```ignore
//! use gpuikit::elements::popover::{popover, PopoverState};
//! use gpuikit::elements::button::button;
//! use gpuikit::traits::portal::AnchorEdge;
//! use gpuikit::layout::v_stack;
//!
//! let popover_state = cx.new(|_cx| PopoverState::new(
//!     popover("my-popover")
//!         .trigger(button("open", "Open"))
//!         .content(|window, cx| {
//!             v_stack()
//!                 .child("Popover content")
//!                 .into_any_element()
//!         })
//!         .preferred_edge(AnchorEdge::Bottom)
//! ));
//! ```

use crate::icons::Icons;
use crate::theme::{ActiveTheme, Themeable};
use crate::traits::portal::AnchorEdge;
use gpui::{
    actions, anchored, deferred, div, prelude::*, px, AnyElement, App, Context, DismissEvent,
    ElementId, EventEmitter, FocusHandle, Focusable, IntoElement, KeyBinding, ParentElement,
    Pixels, Render, SharedString, Styled, Window,
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
    trigger: Option<AnyElement>,
    content: Option<Rc<dyn Fn(&mut Window, &mut App) -> AnyElement>>,
    preferred_edge: AnchorEdge,
    offset: Pixels,
    close_on_escape: bool,
    close_on_click_outside: bool,
    show_arrow: bool,
    title: Option<SharedString>,
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
///     .trigger(button("open", "Open"))
///     .content(|window, cx| div().child("Content").into_any_element())
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
            preferred_edge: AnchorEdge::Bottom,
            offset: px(4.),
            close_on_escape: true,
            close_on_click_outside: true,
            show_arrow: false,
            title: None,
        }
    }

    /// Set the trigger element that opens the popover on click.
    pub fn trigger(mut self, trigger: impl IntoElement) -> Self {
        self.trigger = Some(trigger.into_any_element());
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

    /// Set the preferred edge for positioning the popover relative to the trigger.
    ///
    /// The popover will automatically flip to the opposite edge if it would
    /// overflow the viewport.
    pub fn preferred_edge(mut self, edge: AnchorEdge) -> Self {
        self.preferred_edge = edge;
        self
    }

    /// Set the offset distance between the trigger and the popover content.
    pub fn offset(mut self, offset: Pixels) -> Self {
        self.offset = offset;
        self
    }

    /// Configure whether pressing Escape closes the popover.
    pub fn close_on_escape(mut self, close: bool) -> Self {
        self.close_on_escape = close;
        self
    }

    /// Configure whether clicking outside the popover closes it.
    pub fn close_on_click_outside(mut self, close: bool) -> Self {
        self.close_on_click_outside = close;
        self
    }

    /// Show an arrow pointing toward the trigger element.
    pub fn show_arrow(mut self, show: bool) -> Self {
        self.show_arrow = show;
        self
    }

    /// Set an optional title for the popover header.
    pub fn title(mut self, title: impl Into<SharedString>) -> Self {
        self.title = Some(title.into());
        self
    }
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
    trigger: Option<AnyElement>,
    content: Option<Rc<dyn Fn(&mut Window, &mut App) -> AnyElement>>,
    preferred_edge: AnchorEdge,
    offset: Pixels,
    close_on_escape: bool,
    close_on_click_outside: bool,
    show_arrow: bool,
    title: Option<SharedString>,
    is_open: bool,
    focus_handle: Option<FocusHandle>,
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
            preferred_edge: popover.preferred_edge,
            offset: popover.offset,
            close_on_escape: popover.close_on_escape,
            close_on_click_outside: popover.close_on_click_outside,
            show_arrow: popover.show_arrow,
            title: popover.title,
            is_open: false,
            focus_handle: None,
        }
    }

    /// Check if the popover is currently open.
    pub fn is_open(&self) -> bool {
        self.is_open
    }

    /// Open the popover.
    pub fn open(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.is_open {
            return;
        }

        self.is_open = true;
        let focus_handle = cx.focus_handle();
        window.focus(&focus_handle, cx);
        self.focus_handle = Some(focus_handle);
        cx.emit(PopoverOpened);
        cx.notify();
    }

    /// Close the popover.
    pub fn close(&mut self, cx: &mut Context<Self>) {
        if !self.is_open {
            return;
        }

        self.is_open = false;
        self.focus_handle = None;
        cx.emit(PopoverClosed);
        cx.emit(DismissEvent);
        cx.notify();
    }

    /// Toggle the popover open/closed state.
    pub fn toggle(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.is_open {
            self.close(cx);
        } else {
            self.open(window, cx);
        }
    }

    fn handle_close(&mut self, _: &Close, _window: &mut Window, cx: &mut Context<Self>) {
        if self.close_on_escape {
            self.close(cx);
        }
    }

    /// Get the content margin based on the preferred edge.
    fn get_content_margin(&self) -> (Pixels, Pixels, Pixels, Pixels) {
        let offset = self.offset;
        match self.preferred_edge {
            AnchorEdge::Top => (px(0.), px(0.), offset, px(0.)),    // mb
            AnchorEdge::Bottom => (offset, px(0.), px(0.), px(0.)), // mt
            AnchorEdge::Left => (px(0.), offset, px(0.), px(0.)),   // mr
            AnchorEdge::Right => (px(0.), px(0.), px(0.), offset),  // ml
        }
    }

    /// Render the arrow/pointer element.
    fn render_arrow_element(
        edge: AnchorEdge,
        surface_color: gpui::Hsla,
        border_color: gpui::Hsla,
    ) -> impl IntoElement {
        // Arrow size
        let arrow_size = px(8.);

        div()
            .absolute()
            .size(arrow_size)
            .bg(surface_color)
            .border_1()
            .border_color(border_color)
            // Position based on edge
            .when(edge == AnchorEdge::Top, |d| {
                d.bottom(px(-4.))
                    .left(px(12.))
                    .border_t_0()
                    .border_l_0()
            })
            .when(edge == AnchorEdge::Bottom, |d| {
                d.top(px(-4.)).left(px(12.)).border_b_0().border_r_0()
            })
            .when(edge == AnchorEdge::Left, |d| {
                d.right(px(-4.)).top(px(12.)).border_t_0().border_r_0()
            })
            .when(edge == AnchorEdge::Right, |d| {
                d.left(px(-4.)).top(px(12.)).border_b_0().border_l_0()
            })
    }
}

impl Focusable for PopoverState {
    fn focus_handle(&self, cx: &App) -> FocusHandle {
        self.focus_handle
            .clone()
            .unwrap_or_else(|| cx.focus_handle())
    }
}

impl Render for PopoverState {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let focus_handle = self.focus_handle.clone();
        let close_on_click_outside = self.close_on_click_outside;
        let show_arrow = self.show_arrow;
        let title = self.title.clone();
        let content = self.content.clone();
        let preferred_edge = self.preferred_edge;
        let (mt, mr, mb, ml) = self.get_content_margin();

        let surface_color = theme.surface();
        let border_color = theme.border();
        let fg_color = theme.fg();
        let fg_muted_color = theme.fg_muted();

        // Pre-render arrow if needed
        let arrow_element = if show_arrow {
            Some(Self::render_arrow_element(preferred_edge, surface_color, border_color))
        } else {
            None
        };

        // Pre-render content if open
        let rendered_content = if self.is_open {
            content.as_ref().map(|c| c(window, cx))
        } else {
            None
        };

        div()
            .relative()
            // Trigger area
            .child(
                div()
                    .id(self.id.clone())
                    .cursor_pointer()
                    .on_click(cx.listener(|this, _, window, cx| {
                        this.toggle(window, cx);
                    }))
                    .when_some(self.trigger.take(), |this, trigger| this.child(trigger)),
            )
            // Popover content (when open)
            .when(self.is_open, |this| {
                this.child(
                    deferred(
                        anchored().child(
                            div()
                                .key_context(POPOVER_CONTEXT)
                                .when_some(focus_handle, |this, handle| {
                                    this.track_focus(&handle)
                                        .on_action(cx.listener(Self::handle_close))
                                })
                                // Handle click outside
                                .when(close_on_click_outside, |this| {
                                    this.on_mouse_down_out(cx.listener(|this, _, _window, cx| {
                                        this.close(cx);
                                    }))
                                })
                                .occlude()
                                .mt(mt)
                                .mr(mr)
                                .mb(mb)
                                .ml(ml)
                                // Popover panel styling
                                .child(
                                    div()
                                        .relative()
                                        .min_w(px(120.))
                                        .max_w(px(400.))
                                        .bg(surface_color)
                                        .border_1()
                                        .border_color(border_color)
                                        .rounded_md()
                                        .shadow_lg()
                                        .flex()
                                        .flex_col()
                                        .overflow_hidden()
                                        // Arrow
                                        .when_some(arrow_element, |this, arrow| this.child(arrow))
                                        // Header with title
                                        .when_some(title.clone(), |this, title| {
                                            this.child(
                                                div()
                                                    .flex()
                                                    .items_center()
                                                    .justify_between()
                                                    .px_3()
                                                    .py_2()
                                                    .border_b_1()
                                                    .border_color(border_color)
                                                    .child(
                                                        div()
                                                            .text_sm()
                                                            .font_weight(
                                                                gpui::FontWeight::SEMIBOLD,
                                                            )
                                                            .text_color(fg_color)
                                                            .child(title),
                                                    )
                                                    .child(
                                                        div()
                                                            .id("popover-close")
                                                            .flex_none()
                                                            .size(px(20.))
                                                            .flex()
                                                            .items_center()
                                                            .justify_center()
                                                            .rounded(px(4.))
                                                            .cursor_pointer()
                                                            .hover(|style| {
                                                                style.bg(border_color.opacity(0.5))
                                                            })
                                                            .on_click(cx.listener(
                                                                |this, _, _window, cx| {
                                                                    this.close(cx);
                                                                },
                                                            ))
                                                            .child(
                                                                Icons::cross_1()
                                                                    .size(px(12.))
                                                                    .text_color(fg_muted_color),
                                                            ),
                                                    ),
                                            )
                                        })
                                        // Content
                                        .when_some(rendered_content, |this, content| {
                                            this.child(div().p_3().child(content))
                                        }),
                                ),
                        ),
                    )
                    .with_priority(1),
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
