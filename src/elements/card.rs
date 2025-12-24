use crate::theme::{ActiveTheme, Themeable};
use gpui::{
    div, prelude::FluentBuilder, rems, AnyElement, App, IntoElement, ParentElement, RenderOnce,
    SharedString, Styled, Window,
};

pub fn card() -> Card {
    Card::new()
}

#[derive(IntoElement)]
pub struct Card {
    header: Option<AnyElement>,
    body: Option<AnyElement>,
    footer: Option<AnyElement>,
    title: Option<SharedString>,
    description: Option<SharedString>,
}

impl Card {
    pub fn new() -> Self {
        Card {
            header: None,
            body: None,
            footer: None,
            title: None,
            description: None,
        }
    }

    pub fn title(mut self, title: impl Into<SharedString>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn description(mut self, description: impl Into<SharedString>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn header(mut self, header: impl IntoElement) -> Self {
        self.header = Some(header.into_any_element());
        self
    }

    pub fn body(mut self, body: impl IntoElement) -> Self {
        self.body = Some(body.into_any_element());
        self
    }

    pub fn footer(mut self, footer: impl IntoElement) -> Self {
        self.footer = Some(footer.into_any_element());
        self
    }
}

impl Default for Card {
    fn default() -> Self {
        Self::new()
    }
}

impl RenderOnce for Card {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();

        let has_title_section = self.title.is_some() || self.description.is_some();

        div()
            .flex()
            .flex_col()
            .bg(theme.surface())
            .border_1()
            .border_color(theme.border())
            .rounded(rems(0.5))
            .overflow_hidden()
            .when_some(self.header, |card, header| {
                card.child(
                    div()
                        .p(rems(0.75))
                        .border_b_1()
                        .border_color(theme.border_subtle())
                        .child(header),
                )
            })
            .when(has_title_section, |card| {
                card.child(
                    div()
                        .p(rems(0.75))
                        .flex()
                        .flex_col()
                        .gap(rems(0.25))
                        .when_some(self.title, |section, title| {
                            section.child(
                                div()
                                    .text_sm()
                                    .font_weight(gpui::FontWeight::SEMIBOLD)
                                    .text_color(theme.fg())
                                    .child(title),
                            )
                        })
                        .when_some(self.description, |section, description| {
                            section.child(
                                div()
                                    .text_xs()
                                    .text_color(theme.fg_muted())
                                    .child(description),
                            )
                        }),
                )
            })
            .when_some(self.body, |card, body| {
                card.child(div().p(rems(0.75)).child(body))
            })
            .when_some(self.footer, |card, footer| {
                card.child(
                    div()
                        .p(rems(0.75))
                        .border_t_1()
                        .border_color(theme.border_subtle())
                        .child(footer),
                )
            })
    }
}
