use gpui::{
    div, prelude::FluentBuilder, px, rgb, IntoElement, ParentElement, Point, RenderOnce,
    SharedString, Styled,
};

#[derive(Default, Debug, Clone)]
pub enum Language {
    #[default]
    PlainText,
    Rust,
}

impl Language {
    pub fn label(self) -> SharedString {
        let string: &'static str = match self {
            Language::PlainText => "Plain Text",
            Language::Rust => "Rust",
        };

        string.into()
    }
}

pub struct Selection {
    pub lines: usize,
    pub chars: usize,
}

#[derive(IntoElement)]
pub struct MetaLine {
    cursor_position: Point<usize>,
    language: Language,
    selection: Option<Selection>,
}

impl MetaLine {
    pub fn new(
        cursor_position: Point<usize>,
        language: Language,
        selection: Option<Selection>,
    ) -> Self {
        Self {
            cursor_position,
            language,
            selection,
        }
    }
}

impl RenderOnce for MetaLine {
    fn render(self, _window: &mut gpui::Window, _cx: &mut gpui::App) -> impl gpui::IntoElement {
        div()
            .absolute()
            .right(px(0.0))
            .bottom(px(0.0))
            .h(px(24.0))
            .flex()
            .px_3()
            .child(
                div()
                    .flex()
                    .gap_2()
                    .text_sm()
                    .text_color(rgb(0xaaaaaa))
                    .child(self.language.label())
                    .child(SharedString::from(format!(
                        "{}:{}",
                        self.cursor_position.y + 1,
                        self.cursor_position.x + 1
                    )))
                    .when_some(self.selection, |this, selection| {
                        this.child(SharedString::from(format!("{} chars", selection.chars)))
                    }),
            )
    }
}
