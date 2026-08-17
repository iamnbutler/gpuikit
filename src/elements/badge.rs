use crate::theme::{ActiveTheme, ControlSize, Themeable};
use crate::traits::control_sized::ControlSized;
use gpui::{
    div, App, FontWeight, IntoElement, ParentElement, RenderOnce, SharedString, Styled, Window,
};

pub fn badge(label: impl Into<SharedString>) -> Badge {
    Badge::new(label)
}

#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub enum BadgeVariant {
    #[default]
    Default,
    Secondary,
    Outline,
    Destructive,
}

#[derive(IntoElement)]
pub struct Badge {
    label: SharedString,
    variant: BadgeVariant,
    size: ControlSize,
}

impl Badge {
    pub fn new(label: impl Into<SharedString>) -> Self {
        Badge {
            label: label.into(),
            variant: BadgeVariant::Default,
            size: ControlSize::default(),
        }
    }

    pub fn variant(mut self, variant: BadgeVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn secondary(mut self) -> Self {
        self.variant = BadgeVariant::Secondary;
        self
    }

    pub fn outline(mut self) -> Self {
        self.variant = BadgeVariant::Outline;
        self
    }

    pub fn destructive(mut self) -> Self {
        self.variant = BadgeVariant::Destructive;
        self
    }
}

impl RenderOnce for Badge {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let metrics = theme.control(self.size);

        let (bg, text_color, border_color) = match self.variant {
            BadgeVariant::Default => (theme.accent(), theme.bg(), theme.accent()),
            BadgeVariant::Secondary => (
                theme.surface_secondary(),
                theme.fg(),
                theme.surface_secondary(),
            ),
            BadgeVariant::Outline => (theme.bg().opacity(0.0), theme.fg(), theme.border()),
            BadgeVariant::Destructive => (theme.danger(), theme.bg(), theme.danger()),
        };

        div()
            // A declared height, rather than padding plus a line box and
            // whatever that came to: the badge sits on the same rung as the
            // controls beside it.
            .h(metrics.height)
            .flex()
            .flex_none()
            .items_center()
            .px(metrics.padding_x)
            .rounded(metrics.radius)
            .text_size(metrics.text_size)
            .font_weight(FontWeight::SEMIBOLD)
            .line_height(metrics.line_height)
            .bg(bg)
            .text_color(text_color)
            .border_1()
            .border_color(border_color)
            .whitespace_nowrap()
            .child(self.label)
    }
}

impl ControlSized for Badge {
    fn control_size(mut self, size: ControlSize) -> Self {
        self.size = size;
        self
    }
}
