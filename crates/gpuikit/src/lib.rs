//! gpuikit

pub use gpuikit_theme as theme;

pub mod error;
pub mod layout;
pub mod resource;

pub mod style {
    use gpuikit_theme::Theme;

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
