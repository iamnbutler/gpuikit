//! Separator component for gpuikit
//!
//! A visual divider that can be horizontal or vertical.

use crate::theme::{ActiveTheme, Themeable};
use gpui::{div, px, IntoElement, Pixels, Styled};

/// Orientation of the separator
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Orientation {
    #[default]
    Horizontal,
    Vertical,
}

/// A separator component for visually dividing content.
#[derive(IntoElement)]
pub struct Separator {
    orientation: Orientation,
    size: Option<Pixels>,
    inset: Option<Pixels>,
}

impl Default for Separator {
    fn default() -> Self {
        Self::new()
    }
}

impl Separator {
    pub fn new() -> Self {
        Self {
            orientation: Orientation::Horizontal,
            size: None,
            inset: None,
        }
    }

    /// Create a horizontal separator
    pub fn horizontal() -> Self {
        Self::new().orientation(Orientation::Horizontal)
    }

    /// Create a vertical separator
    pub fn vertical() -> Self {
        Self::new().orientation(Orientation::Vertical)
    }

    /// Set the orientation of the separator
    pub fn orientation(mut self, orientation: Orientation) -> Self {
        self.orientation = orientation;
        self
    }

    /// Set the size (length) of the separator.
    /// For horizontal separators, this sets the width.
    /// For vertical separators, this sets the height.
    /// If not set, the separator will fill its container.
    pub fn size(mut self, size: impl Into<Pixels>) -> Self {
        self.size = Some(size.into());
        self
    }

    /// Set inset (margin) on both ends of the separator
    pub fn inset(mut self, inset: impl Into<Pixels>) -> Self {
        self.inset = Some(inset.into());
        self
    }
}

impl gpui::RenderOnce for Separator {
    fn render(self, _window: &mut gpui::Window, cx: &mut gpui::App) -> impl IntoElement {
        let theme = cx.theme();
        let border_color = theme.border_subtle();

        let base = div().flex_shrink_0().bg(border_color);

        match self.orientation {
            Orientation::Horizontal => {
                let el = base.h(px(1.0)).w_full();
                let el = if let Some(size) = self.size {
                    el.w(size)
                } else {
                    el
                };
                if let Some(inset) = self.inset {
                    el.mx(inset)
                } else {
                    el
                }
            }
            Orientation::Vertical => {
                let el = base.w(px(1.0)).h_full();
                let el = if let Some(size) = self.size {
                    el.h(size)
                } else {
                    el
                };
                if let Some(inset) = self.inset {
                    el.my(inset)
                } else {
                    el
                }
            }
        }
    }
}

/// Convenience function to create a horizontal separator
pub fn separator() -> Separator {
    Separator::horizontal()
}

/// Convenience function to create a vertical separator
pub fn vertical_separator() -> Separator {
    Separator::vertical()
}
