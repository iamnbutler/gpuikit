//! Image element for markdown.

use gpui::{img, prelude::*, px, App, SharedString, Styled};

/// Render an image element.
///
/// Uses GPUI's `img` element to load and display images from URLs or data URIs.
pub fn image(src: impl Into<SharedString>, alt: Option<&str>, _cx: &App) -> impl IntoElement {
    // TODO: Handle alt text accessibility
    // TODO: Add loading/error states
    let _ = alt;

    let src: SharedString = src.into();
    img(src).max_w(px(600.0)).rounded(px(4.0))
}
