use crate::theme::{ActiveTheme, Themeable};
use gpui::{
    div, hsla, prelude::FluentBuilder, px, rems, App, FontWeight, Hsla, IntoElement, ParentElement,
    RenderOnce, SharedString, Styled, Svg, Window,
};

pub fn alert() -> Alert {
    Alert::new()
}

#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub enum AlertVariant {
    #[default]
    Info,
    Success,
    Warning,
    Destructive,
}

#[derive(IntoElement)]
pub struct Alert {
    variant: AlertVariant,
    icon: Option<Svg>,
    title: Option<SharedString>,
    description: Option<SharedString>,
    dismissible: bool,
    on_dismiss: Option<Box<dyn Fn(&mut Window, &mut App) + 'static>>,
}

impl Alert {
    pub fn new() -> Self {
        Alert {
            variant: AlertVariant::Info,
            icon: None,
            title: None,
            description: None,
            dismissible: false,
            on_dismiss: None,
        }
    }

    pub fn variant(mut self, variant: AlertVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn info(mut self) -> Self {
        self.variant = AlertVariant::Info;
        self
    }

    pub fn success(mut self) -> Self {
        self.variant = AlertVariant::Success;
        self
    }

    pub fn warning(mut self) -> Self {
        self.variant = AlertVariant::Warning;
        self
    }

    pub fn destructive(mut self) -> Self {
        self.variant = AlertVariant::Destructive;
        self
    }

    pub fn icon(mut self, icon: Svg) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn title(mut self, title: impl Into<SharedString>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn description(mut self, description: impl Into<SharedString>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn dismissible(mut self) -> Self {
        self.dismissible = true;
        self
    }

    pub fn on_dismiss(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.dismissible = true;
        self.on_dismiss = Some(Box::new(handler));
        self
    }
}

impl Default for Alert {
    fn default() -> Self {
        Self::new()
    }
}

struct AlertColors {
    bg: Hsla,
    border: Hsla,
    icon: Hsla,
    title: Hsla,
    description: Hsla,
}

fn get_alert_colors(variant: AlertVariant, theme: &impl Themeable) -> AlertColors {
    match variant {
        AlertVariant::Info => {
            let accent = theme.accent();
            AlertColors {
                bg: accent.opacity(0.1),
                border: accent.opacity(0.3),
                icon: accent,
                title: theme.fg(),
                description: theme.fg_muted(),
            }
        }
        AlertVariant::Success => {
            // Green color for success
            let success = hsla(120.0 / 360.0, 0.6, 0.4, 1.0);
            AlertColors {
                bg: success.opacity(0.1),
                border: success.opacity(0.3),
                icon: success,
                title: theme.fg(),
                description: theme.fg_muted(),
            }
        }
        AlertVariant::Warning => {
            // Yellow/amber color for warning
            let warning = hsla(45.0 / 360.0, 0.9, 0.5, 1.0);
            AlertColors {
                bg: warning.opacity(0.1),
                border: warning.opacity(0.3),
                icon: warning,
                title: theme.fg(),
                description: theme.fg_muted(),
            }
        }
        AlertVariant::Destructive => {
            let danger = theme.danger();
            AlertColors {
                bg: danger.opacity(0.1),
                border: danger.opacity(0.3),
                icon: danger,
                title: theme.fg(),
                description: theme.fg_muted(),
            }
        }
    }
}

impl RenderOnce for Alert {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let colors = get_alert_colors(self.variant, theme.as_ref());

        let has_content = self.title.is_some() || self.description.is_some();

        div()
            .flex()
            .w_full()
            .p(rems(0.75))
            .gap(rems(0.75))
            .rounded(rems(0.375))
            .bg(colors.bg)
            .border_1()
            .border_color(colors.border)
            .when_some(self.icon, |container, icon| {
                container.child(
                    div()
                        .flex_shrink_0()
                        .child(icon.size(px(16.0)).text_color(colors.icon)),
                )
            })
            .when(has_content, |container| {
                container.child(
                    div()
                        .flex()
                        .flex_col()
                        .flex_1()
                        .gap(rems(0.25))
                        .when_some(self.title, |content, title| {
                            content.child(
                                div()
                                    .text_sm()
                                    .font_weight(FontWeight::MEDIUM)
                                    .text_color(colors.title)
                                    .child(title),
                            )
                        })
                        .when_some(self.description, |content, description| {
                            content.child(
                                div()
                                    .text_sm()
                                    .text_color(colors.description)
                                    .child(description),
                            )
                        }),
                )
            })
    }
}
