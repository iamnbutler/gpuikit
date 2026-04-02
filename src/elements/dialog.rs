//! Dialog component for modal interactions
//!
//! Provides a modal overlay with a semi-transparent backdrop, centered content panel,
//! and configurable close behavior (Escape key, backdrop click).
//!
//! # Example
//!
//! ```ignore
//! use gpuikit::elements::dialog::{dialog, DialogState};
//! use gpuikit::elements::button::button;
//! use gpuikit::layout::h_stack;
//!
//! let dialog_state = cx.new(|_cx| DialogState::new(
//!     dialog("confirm-dialog")
//!         .title("Are you sure?")
//!         .description("This action cannot be undone.")
//!         .content(|window, cx| {
//!             div().child("Custom body content here").into_any_element()
//!         })
//!         .footer(|window, cx| {
//!             h_stack()
//!                 .gap_2()
//!                 .child(button("cancel", "Cancel"))
//!                 .child(button("confirm", "Confirm"))
//!                 .into_any_element()
//!         })
//! ));
//!
//! // Open the dialog
//! dialog_state.update(cx, |state, cx| state.open(window, cx));
//! ```

use crate::icons::Icons;
use crate::theme::{ActiveTheme, Themeable};
use gpui::{
    actions, deferred, div, prelude::*, px, AnyElement, App, Context, DismissEvent, ElementId,
    EventEmitter, FocusHandle, Focusable, IntoElement, KeyBinding, ParentElement, Render,
    SharedString, Styled, Window,
};
use std::rc::Rc;

actions!(dialog, [Close]);

/// The key context used for dialog keybindings.
pub const DIALOG_CONTEXT: &str = "Dialog";

/// Event emitted when the dialog is opened.
pub struct DialogOpened;

/// Event emitted when the dialog is closed.
pub struct DialogClosed;

/// Builder for creating a dialog component.
///
/// Use the [`dialog`] function to create an instance.
pub struct Dialog {
    id: ElementId,
    title: Option<SharedString>,
    description: Option<SharedString>,
    content: Option<Rc<dyn Fn(&mut Window, &mut App) -> AnyElement>>,
    footer: Option<Rc<dyn Fn(&mut Window, &mut App) -> AnyElement>>,
    close_on_escape: bool,
    close_on_backdrop_click: bool,
    show_close_button: bool,
}

/// Creates a new dialog builder.
///
/// # Arguments
///
/// * `id` - Unique identifier for the dialog
pub fn dialog(id: impl Into<ElementId>) -> Dialog {
    Dialog::new(id)
}

impl Dialog {
    /// Create a new dialog builder.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            title: None,
            description: None,
            content: None,
            footer: None,
            close_on_escape: true,
            close_on_backdrop_click: true,
            show_close_button: true,
        }
    }

    /// Set the dialog title.
    pub fn title(mut self, title: impl Into<SharedString>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Set the dialog description.
    pub fn description(mut self, description: impl Into<SharedString>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Set arbitrary body content rendered between description and footer.
    pub fn content(
        mut self,
        content: impl Fn(&mut Window, &mut App) -> AnyElement + 'static,
    ) -> Self {
        self.content = Some(Rc::new(content));
        self
    }

    /// Set the footer content (typically action buttons).
    pub fn footer(
        mut self,
        footer: impl Fn(&mut Window, &mut App) -> AnyElement + 'static,
    ) -> Self {
        self.footer = Some(Rc::new(footer));
        self
    }

    /// Configure whether pressing Escape closes the dialog.
    pub fn close_on_escape(mut self, close: bool) -> Self {
        self.close_on_escape = close;
        self
    }

    /// Configure whether clicking the backdrop closes the dialog.
    pub fn close_on_backdrop_click(mut self, close: bool) -> Self {
        self.close_on_backdrop_click = close;
        self
    }

    /// Configure whether to show the close button in the header.
    pub fn show_close_button(mut self, show: bool) -> Self {
        self.show_close_button = show;
        self
    }
}

/// Stateful dialog component that manages open/close state.
///
/// Create using [`Dialog`] and wrap in an Entity:
///
/// ```ignore
/// let state = cx.new(|_cx| DialogState::new(dialog("my-dialog").title("Hello")));
/// ```
pub struct DialogState {
    id: ElementId,
    title: Option<SharedString>,
    description: Option<SharedString>,
    content: Option<Rc<dyn Fn(&mut Window, &mut App) -> AnyElement>>,
    footer: Option<Rc<dyn Fn(&mut Window, &mut App) -> AnyElement>>,
    close_on_escape: bool,
    close_on_backdrop_click: bool,
    show_close_button: bool,
    is_open: bool,
    focus_handle: Option<FocusHandle>,
}

impl EventEmitter<DialogOpened> for DialogState {}
impl EventEmitter<DialogClosed> for DialogState {}
impl EventEmitter<DismissEvent> for DialogState {}

impl DialogState {
    /// Create a new dialog state from a Dialog builder.
    pub fn new(dialog: Dialog) -> Self {
        Self {
            id: dialog.id,
            title: dialog.title,
            description: dialog.description,
            content: dialog.content,
            footer: dialog.footer,
            close_on_escape: dialog.close_on_escape,
            close_on_backdrop_click: dialog.close_on_backdrop_click,
            show_close_button: dialog.show_close_button,
            is_open: false,
            focus_handle: None,
        }
    }

    /// Check if the dialog is currently open.
    pub fn is_open(&self) -> bool {
        self.is_open
    }

    /// Open the dialog.
    pub fn open(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.is_open {
            return;
        }

        self.is_open = true;
        let focus_handle = cx.focus_handle();
        window.focus(&focus_handle, cx);
        self.focus_handle = Some(focus_handle);
        cx.emit(DialogOpened);
        cx.notify();
    }

    /// Close the dialog.
    pub fn close(&mut self, cx: &mut Context<Self>) {
        if !self.is_open {
            return;
        }

        self.is_open = false;
        self.focus_handle = None;
        cx.emit(DialogClosed);
        cx.emit(DismissEvent);
        cx.notify();
    }

    /// Toggle the dialog open/closed state.
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
}

impl Focusable for DialogState {
    fn focus_handle(&self, cx: &App) -> FocusHandle {
        self.focus_handle
            .clone()
            .unwrap_or_else(|| cx.focus_handle())
    }
}

impl Render for DialogState {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();

        if !self.is_open {
            return div().into_any_element();
        }

        let focus_handle = self.focus_handle.clone();
        let close_on_backdrop_click = self.close_on_backdrop_click;
        let show_close_button = self.show_close_button;
        let title = self.title.clone();
        let description = self.description.clone();
        let content = self.content.clone();
        let footer = self.footer.clone();

        let backdrop_color = theme.overlay();
        let surface_color = theme.surface();
        let border_color = theme.border();
        let fg_color = theme.fg();
        let fg_muted_color = theme.fg_muted();

        // Render the dialog overlay using deferred() for proper layering
        deferred(
            div()
                .id(self.id.clone())
                .key_context(DIALOG_CONTEXT)
                .when_some(focus_handle, |this, handle| {
                    this.track_focus(&handle)
                        .on_action(cx.listener(Self::handle_close))
                })
                // Full-screen backdrop using absolute positioning
                .absolute()
                .top_0()
                .left_0()
                .right_0()
                .bottom_0()
                .size_full()
                .flex()
                .items_center()
                .justify_center()
                .bg(backdrop_color)
                // Handle backdrop click
                .when(close_on_backdrop_click, |this| {
                    this.on_mouse_down(
                        gpui::MouseButton::Left,
                        cx.listener(|this, _event, _window, cx| {
                            // This will be prevented from closing if clicking on the panel
                            this.close(cx);
                        }),
                    )
                })
                // Dialog panel
                .child(
                    div()
                        .id("dialog-panel")
                        // Prevent backdrop click from closing when clicking on panel
                        .on_mouse_down(gpui::MouseButton::Left, |_, _, cx| {
                            cx.stop_propagation();
                        })
                        .min_w(px(320.))
                        .max_w(px(500.))
                        .bg(surface_color)
                        .border_1()
                        .border_color(border_color)
                        .rounded_lg()
                        .shadow_xl()
                        .flex()
                        .flex_col()
                        // Header
                        .when(title.is_some() || show_close_button, |this| {
                            this.child(
                                div()
                                    .flex()
                                    .items_start()
                                    .justify_between()
                                    .p_4()
                                    .when(
                                        description.is_some()
                                            || content.is_some()
                                            || footer.is_some(),
                                        |this| this.pb_0(),
                                    )
                                    // Title
                                    .when_some(title.clone(), |this, title| {
                                        this.child(
                                            div()
                                                .flex_1()
                                                .text_base()
                                                .font_weight(gpui::FontWeight::SEMIBOLD)
                                                .text_color(fg_color)
                                                .child(title),
                                        )
                                    })
                                    // Close button
                                    .when(show_close_button, |this| {
                                        this.child(
                                            div()
                                                .id("dialog-close")
                                                .flex_none()
                                                .size(px(24.))
                                                .flex()
                                                .items_center()
                                                .justify_center()
                                                .rounded(px(4.))
                                                .cursor_pointer()
                                                .hover(|style| {
                                                    style.bg(border_color.opacity(0.5))
                                                })
                                                .on_click(cx.listener(|this, _, _window, cx| {
                                                    this.close(cx);
                                                }))
                                                .child(
                                                    Icons::cross_1()
                                                        .size(px(14.))
                                                        .text_color(fg_muted_color),
                                                ),
                                        )
                                    }),
                            )
                        })
                        // Description
                        .when_some(description, |this, desc| {
                            this.child(
                                div()
                                    .px_4()
                                    .pt_2()
                                    .when(content.is_none() && footer.is_none(), |this| {
                                        this.pb_4()
                                    })
                                    .text_sm()
                                    .text_color(fg_muted_color)
                                    .child(desc),
                            )
                        })
                        // Content
                        .when_some(content, |this, content| {
                            this.child(
                                div()
                                    .px_4()
                                    .pt_2()
                                    .when(footer.is_none(), |this| this.pb_4())
                                    .child(content(window, cx)),
                            )
                        })
                        // Footer
                        .when_some(footer, |this, footer| {
                            this.child(
                                div()
                                    .flex()
                                    .justify_end()
                                    .gap_2()
                                    .p_4()
                                    .child(footer(window, cx)),
                            )
                        }),
                ),
        )
        .with_priority(10)
        .into_any_element()
    }
}

/// Binds the dialog keybindings to the application.
///
/// Call this in your application's initialization to enable escape-to-close functionality.
///
/// # Example
///
/// ```ignore
/// use gpuikit::elements::dialog::bind_dialog_keys;
///
/// fn main() {
///     Application::new().run(|cx| {
///         bind_dialog_keys(cx);
///         // ... rest of app initialization
///     });
/// }
/// ```
pub fn bind_dialog_keys(cx: &mut App) {
    cx.bind_keys([KeyBinding::new("escape", Close, Some(DIALOG_CONTEXT))]);
}
