//! Popover component for positioning overlay content relative to a trigger element.
//!
//! # Example
//!
//! ```ignore
//! use gpuikit::elements::popover::{popover, PopoverState};
//!
//! let popover_state = cx.new(|_cx| {
//!     PopoverState::new(
//!         popover("my-popover")
//!             .trigger(|_window, _cx| {
//!                 button("open", "Open Popover").into_any_element()
//!             })
//!             .content(|_window, _cx| {
//!                 v_stack()
//!                     .p_4()
//!                     .child("Popover content")
//!                     .into_any_element()
//!             })
//!     )
//! });
//! ```

use crate::theme::{ActiveTheme, Themeable};
use gpui::{
    anchored, deferred, div, point, prelude::*, px, AnyElement, App, Context, DismissEvent,
    ElementId, Entity, EventEmitter, FocusHandle, Focusable, IntoElement, ParentElement, Pixels,
    Point, Render, Styled, Window,
};
use std::rc::Rc;

/// A render callback that produces an element.
pub type RenderCallback = Rc<dyn Fn(&mut Window, &mut App) -> AnyElement>;

/// Event emitted when the popover open state changes.
pub struct PopoverChanged {
    pub open: bool,
}

/// The content panel that displays inside the popover.
pub struct PopoverPanel {
    content_render: RenderCallback,
    focus_handle: FocusHandle,
}

impl EventEmitter<DismissEvent> for PopoverPanel {}

impl Focusable for PopoverPanel {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl PopoverPanel {
    pub fn build(
        content_render: RenderCallback,
        window: &mut Window,
        cx: &mut App,
    ) -> Entity<Self> {
        cx.new(|cx| {
            let focus_handle = cx.focus_handle();
            window.focus(&focus_handle, cx);
            Self {
                content_render,
                focus_handle,
            }
        })
    }

    fn dismiss(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        cx.emit(DismissEvent);
    }
}

impl Render for PopoverPanel {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let focus_handle = self.focus_handle.clone();

        let theme = cx.theme();
        let surface_color = theme.surface();
        let border_color = theme.border();

        let content = (self.content_render)(window, cx);

        div()
            .id("popover-panel")
            .track_focus(&focus_handle)
            .on_mouse_down_out(cx.listener(|this, _, window, cx| {
                this.dismiss(window, cx);
            }))
            .on_key_down(cx.listener(|this, event: &gpui::KeyDownEvent, window, cx| {
                if event.keystroke.key == "escape" {
                    this.dismiss(window, cx);
                }
            }))
            .flex()
            .flex_col()
            .bg(surface_color)
            .border_1()
            .border_color(border_color)
            .rounded_md()
            .shadow_lg()
            .overflow_hidden()
            .child(content)
    }
}

/// Builder for creating a Popover component.
///
/// Use the [`popover`] function to create an instance.
pub struct Popover {
    id: ElementId,
    trigger_render: Option<RenderCallback>,
    content_render: Option<RenderCallback>,
    offset: Point<Pixels>,
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
///     .trigger(|_window, _cx| button("open", "Open").into_any_element())
///     .content(|_window, _cx| div().child("Content").into_any_element())
/// ```
pub fn popover(id: impl Into<ElementId>) -> Popover {
    Popover::new(id)
}

impl Popover {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            trigger_render: None,
            content_render: None,
            offset: point(px(0.), px(4.)),
        }
    }

    /// Set the trigger element via a render callback.
    ///
    /// The trigger element is what the user clicks to open/close the popover.
    pub fn trigger(
        mut self,
        render: impl Fn(&mut Window, &mut App) -> AnyElement + 'static,
    ) -> Self {
        self.trigger_render = Some(Rc::new(render));
        self
    }

    /// Set the popover content via a render callback.
    ///
    /// The content is displayed in the positioned overlay panel.
    pub fn content(
        mut self,
        render: impl Fn(&mut Window, &mut App) -> AnyElement + 'static,
    ) -> Self {
        self.content_render = Some(Rc::new(render));
        self
    }

    /// Set a custom offset from the trigger element.
    pub fn offset(mut self, offset: Point<Pixels>) -> Self {
        self.offset = offset;
        self
    }
}

/// Stateful popover component that manages the panel popup.
///
/// Create using [`Popover`] and wrap in an Entity:
///
/// ```ignore
/// let state = cx.new(|_cx| PopoverState::new(popover(...)));
/// ```
pub struct PopoverState {
    id: ElementId,
    trigger_render: Option<RenderCallback>,
    content_render: Option<RenderCallback>,
    offset: Point<Pixels>,
    open: bool,
    panel: Option<Entity<PopoverPanel>>,
}

impl EventEmitter<PopoverChanged> for PopoverState {}

impl PopoverState {
    pub fn new(popover: Popover) -> Self {
        Self {
            id: popover.id,
            trigger_render: popover.trigger_render,
            content_render: popover.content_render,
            offset: popover.offset,
            open: false,
            panel: None,
        }
    }

    /// Check if the popover is currently open.
    pub fn is_open(&self) -> bool {
        self.open
    }

    /// Open the popover.
    pub fn open(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.open {
            return;
        }
        self.show_panel(window, cx);
    }

    /// Close the popover.
    pub fn close(&mut self, cx: &mut Context<Self>) {
        if !self.open {
            return;
        }
        self.open = false;
        self.panel = None;
        cx.emit(PopoverChanged { open: false });
        cx.notify();
    }

    /// Toggle the popover open/closed.
    pub fn toggle(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.open {
            self.close(cx);
        } else {
            self.show_panel(window, cx);
        }
    }

    fn show_panel(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(content_render) = self.content_render.clone() else {
            return;
        };

        let panel = PopoverPanel::build(
            content_render,
            window,
            cx,
        );

        cx.subscribe_in(
            &panel,
            window,
            |this, _, _event: &DismissEvent, _window, cx| {
                this.open = false;
                this.panel = None;
                cx.emit(PopoverChanged { open: false });
                cx.notify();
            },
        )
        .detach();

        self.panel = Some(panel);
        self.open = true;
        cx.emit(PopoverChanged { open: true });
        cx.notify();
    }
}

impl Render for PopoverState {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let trigger_element = if let Some(ref render) = self.trigger_render {
            Some(render(window, cx))
        } else {
            None
        };

        let offset = self.offset;

        div()
            .relative()
            .child(
                div()
                    .id(self.id.clone())
                    .cursor_pointer()
                    .on_click(cx.listener(|this, _, window, cx| {
                        this.toggle(window, cx);
                    }))
                    .when_some(trigger_element, |this, trigger| this.child(trigger)),
            )
            .when_some(self.panel.clone(), |this, panel| {
                this.child(
                    deferred(
                        anchored().child(
                            div()
                                .occlude()
                                .mt(offset.y)
                                .ml(offset.x)
                                .child(panel),
                        ),
                    )
                    .with_priority(1),
                )
            })
    }
}
