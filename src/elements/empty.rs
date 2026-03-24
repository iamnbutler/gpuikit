use crate::theme::{ActiveTheme, Themeable};
use gpui::{
    div, prelude::FluentBuilder, px, rems, AnyElement, App, IntoElement, ParentElement, RenderOnce,
    SharedString, Styled, Svg, Window,
};

pub fn empty() -> Empty {
    Empty::new()
}

#[derive(IntoElement)]
pub struct Empty {
    icon: Option<Svg>,
    title: Option<SharedString>,
    description: Option<SharedString>,
    action: Option<AnyElement>,
}

impl Empty {
    pub fn new() -> Self {
        Empty {
            icon: None,
            title: None,
            description: None,
            action: None,
        }
    }

    /// Set the icon to display above the title.
    pub fn icon(mut self, icon: Svg) -> Self {
        self.icon = Some(icon);
        self
    }

    /// Set the main title/message.
    pub fn title(mut self, title: impl Into<SharedString>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Set the secondary description text.
    pub fn description(mut self, description: impl Into<SharedString>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Set an action element (typically a button).
    pub fn action(mut self, action: impl IntoElement) -> Self {
        self.action = Some(action.into_any_element());
        self
    }
}

impl Default for Empty {
    fn default() -> Self {
        Self::new()
    }
}

impl RenderOnce for Empty {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();

        div()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .py(rems(2.0))
            .px(rems(1.5))
            .gap(rems(0.75))
            .when_some(self.icon, |container, icon| {
                container.child(
                    icon.size(px(48.0))
                        .text_color(theme.fg_muted().opacity(0.6)),
                )
            })
            .when_some(self.title, |container, title| {
                container.child(
                    div()
                        .text_base()
                        .font_weight(gpui::FontWeight::MEDIUM)
                        .text_color(theme.fg())
                        .text_center()
                        .child(title),
                )
            })
            .when_some(self.description, |container, description| {
                container.child(
                    div()
                        .text_sm()
                        .text_color(theme.fg_muted())
                        .text_center()
                        .max_w(px(300.0))
                        .child(description),
                )
            })
            .when_some(self.action, |container, action| {
                container.child(div().pt(rems(0.5)).child(action))
            })
    }
}
