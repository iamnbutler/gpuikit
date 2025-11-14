//! Asset management for GPUI applications
//!
//! This crate provides utilities for embedding and managing assets in GPUI applications.

use anyhow::{anyhow, Result};
use gpui::{AssetSource, SharedString};
use rust_embed::RustEmbed;
use std::borrow::Cow;

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
pub struct AssetManager {}

impl AssetManager {
    /// Create a new asset manager
    pub fn new() -> Self {
        Self {}
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
pub struct EmbeddedAssetSource<T: EmbeddedAssets + Send + Sync> {
    _phantom: std::marker::PhantomData<T>,
}

impl<T: EmbeddedAssets + Send + Sync> Default for EmbeddedAssetSource<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: EmbeddedAssets + Send + Sync> EmbeddedAssetSource<T> {
    /// Create a new embedded asset source
    pub fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<T: EmbeddedAssets + Send + Sync + 'static> AssetSource for EmbeddedAssetSource<T> {
    fn load(&self, path: &str) -> Result<Option<Cow<'static, [u8]>>> {
        Ok(T::get_asset(path))
    }

    fn list(&self, prefix: &str) -> Result<Vec<SharedString>> {
        Ok(T::list_assets(Some(prefix))
            .into_iter()
            .map(|s| s.into())
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_asset_manager() {
        let _manager = AssetManager::new();
        // Just verify that we can create an asset manager
    }
}
