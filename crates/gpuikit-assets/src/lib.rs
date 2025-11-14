//! Asset management and embedding for GPUI applications
//!
//! This crate provides utilities for embedding and managing static assets
//! in GPUI applications using rust-embed.

use anyhow::{anyhow, Result};
use gpui::AssetSource;
use rust_embed::RustEmbed;
use std::borrow::Cow;
use std::path::{Path, PathBuf};

/// Trait for types that can serve as embedded asset sources
pub trait EmbeddedAssets: RustEmbed {
    /// Get an asset by path
    fn get_asset(path: &str) -> Option<Cow<'static, [u8]>> {
        Self::get(path).map(|file| file.data)
    }

    /// Check if an asset exists
    fn has_asset(path: &str) -> bool {
        Self::get(path).is_some()
    }

    /// List all assets matching a pattern
    fn list_assets(pattern: Option<&str>) -> Vec<String> {
        let iter = Self::iter();
        match pattern {
            Some(pat) => iter
                .filter(|path| path.contains(pat))
                .map(|s| s.to_string())
                .collect(),
            None => iter.map(|s| s.to_string()).collect(),
        }
    }
}

/// Default implementation for all RustEmbed types
impl<T: RustEmbed> EmbeddedAssets for T {}

/// Asset manager for loading and caching embedded assets
#[derive(Debug, Clone)]
pub struct AssetManager {
    name: String,
}

impl AssetManager {
    /// Create a new asset manager with a given name
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }

    /// Load an embedded asset
    pub fn load_embedded<T: EmbeddedAssets>(&self, path: &str) -> Result<Vec<u8>> {
        T::get_asset(path)
            .map(|data| data.to_vec())
            .ok_or_else(|| anyhow!("Asset not found: {}", path))
    }

    /// Load an embedded text asset
    pub fn load_text<T: EmbeddedAssets>(&self, path: &str) -> Result<String> {
        let data = self.load_embedded::<T>(path)?;
        String::from_utf8(data).map_err(|e| anyhow!("Invalid UTF-8 in asset {}: {}", path, e))
    }

    /// List all embedded assets
    pub fn list_embedded<T: EmbeddedAssets>(&self, pattern: Option<&str>) -> Vec<String> {
        T::list_assets(pattern)
    }
}

/// GPUI AssetSource implementation for embedded assets
pub struct EmbeddedAssetSource<T: EmbeddedAssets> {
    _phantom: std::marker::PhantomData<T>,
}

impl<T: EmbeddedAssets> Default for EmbeddedAssetSource<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: EmbeddedAssets> EmbeddedAssetSource<T> {
    /// Create a new embedded asset source
    pub fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<T: EmbeddedAssets + 'static> AssetSource for EmbeddedAssetSource<T> {
    fn load(&self, path: &str) -> Result<Cow<'static, [u8]>> {
        T::get_asset(path).ok_or_else(|| anyhow!("Asset not found: {}", path))
    }

    fn list(&self, prefix: &str) -> Result<Vec<PathBuf>> {
        Ok(T::list_assets(Some(prefix))
            .into_iter()
            .map(PathBuf::from)
            .collect())
    }
}

/// Helper macro to define an embedded assets module
#[macro_export]
macro_rules! embed_assets {
    ($name:ident, $folder:expr) => {
        #[derive(::rust_embed::RustEmbed)]
        #[folder = $folder]
        pub struct $name;

        impl $name {
            /// Create a GPUI asset source for these embedded assets
            pub fn asset_source() -> $crate::EmbeddedAssetSource<Self> {
                $crate::EmbeddedAssetSource::new()
            }
        }
    };
}

/// Common asset types
pub mod types {
    use super::*;

    /// Icon asset metadata
    #[cfg(feature = "manifest")]
    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
    pub struct IconAsset {
        pub name: String,
        pub path: String,
        pub size: Option<(u32, u32)>,
        pub category: Option<String>,
    }

    /// Font asset metadata
    #[cfg(feature = "manifest")]
    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
    pub struct FontAsset {
        pub name: String,
        pub path: String,
        pub family: String,
        pub weight: Option<u32>,
        pub style: Option<String>,
    }

    /// Asset manifest for cataloging embedded assets
    #[cfg(feature = "manifest")]
    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
    pub struct AssetManifest {
        pub icons: Vec<IconAsset>,
        pub fonts: Vec<FontAsset>,
        pub other: Vec<String>,
    }
}

#[cfg(feature = "icons")]
pub mod icons {
    use super::*;

    embed_assets!(DefaultIcons, "assets/icons");

    /// Load a default icon by name
    pub fn load_icon(name: &str) -> Result<Vec<u8>> {
        let manager = AssetManager::new("default_icons");
        let path = format!("{}.svg", name);
        manager.load_embedded::<DefaultIcons>(&path)
    }

    /// List all available default icons
    pub fn list_icons() -> Vec<String> {
        DefaultIcons::list_assets(Some(".svg"))
            .into_iter()
            .map(|path| {
                Path::new(&path)
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("")
                    .to_string()
            })
            .collect()
    }
}

#[cfg(feature = "fonts")]
pub mod fonts {
    use super::*;

    embed_assets!(DefaultFonts, "assets/fonts");

    /// Load a default font by name
    pub fn load_font(name: &str) -> Result<Vec<u8>> {
        let manager = AssetManager::new("default_fonts");
        manager.load_embedded::<DefaultFonts>(name)
    }

    /// List all available default fonts
    pub fn list_fonts() -> Vec<String> {
        DefaultFonts::list_assets(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    embed_assets!(TestAssets, "tests/assets");

    #[test]
    fn test_embed_assets_macro() {
        // This test will only work if tests/assets exists with some files
        let _source = TestAssets::asset_source();
    }

    #[test]
    fn test_asset_manager() {
        let manager = AssetManager::new("test");
        assert_eq!(manager.name, "test");
    }
}
