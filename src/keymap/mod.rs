//! Keymap module for managing keyboard shortcuts and their associated actions
//!
//! This module provides JSON-based keymap configuration support, allowing
//! keybindings to be loaded from external files rather than hardcoded.

use anyhow::{anyhow, Context as _, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub mod extensions;

/// Represents a complete keymap configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Keymap {
    /// Optional context where these bindings apply (e.g., "Editor", "Menu")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<String>,

    /// The key bindings in this keymap
    pub bindings: HashMap<String, String>,
}

impl Keymap {
    /// Create a new keymap with the given bindings
    pub fn new(bindings: HashMap<String, String>) -> Self {
        Self {
            context: None,
            bindings,
        }
    }

    /// Create a new keymap with context
    pub fn with_context(context: impl Into<String>, bindings: HashMap<String, String>) -> Self {
        Self {
            context: Some(context.into()),
            bindings,
        }
    }
}

/// A collection of keymaps, typically loaded from multiple files
#[derive(Debug, Default)]
pub struct KeymapCollection {
    keymaps: Vec<Keymap>,
}

impl KeymapCollection {
    /// Create a new empty keymap collection
    pub fn new() -> Self {
        Self::default()
    }

    /// Load a keymap from a JSON file
    pub fn load_file<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        let path = path.as_ref();
        let contents = fs::read_to_string(path)
            .with_context(|| format!("Failed to read keymap file: {}", path.display()))?;

        self.load_json(&contents)
            .with_context(|| format!("Failed to parse keymap file: {}", path.display()))?;

        Ok(())
    }

    /// Load keymaps from a JSON string
    pub fn load_json(&mut self, json: &str) -> Result<()> {
        // Try parsing as an array first (multiple keymaps)
        if let Ok(keymaps) = serde_json::from_str::<Vec<Keymap>>(json) {
            self.keymaps.extend(keymaps);
            return Ok(());
        }

        // Try parsing as a single keymap
        if let Ok(keymap) = serde_json::from_str::<Keymap>(json) {
            self.keymaps.push(keymap);
            return Ok(());
        }

        Err(anyhow!("Invalid keymap JSON format"))
    }

    /// Load default keymaps
    pub fn load_defaults(&mut self) -> Result<()> {
        let default_keymap = include_str!("../../assets/default-keymap.json");
        self.load_json(default_keymap)?;
        Ok(())
    }

    /// Get all key binding specifications from this collection
    ///
    /// Returns a list of binding specifications that can be used to create
    /// actual GPUI key bindings with concrete action types.
    pub fn get_binding_specs(&self) -> Vec<BindingSpec> {
        let mut specs = Vec::new();

        for keymap in &self.keymaps {
            let context = keymap.context.as_deref();

            for (keystrokes, action_name) in &keymap.bindings {
                specs.push(BindingSpec {
                    keystrokes: keystrokes.clone(),
                    action_name: action_name.clone(),
                    context: context.map(String::from),
                });
            }
        }

        specs
    }

    /// Get all keymaps in this collection
    pub fn keymaps(&self) -> &[Keymap] {
        &self.keymaps
    }

    /// Add a keymap to this collection
    pub fn add(&mut self, keymap: Keymap) {
        self.keymaps.push(keymap);
    }

    /// Clear all keymaps from this collection
    pub fn clear(&mut self) {
        self.keymaps.clear();
    }

    /// Find all bindings for a given action
    pub fn find_bindings_for_action(&self, action_name: &str) -> Vec<BindingSpec> {
        self.get_binding_specs()
            .into_iter()
            .filter(|spec| spec.action_name == action_name)
            .collect()
    }

    /// Find the action for a given keystroke in a context
    pub fn find_action(&self, keystrokes: &str, context: Option<&str>) -> Option<&str> {
        // First try to find a binding with matching context
        if let Some(context) = context {
            for keymap in &self.keymaps {
                if keymap.context.as_deref() == Some(context) {
                    if let Some(action) = keymap.bindings.get(keystrokes) {
                        return Some(action);
                    }
                }
            }
        }

        // Then try bindings without context (global)
        for keymap in &self.keymaps {
            if keymap.context.is_none() {
                if let Some(action) = keymap.bindings.get(keystrokes) {
                    return Some(action);
                }
            }
        }

        None
    }
}

/// Specification for a key binding
#[derive(Debug, Clone)]
pub struct BindingSpec {
    /// The keystroke sequence (e.g., "cmd-s", "ctrl-shift-p")
    pub keystrokes: String,
    /// The action name to trigger
    pub action_name: String,
    /// Optional context where this binding applies
    pub context: Option<String>,
}

/// Helper function to create a simple binding
pub fn binding(key: impl Into<String>, action: impl Into<String>) -> (String, String) {
    (key.into(), action.into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_keymap() {
        let json = r#"{
            "context": "Editor",
            "bindings": {
                "cmd-s": "Save",
                "cmd-z": "Undo"
            }
        }"#;

        let keymap: Keymap = serde_json::from_str(json).unwrap();
        assert_eq!(keymap.context, Some("Editor".to_string()));
        assert_eq!(keymap.bindings.len(), 2);
        assert_eq!(keymap.bindings.get("cmd-s"), Some(&"Save".to_string()));
        assert_eq!(keymap.bindings.get("cmd-z"), Some(&"Undo".to_string()));
    }

    #[test]
    fn test_parse_multiple_keymaps() {
        let json = r#"[
            {
                "bindings": {
                    "cmd-s": "Save",
                    "cmd-z": "Undo"
                }
            },
            {
                "context": "Menu",
                "bindings": {
                    "enter": "Select",
                    "escape": "Cancel"
                }
            }
        ]"#;

        let keymaps: Vec<Keymap> = serde_json::from_str(json).unwrap();
        assert_eq!(keymaps.len(), 2);
        assert_eq!(keymaps[0].context, None);
        assert_eq!(keymaps[1].context, Some("Menu".to_string()));
    }

    #[test]
    fn test_keymap_collection() {
        let mut collection = KeymapCollection::new();

        let json1 = r#"{ "bindings": { "cmd-s": "Save" } }"#;
        let json2 = r#"{ "context": "Menu", "bindings": { "enter": "Select" } }"#;

        collection.load_json(json1).unwrap();
        collection.load_json(json2).unwrap();

        assert_eq!(collection.keymaps().len(), 2);
        assert_eq!(collection.keymaps()[0].context, None);
        assert_eq!(collection.keymaps()[1].context, Some("Menu".to_string()));

        let specs = collection.get_binding_specs();
        assert_eq!(specs.len(), 2);
        assert_eq!(specs[0].keystrokes, "cmd-s");
        assert_eq!(specs[0].action_name, "Save");
        assert_eq!(specs[0].context, None);
    }

    #[test]
    fn test_find_action() {
        let mut collection = KeymapCollection::new();

        collection.add(Keymap::new(
            [("cmd-s", "Save"), ("cmd-z", "Undo")]
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
        ));

        collection.add(Keymap::with_context(
            "Editor",
            [("cmd-x", "Cut")]
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
        ));

        // Global binding
        assert_eq!(collection.find_action("cmd-s", None), Some("Save"));
        assert_eq!(
            collection.find_action("cmd-s", Some("Editor")),
            Some("Save")
        );

        // Context-specific binding
        assert_eq!(collection.find_action("cmd-x", Some("Editor")), Some("Cut"));
        assert_eq!(collection.find_action("cmd-x", None), None);
        assert_eq!(collection.find_action("cmd-x", Some("Menu")), None);
    }

    #[test]
    fn test_serialize_keymap() {
        let mut bindings = HashMap::new();
        bindings.insert("cmd-s".to_string(), "Save".to_string());
        bindings.insert("cmd-z".to_string(), "Undo".to_string());

        let keymap = Keymap::with_context("Editor", bindings);

        let json = serde_json::to_string_pretty(&keymap).unwrap();
        let parsed: Keymap = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.context, keymap.context);
        assert_eq!(parsed.bindings, keymap.bindings);
    }
}
