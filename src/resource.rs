//! Resource management for gpui apps
//!
//! Embed icons, images, fonts, etc...

use anyhow::Result;
use gpui::{AssetSource, SharedString};
use rust_embed::RustEmbed;
use std::borrow::Cow;

/// Trait for types that can serve as embedded asset sources
pub trait Resource: RustEmbed {
    /// Get an asset by path
    fn by_path(path: &str) -> Option<Cow<'static, [u8]>> {
        Self::get(path).map(|file| file.data)
    }

    /// Check if an asset exists
    fn has(path: &str) -> bool {
        Self::get(path).is_some()
    }

    /// List all assets
    fn list(prefix: Option<&str>) -> Vec<String> {
        match prefix {
            Some(prefix) => Self::iter()
                .filter(|path| path.starts_with(prefix))
                .map(|s| s.to_string())
                .collect(),
            None => Self::iter().map(|s| s.to_string()).collect(),
        }
    }
}

impl<T: RustEmbed> Resource for T {}

pub struct ResourceSource<T: Resource + Send + Sync> {
    _phantom: std::marker::PhantomData<T>,
}

impl<T: Resource + Send + Sync> Default for ResourceSource<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Resource + Send + Sync> ResourceSource<T> {
    /// Create a new embedded asset source
    pub fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<T: Resource + Send + Sync + 'static> AssetSource for ResourceSource<T> {
    fn load(&self, path: &str) -> Result<Option<Cow<'static, [u8]>>> {
        Ok(T::by_path(path))
    }

    fn list(&self, prefix: &str) -> Result<Vec<SharedString>> {
        Ok(T::list(Some(prefix))
            .into_iter()
            .map(|s| s.into())
            .collect())
    }
}
