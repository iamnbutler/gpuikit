//! A panel docked to an edge of the window, with a collapsed rail.
//!
//! # What this is, and what it deliberately is not
//!
//! shadcn's Sidebar is roughly twenty exported parts — provider, trigger, rail,
//! inset, header, footer, group, group label, group action, menu, menu item,
//! menu button, sub-menu, skeleton. That is a layout framework wearing a
//! component's name, and most of those parts are `List`, `Button` and
//! `Separator` with a prefix. What was actually missing from this crate is
//! smaller: **a docked panel** — an edge, a width, a collapsed state, and a
//! push-versus-overlay behaviour.
//!
//! So there are no menu/group/header/footer sub-components here.
//! [`List`](crate::elements::list::List) (with
//! [`ListEntry::header`](crate::elements::list::ListEntry::header)),
//! [`Separator`](crate::elements::separator) and
//! [`Button`](crate::elements::button) are the contents, and both the
//! showcase's Sidebar page and the showcase's own navigation are composed
//! exactly that way. The one sub-component that ships,
//! [`SidebarTrigger`], ships for an accessibility reason: the panel reports
//! [`Role::Complementary`] with an accessible name, and `aria-expanded` has to
//! be on the control that changes the state.
//!
//! # State is the caller's
//!
//! The element stores nothing across frames. [`SidebarState`] is a value the
//! caller owns, which is what makes the width persistable and what lets the
//! same collapse button live anywhere in the app.
//!
//! ```ignore
//! sidebar("app-nav")
//!     .label("Navigation")
//!     .state(self.nav_open)
//!     .width(rems(12.5))
//!     .rail(
//!         v_stack()
//!             .child(sidebar_trigger("app-nav-rail-trigger", self.nav_open).on_click(toggle))
//!             .child(icon_button("nav-home", Icons::home())),
//!     )
//!     .child(sidebar_trigger("app-nav-trigger", self.nav_open).on_click(toggle))
//!     .child(List::new("nav", entries).render(window, cx))
//! ```
//!
//! The panel shows its children when expanded and only the rail when
//! collapsed. A trigger that lives among the children is gone the moment it
//! is used, so either the rail carries one too, or the trigger lives outside
//! the panel altogether.
//!
//! # Overlays
//!
//! The drawer at narrow widths follows `docs/overlays.md`: `deferred()` over
//! `anchored()`, `.occlude()` on the panel, and a draw priority that is a rung
//! of the ladder that document states.

use gpui::{
    AnyElement, App, ClickEvent, Div, ElementId, FocusHandle, IntoElement, ParentElement, Pixels,
    Rems, RenderOnce, Role, SharedString, Stateful, Styled, Svg, Window, anchored, deferred, div,
    point, prelude::*, px,
};

use crate::a11y::{A11y, Announce};
use crate::element_id::scoped;
use crate::icons::Icons;
use crate::theme::{ActiveTheme, ControlMetrics, ControlSize, Themeable, focus_ring};
use crate::traits::accessible::Accessible;
use crate::traits::clickable::Clickable;
use crate::traits::control_sized::ControlSized;
use crate::traits::disableable::Disableable;

/// The width a sidebar takes when nothing says otherwise.
const DEFAULT_WIDTH: Rems = Rems(16.0);

/// Below this window width an expanded sidebar becomes a dismissible drawer.
///
/// A phone-ish breakpoint. `Sidebar::overlay_below` moves it and
/// `Sidebar::never_overlay` turns the behaviour off.
const DEFAULT_OVERLAY_BELOW: Pixels = px(640.);

/// Which edge of the window the panel is docked to.
///
/// Deliberately not `Top`/`Bottom`: a top dock's cross-axis size is a *height*
/// rather than a width, and a rail is a *column* of controls, so neither
/// survives the rotation. That is a different component.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SidebarEdge {
    /// Docked to the left of the window. The default.
    #[default]
    Left,
    /// Docked to the right of the window.
    Right,
}

impl SidebarEdge {
    /// Whether this is the left edge.
    pub fn is_left(self) -> bool {
        matches!(self, SidebarEdge::Left)
    }
}

/// Whether the panel shows its content or its rail.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SidebarState {
    /// Full width, showing the panel's children.
    #[default]
    Expanded,
    /// A rail the width of one icon control, showing [`Sidebar::rail`].
    Collapsed,
}

impl SidebarState {
    /// Whether the panel is expanded.
    pub fn is_expanded(self) -> bool {
        matches!(self, SidebarState::Expanded)
    }

    /// The other state.
    pub fn toggled(self) -> Self {
        match self {
            SidebarState::Expanded => SidebarState::Collapsed,
            SidebarState::Collapsed => SidebarState::Expanded,
        }
    }
}

impl From<bool> for SidebarState {
    /// `true` is expanded, so `state(self.nav_open)` reads the way it looks.
    fn from(expanded: bool) -> Self {
        if expanded {
            SidebarState::Expanded
        } else {
            SidebarState::Collapsed
        }
    }
}

/// The dimensions this component adds to a rung of the shared control scale.
///
/// Lives here rather than in `src/theme/control.rs` per the "What belongs
/// here" note at the top of that file: a rail is specific to this component's
/// shape. Every value is derived from the rung, so nothing here is a named
/// dimension.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SidebarMetrics {
    /// The width of the collapsed rail: one icon control's box plus the
    /// panel's padding on both sides.
    pub rail_width: Rems,
    /// Padding inside the panel, on both axes.
    pub padding: Rems,
    /// Gap between the things stacked inside the panel.
    pub gap: Rems,
}

impl SidebarMetrics {
    /// Derive the panel's shape from a rung.
    pub fn from_control(metrics: ControlMetrics) -> Self {
        Self {
            rail_width: metrics.height + metrics.padding_x * 2.0,
            padding: metrics.padding_x,
            gap: metrics.gap,
        }
    }
}

/// How a sidebar occupies the window this frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SidebarPresentation {
    /// In flow: the panel is a flex child and the content beside it is
    /// narrower by exactly its width.
    Push,
    /// A drawer over the content, with a dismissing scrim.
    Overlay,
}

/// What [`SidebarLayout::resolve`] decided.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SidebarLayout {
    /// Push or overlay.
    pub presentation: SidebarPresentation,
    /// The width this sidebar takes *in the layout*. Equal to `panel` when
    /// pushing; the rail width when overlaying, which is the gutter that keeps
    /// the content from reflowing as the drawer opens and closes.
    pub footprint: Pixels,
    /// The width the panel itself is drawn at.
    pub panel: Pixels,
}

impl SidebarLayout {
    /// The whole of this component's interesting behaviour, as a pure
    /// function — so it is unit-testable with no window, and so there is one
    /// place to argue with if push-versus-overlay should key off something
    /// else.
    ///
    /// - Collapsed is always a rail, and always pushes: a rail is narrow
    ///   enough that taking it out of flow buys nothing.
    /// - Expanded overlays when the *window* is narrower than `overlay_below`.
    ///   `None` disables the transition.
    /// - The requested width is clamped into `[rail_width, viewport_width]`.
    pub fn resolve(
        state: SidebarState,
        requested_width: Pixels,
        rail_width: Pixels,
        viewport_width: Pixels,
        overlay_below: Option<Pixels>,
    ) -> Self {
        if state == SidebarState::Collapsed {
            return Self {
                presentation: SidebarPresentation::Push,
                footprint: rail_width,
                panel: rail_width,
            };
        }

        // `f32::clamp` panics when its bounds cross, and a window narrower
        // than the rail is a real state during a resize.
        let ceiling = viewport_width.max(rail_width);
        let panel = px(requested_width
            .as_f32()
            .clamp(rail_width.as_f32(), ceiling.as_f32()));

        let overlaying = overlay_below.is_some_and(|breakpoint| viewport_width < breakpoint);

        if overlaying {
            Self {
                presentation: SidebarPresentation::Overlay,
                // A gutter the width of the rail, so opening the drawer does
                // not reflow the content behind it.
                footprint: rail_width,
                panel,
            }
        } else {
            Self {
                presentation: SidebarPresentation::Push,
                footprint: panel,
                panel,
            }
        }
    }
}

/// A click handler, as both `Sidebar` and `SidebarTrigger` store one.
type ClickHandler = Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;

/// Create a sidebar. The `id` scopes everything the panel draws, including
/// the drawer, so it has to be unique among what is on screen.
pub fn sidebar(id: impl Into<ElementId>) -> Sidebar {
    Sidebar::new(id)
}

/// A panel docked to the left or right edge of the window, with an
/// expanded/collapsed state the caller owns.
#[derive(IntoElement)]
pub struct Sidebar {
    id: ElementId,
    edge: SidebarEdge,
    state: SidebarState,
    width: Rems,
    overlay_below: Option<Pixels>,
    label: Option<SharedString>,
    size: ControlSize,
    children: Vec<AnyElement>,
    rail: Option<AnyElement>,
    on_dismiss: Option<ClickHandler>,
}

impl Sidebar {
    /// Create a sidebar with the default edge, width and state.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            edge: SidebarEdge::default(),
            state: SidebarState::default(),
            width: DEFAULT_WIDTH,
            overlay_below: Some(DEFAULT_OVERLAY_BELOW),
            label: None,
            size: ControlSize::default(),
            children: Vec::new(),
            rail: None,
            on_dismiss: None,
        }
    }

    /// Dock this panel to `edge`.
    pub fn edge(mut self, edge: SidebarEdge) -> Self {
        self.edge = edge;
        self
    }

    /// Dock to the left edge.
    pub fn left(self) -> Self {
        self.edge(SidebarEdge::Left)
    }

    /// Dock to the right edge.
    pub fn right(self) -> Self {
        self.edge(SidebarEdge::Right)
    }

    /// The expanded width. Clamped into `[rail width, window width]` at
    /// layout, so a value wider than the window is a wide sidebar rather than
    /// a broken one.
    pub fn width(mut self, width: impl Into<Rems>) -> Self {
        self.width = width.into();
        self
    }

    /// Expanded or collapsed. Takes a `bool` too, where `true` is expanded.
    pub fn state(mut self, state: impl Into<SidebarState>) -> Self {
        self.state = state.into();
        self
    }

    /// Shorthand for `state(!collapsed)`.
    pub fn collapsed(self, collapsed: bool) -> Self {
        self.state(!collapsed)
    }

    /// Move the window width below which an expanded panel becomes a drawer.
    pub fn overlay_below(mut self, width: impl Into<Pixels>) -> Self {
        self.overlay_below = Some(width.into());
        self
    }

    /// Always push, however narrow the window gets.
    pub fn never_overlay(mut self) -> Self {
        self.overlay_below = None;
        self
    }

    /// The accessible name of the region — what a screen reader announces
    /// this panel as. Without it the panel is an unnamed `Complementary`
    /// landmark, which is worth avoiding when there is more than one.
    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// What the panel shows when collapsed: a *rail* of icon controls, rather
    /// than nothing.
    ///
    /// The rail is what makes this more than a `when(open, …)`. Without one, a
    /// collapsed panel is an empty strip.
    pub fn rail(mut self, rail: impl IntoElement) -> Self {
        self.rail = Some(rail.into_any_element());
        self
    }

    /// Called when the drawer's scrim is clicked. Only reachable while
    /// overlaying — a pushed panel has no scrim to click.
    pub fn on_dismiss(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_dismiss = Some(Box::new(handler));
        self
    }

    /// The panel itself: the box that carries the role, the border and the
    /// content or the rail.
    fn region(self, layout: SidebarLayout, metrics: SidebarMetrics, cx: &App) -> Stateful<Div> {
        let theme = cx.theme();
        let expanded = self.state.is_expanded();
        let a11y = self.a11y();

        let mut region = div()
            .id(self.id.clone())
            .announce(a11y)
            .flex()
            .flex_col()
            .flex_none()
            .w(layout.panel)
            .h_full()
            .gap(metrics.gap)
            .p(metrics.padding)
            .bg(theme.surface())
            .overflow_hidden();

        region = if self.edge.is_left() {
            region.border_r_1()
        } else {
            region.border_l_1()
        }
        .border_color(theme.border());

        if expanded {
            region.children(self.children)
        } else {
            region.children(self.rail)
        }
    }
}

/// A landmark is named by what it contains, so the label stays optional here
/// — `crate::a11y::role_requires_a_name` does not list `Complementary`. The
/// expanded state deliberately is *not* reported on the region: it belongs on
/// [`SidebarTrigger`], the control that changes it.
impl Accessible for Sidebar {
    fn a11y(&self) -> A11y {
        let a11y = A11y::new(Role::Complementary);

        match self.label.clone() {
            Some(label) => a11y.name(label),
            None => a11y,
        }
    }
}

impl ParentElement for Sidebar {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements)
    }
}

impl ControlSized for Sidebar {
    fn control_size(mut self, size: ControlSize) -> Self {
        self.size = size;
        self
    }
}

impl RenderOnce for Sidebar {
    fn render(mut self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let metrics = SidebarMetrics::from_control(cx.theme().control(self.size));
        let rem_size = window.rem_size();

        // The *window's* width, because that is all a `RenderOnce` gets —
        // nothing about its parent's box. So a sidebar inside a 400px
        // container on a 1400px window still pushes, and a sidebar not docked
        // to a window edge overlays at the window edge anyway.
        let viewport_width = window.viewport_size().width;
        let layout = SidebarLayout::resolve(
            self.state,
            self.width.to_pixels(rem_size),
            metrics.rail_width.to_pixels(rem_size),
            viewport_width,
            self.overlay_below,
        );

        match layout.presentation {
            SidebarPresentation::Push => self.region(layout, metrics, cx).into_any_element(),
            SidebarPresentation::Overlay => {
                let id = self.id.clone();
                let edge = self.edge;
                let on_dismiss = self.on_dismiss.take();
                let scrim_color = cx.theme().overlay();
                let viewport = window.viewport_size();

                let region = self.region(layout, metrics, cx);

                let drawer = div()
                    // The full viewport, so a click anywhere outside the
                    // drawer dismisses it. `occlude` keeps that click off
                    // whatever is underneath.
                    .id(scoped(&id, "scrim"))
                    .occlude()
                    .w(viewport.width)
                    .h(viewport.height)
                    .flex()
                    .when(!edge.is_left(), |this| this.justify_end())
                    .bg(scrim_color)
                    .when_some(on_dismiss, |this, on_dismiss| {
                        this.on_mouse_down(gpui::MouseButton::Left, move |event, window, cx| {
                            cx.stop_propagation();
                            on_dismiss(
                                &ClickEvent::Mouse(gpui::MouseClickEvent {
                                    down: event.clone(),
                                    up: gpui::MouseUpEvent {
                                        button: event.button,
                                        position: event.position,
                                        modifiers: event.modifiers,
                                        click_count: event.click_count,
                                    },
                                }),
                                window,
                                cx,
                            );
                        })
                    })
                    .child(
                        // Clicking the drawer itself must not dismiss it —
                        // the same shape `Dialog` uses for its panel.
                        div()
                            .id(scoped(&id, "drawer"))
                            .occlude()
                            .on_mouse_down(gpui::MouseButton::Left, |_, _, cx| {
                                cx.stop_propagation();
                            })
                            .child(region),
                    );

                // An in-flow gutter the width of the rail, carrying **no
                // role**: the panel this frame is the drawer, and two
                // `Complementary` regions for one panel would tell a screen
                // reader about a panel that is really the drawer's shadow.
                div()
                    .id(scoped(&id, "gutter"))
                    .flex_none()
                    .w(layout.footprint)
                    .h_full()
                    .child(
                        deferred(
                            anchored()
                                .position(point(px(0.), px(0.)))
                                .snap_to_window()
                                .child(drawer),
                        )
                        // Below `Dialog`'s 10, so a modal opened from the
                        // sidebar still draws over the drawer.
                        .with_priority(2),
                    )
                    .into_any_element()
            }
        }
    }
}

/// Create the control that expands and collapses a [`Sidebar`].
///
/// This is the only sub-component, and it ships for an accessibility reason:
/// `aria-expanded` belongs on the control that changes the state, not on the
/// region. Resist adding a second — the rail's icons are plain
/// [`IconButton`](crate::elements::icon_button::IconButton)s.
pub fn sidebar_trigger(id: impl Into<ElementId>, state: SidebarState) -> SidebarTrigger {
    SidebarTrigger::new(id, state)
}

/// The control that expands and collapses a [`Sidebar`].
#[derive(IntoElement)]
pub struct SidebarTrigger {
    id: ElementId,
    state: SidebarState,
    edge: SidebarEdge,
    label: Option<SharedString>,
    disabled: bool,
    size: ControlSize,
    handler: Option<ClickHandler>,
    focus_handle: Option<FocusHandle>,
}

impl SidebarTrigger {
    /// Create a trigger reporting `state`.
    pub fn new(id: impl Into<ElementId>, state: SidebarState) -> Self {
        Self {
            id: id.into(),
            state,
            edge: SidebarEdge::default(),
            label: None,
            disabled: false,
            size: ControlSize::default(),
            handler: None,
            focus_handle: None,
        }
    }

    /// Focus this trigger through a handle the caller owns. Optional for the
    /// same reason [`crate::elements::button::Button::focus_handle`] is.
    pub fn focus_handle(mut self, handle: FocusHandle) -> Self {
        self.focus_handle = Some(handle);
        self
    }

    /// Mirror the glyph for a sidebar docked to the other edge.
    pub fn edge(mut self, edge: SidebarEdge) -> Self {
        self.edge = edge;
        self
    }

    /// The accessible name. Defaults to "Toggle sidebar".
    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Register a click handler without importing [`Clickable`] — the shape
    /// `IconButton` already has.
    pub fn on_click(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.handler = Some(Box::new(handler));
        self
    }

    fn glyph(&self) -> Svg {
        match (self.edge, self.state) {
            (SidebarEdge::Left, SidebarState::Expanded) => Icons::panel_left(),
            (SidebarEdge::Left, SidebarState::Collapsed) => Icons::panel_left_minimized(),
            (SidebarEdge::Right, SidebarState::Expanded) => Icons::panel_right(),
            (SidebarEdge::Right, SidebarState::Collapsed) => Icons::panel_right_minimized(),
        }
    }
}

/// The name has a default rather than being required of the caller, which is
/// the one liberty this element takes with `crate::a11y`'s section 2: the
/// glyph is always the same glyph, so "Toggle sidebar" is always true of it.
/// `aria-expanded` is here rather than on the region because this is the
/// control that changes the state.
impl Accessible for SidebarTrigger {
    fn a11y(&self) -> A11y {
        let a11y = A11y::new(Role::Button)
            .name(
                self.label
                    .clone()
                    .unwrap_or_else(|| "Toggle sidebar".into()),
            )
            .expanded(self.state.is_expanded());

        // The same answer `Button` gives, for the same reason: a control that
        // announces a role a keyboard cannot reach is the defect
        // `crate::a11y`'s section 4 exists for, and a disabled one leaves the
        // tab order because gpui has no `aria_disabled` to announce instead.
        if self.disabled {
            a11y.not_focusable("a disabled trigger has nothing for a keyboard to do")
        } else if let Some(handle) = self.focus_handle.clone() {
            a11y.focus_handle(handle)
        } else {
            a11y.focusable()
        }
    }
}

impl RenderOnce for SidebarTrigger {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let metrics = theme.control(self.size);
        let glyph = self.glyph();
        let disabled = self.disabled;
        // Before `self`'s fields are moved into the element below.
        let a11y = self.a11y();
        let handler = self.handler;

        let color = if disabled {
            theme.fg_disabled()
        } else {
            theme.fg_muted()
        };

        div()
            .id(self.id)
            .announce(a11y)
            .w(metrics.height)
            .h(metrics.height)
            .flex()
            .flex_none()
            .items_center()
            .justify_center()
            .rounded(metrics.radius)
            .focus_visible(|style| style.shadow(focus_ring(theme.accent())))
            .when(disabled, |this| this.cursor_not_allowed())
            .when(!disabled, |this| {
                this.cursor_pointer()
                    .hover(|this| this.bg(theme.surface_secondary()))
                    .when_some(handler, |this, handler| {
                        this.on_click(move |event, window, cx| {
                            cx.stop_propagation();
                            handler(event, window, cx);
                        })
                    })
            })
            .child(glyph.size(metrics.ink).text_color(color))
    }
}

impl Clickable for SidebarTrigger {
    fn on_click(mut self, handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static) -> Self {
        self.handler = Some(Box::new(handler));
        self
    }
}

impl Disableable for SidebarTrigger {
    fn is_disabled(&self) -> bool {
        self.disabled
    }

    fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

impl ControlSized for SidebarTrigger {
    fn control_size(mut self, size: ControlSize) -> Self {
        self.size = size;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::elements::button::button;
    use crate::elements::icon_button::icon_button;
    use crate::elements::list::{List, ListEntry};
    use crate::elements::separator::separator;
    use crate::theme::ControlScale;
    use gpui::{Context, Entity, Render, TestAppContext, VisualTestContext, size};

    // --- what it announces ---

    /// The migration onto `crate::a11y` is behaviour-preserving: the trigger
    /// still announces a named button carrying the panel's expanded state.
    ///
    /// Only the trigger is checked this way. `Sidebar::render` returns an
    /// `AnyElement`, which does not forward `a11y_role` to what it wraps, so
    /// `announced` cannot see the region's landmark — see that helper's docs.
    /// `Sidebar::a11y` is checked directly instead.
    #[gpui::test]
    fn the_trigger_announces_a_named_button_with_its_state(cx: &mut TestAppContext) {
        cx.update(crate::theme::init);
        let cx = cx.add_empty_window();

        for (state, expanded) in [
            (SidebarState::Expanded, true),
            (SidebarState::Collapsed, false),
        ] {
            let announced = cx.update(|window, cx| {
                crate::a11y::test_support::announced(
                    sidebar_trigger("nav-toggle", state),
                    window,
                    cx,
                )
            });

            assert_eq!(announced.role, Some(Role::Button));
            assert_eq!(announced.name(), Some("Toggle sidebar"));
            assert_eq!(
                announced.node.as_ref().and_then(|node| node.is_expanded()),
                Some(expanded),
                "the state belongs on the control that changes it"
            );
        }
    }

    /// The trigger is a `Role::Button`, so `crate::a11y`'s section 4 requires
    /// it to answer the focus question — and answering it wrong is the defect
    /// that section exists for.
    #[test]
    fn the_trigger_takes_focus_and_a_disabled_one_declines_it() {
        assert!(
            sidebar_trigger("nav-toggle", SidebarState::Expanded)
                .a11y()
                .is_focusable()
        );

        let disabled = sidebar_trigger("nav-toggle", SidebarState::Expanded)
            .disabled(true)
            .a11y();
        assert!(!disabled.is_focusable());
        assert!(disabled.focus_declined_because().is_some());
        assert!(!disabled.is_missing_a_focus_decision());
    }

    /// The panel is a landmark, so it is never asked the question.
    #[test]
    fn the_region_is_not_asked_about_keyboard_focus() {
        assert!(
            !sidebar("app-nav")
                .label("Navigation")
                .a11y()
                .is_missing_a_focus_decision()
        );
    }

    #[test]
    fn the_region_is_a_landmark_named_by_its_label() {
        let named = sidebar("app-nav").label("Navigation").a11y();
        assert_eq!(named.role(), Role::Complementary);
        assert_eq!(named.accessible_name(), Some(&"Navigation".into()));

        // A landmark is named by what it contains, so this is legal — and the
        // convention's naming rule agrees.
        let unnamed = sidebar("app-nav").a11y();
        assert_eq!(unnamed.accessible_name(), None);
        assert!(!unnamed.is_missing_a_required_name());
    }

    // --- the layout maths, with no window ---

    fn resolve(
        state: SidebarState,
        width: f32,
        viewport: f32,
        overlay_below: Option<f32>,
    ) -> SidebarLayout {
        SidebarLayout::resolve(
            state,
            px(width),
            px(48.),
            px(viewport),
            overlay_below.map(px),
        )
    }

    #[test]
    fn a_collapsed_sidebar_is_a_rail_and_always_pushes() {
        // Even in a window far narrower than the breakpoint: a rail is narrow
        // enough that taking it out of flow buys nothing.
        for viewport in [320., 1400.] {
            let layout = resolve(SidebarState::Collapsed, 240., viewport, Some(640.));

            assert_eq!(layout.presentation, SidebarPresentation::Push);
            assert_eq!(layout.panel, px(48.));
            assert_eq!(layout.footprint, px(48.));
        }
    }

    #[test]
    fn an_expanded_sidebar_pushes_in_a_wide_window() {
        let layout = resolve(SidebarState::Expanded, 240., 1400., Some(640.));

        assert_eq!(layout.presentation, SidebarPresentation::Push);
        assert_eq!(layout.panel, px(240.));
        assert_eq!(
            layout.footprint,
            px(240.),
            "a pushed panel's footprint is its width"
        );
    }

    #[test]
    fn an_expanded_sidebar_overlays_below_the_breakpoint() {
        let layout = resolve(SidebarState::Expanded, 240., 480., Some(640.));

        assert_eq!(layout.presentation, SidebarPresentation::Overlay);
        assert_eq!(layout.panel, px(240.));
        assert_eq!(
            layout.footprint,
            px(48.),
            "the in-flow gutter keeps the content from reflowing as the drawer opens"
        );
    }

    #[test]
    fn never_overlay_pushes_however_narrow_the_window_is() {
        let layout = resolve(SidebarState::Expanded, 240., 200., None);

        assert_eq!(layout.presentation, SidebarPresentation::Push);
    }

    #[test]
    fn the_width_is_clamped_into_the_window() {
        let wide = resolve(SidebarState::Expanded, 2000., 800., None);
        assert_eq!(wide.panel, px(800.));

        let narrow = resolve(SidebarState::Expanded, 10., 800., None);
        assert_eq!(narrow.panel, px(48.), "never narrower than the rail");
    }

    /// `f32::clamp` panics when its bounds cross, and a window narrower than
    /// the rail is a real state during a resize.
    #[test]
    fn a_window_narrower_than_the_rail_does_not_panic() {
        let layout = resolve(SidebarState::Expanded, 240., 20., None);

        assert_eq!(layout.panel, px(48.));
    }

    #[test]
    fn the_rail_is_one_icon_control_plus_the_panels_padding() {
        for size in ControlSize::ALL {
            let control = ControlScale::default().metrics(size);
            let metrics = SidebarMetrics::from_control(control);

            assert_eq!(
                metrics.rail_width.0,
                control.height.0 + control.padding_x.0 * 2.0,
                "{}: the rail must fit an icon button and the panel's padding",
                size.name(),
            );
            assert!(
                metrics.rail_width.0 > control.height.0,
                "{}: a rail exactly one control wide leaves no padding",
                size.name(),
            );
        }
    }

    #[test]
    fn a_state_round_trips_through_bool_and_back() {
        assert_eq!(SidebarState::from(true), SidebarState::Expanded);
        assert_eq!(SidebarState::from(false), SidebarState::Collapsed);
        assert_eq!(SidebarState::Expanded.toggled(), SidebarState::Collapsed);
        assert_eq!(
            SidebarState::Collapsed.toggled().toggled(),
            SidebarState::Collapsed
        );
    }

    // --- drawing ---
    //
    // An element that has *both* a role and a mouse listener cannot be drawn
    // with `VisualTestContext::draw`: registering a mouse listener reads
    // `Window::current_view`, which is only set while a *view* renders, so a
    // bare draw panics inside gpui with an opaque `Option::unwrap()` on
    // `None`. Hence a real view.

    type Build = Box<dyn Fn(&mut Window, &mut App) -> AnyElement>;

    struct Harness {
        build: Build,
        drawn: Entity<usize>,
    }

    impl Render for Harness {
        fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            self.drawn.update(cx, |count, _| *count += 1);
            (self.build)(window, cx)
        }
    }

    /// Draw `build` in a window of `window_size`, and assert it really drew —
    /// a harness that never rendered would otherwise pass every test below
    /// silently.
    fn draw(
        cx: &mut TestAppContext,
        window_size: gpui::Size<Pixels>,
        build: impl Fn(&mut Window, &mut App) -> AnyElement + 'static,
    ) -> &mut VisualTestContext {
        cx.update(crate::theme::init);

        let drawn = cx.update(|cx| cx.new(|_| 0usize));
        let counter = drawn.clone();
        let window = cx.open_window(window_size, move |_window, _cx| Harness {
            build: Box::new(build),
            drawn: counter,
        });

        let cx = VisualTestContext::from_window(*std::ops::Deref::deref(&window), cx).into_mut();
        cx.run_until_parked();

        assert!(
            drawn.read_with(cx, |count, _| *count) > 0,
            "the harness never drew, so this test is checking nothing"
        );
        cx
    }

    fn panel(
        state: SidebarState,
        window: gpui::Size<Pixels>,
    ) -> impl Fn(&mut Window, &mut App) -> AnyElement {
        move |_window, _cx| {
            let _ = window;
            sidebar("test-sidebar")
                .label("Navigation")
                .state(state)
                .rail(div().child("rail"))
                .child(div().child("content"))
                .into_any_element()
        }
    }

    #[gpui::test]
    fn an_expanded_panel_draws_in_a_wide_window(cx: &mut TestAppContext) {
        let window = size(px(1200.), px(800.));
        draw(cx, window, panel(SidebarState::Expanded, window));
    }

    #[gpui::test]
    fn a_collapsed_panel_draws_its_rail(cx: &mut TestAppContext) {
        let window = size(px(1200.), px(800.));
        draw(cx, window, panel(SidebarState::Collapsed, window));
    }

    /// The overlay branch, which is the one with a deferred anchored subtree,
    /// a scrim and two mouse listeners in it.
    #[gpui::test]
    fn an_expanded_panel_draws_as_a_drawer_in_a_narrow_window(cx: &mut TestAppContext) {
        let window = size(px(420.), px(700.));
        draw(cx, window, panel(SidebarState::Expanded, window));
    }

    /// The composition the issue asked for — `List` + `Separator` + `Button`
    /// and no sub-components — drawn inside an *overlaying* panel, because
    /// `deferred` keeps the ambient element-id stack and a `List`'s
    /// `uniform_list` keeps per-element state under that path.
    #[gpui::test]
    fn the_documented_composition_draws_inside_a_drawer(cx: &mut TestAppContext) {
        let window = size(px(420.), px(700.));

        draw(cx, window, move |window, cx| {
            let entries = vec![
                ListEntry::header("Section"),
                ListEntry::item("row-one", |_w, _cx| div().child("One").into_any_element()),
                ListEntry::item("row-two", |_w, _cx| div().child("Two").into_any_element()),
            ];

            sidebar("composed-sidebar")
                .label("Navigation")
                .state(SidebarState::Expanded)
                .on_dismiss(|_, _, _| {})
                .rail(icon_button("rail-home", Icons::home()))
                .child(sidebar_trigger("composed-trigger", SidebarState::Expanded))
                .child(List::new("composed-list", entries).render(window, cx))
                .child(separator())
                .child(button("composed-action", "Action"))
                .into_any_element()
        });
    }
}
