//! Flexible theming system for GPUI applications
//!
//! This crate provides a comprehensive theming system with built-in themes,
//! custom theme support, and dynamic theme switching capabilities.

use anyhow::{anyhow, Result};
use gpui::{hsla, point, px, Hsla, Pixels, Point, Rgba, SharedString};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

pub mod colors;
pub mod styles;

// Re-export commonly used types
pub use colors::{ColorPalette, ColorScale, SystemColors};
pub use styles::{BorderStyle, Spacing, Typography};

/// A complete theme definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    /// Unique identifier for the theme
    pub id: String,
    /// Display name of the theme
    pub name: String,
    /// Theme appearance (light or dark)
    pub appearance: ThemeAppearance,
    /// Color definitions
    pub colors: ThemeColors,
    /// Typography settings
    pub typography: Typography,
    /// Spacing and sizing
    pub spacing: Spacing,
    /// Component-specific styles
    pub components: ComponentStyles,
    /// Custom metadata
    #[serde(default)]
    pub metadata: ThemeMetadata,
}

/// Theme appearance mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ThemeAppearance {
    Light,
    Dark,
    Auto,
}

/// Core theme colors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeColors {
    /// Background colors
    pub background: ColorScale,
    /// Foreground/text colors
    pub foreground: ColorScale,
    /// Primary accent color
    pub primary: ColorScale,
    /// Secondary accent color
    pub secondary: ColorScale,
    /// Success/positive color
    pub success: ColorScale,
    /// Warning color
    pub warning: ColorScale,
    /// Error/danger color
    pub error: ColorScale,
    /// Info color
    pub info: ColorScale,
    /// Border colors
    pub border: ColorScale,
    /// System colors (window, controls, etc.)
    pub system: SystemColors,
}

/// Component-specific styling
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ComponentStyles {
    /// Button styles
    #[serde(default)]
    pub button: ButtonStyle,
    /// Input field styles
    #[serde(default)]
    pub input: InputStyle,
    /// Panel/container styles
    #[serde(default)]
    pub panel: PanelStyle,
    /// Custom component styles
    #[serde(default)]
    pub custom: HashMap<String, serde_json::Value>,
}

/// Button styling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ButtonStyle {
    pub padding: Spacing,
    pub border_radius: Pixels,
    pub border_width: Pixels,
    #[serde(default)]
    pub variants: HashMap<String, ButtonVariant>,
}

impl Default for ButtonStyle {
    fn default() -> Self {
        Self {
            padding: Spacing::default(),
            border_radius: px(4.0),
            border_width: px(1.0),
            variants: HashMap::new(),
        }
    }
}

/// Button variant styling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ButtonVariant {
    pub background: Hsla,
    pub foreground: Hsla,
    pub border: Hsla,
    #[serde(default)]
    pub hover_background: Option<Hsla>,
    #[serde(default)]
    pub hover_foreground: Option<Hsla>,
    #[serde(default)]
    pub active_background: Option<Hsla>,
}

/// Input field styling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputStyle {
    pub padding: Spacing,
    pub border_radius: Pixels,
    pub border_width: Pixels,
    pub background: Hsla,
    pub foreground: Hsla,
    pub border: Hsla,
    pub placeholder: Hsla,
}

impl Default for InputStyle {
    fn default() -> Self {
        Self {
            padding: Spacing::default(),
            border_radius: px(4.0),
            border_width: px(1.0),
            background: hsla(0.0, 0.0, 1.0, 1.0),
            foreground: hsla(0.0, 0.0, 0.0, 1.0),
            border: hsla(0.0, 0.0, 0.8, 1.0),
            placeholder: hsla(0.0, 0.0, 0.5, 1.0),
        }
    }
}

/// Panel/container styling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PanelStyle {
    pub padding: Spacing,
    pub background: Hsla,
    pub border: Option<BorderStyle>,
}

impl Default for PanelStyle {
    fn default() -> Self {
        Self {
            padding: Spacing::default(),
            background: hsla(0.0, 0.0, 1.0, 1.0),
            border: None,
        }
    }
}

/// Theme metadata
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ThemeMetadata {
    /// Theme author
    #[serde(default)]
    pub author: Option<String>,
    /// Theme version
    #[serde(default)]
    pub version: Option<String>,
    /// Theme description
    #[serde(default)]
    pub description: Option<String>,
    /// Parent theme (for variants)
    #[serde(default)]
    pub parent: Option<String>,
    /// Custom metadata fields
    #[serde(flatten)]
    pub custom: HashMap<String, serde_json::Value>,
}

/// Theme manager for loading and switching themes
pub struct ThemeManager {
    themes: HashMap<String, Arc<Theme>>,
    active_theme: Arc<Theme>,
}

impl ThemeManager {
    /// Create a new theme manager with default theme
    pub fn new() -> Self {
        let default_theme = Self::default_theme();
        let mut themes = HashMap::new();
        themes.insert(default_theme.id.clone(), Arc::new(default_theme.clone()));

        Self {
            themes,
            active_theme: Arc::new(default_theme),
        }
    }

    /// Register a new theme
    pub fn register(&mut self, theme: Theme) {
        self.themes.insert(theme.id.clone(), Arc::new(theme));
    }

    /// Load theme from JSON
    pub fn load_from_json(&mut self, json: &str) -> Result<()> {
        let theme: Theme = serde_json::from_str(json)?;
        self.register(theme);
        Ok(())
    }

    /// Set active theme by ID
    pub fn set_active(&mut self, theme_id: &str) -> Result<()> {
        let theme = self
            .themes
            .get(theme_id)
            .ok_or_else(|| anyhow!("Theme not found: {}", theme_id))?;
        self.active_theme = Arc::clone(theme);
        Ok(())
    }

    /// Get the active theme
    pub fn active(&self) -> Arc<Theme> {
        Arc::clone(&self.active_theme)
    }

    /// Get a theme by ID
    pub fn get(&self, theme_id: &str) -> Option<Arc<Theme>> {
        self.themes.get(theme_id).cloned()
    }

    /// List all available themes
    pub fn list(&self) -> Vec<&str> {
        self.themes.keys().map(|s| s.as_str()).collect()
    }

    /// Get default light theme
    fn default_theme() -> Theme {
        Theme {
            id: "default-light".to_string(),
            name: "Default Light".to_string(),
            appearance: ThemeAppearance::Light,
            colors: ThemeColors::default_light(),
            typography: Typography::default(),
            spacing: Spacing::default(),
            components: ComponentStyles::default(),
            metadata: ThemeMetadata::default(),
        }
    }
}

impl Default for ThemeManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ThemeColors {
    /// Create default light theme colors
    pub fn default_light() -> Self {
        Self {
            background: ColorScale::grayscale_light(),
            foreground: ColorScale::grayscale_dark(),
            primary: ColorScale::blue(),
            secondary: ColorScale::purple(),
            success: ColorScale::green(),
            warning: ColorScale::yellow(),
            error: ColorScale::red(),
            info: ColorScale::cyan(),
            border: ColorScale::grayscale_light(),
            system: SystemColors::default_light(),
        }
    }

    /// Create default dark theme colors
    pub fn default_dark() -> Self {
        Self {
            background: ColorScale::grayscale_dark(),
            foreground: ColorScale::grayscale_light(),
            primary: ColorScale::blue(),
            secondary: ColorScale::purple(),
            success: ColorScale::green(),
            warning: ColorScale::yellow(),
            error: ColorScale::red(),
            info: ColorScale::cyan(),
            border: ColorScale::grayscale_dark(),
            system: SystemColors::default_dark(),
        }
    }
}

/// Built-in themes module
#[cfg(feature = "builtin-themes")]
pub mod builtin {
    use super::*;

    /// Get all built-in themes
    pub fn all() -> Vec<Theme> {
        vec![light(), dark(), solarized_light(), solarized_dark(), nord()]
    }

    /// Default light theme
    pub fn light() -> Theme {
        Theme {
            id: "builtin-light".to_string(),
            name: "Light".to_string(),
            appearance: ThemeAppearance::Light,
            colors: ThemeColors::default_light(),
            typography: Typography::default(),
            spacing: Spacing::default(),
            components: ComponentStyles::default(),
            metadata: ThemeMetadata {
                author: Some("GPUIKit".to_string()),
                version: Some("1.0.0".to_string()),
                description: Some("Default light theme".to_string()),
                ..Default::default()
            },
        }
    }

    /// Default dark theme
    pub fn dark() -> Theme {
        Theme {
            id: "builtin-dark".to_string(),
            name: "Dark".to_string(),
            appearance: ThemeAppearance::Dark,
            colors: ThemeColors::default_dark(),
            typography: Typography::default(),
            spacing: Spacing::default(),
            components: ComponentStyles::default(),
            metadata: ThemeMetadata {
                author: Some("GPUIKit".to_string()),
                version: Some("1.0.0".to_string()),
                description: Some("Default dark theme".to_string()),
                ..Default::default()
            },
        }
    }

    /// Solarized light theme
    pub fn solarized_light() -> Theme {
        // Implementation would include Solarized color scheme
        light() // Placeholder
    }

    /// Solarized dark theme
    pub fn solarized_dark() -> Theme {
        // Implementation would include Solarized color scheme
        dark() // Placeholder
    }

    /// Nord theme
    pub fn nord() -> Theme {
        // Implementation would include Nord color scheme
        dark() // Placeholder
    }
}

/// Hot-reload support for theme development
#[cfg(feature = "hot-reload")]
pub mod hot_reload {
    use super::*;
    use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
    use std::path::PathBuf;
    use std::sync::mpsc;

    /// Theme watcher for hot-reloading
    pub struct ThemeWatcher {
        watcher: RecommendedWatcher,
        rx: mpsc::Receiver<notify::Result<Event>>,
    }

    impl ThemeWatcher {
        /// Create a new theme watcher
        pub fn new(theme_path: PathBuf) -> Result<Self> {
            let (tx, rx) = mpsc::channel();
            let mut watcher = notify::recommended_watcher(tx)?;
            watcher.watch(&theme_path, RecursiveMode::NonRecursive)?;

            Ok(Self { watcher, rx })
        }

        /// Check for theme changes
        pub fn check_changes(&self) -> Option<PathBuf> {
            if let Ok(Ok(event)) = self.rx.try_recv() {
                if let EventKind::Modify(_) = event.kind {
                    event.paths.first().cloned()
                } else {
                    None
                }
            } else {
                None
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_creation() {
        let theme = Theme {
            id: "test".to_string(),
            name: "Test Theme".to_string(),
            appearance: ThemeAppearance::Light,
            colors: ThemeColors::default_light(),
            typography: Typography::default(),
            spacing: Spacing::default(),
            components: ComponentStyles::default(),
            metadata: ThemeMetadata::default(),
        };

        assert_eq!(theme.id, "test");
        assert_eq!(theme.appearance, ThemeAppearance::Light);
    }

    #[test]
    fn test_theme_manager() {
        let mut manager = ThemeManager::new();
        assert!(!manager.list().is_empty());

        let theme = Theme {
            id: "custom".to_string(),
            name: "Custom Theme".to_string(),
            appearance: ThemeAppearance::Dark,
            colors: ThemeColors::default_dark(),
            typography: Typography::default(),
            spacing: Spacing::default(),
            components: ComponentStyles::default(),
            metadata: ThemeMetadata::default(),
        };

        manager.register(theme);
        assert!(manager.get("custom").is_some());
    }
}
