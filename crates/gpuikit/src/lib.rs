//! gpuikit

pub use gpuikit_assets as assets;
pub use gpuikit_theme as theme;
pub use gpuikit_utils as utils;

pub mod error;
pub mod layout;
pub mod resource;

pub mod style {
    use crate::theme::Theme;

    pub trait Themed {
        fn themed(self, theme: &Theme) -> Self;
    }

    // todo: is Themed useful?
    //
    // I could see most gpuikit components being something like:
    //
    // pub trait Component: IntoElement + Themed {}
    //
    // where Themed eventually gets more style helpers...
}
