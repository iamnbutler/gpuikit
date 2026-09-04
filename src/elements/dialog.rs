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

use crate::a11y::{A11y, Announce};
use crate::element_id::scoped;
use crate::elements::button::button;
use crate::elements::icon_button::icon_button;
use crate::icons::Icons;
use crate::layout::h_stack;
use crate::theme::{ActiveTheme, ControlSize, Themeable};
use crate::traits::accessible::Accessible;
use crate::traits::control_sized::ControlSized;
use gpui::{
    AnyElement, App, Context, DismissEvent, ElementId, EventEmitter, FocusHandle, Focusable,
    IntoElement, KeyBinding, ParentElement, Render, Role, SharedString, Styled, Window, actions,
    deferred, div, prelude::*, px,
};
use std::rc::Rc;

actions!(dialog, [Close]);

/// The key context used for dialog keybindings.
pub const DIALOG_CONTEXT: &str = "Dialog";

/// Event emitted when the dialog is opened.
pub struct DialogOpened;

/// Event emitted when the dialog is closed.
pub struct DialogClosed;

/// Event emitted when a confirmation dialog's confirming answer was chosen.
///
/// Only [`DialogState::confirm`] emits this, and only the confirm button calls
/// it. Escape, the backdrop and the header's close button all route through
/// `dismiss`, so there is no path in this module from a stray key or click to
/// a confirmation.
pub struct DialogConfirmed;

/// Event emitted when a confirmation dialog was answered "no" — by the cancel
/// button, by Escape, or by a click on the backdrop.
pub struct DialogCancelled;

/// The confirmation half of a [`Dialog`]: a question, and two answers of which
/// one destroys something.
///
/// Built by [`Dialog::confirm`], which takes the question and its detail
/// together — that pairing is the point. Everywhere else in the builder the
/// title and description are independent `Option`s, and a confirmation with
/// one and not the other is a dialog that either asks nothing or explains
/// nothing. The refining builders ([`Dialog::confirm_label`] and friends)
/// deliberately **refuse** to create one, so `.confirm_label("Delete")` on a
/// plain dialog cannot conjure an alert with no question in it.
///
/// # `destructive` defaults to `true`
///
/// A caller who has not thought about it gets the louder answer rather than
/// two identical buttons, because the dialog this element exists for is the
/// one guarding something unrecoverable. That default has a cost worth
/// knowing: a library whose every confirmation is red teaches people that red
/// means nothing. A confirmation that merely needs acknowledging — "Save
/// changes?", "Continue?" — should say `Dialog::destructive(false)`, and
/// that is what the flag is for.
#[derive(Clone)]
pub struct Confirmation {
    confirm_label: SharedString,
    cancel_label: SharedString,
    destructive: bool,
    on_confirm: Option<Rc<dyn Fn(&mut Window, &mut App)>>,
    on_cancel: Option<Rc<dyn Fn(&mut Window, &mut App)>>,
}

impl Default for Confirmation {
    fn default() -> Self {
        Self {
            // "Confirm" is the fallback, not the recommendation: name the verb
            // — "Delete", "Discard", "Revoke" — so the answer is readable
            // without the question.
            confirm_label: "Confirm".into(),
            cancel_label: "Cancel".into(),
            destructive: true,
            on_confirm: None,
            on_cancel: None,
        }
    }
}

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
    confirmation: Option<Confirmation>,
    size: ControlSize,
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
            confirmation: None,
            size: ControlSize::default(),
        }
    }

    /// Turn this into a **confirmation**: a question, its consequence, and two
    /// answers.
    ///
    /// One call, because the question and the detail are a pair — see
    /// [`Confirmation`]. It also turns the header's close button off: Cancel
    /// is already on screen, and two ways to say no in the same corner is one
    /// too many.
    ///
    /// ```ignore
    /// dialog("delete-project")
    ///     .confirm("Delete this project?", "Its 42 tasks are deleted with it. This cannot be undone.")
    ///     .confirm_label("Delete")
    ///     .on_confirm(|_window, _cx| { /* … */ })
    /// ```
    pub fn confirm(
        mut self,
        question: impl Into<SharedString>,
        consequence: impl Into<SharedString>,
    ) -> Self {
        self.title = Some(question.into());
        self.description = Some(consequence.into());
        self.confirmation = Some(self.confirmation.take().unwrap_or_default());
        self.show_close_button = false;
        self
    }

    /// Refine the confirmation, or complain in a debug build that there is
    /// none to refine.
    ///
    /// This is what keeps `.confirm_label("Delete")` from producing an alert
    /// dialog with no question in it: the refining builders can only ever
    /// adjust a confirmation [`Dialog::confirm`] already made.
    fn with_confirmation(mut self, refine: impl FnOnce(&mut Confirmation)) -> Self {
        debug_assert!(
            self.confirmation.is_some(),
            "this builder refines a confirmation and does not create one — call \
             `.confirm(question, consequence)` first, so the dialog has something to \
             confirm"
        );
        if let Some(confirmation) = self.confirmation.as_mut() {
            refine(confirmation);
        }
        self
    }

    /// Label the confirming answer. Defaults to "Confirm"; name the verb.
    pub fn confirm_label(self, label: impl Into<SharedString>) -> Self {
        let label = label.into();
        self.with_confirmation(|confirmation| confirmation.confirm_label = label)
    }

    /// Label the safe answer. Defaults to "Cancel".
    pub fn cancel_label(self, label: impl Into<SharedString>) -> Self {
        let label = label.into();
        self.with_confirmation(|confirmation| confirmation.cancel_label = label)
    }

    /// Whether the confirming answer is styled as destructive. Defaults to
    /// `true` — see [`Confirmation`] for the argument, and for why a
    /// confirmation that merely needs acknowledging should say `false`.
    pub fn destructive(self, destructive: bool) -> Self {
        self.with_confirmation(|confirmation| confirmation.destructive = destructive)
    }

    /// What to run when the confirming answer is chosen.
    pub fn on_confirm(self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        let handler = Rc::new(handler);
        self.with_confirmation(|confirmation| confirmation.on_confirm = Some(handler))
    }

    /// What to run when the dialog is cancelled — by the button, by Escape, or
    /// by the backdrop.
    pub fn on_cancel(self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        let handler = Rc::new(handler);
        self.with_confirmation(|confirmation| confirmation.on_cancel = Some(handler))
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
    ///
    /// Passing `false` to a **confirmation** dialog voids the "Escape cancels"
    /// promise this module's docs make for it: the Escape handler gates on this
    /// flag, so Escape becomes inert and the only way out is a footer button.
    /// Turn it off for a plain dialog if you like; leave it on for a
    /// confirmation.
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

/// The rung sizes the two action buttons of a confirmation and the gap between
/// them, and nothing else.
///
/// A dialog is not a row control: taken literally, "every dimension from the
/// rung" would give a modal surface a 20px height. The panel's own padding
/// stays component-specific, which is what `src/theme/control.rs`'s "What
/// belongs here" note asks for.
impl ControlSized for Dialog {
    fn control_size(mut self, size: ControlSize) -> Self {
        self.size = size;
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
    confirmation: Option<Confirmation>,
    size: ControlSize,
    is_open: bool,
    focus_handle: Option<FocusHandle>,
    /// The safe answer's handle, minted alongside the root's and focused after
    /// it, so a confirmation opens on Cancel rather than on Delete.
    cancel_focus_handle: Option<FocusHandle>,
}

impl EventEmitter<DialogOpened> for DialogState {}
impl EventEmitter<DialogClosed> for DialogState {}
impl EventEmitter<DialogConfirmed> for DialogState {}
impl EventEmitter<DialogCancelled> for DialogState {}
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
            confirmation: dialog.confirmation,
            size: dialog.size,
            is_open: false,
            focus_handle: None,
            cancel_focus_handle: None,
        }
    }

    /// Check if the dialog is currently open.
    pub fn is_open(&self) -> bool {
        self.is_open
    }

    /// Whether this dialog asks a question with two answers.
    pub fn is_confirmation(&self) -> bool {
        self.confirmation.is_some()
    }

    /// Open the dialog.
    pub fn open(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.is_open {
            return;
        }

        self.is_open = true;
        let focus_handle = cx.focus_handle();
        // The root is focused first and tracked by the element below, because
        // it is what carries `DIALOG_CONTEXT` and the `Close` action. Focus
        // then moves down to the safe answer, which is a descendant of that
        // element — so Escape still dispatches to this view.
        window.focus(&focus_handle, cx);
        self.focus_handle = Some(focus_handle);

        if self.is_confirmation() {
            let cancel = cx.focus_handle();
            window.focus(&cancel, cx);
            self.cancel_focus_handle = Some(cancel);
        }

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
        self.cancel_focus_handle = None;
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

    /// Choose the confirming answer: close, emit [`DialogConfirmed`], run the
    /// handler.
    ///
    /// A no-op on a closed dialog and on a dialog that is not a confirmation,
    /// so a second press of a button that is no longer on screen cannot run
    /// the handler twice.
    pub fn confirm(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if !self.is_open {
            return;
        }
        let Some(handler) = self
            .confirmation
            .as_ref()
            .map(|confirmation| confirmation.on_confirm.clone())
        else {
            return;
        };

        self.close(cx);
        cx.emit(DialogConfirmed);
        if let Some(handler) = handler {
            handler(window, cx);
        }
    }

    /// Choose the safe answer: close, emit [`DialogCancelled`], run the
    /// handler. A no-op on a closed dialog.
    pub fn cancel(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if !self.is_open {
            return;
        }
        let handler = self
            .confirmation
            .as_ref()
            .and_then(|confirmation| confirmation.on_cancel.clone());

        self.close(cx);
        cx.emit(DialogCancelled);
        if let Some(handler) = handler {
            handler(window, cx);
        }
    }

    /// The one way out that is not the confirm button.
    ///
    /// Escape, the backdrop and the header's close button all come through
    /// here: a confirmation is cancelled, a plain dialog is closed. Keeping
    /// them on one path is what guarantees there is no route from a key or a
    /// stray click to [`DialogConfirmed`].
    fn dismiss(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.is_confirmation() {
            self.cancel(window, cx);
        } else {
            self.close(cx);
        }
    }

    fn handle_close(&mut self, _: &Close, window: &mut Window, cx: &mut Context<Self>) {
        if self.close_on_escape {
            self.dismiss(window, cx);
        }
    }
}

/// A confirmation announces `Role::AlertDialog`, named by its question and
/// described by its consequence. A plain dialog announces **nothing** — see
/// the note on [`Render for DialogState`](DialogState#impl-Render-for-DialogState)
/// below and the paragraph in `render`.
impl Accessible for DialogState {
    fn a11y(&self) -> A11y {
        let role = if self.is_confirmation() {
            Role::AlertDialog
        } else {
            Role::Dialog
        };

        let mut a11y = A11y::new(role);
        if let Some(title) = self.title.clone() {
            a11y = a11y.name(title);
        }
        if let Some(description) = self.description.clone() {
            a11y = a11y.description(description);
        }
        a11y
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
        let cancel_focus_handle = self.cancel_focus_handle.clone();
        let close_on_backdrop_click = self.close_on_backdrop_click;
        let show_close_button = self.show_close_button;
        let title = self.title.clone();
        let description = self.description.clone();
        let content = self.content.clone();
        let confirmation = self.confirmation.clone();
        // A caller-supplied footer is *ignored* in confirm mode rather than
        // stacked under the two answers: a confirmation with three buttons is
        // no longer a confirmation.
        let footer = self.footer.clone().filter(|_| confirmation.is_none());
        let size = self.size;
        let footer_gap = theme.control(size).gap;

        // What the panel announces, and why a plain dialog announces nothing.
        //
        // `ELEMENTS_WITHOUT_A_ROLE` used to excuse this module with: *"would be
        // Role::Dialog with a required name and `modal`; gpui has no
        // `aria_modal` builder, and a dialog that announces itself unmodal is
        // worse than one that waits."* gpui still has no `aria_modal` — the
        // `aria_*` family on `Div` is label, description, keyshortcuts,
        // active_descendant, selected, expanded, toggled, the numeric-value
        // family, orientation, level, position_in_set, size_of_set and the
        // row/column family, and nothing may reach around `A11y` to fake one.
        //
        // So that sentence is answered for one half and left standing for the
        // other. A **confirmation** announces `Role::AlertDialog` anyway: an
        // alert that a screen reader hears about, unmodal, conveys more than
        // silence, and the affordance is the whole reason this mode exists. A
        // **plain dialog** still waits — the recorded reason applies to it
        // unchanged, and announcing an unmodal `Role::Dialog` is exactly what
        // this module wrote down as worse than waiting. When `aria_modal`
        // lands upstream, the guard below is the one line to delete.
        let announcement = confirmation.is_some().then(|| self.a11y());

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
                        cx.listener(|this, _event, window, cx| {
                            // This will be prevented from closing if clicking on the panel.
                            // `dismiss`, not `close`: a backdrop click on a
                            // confirmation is a "no", not a shrug.
                            this.dismiss(window, cx);
                        }),
                    )
                })
                // Dialog panel
                .child(
                    div()
                        // Was unique only because it sits under the backdrop's
                        // `.id(self.id)`; it derives from that id directly now.
                        .id(scoped(&self.id, "panel"))
                        // On the panel, not the scrim: the panel is what the
                        // title names.
                        .when_some(announcement, |this, a11y| this.announce(a11y))
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
                                            icon_button(
                                                scoped(&self.id, "close"),
                                                Icons::cross_1(),
                                            )
                                            .on_click(
                                                cx.listener(|this, _, window, cx| {
                                                    this.dismiss(window, cx);
                                                }),
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
                                    .when(
                                        content.is_none()
                                            && footer.is_none()
                                            && confirmation.is_none(),
                                        |this| this.pb_4(),
                                    )
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
                                    .when(footer.is_none() && confirmation.is_none(), |this| {
                                        this.pb_4()
                                    })
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
                        })
                        // The confirmation's own footer: the safe answer
                        // first, and the only route in this module to
                        // `DialogConfirmed`.
                        .when_some(confirmation, |this, confirmation| {
                            this.child(
                                h_stack()
                                    .justify_end()
                                    .gap(footer_gap)
                                    .p_4()
                                    .child(
                                        button(
                                            scoped(&self.id, "cancel"),
                                            confirmation.cancel_label.clone(),
                                        )
                                        .control_size(size)
                                        .when_some(cancel_focus_handle, |cancel, handle| {
                                            cancel.focus_handle(handle)
                                        })
                                        .on_click(
                                            cx.listener(|this, _, window, cx| {
                                                this.cancel(window, cx);
                                            }),
                                        ),
                                    )
                                    .child(
                                        button(
                                            scoped(&self.id, "confirm"),
                                            confirmation.confirm_label.clone(),
                                        )
                                        .control_size(size)
                                        .when(confirmation.destructive, |confirm| {
                                            confirm.destructive()
                                        })
                                        .on_click(
                                            cx.listener(|this, _, window, cx| {
                                                this.confirm(window, cx);
                                            }),
                                        ),
                                    ),
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

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::TestAppContext;
    use std::cell::RefCell;

    fn confirmation() -> Dialog {
        dialog("delete-project").confirm(
            "Delete this project?",
            "Its 42 tasks are deleted with it. This cannot be undone.",
        )
    }

    /// The whole announcement, compared as one value.
    ///
    /// `A11y` exposes no description getter but does derive `PartialEq`, so
    /// comparing the built value is how the description is pinned at all.
    #[test]
    fn a_confirmation_announces_an_alert_dialog_named_by_its_question() {
        let state = DialogState::new(confirmation());

        assert!(state.is_confirmation());
        assert_eq!(
            state.a11y(),
            A11y::new(Role::AlertDialog)
                .name("Delete this project?")
                .description("Its 42 tasks are deleted with it. This cannot be undone.")
        );
    }

    /// A plain dialog is `Role::Dialog` — and is deliberately **not**
    /// announced, which is the half of `ELEMENTS_WITHOUT_A_ROLE`'s deleted
    /// reason that still stands: gpui has no `aria_modal`, and an unmodal
    /// `Role::Dialog` is what this crate wrote down as worse than waiting.
    /// `render` guards the announcement on there being a confirmation.
    #[test]
    fn a_plain_dialog_is_not_announced() {
        let state = DialogState::new(dialog("plain").title("Settings"));

        assert!(!state.is_confirmation());
        assert_eq!(state.a11y(), A11y::new(Role::Dialog).name("Settings"));
    }

    /// The guard is not dead code: an untitled dialog has nothing to announce,
    /// and `role_requires_a_name` covers `Role::Dialog`, so announcing it
    /// would trip `announce`'s own `debug_assert!`.
    #[test]
    fn an_untitled_dialog_has_no_name_to_announce() {
        assert!(
            DialogState::new(dialog("plain"))
                .a11y()
                .is_missing_a_required_name()
        );
    }

    /// The refining builders adjust a confirmation and refuse to create one,
    /// so `.confirm_label("Delete")` alone cannot produce an alert dialog with
    /// no question in it.
    #[test]
    #[should_panic(expected = "refines a confirmation and does not create one")]
    fn a_label_builder_does_not_conjure_a_confirmation() {
        let _ = dialog("plain").title("Settings").confirm_label("Delete");
    }

    #[test]
    fn the_confirming_answer_is_destructive_unless_told_otherwise() {
        let loud = DialogState::new(confirmation());
        assert!(loud.confirmation.as_ref().unwrap().destructive);

        let quiet = DialogState::new(confirmation().destructive(false));
        assert!(!quiet.confirmation.as_ref().unwrap().destructive);
    }

    #[test]
    fn a_destructive_button_reports_its_variant() {
        use crate::elements::button::{ButtonVariant, button};

        // `b.variant()` with no argument resolves to the inherent *builder*
        // and fails to compile; the getter is reached through the trait.
        let plain = button("save", "Save");
        assert_eq!(
            crate::traits::button::Button::variant(&plain),
            ButtonVariant::Filled
        );
        assert_eq!(
            crate::traits::button::Button::variant(&button("delete", "Delete").destructive()),
            ButtonVariant::Destructive
        );
    }

    /// Escape's route, taken as the action handler gpui dispatches `Close` to.
    ///
    /// A confirmation is *cancelled*, never confirmed. There is no path in
    /// this module from a key to `DialogConfirmed`: `handle_close`, the
    /// backdrop listener and the header's close button all call `dismiss`, and
    /// only the confirm button calls `confirm`.
    #[gpui::test]
    fn escape_cancels_a_confirmation_and_never_confirms_it(cx: &mut TestAppContext) {
        // `add_window` renders, and `render` reads the theme.
        cx.update(crate::theme::init);

        let confirmed = Rc::new(RefCell::new(0usize));
        let cancelled = Rc::new(RefCell::new(0usize));
        let (yes, no) = (confirmed.clone(), cancelled.clone());

        let window = cx.add_window(move |_window, _cx| {
            DialogState::new(
                confirmation()
                    .on_confirm(move |_, _| *yes.borrow_mut() += 1)
                    .on_cancel(move |_, _| *no.borrow_mut() += 1),
            )
        });

        window
            .update(cx, |state, window, cx| {
                state.open(window, cx);
                assert!(state.is_open());
                state.handle_close(&Close, window, cx);
            })
            .unwrap();

        assert_eq!(*confirmed.borrow(), 0, "Escape confirmed a destruction");
        assert_eq!(*cancelled.borrow(), 1);
        window
            .update(cx, |state, _window, _cx| assert!(!state.is_open()))
            .unwrap();
    }

    /// `dismiss` is the one way out that is not the confirm button, and it
    /// plainly closes a dialog that has nothing to cancel.
    #[gpui::test]
    fn dismissing_a_plain_dialog_just_closes_it(cx: &mut TestAppContext) {
        // `add_window` renders, and `render` reads the theme.
        cx.update(crate::theme::init);

        let window =
            cx.add_window(|_window, _cx| DialogState::new(dialog("plain").title("Settings")));

        window
            .update(cx, |state, window, cx| {
                state.open(window, cx);
                state.dismiss(window, cx);
                assert!(!state.is_open());
            })
            .unwrap();
    }

    #[gpui::test]
    fn confirming_runs_its_handler_exactly_once(cx: &mut TestAppContext) {
        // `add_window` renders, and `render` reads the theme.
        cx.update(crate::theme::init);

        let ran = Rc::new(RefCell::new(0usize));
        let counter = ran.clone();

        let window = cx.add_window(move |_window, _cx| {
            DialogState::new(confirmation().on_confirm(move |_, _| *counter.borrow_mut() += 1))
        });

        window
            .update(cx, |state, window, cx| {
                state.open(window, cx);
                state.confirm(window, cx);
                // The dialog is off screen by now, but a second press of a
                // button that has not been re-rendered yet must not run the
                // handler again.
                state.confirm(window, cx);
                assert!(!state.is_open());
            })
            .unwrap();

        assert_eq!(*ran.borrow(), 1);
    }

    /// The property this feature exists for: a destructive question opens on
    /// the safe answer.
    #[gpui::test]
    fn a_confirmation_opens_with_the_safe_action_focused(cx: &mut TestAppContext) {
        // `add_window` renders, and `render` reads the theme.
        cx.update(crate::theme::init);

        let window = cx.add_window(|_window, _cx| DialogState::new(confirmation()));

        window
            .update(cx, |state, window, cx| {
                state.open(window, cx);

                let cancel = state
                    .cancel_focus_handle
                    .clone()
                    .expect("a confirmation mints a handle for its safe answer");
                assert_eq!(
                    window.focused(cx),
                    Some(cancel),
                    "a confirmation opened on something other than Cancel"
                );
            })
            .unwrap();
    }

    /// A plain dialog mints no second handle, so nothing is competing with the
    /// root for focus.
    #[gpui::test]
    fn a_plain_dialog_focuses_only_its_root(cx: &mut TestAppContext) {
        // `add_window` renders, and `render` reads the theme.
        cx.update(crate::theme::init);

        let window =
            cx.add_window(|_window, _cx| DialogState::new(dialog("plain").title("Settings")));

        window
            .update(cx, |state, window, cx| {
                state.open(window, cx);
                assert!(state.cancel_focus_handle.is_none());
                assert_eq!(window.focused(cx), state.focus_handle.clone());
            })
            .unwrap();
    }
}
