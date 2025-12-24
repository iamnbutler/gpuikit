use crate::theme::{ActiveTheme, Themeable};
use gpui::{
    div, px, rems, App, FontWeight, IntoElement, ParentElement, RenderOnce, SharedString, Styled,
    Window,
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
}

impl Badge {
    pub fn new(label: impl Into<SharedString>) -> Self {
        Badge {
            label: label.into(),
            variant: BadgeVariant::Default,
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
            .px(rems(0.375))
            .py(px(2.0))
            .rounded(rems(0.25))
            .text_xs()
            .font_weight(FontWeight::SEMIBOLD)
            .line_height(rems(1.0))
            .bg(bg)
            .text_color(text_color)
            .border_1()
            .border_color(border_color)
            .whitespace_nowrap()
            .child(self.label)
    }
}
