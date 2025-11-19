use gpui::{
    div, img, prelude::FluentBuilder, px, rems, AbsoluteLength, App, Hsla, ImageSource, Img,
    IntoElement, ParentElement, RenderOnce, Styled, Window,
};
use gpuikit_theme::ActiveTheme;

#[derive(IntoElement)]
pub struct Avatar {
    image: Img,
    size: Option<AbsoluteLength>,
    border_color: Option<Hsla>,
}

impl Avatar {
    pub fn new(src: impl Into<ImageSource>) -> Self {
        Avatar {
            image: img(src),
            size: None,
            border_color: None,
        }
    }

    pub fn border_color(mut self, color: impl Into<Hsla>) -> Self {
        self.border_color = Some(color.into());
        self
    }

    pub fn size<L: Into<AbsoluteLength>>(mut self, size: impl Into<Option<L>>) -> Self {
        self.size = size.into().map(Into::into);
        self
    }
}

impl RenderOnce for Avatar {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let border_width = if self.border_color.is_some() {
            px(2.)
        } else {
            px(0.)
        };

        let image_size = self.size.unwrap_or_else(|| rems(1.).into());
        let container_size = image_size.to_pixels(window.rem_size()) + border_width * 2.;

        div()
            .size(container_size)
            .rounded_full()
            .when_some(self.border_color, |this, color| {
                this.border(border_width).border_color(color)
            })
            .child(
                self.image
                    .size(image_size)
                    .rounded_full()
                    .bg(cx.theme().surface),
            )
    }
}
