use crate::theme::{ActiveTheme, Themeable};
use gpui::{
    div, prelude::FluentBuilder, px, relative, App, IntoElement, ParentElement, Pixels, RenderOnce,
    Styled, Window,
};

pub fn progress(value: f32) -> Progress {
    Progress::new(value)
}

#[derive(Clone, Copy, Default)]
pub enum ProgressVariant {
    #[default]
    Default,
    Success,
    Danger,
}

#[derive(IntoElement)]
pub struct Progress {
    value: f32,
    variant: ProgressVariant,
    height: Option<Pixels>,
    width: Option<Pixels>,
}

impl Progress {
    pub fn new(value: f32) -> Self {
        Self {
            value: value.clamp(0.0, 1.0),
            variant: ProgressVariant::Default,
            height: None,
            width: None,
        }
    }

    pub fn variant(mut self, variant: ProgressVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn height(mut self, height: impl Into<Pixels>) -> Self {
        self.height = Some(height.into());
        self
    }

    pub fn width(mut self, width: impl Into<Pixels>) -> Self {
        self.width = Some(width.into());
        self
    }
}

impl RenderOnce for Progress {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let height = self.height.unwrap_or(px(8.0));
        let border_radius = height / 2.0;

        let fill_color = match self.variant {
            ProgressVariant::Default => theme.accent(),
            ProgressVariant::Success => theme.accent(),
            ProgressVariant::Danger => theme.danger(),
        };

        let fill_width = self.value.max(0.02);

        div()
            .w_full()
            .when_some(self.width, |this, width| this.w(width))
            .h(height)
            .bg(theme.surface_secondary())
            .rounded(border_radius)
            .overflow_hidden()
            .child(
                div()
                    .h_full()
                    .w(relative(fill_width))
                    .bg(fill_color)
                    .rounded(border_radius),
            )
    }
}
