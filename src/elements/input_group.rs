//! A component for combining inputs with addons (buttons, icons, text).
//!
//! InputGroup allows you to attach addons to the left or right side of an input,
//! creating connected UI elements with seamless borders.
//!
//! # Example
//!
//! ```ignore
//! use gpuikit::elements::input_group::{input_group, InputAddon};
//! use gpuikit::DefaultIcons;
//!
//! // Input with icon addon
//! input_group(input_state)
//!     .left_addon(InputAddon::icon(DefaultIcons::magnifying_glass()))
//!
//! // Input with text addon
//! input_group(input_state)
//!     .left_addon(InputAddon::text("https://"))
//!
//! // Input with button addon
//! input_group(input_state)
//!     .right_addon(InputAddon::button(button("submit", "Go")))
//! ```

use crate::elements::button::Button;
use crate::elements::input::input;
use crate::input::InputState;
use crate::layout::h_stack;
use crate::theme::{ActiveTheme, Themeable};
use crate::traits::disableable::Disableable;
use gpui::{
    prelude::FluentBuilder, px, rems, App, Entity, IntoElement, ParentElement, RenderOnce,
    SharedString, Styled, Svg, Window,
};

/// Creates a new InputGroup wrapping the given input state.
pub fn input_group(input_state: &Entity<InputState>, cx: &App) -> InputGroup {
    InputGroup::new(input_state, cx)
}

/// Type of addon that can be attached to an input.
pub enum InputAddon {
    /// An icon addon
    Icon(Svg),
    /// A text label addon
    Text(SharedString),
    /// A button addon
    Button(Button),
}

impl InputAddon {
    /// Create an icon addon.
    pub fn icon(icon: Svg) -> Self {
        InputAddon::Icon(icon)
    }

    /// Create a text addon.
    pub fn text(text: impl Into<SharedString>) -> Self {
        InputAddon::Text(text.into())
    }

    /// Create a button addon.
    pub fn button(button: Button) -> Self {
        InputAddon::Button(button)
    }
}

/// A component for combining inputs with addons.
#[derive(IntoElement)]
pub struct InputGroup {
    input_state: Entity<InputState>,
    left_addon: Option<InputAddon>,
    right_addon: Option<InputAddon>,
    disabled: bool,
}

impl InputGroup {
    /// Create a new InputGroup with the given input state.
    pub fn new(input_state: &Entity<InputState>, _cx: &App) -> Self {
        InputGroup {
            input_state: input_state.clone(),
            left_addon: None,
            right_addon: None,
            disabled: false,
        }
    }

    /// Add an addon to the left side of the input.
    pub fn left_addon(mut self, addon: InputAddon) -> Self {
        self.left_addon = Some(addon);
        self
    }

    /// Add an addon to the right side of the input.
    pub fn right_addon(mut self, addon: InputAddon) -> Self {
        self.right_addon = Some(addon);
        self
    }
}

impl Disableable for InputGroup {
    fn is_disabled(&self) -> bool {
        self.disabled
    }

    fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

impl RenderOnce for InputGroup {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let has_left = self.left_addon.is_some();
        let has_right = self.right_addon.is_some();
        let disabled = self.disabled;

        h_stack()
            .items_center()
            .h(px(36.))
            .rounded(rems(0.25))
            .border_1()
            .border_color(theme.input_border())
            .bg(theme.input_bg())
            .overflow_hidden()
            .when(disabled, |el| el.opacity(0.65).cursor_not_allowed())
            // Left addon
            .when_some(self.left_addon, |el, addon| {
                el.child(render_addon(addon, AddonPosition::Left, disabled, cx))
            })
            // Input element (with adjusted border radius based on addons)
            .child(
                input(&self.input_state, cx)
                    .border_0()
                    .bg(gpui::transparent_black())
                    .flex_1()
                    .size_full()
                    .min_w(px(100.))
                    .when(has_left, |el| el.rounded_l_none())
                    .when(has_right, |el| el.rounded_r_none()),
            )
            // Right addon
            .when_some(self.right_addon, |el, addon| {
                el.child(render_addon(addon, AddonPosition::Right, disabled, cx))
            })
    }
}

#[derive(Clone, Copy)]
enum AddonPosition {
    Left,
    Right,
}

fn render_addon(
    addon: InputAddon,
    position: AddonPosition,
    disabled: bool,
    cx: &App,
) -> impl IntoElement {
    let theme = cx.theme();

    match addon {
        InputAddon::Icon(icon) => gpui::div()
            .flex()
            .items_center()
            .justify_center()
            .px(rems(0.5))
            .h_full()
            .bg(theme.surface_secondary())
            .border_color(theme.border_subtle())
            .when(matches!(position, AddonPosition::Left), |el| {
                el.border_r_1()
            })
            .when(matches!(position, AddonPosition::Right), |el| {
                el.border_l_1()
            })
            .child(
                icon.size(px(16.))
                    .text_color(if disabled {
                        theme.fg_disabled()
                    } else {
                        theme.fg_muted()
                    }),
            )
            .into_any_element(),

        InputAddon::Text(text) => gpui::div()
            .flex()
            .items_center()
            .px(rems(0.5))
            .h_full()
            .bg(theme.surface_secondary())
            .border_color(theme.border_subtle())
            .text_sm()
            .text_color(if disabled {
                theme.fg_disabled()
            } else {
                theme.fg_muted()
            })
            .whitespace_nowrap()
            .when(matches!(position, AddonPosition::Left), |el| {
                el.border_r_1()
            })
            .when(matches!(position, AddonPosition::Right), |el| {
                el.border_l_1()
            })
            .child(text)
            .into_any_element(),

        InputAddon::Button(button) => {
            // For buttons, we render them inline and adjust their styling
            gpui::div()
                .flex()
                .items_center()
                .h_full()
                .border_color(theme.border_subtle())
                .when(matches!(position, AddonPosition::Left), |el| {
                    el.border_r_1()
                })
                .when(matches!(position, AddonPosition::Right), |el| {
                    el.border_l_1()
                })
                .child(button.disabled(disabled))
                .into_any_element()
        }
    }
}
