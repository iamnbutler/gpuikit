use crate::theme::{ActiveTheme, ControlSize, Themeable};
use crate::traits::{
    clickable::Clickable, control_sized::ControlSized, disableable::Disableable,
    selectable::Selectable,
};
use gpui::{
    prelude::FluentBuilder, AnyView, App, ClickEvent, Context, ElementId, Entity,
    InteractiveElement, IntoElement, MouseButton, ParentElement, Rems, RenderOnce,
    StatefulInteractiveElement, Styled, Svg, Window,
};

pub fn icon_button(id: impl Into<ElementId>, icon: Svg) -> IconButton {
    IconButton::new(id, icon)
}

/// Internal state for toggle behavior, managed by gpui's element state system
struct ToggleState {
    toggled: bool,
}

#[derive(IntoElement)]
pub struct IconButton {
    id: ElementId,
    icon: Svg,
    /// External selected state (takes precedence if set)
    selected: Option<bool>,
    disabled: bool,
    /// Whether internal state management is enabled
    use_internal_state: bool,
    handler: Option<Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>>,
    /// Called when toggle state changes (only used with internal state)
    on_toggle: Option<Box<dyn Fn(&bool, &mut Window, &mut App) + 'static>>,
    tooltip: Option<Box<dyn Fn(&mut Window, &mut App) -> AnyView + 'static>>,
    /// The rung this button sits on. Everything below is an override of what
    /// the rung says.
    size: ControlSize,
    /// Button box width, overriding the rung's height.
    width: Option<Rems>,
    /// Button box height, overriding the rung's height.
    height: Option<Rems>,
    /// Rendered icon size, overriding the rung's ink.
    icon_size: Option<Rems>,
}

impl IconButton {
    pub fn new(id: impl Into<ElementId>, icon: Svg) -> Self {
        IconButton {
            id: id.into(),
            icon,
            selected: None,
            disabled: false,
            use_internal_state: false,
            handler: None,
            on_toggle: None,
            tooltip: None,
            size: ControlSize::default(),
            width: None,
            height: None,
            icon_size: None,
        }
    }

    /// Force the button box to a square of the given size, overriding the rung.
    ///
    /// Named `box_size` rather than `size` because `.size()` read as though it
    /// set the *control* size; that is [`ControlSized::control_size`] and its
    /// `.small()` / `.medium()` / `.large()` shorthands, which is what you
    /// almost always want.
    pub fn box_size(mut self, size: impl Into<Rems>) -> Self {
        let size = size.into();
        self.width = Some(size);
        self.height = Some(size);
        self
    }

    /// Force the button box width, overriding the rung.
    pub fn width(mut self, width: impl Into<Rems>) -> Self {
        self.width = Some(width.into());
        self
    }

    /// Force the button box height, overriding the rung.
    pub fn height(mut self, height: impl Into<Rems>) -> Self {
        self.height = Some(height.into());
        self
    }

    /// Force the rendered icon size within the button, overriding the rung's
    /// ink.
    pub fn icon_size(mut self, size: impl Into<Rems>) -> Self {
        self.icon_size = Some(size.into());
        self
    }

    /// Enable internal state management for toggle behavior.
    /// When enabled, the button will track its own toggled state across frames.
    /// Use `on_toggle` to respond to state changes.
    pub fn use_state(mut self) -> Self {
        self.use_internal_state = true;
        self
    }

    /// Set the selected state externally.
    /// This takes precedence over internal state management.
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = Some(selected);
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Register a callback for click events.
    /// For toggle buttons using internal state, prefer `on_toggle` instead.
    pub fn on_click(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.handler = Some(Box::new(handler));
        self
    }

    /// Register a callback for when the toggle state changes.
    /// Only fires when using internal state management via `use_state()`.
    /// The callback receives the new toggled state.
    /// Accepts `&bool` to enable use with `cx.listener()`.
    pub fn on_toggle(mut self, handler: impl Fn(&bool, &mut Window, &mut App) + 'static) -> Self {
        self.on_toggle = Some(Box::new(handler));
        self
    }

    pub fn tooltip(mut self, tooltip: impl Fn(&mut Window, &mut App) -> AnyView + 'static) -> Self {
        self.tooltip = Some(Box::new(tooltip));
        self
    }
}

impl RenderOnce for IconButton {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        // Get or create persistent state for this button if internal state is enabled
        // Note: must happen before borrowing theme to avoid borrow conflicts
        // Uses use_keyed_state with the button's ID to ensure each button has its own state
        let state: Option<Entity<ToggleState>> = if self.use_internal_state {
            Some(window.use_keyed_state(
                self.id.clone(),
                cx,
                |_window, _cx: &mut Context<ToggleState>| ToggleState { toggled: false },
            ))
        } else {
            None
        };

        // Resolve the selected state:
        // 1. External `selected` takes precedence
        // 2. Otherwise use internal state if enabled
        // 3. Default to false
        let is_selected = self
            .selected
            .unwrap_or_else(|| state.as_ref().map(|s| s.read(cx).toggled).unwrap_or(false));

        let theme = cx.theme();
        let metrics = theme.control(self.size);
        let icon_color = match (is_selected, self.disabled) {
            (true, true) => theme.accent().opacity(0.5),
            (true, false) => theme.accent(),
            (false, true) => theme.fg_disabled(),
            (false, false) => theme.fg_muted(),
        };

        let disabled = self.disabled;
        let on_toggle = self.on_toggle;
        let handler = self.handler;

        gpui::div()
            .id(self.id)
            .w(self.width.unwrap_or(metrics.height))
            .h(self.height.unwrap_or(metrics.height))
            .flex()
            .flex_none()
            .items_center()
            .justify_center()
            .rounded(metrics.radius)
            .when(!disabled, |div| {
                div.hover(|div| div.bg(theme.surface_secondary()))
                    .cursor_pointer()
            })
            .when(disabled, |div| div.cursor_not_allowed())
            .when(!disabled, |button| {
                button
                    .on_mouse_down(MouseButton::Left, |_, window, _| window.prevent_default())
                    .on_click(move |event, window, cx| {
                        cx.stop_propagation();

                        // Handle internal state toggle
                        if let Some(ref state) = state {
                            state.update(cx, |s, _cx| {
                                s.toggled = !s.toggled;
                            });

                            let new_state = state.read(cx).toggled;
                            if let Some(ref on_toggle) = on_toggle {
                                on_toggle(&new_state, window, cx);
                            }
                        }

                        // Also call the regular click handler if present
                        if let Some(ref handler) = handler {
                            handler(event, window, cx);
                        }
                    })
            })
            .when_some(self.tooltip, |el, tooltip| el.tooltip(tooltip))
            .child(
                self.icon
                    .size(self.icon_size.unwrap_or(metrics.ink))
                    .text_color(icon_color),
            )
    }
}

impl Clickable for IconButton {
    fn on_click(mut self, handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static) -> Self {
        self.handler = Some(Box::new(handler));
        self
    }
}

impl Disableable for IconButton {
    fn is_disabled(&self) -> bool {
        self.disabled
    }

    fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

impl ControlSized for IconButton {
    fn control_size(mut self, size: ControlSize) -> Self {
        self.size = size;
        self
    }
}

impl Selectable for IconButton {
    fn is_selected(&self) -> bool {
        self.selected.unwrap_or(false)
    }

    fn selected(mut self, selected: bool) -> Self {
        self.selected = Some(selected);
        self
    }
}
