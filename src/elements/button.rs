use crate::a11y::{A11y, Announce};
use crate::theme::{focus_ring, ActiveTheme, ControlSize, Themeable};
use crate::{
    layout::h_stack, traits, traits::accessible::Accessible, traits::control_sized::ControlSized,
    traits::disableable::Disableable,
};
use gpui::{
    prelude::FluentBuilder, AnyView, App, ClickEvent, ElementId, FocusHandle, FontWeight,
    InteractiveElement, IntoElement, MouseButton, ParentElement, RenderOnce, Role, SharedString,
    StatefulInteractiveElement, Styled, Window,
};

pub fn button(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Button {
    let label = label.into();
    let id = id.into();
    Button::new(id, label)
}

// todo: style through ButtonVariant
#[derive(Default)]
pub enum ButtonVariant {
    #[default]
    Filled,
}

#[derive(IntoElement)]
pub struct Button {
    id: ElementId,
    label: SharedString,
    disabled: bool,
    size: ControlSize,
    handler: Option<Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>>,
    tooltip: Option<Box<dyn Fn(&mut Window, &mut App) -> AnyView + 'static>>,
    focus_handle: Option<FocusHandle>,
}

impl Button {
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        let id = id.into();
        let label = label.into();

        Button {
            id,
            label,
            disabled: false,
            size: ControlSize::default(),
            handler: None,
            tooltip: None,
            focus_handle: None,
        }
    }

    pub fn on_click(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.handler = Some(Box::new(handler));
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn tooltip(mut self, tooltip: impl Fn(&mut Window, &mut App) -> AnyView + 'static) -> Self {
        self.tooltip = Some(Box::new(tooltip));
        self
    }

    /// Focus this button through a handle the caller owns, rather than the one
    /// gpui mints and keeps in the element's element state.
    ///
    /// Optional, and most callers want nothing here: a `RenderOnce` control is
    /// the same focus target across frames without anyone holding state — see
    /// `crate::a11y`'s module docs, section 4. Supply one when something else
    /// has to move focus *to* this button.
    pub fn focus_handle(mut self, handle: FocusHandle) -> Self {
        self.focus_handle = Some(handle);
        self
    }
}

/// The worked example for `crate::a11y`: the label is the accessible name, so
/// there is no second string to keep in step with it, and a button built with
/// an empty label is a `debug_assert!` rather than a control that announces
/// "button" and nothing else.
///
/// Nothing here reports `disabled` — gpui has no `aria_disabled` (see the
/// `a11y` module docs). What a disabled button does report is the absence of
/// `Action::Click`, because `render` drops the click handler below, and the
/// absence of a tab stop, because it declines focus here.
///
/// Keyboard focus is declared, not applied: `announce` is what turns it into a
/// tab stop, so the role and the focus answer cannot drift apart. A disabled
/// button leaving the tab order is the weaker of the two answers ARIA allows,
/// and is forced by gpui having no `aria_disabled` to announce the other with.
impl Accessible for Button {
    fn a11y(&self) -> A11y {
        let a11y = A11y::new(Role::Button).name(self.label.clone());

        if self.disabled {
            a11y.not_focusable("a disabled button has nothing for a keyboard to do")
        } else if let Some(handle) = self.focus_handle.clone() {
            a11y.focus_handle(handle)
        } else {
            a11y.focusable()
        }
    }
}

impl RenderOnce for Button {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let metrics = theme.control(self.size);
        // Taken before `self`'s fields are moved into the element below.
        let a11y = self.a11y();

        h_stack()
            .id(self.id)
            .announce(a11y)
            .h(metrics.height)
            .px(metrics.padding_x)
            .gap(metrics.gap)
            .flex_none()
            .items_center()
            .justify_center()
            .rounded(metrics.radius)
            .text_size(metrics.text_size)
            .line_height(metrics.line_height)
            .font_weight(FontWeight::MEDIUM)
            .bg(theme.button_bg())
            .text_color(theme.fg())
            .whitespace_nowrap()
            // `focus_visible`, not `focus`: clicking a button should not leave
            // a ring behind it. A spread shadow rather than a border, so
            // arriving focus does not resize the control — see
            // `theme::focus_ring`.
            .focus_visible(|style| style.shadow(focus_ring(theme.accent())))
            .when(!self.disabled, |button| {
                button
                    .hover(|div| div.bg(theme.button_bg_hover()))
                    .active(|div| div.bg(theme.button_bg_active()))
                    .cursor_pointer()
            })
            .when(self.disabled, |button| {
                button
                    .opacity(0.65)
                    .cursor_not_allowed()
                    .text_color(theme.fg_muted())
            })
            .when_some(
                self.handler.filter(|_| !self.disabled),
                |button, handler| {
                    button
                        .on_mouse_down(MouseButton::Left, |_, window, _| window.prevent_default())
                        .on_click(move |event, window, cx| {
                            cx.stop_propagation();
                            handler(event, window, cx)
                        })
                },
            )
            .when_some(self.tooltip, |button, tooltip| button.tooltip(tooltip))
            .child(self.label)
    }
}

impl traits::clickable::Clickable for Button {
    fn on_click(mut self, handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static) -> Self {
        self.handler = Some(Box::new(handler));
        self
    }
}

impl traits::button::Button for Button {
    type Variant = ButtonVariant;

    fn variant(&self) -> Self::Variant {
        ButtonVariant::default()
    }
}

impl Disableable for Button {
    fn is_disabled(&self) -> bool {
        self.disabled
    }

    fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

impl ControlSized for Button {
    fn control_size(mut self, size: ControlSize) -> Self {
        self.size = size;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::a11y::test_support::announced;
    use crate::a11y::FocusNavigation;
    use gpui::{
        accesskit, div, px, size, AnyElement, Context, ElementId, FocusHandle, KeyUpEvent,
        Keystroke, PlatformInput, Render, TestAppContext, VisualTestContext,
    };
    use std::cell::RefCell;
    use std::rc::Rc;

    /// The whole announcement: a role, and the label as the name.
    #[gpui::test]
    fn a_button_announces_its_label(cx: &mut TestAppContext) {
        cx.update(crate::theme::init);
        let cx = cx.add_empty_window();

        let announced = cx.update(|window, cx| announced(button("save", "Save"), window, cx));

        assert_eq!(announced.role, Some(Role::Button));
        assert_eq!(announced.name(), Some("Save"));
    }

    /// gpui builds a node only for an element that has *both* a role and an
    /// id, and it hashes the id path into the node id. So the role has to sit
    /// on the element carrying the caller's id, not on some inner box.
    #[gpui::test]
    fn the_role_sits_on_the_element_the_caller_named(cx: &mut TestAppContext) {
        cx.update(crate::theme::init);
        let cx = cx.add_empty_window();

        let announced = cx.update(|window, cx| announced(button("save", "Save"), window, cx));

        assert_eq!(announced.id, Some(ElementId::Name("save".into())));
        assert!(announced.node.is_some(), "an id and a role make a node");
    }

    /// With no `aria_disabled` in gpui, the click action is the only thing
    /// that tells the two apart — see the `a11y` module docs, section 3.
    #[gpui::test]
    fn a_disabled_button_offers_no_click_action(cx: &mut TestAppContext) {
        cx.update(crate::theme::init);
        let cx = cx.add_empty_window();

        let (enabled, disabled) = cx.update(|window, cx| {
            (
                announced(button("save", "Save").on_click(|_, _, _| {}), window, cx),
                announced(
                    button("save", "Save").on_click(|_, _, _| {}).disabled(true),
                    window,
                    cx,
                ),
            )
        });

        assert!(enabled.supports(accesskit::Action::Click));
        assert!(!disabled.supports(accesskit::Action::Click));
        assert_eq!(
            disabled.name(),
            Some("Save"),
            "a disabled button is still announced, just not actionable"
        );
    }

    /// What the element *declares*, as distinct from what reaches the tab
    /// order. `announced` cannot see the second — see the `a11y` module docs,
    /// section 5 — so the two halves are tested separately.
    #[test]
    fn a_button_declares_focus_and_a_disabled_one_declines_it_in_writing() {
        assert!(button("save", "Save").a11y().is_focusable());

        let disabled = button("save", "Save").disabled(true).a11y();
        assert!(!disabled.is_focusable());
        assert!(
            disabled.focus_declined_because().is_some(),
            "declining without a reason is how a decision becomes a way of silencing the \
             assertion"
        );
        assert!(!disabled.is_missing_a_focus_decision());
    }

    // --- the keyboard ---
    //
    // Everything below draws a real window and presses real keys, because
    // `announced` cannot see focus: it calls two `Element` methods and never
    // lays out or paints, so neither the minted handle nor the tab-stop
    // registration exists for it to look at. What the element *declares* is
    // checked through `Accessible` above; that the declaration reaches the tab
    // order is checked here.
    //
    // A real *view*, not `VisualTestContext::draw` — registering a mouse
    // listener reads `Window::current_view`, which is only set while a view
    // renders. That is the note `elements::sidebar`'s harness carries too.

    type Build = Box<dyn Fn(&mut Window, &mut App) -> AnyElement>;

    struct Harness {
        build: Build,
    }

    impl Render for Harness {
        fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            (self.build)(window, cx)
        }
    }

    /// Draw `build` in a real window with **no focus scaffolding at all**:
    /// nothing is focused and the root element tracks no handle. That is the
    /// state `tab_does_nothing_before_anything_is_focused` is about, and it is
    /// not the state an app is in.
    fn draw_bare(
        cx: &mut TestAppContext,
        build: impl Fn(&mut Window, &mut App) -> AnyElement + 'static,
    ) -> &'static mut VisualTestContext {
        cx.update(crate::init);

        let window = cx.open_window(size(px(400.), px(300.)), move |_window, _cx| Harness {
            build: Box::new(build),
        });
        let cx = VisualTestContext::from_window(*std::ops::Deref::deref(&window), cx).into_mut();
        cx.run_until_parked();
        cx
    }

    /// One drawn window, with the app-shaped focus scaffolding in place.
    struct Drawn {
        cx: &'static mut VisualTestContext,
        /// The root's handle, focused. Focus starts here and Tab moves it on,
        /// which is what `examples/showcase.rs` sets up for real.
        root: FocusHandle,
    }

    /// Draw `build` under a focused root that answers Tab.
    ///
    /// The scaffolding is not decoration. With *nothing* focused, gpui
    /// dispatches `FocusNext` to the node belonging to its own wrapper around
    /// the view, which is above every element in it — so the listener
    /// `announce` puts on the button is not in the dispatch path and the first
    /// Tab reaches nothing. An app answers this by tracking the handle it
    /// focuses at startup, and so does this.
    fn draw(
        cx: &mut TestAppContext,
        build: impl Fn(&mut Window, &mut App) -> AnyElement + 'static,
    ) -> Drawn {
        let slot: Rc<RefCell<Option<FocusHandle>>> = Rc::new(RefCell::new(None));
        let for_render = slot.clone();

        let cx = draw_bare(cx, move |window, app| {
            let root = for_render
                .borrow_mut()
                .get_or_insert_with(|| app.focus_handle())
                .clone();

            div()
                .id("harness-root")
                .track_focus(&root)
                .moves_focus_on_tab()
                .child(build(window, app))
                .into_any_element()
        });

        let root = slot.borrow().clone().expect("the harness drew");
        cx.update(|window, app| window.focus(&root, app));
        cx.run_until_parked();

        Drawn { cx, root }
    }

    /// Press and release `key`.
    ///
    /// `simulate_keystrokes` sends only `KeyDown`, and gpui's Enter/Space
    /// activation needs a matched key *up* on the same element before it
    /// synthesises a click.
    fn press(cx: &mut VisualTestContext, key: &str) {
        cx.simulate_keystrokes(key);
        let keystroke = Keystroke::parse(key).expect("a parseable keystroke");
        cx.update(|window, cx| {
            window.dispatch_event(PlatformInput::KeyUp(KeyUpEvent { keystroke }), cx);
        });
        cx.run_until_parked();
    }

    /// Whatever the window has focused, if anything.
    fn focused(cx: &mut VisualTestContext) -> Option<FocusHandle> {
        cx.update(|window, cx| window.focused(cx))
    }

    /// The gotcha, stated: Tab does nothing until something is focused, and a
    /// root that answers Tab but tracks no handle is not enough. The
    /// counterpart is `a_root_that_tracks_a_handle_answers_the_first_tab`.
    #[gpui::test]
    fn tab_does_nothing_before_anything_is_focused(cx: &mut TestAppContext) {
        let cx = draw_bare(cx, |_window, _cx| {
            div()
                .id("untracked-root")
                .moves_focus_on_tab()
                .child(button("save", "Save").on_click(|_, _, _| {}))
                .into_any_element()
        });

        assert!(focused(cx).is_none());
        cx.simulate_keystrokes("tab");
        cx.run_until_parked();

        assert!(
            focused(cx).is_none(),
            "with nothing focused, gpui dispatches above the root element, so a listener on \
             the root is not in the path — the root has to track a handle something focuses"
        );
    }

    #[gpui::test]
    fn tab_reaches_a_button(cx: &mut TestAppContext) {
        let Drawn { cx, root } = draw(cx, |_window, _cx| {
            button("save", "Save")
                .on_click(|_, _, _| {})
                .into_any_element()
        });

        assert_eq!(focused(cx), Some(root.clone()), "focus starts on the root");
        cx.simulate_keystrokes("tab");
        cx.run_until_parked();

        let now = focused(cx);
        assert!(now.is_some() && now != Some(root), "Tab did not reach the button. `announce` makes a focusable element a tab stop with `.tab_stop(true)`; a focusable element that is not a stop is walked straight past by `TabStopMap::next`");
    }

    /// The caller-handle path, which `track_focus` does *not* make a stop on
    /// its own — the handle has to be made one.
    #[gpui::test]
    fn tab_reaches_a_button_holding_a_caller_supplied_handle(cx: &mut TestAppContext) {
        let handle: Rc<RefCell<Option<FocusHandle>>> = Rc::new(RefCell::new(None));
        let for_render = handle.clone();

        let Drawn { cx, .. } = draw(cx, move |_window, cx| {
            let button_handle = for_render
                .borrow_mut()
                .get_or_insert_with(|| cx.focus_handle())
                .clone();
            button("save", "Save")
                .focus_handle(button_handle)
                .on_click(|_, _, _| {})
                .into_any_element()
        });

        cx.simulate_keystrokes("tab");
        cx.run_until_parked();

        let expected = handle.borrow().clone().expect("the button was rendered");
        assert_eq!(
            focused(cx),
            Some(expected),
            "Tab did not reach the caller's handle. `track_focus` does not push the element's \
             `tab_stop` onto the handle, so it has to be set on the handle itself"
        );
    }

    /// The disabled answer, all the way through: `not_focusable` is a decision
    /// and it keeps the control out of the tab order.
    #[gpui::test]
    fn tab_skips_a_disabled_button(cx: &mut TestAppContext) {
        let Drawn { cx, root } = draw(cx, |_window, _cx| {
            button("save", "Save")
                .disabled(true)
                .on_click(|_, _, _| {})
                .into_any_element()
        });

        cx.simulate_keystrokes("tab");
        cx.run_until_parked();

        assert_eq!(
            focused(cx),
            Some(root),
            "a disabled button is not a tab stop, so there was nowhere for focus to go — see \
             `Button`'s `Accessible` impl"
        );
    }

    #[gpui::test]
    fn tab_walks_from_one_button_to_the_next(cx: &mut TestAppContext) {
        let Drawn { cx, root } = draw(cx, |_window, _cx| {
            div()
                .child(button("first", "First").on_click(|_, _, _| {}))
                .child(button("second", "Second").on_click(|_, _, _| {}))
                .into_any_element()
        });

        cx.simulate_keystrokes("tab");
        cx.run_until_parked();
        let first = focused(cx);

        cx.simulate_keystrokes("tab");
        cx.run_until_parked();
        let second = focused(cx);

        assert!(first.is_some() && first != Some(root.clone()));
        assert!(second.is_some() && second != Some(root));
        assert_ne!(first, second, "the second Tab did not move focus on");
    }

    #[gpui::test]
    fn shift_tab_walks_back(cx: &mut TestAppContext) {
        let Drawn { cx, .. } = draw(cx, |_window, _cx| {
            div()
                .child(button("first", "First").on_click(|_, _, _| {}))
                .child(button("second", "Second").on_click(|_, _, _| {}))
                .into_any_element()
        });

        cx.simulate_keystrokes("tab");
        cx.run_until_parked();
        let first = focused(cx);

        cx.simulate_keystrokes("tab");
        cx.run_until_parked();
        let second = focused(cx);

        cx.simulate_keystrokes("shift-tab");
        cx.run_until_parked();
        let back = focused(cx);

        assert_ne!(first, second);
        assert_eq!(
            back, first,
            "Shift-Tab did not walk back to the first button"
        );
    }

    /// Enter and Space activation is gpui's, not this crate's: a focused
    /// element with a click listener synthesises a click from a matched key
    /// down and key up. What this pins is that focus arrives in the first
    /// place, which is the half `Button` was missing.
    #[gpui::test]
    fn enter_and_space_activate_a_focused_button(cx: &mut TestAppContext) {
        for key in ["enter", "space"] {
            let clicks = Rc::new(RefCell::new(0usize));
            let counter = clicks.clone();

            let Drawn { cx, .. } = draw(cx, move |_window, _cx| {
                let counter = counter.clone();
                button("save", "Save")
                    .on_click(move |_, _, _| *counter.borrow_mut() += 1)
                    .into_any_element()
            });

            cx.simulate_keystrokes("tab");
            cx.run_until_parked();
            press(cx, key);

            assert_eq!(*clicks.borrow(), 1, "{key} did not activate the button");
        }
    }

    /// The lifecycle question #173 raised, answered by gpui rather than by an
    /// API change: `Interactivity::request_layout` mints the handle and keeps
    /// it in element state "as long as frames contain an element with this id",
    /// so a `RenderOnce` control is the same focus target across frames without
    /// anyone above it holding one. That is why `focus_handle` is optional.
    #[gpui::test]
    fn focus_survives_a_redraw(cx: &mut TestAppContext) {
        let Drawn { cx, root } = draw(cx, |_window, _cx| {
            button("save", "Save")
                .on_click(|_, _, _| {})
                .into_any_element()
        });

        cx.simulate_keystrokes("tab");
        cx.run_until_parked();
        let before = focused(cx);
        assert!(before.is_some() && before != Some(root));

        cx.update(|window, _| window.refresh());
        cx.run_until_parked();

        assert_eq!(
            focused(cx),
            before,
            "the button lost focus across a redraw, so gpui is not keeping its handle in \
             element state and a caller would have to hold one"
        );
    }

    /// The other half of `tab_does_nothing_before_anything_is_focused`: put the
    /// listener on an element that tracks a handle something actually focuses,
    /// and the first Tab works. This is what `examples/showcase.rs` does, and
    /// what `draw` above stands in for.
    #[gpui::test]
    fn a_root_that_tracks_a_handle_answers_the_first_tab(cx: &mut TestAppContext) {
        let handle: Rc<RefCell<Option<FocusHandle>>> = Rc::new(RefCell::new(None));
        let for_render = handle.clone();

        let cx = draw_bare(cx, move |_window, cx| {
            let root = for_render
                .borrow_mut()
                .get_or_insert_with(|| cx.focus_handle())
                .clone();
            div()
                .id("root")
                .track_focus(&root)
                .moves_focus_on_tab()
                .child(button("save", "Save").on_click(|_, _, _| {}))
                .into_any_element()
        });

        let root = handle.borrow().clone().expect("the root was rendered");
        cx.update(|window, cx| window.focus(&root, cx));
        cx.run_until_parked();

        cx.simulate_keystrokes("tab");
        cx.run_until_parked();

        assert_ne!(
            focused(cx),
            Some(root),
            "Tab from the app's own focused root did not move focus into the window"
        );
    }
}
