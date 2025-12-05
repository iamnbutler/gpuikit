use gpui::{
    div, img, rems, AbsoluteLength, App, ImageSource, Img, IntoElement, ParentElement, RenderOnce,
    Styled, Window,
};
use gpuikit_theme::{ActiveTheme, Themeable};

pub fn avatar(src: impl Into<ImageSource>) -> Avatar {
    Avatar::new(src)
}

#[derive(IntoElement)]
pub struct Avatar {
    image: Img,
    size: Option<AbsoluteLength>,
}

impl Avatar {
    pub fn new(src: impl Into<ImageSource>) -> Self {
        Avatar {
            image: img(src),
            size: None,
        }
    }

    pub fn size<L: Into<AbsoluteLength>>(mut self, size: impl Into<Option<L>>) -> Self {
        self.size = size.into().map(Into::into);
        self
    }
}

impl RenderOnce for Avatar {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let image_size = self.size.unwrap_or_else(|| rems(1.).into());
        let container_size = image_size.to_pixels(window.rem_size());

        div().size(container_size).rounded_full().child(
            self.image
                .size(image_size)
                .rounded_full()
                .bg(cx.theme().surface()),
        )
    }
}
