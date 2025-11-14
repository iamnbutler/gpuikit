//! gpuikit

pub use gpuikit_assets as assets;
pub use gpuikit_theme as theme;
pub use gpuikit_utils as utils;

// Re-export commonly used types from theme
pub use theme::{
    BorderStyle, Shadow, Spacing, Theme, ThemeAppearance, ThemeColors, ThemeManager, ThemeMetadata,
    Typography,
};

// Re-export commonly used types from assets
pub use assets::{AssetManager, EmbeddedAssetSource, EmbeddedAssets};

// Re-export commonly used utilities
pub use utils::{
    geometry::{center_rect, expand_bounds, point_in_bounds},
    string::{truncate_string, wrap_text},
    task::{debounce, throttle},
};

/// Prelude module for convenient imports
pub mod prelude {
    pub use crate::assets::{AssetManager, EmbeddedAssets};
    pub use crate::theme::{Theme, ThemeAppearance, ThemeManager};
    pub use crate::utils::{center_rect, point_in_bounds};

    // Re-export gpui for convenience
    pub use gpui;
}

/// UI Components module
pub mod components {
    //! Pre-built UI components for GPUI applications
    //!
    //! This module will contain reusable UI components like buttons,
    //! inputs, panels, etc. Currently being developed.

    use crate::theme::Theme;
    use gpui::*;

    /// Button component (placeholder for future implementation)
    #[allow(dead_code)]
    pub struct Button {
        label: SharedString,
        theme: Theme,
    }

    /// Input field component (placeholder for future implementation)
    #[allow(dead_code)]
    pub struct Input {
        placeholder: Option<SharedString>,
        theme: Theme,
    }

    /// Panel component (placeholder for future implementation)
    #[allow(dead_code)]
    pub struct Panel {
        theme: Theme,
    }
}

/// Styling utilities and helpers
pub mod style {
    //! Styling utilities for consistent component appearance

    use crate::theme::Theme;
    use gpui::*;

    /// Apply theme colors to an element
    pub trait Themed {
        /// Apply theme styling
        fn themed(self, theme: &Theme) -> Self;
    }

    /// Get the appropriate text color for a background
    pub fn contrast_color(background: Hsla) -> Hsla {
        if background.l > 0.5 {
            // Light background, use dark text
            hsla(0.0, 0.0, 0.13, 1.0)
        } else {
            // Dark background, use light text
            hsla(0.0, 0.0, 0.91, 1.0)
        }
    }

    /// Create a focus ring style
    pub fn focus_ring(theme: &Theme) -> BoxStyle {
        BoxStyle {
            border_color: Some(theme.colors.primary.shade_500),
            border_width: Some(px(2.0)),
            ..Default::default()
        }
    }

    /// Box styling helper
    #[derive(Debug, Clone, Default)]
    pub struct BoxStyle {
        /// Background color
        pub background: Option<Hsla>,
        /// Border color
        pub border_color: Option<Hsla>,
        /// Border width
        pub border_width: Option<Pixels>,
        /// Border radius
        pub border_radius: Option<Pixels>,
        /// Padding
        pub padding: Option<Pixels>,
    }
}

/// Layout utilities and helpers
pub mod layout {
    //! Layout utilities for positioning and sizing elements

    use gpui::*;

    /// Stack layout direction
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum StackDirection {
        /// Horizontal (row) layout
        Horizontal,
        /// Vertical (column) layout
        Vertical,
    }

    /// Alignment options
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum Alignment {
        /// Start alignment
        Start,
        /// Center alignment
        Center,
        /// End alignment
        End,
        /// Stretch to fill
        Stretch,
        /// Space between items
        SpaceBetween,
        /// Space around items
        SpaceAround,
        /// Space evenly
        SpaceEvenly,
    }

    /// Create a flex container with common settings
    pub fn flex_container(direction: StackDirection, gap: Pixels) -> Div {
        let mut container = div();

        match direction {
            StackDirection::Horizontal => {
                container = container.flex_row();
            }
            StackDirection::Vertical => {
                container = container.flex_col();
            }
        }

        container.gap(gap)
    }

    /// Center an element within its container
    pub fn center(child: impl IntoElement) -> Div {
        div().flex().items_center().justify_center().child(child)
    }
}

/// Animation utilities
pub mod animation {
    //! Animation and transition utilities

    use std::time::Duration;

    /// Standard animation durations
    pub struct Durations;

    impl Durations {
        /// Fast animation (150ms)
        pub const FAST: Duration = Duration::from_millis(150);
        /// Normal animation (250ms)
        pub const NORMAL: Duration = Duration::from_millis(250);
        /// Slow animation (350ms)
        pub const SLOW: Duration = Duration::from_millis(350);
    }

    /// Easing functions for animations
    pub mod easing {
        /// Linear easing
        pub fn linear(t: f32) -> f32 {
            t
        }

        /// Ease in (accelerate)
        pub fn ease_in(t: f32) -> f32 {
            t * t
        }

        /// Ease out (decelerate)
        pub fn ease_out(t: f32) -> f32 {
            t * (2.0 - t)
        }

        /// Ease in-out (accelerate then decelerate)
        pub fn ease_in_out(t: f32) -> f32 {
            if t < 0.5 {
                2.0 * t * t
            } else {
                -1.0 + (4.0 - 2.0 * t) * t
            }
        }
    }
}

/// Error types for GPUIKit
pub mod error {
    //! Error types used throughout GPUIKit

    use std::fmt;

    /// General GPUIKit error type
    #[derive(Debug)]
    pub enum Error {
        /// Theme not found
        ThemeNotFound(String),
        /// Asset not found
        AssetNotFound(String),
        /// Invalid configuration
        InvalidConfig(String),
        /// Other error with message
        Other(String),
    }

    impl fmt::Display for Error {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                Error::ThemeNotFound(name) => write!(f, "Theme not found: {}", name),
                Error::AssetNotFound(path) => write!(f, "Asset not found: {}", path),
                Error::InvalidConfig(msg) => write!(f, "Invalid configuration: {}", msg),
                Error::Other(msg) => write!(f, "Error: {}", msg),
            }
        }
    }

    impl std::error::Error for Error {}

    /// Result type alias for GPUIKit operations
    pub type Result<T> = std::result::Result<T, Error>;
}

// Version information
/// Current version of GPUIKit
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Get version information
pub fn version() -> &'static str {
    VERSION
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert!(!version().is_empty());
    }

    #[test]
    fn test_contrast_color() {
        use gpui::hsla;

        let light_bg = hsla(0.0, 0.0, 0.9, 1.0);
        let dark_bg = hsla(0.0, 0.0, 0.1, 1.0);

        let light_text = style::contrast_color(dark_bg);
        let dark_text = style::contrast_color(light_bg);

        assert!(light_text.l > dark_text.l);
    }
}
