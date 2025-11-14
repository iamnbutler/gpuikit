//! Keymap module for managing keyboard shortcuts and their associated actions
//!
//! This module provides JSON-based keymap configuration support, allowing
//! keybindings to be loaded from external files rather than hardcoded.

use anyhow::{anyhow, Context as _, Result};
use gpui::Action;
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

    /// Whether to use platform-specific key equivalents
    #[serde(default)]
    pub use_key_equivalents: bool,

    /// The key bindings in this keymap
    pub bindings: KeyBindings,
}

/// Represents the bindings section of a keymap
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum KeyBindings {
    /// Simple map of keystroke -> action
    Simple(HashMap<String, ActionValue>),
    /// List of individual key binding entries (for more complex configurations)
    Complex(Vec<KeyBindingEntry>),
}

/// Represents a single key binding entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyBindingEntry {
    /// The keystroke sequence (e.g., "cmd-s", "ctrl-shift-p")
    pub key: String,

    /// The action to trigger
    pub action: ActionValue,

    /// Optional context where this binding applies
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<String>,
}

/// Represents an action value in the keymap
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ActionValue {
    /// Simple action name
    Simple(String),
    /// Action with parameters
    WithParams(Vec<serde_json::Value>),
}

impl ActionValue {
    /// Get the action name from this value
    pub fn action_name(&self) -> Result<String> {
        match self {
            ActionValue::Simple(name) => Ok(name.clone()),
            ActionValue::WithParams(values) => {
                if let Some(first) = values.first() {
                    if let Some(name) = first.as_str() {
                        return Ok(name.to_string());
                    }
                }
                Err(anyhow!(
                    "Invalid action format: expected action name as first element"
                ))
            }
        }
    }

    /// Get the action parameters if any
    pub fn params(&self) -> Option<serde_json::Value> {
        match self {
            ActionValue::Simple(_) => None,
            ActionValue::WithParams(values) => {
                if values.len() > 1 {
                    Some(values[1].clone())
                } else {
                    None
                }
            }
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

    /// Load default keymaps for the current platform
    pub fn load_defaults(&mut self) -> Result<()> {
        let default_keymap = default_keymap_json();
        self.load_json(default_keymap)?;
        Ok(())
    }

    /// Get all key binding specifications from this collection
    ///
    /// Returns a list of binding specifications that can be used to create
    /// actual GPUI key bindings with concrete action types.
    pub fn get_binding_specs(&self) -> Result<Vec<BindingSpec>> {
        let mut specs = Vec::new();

        for keymap in &self.keymaps {
            let context = keymap.context.as_deref();

            match &keymap.bindings {
                KeyBindings::Simple(map) => {
                    for (key, action_value) in map {
                        let action_name = action_value.action_name()?;
                        specs.push(BindingSpec {
                            keystrokes: key.clone(),
                            action_name,
                            action_params: action_value.params(),
                            context: context.map(String::from),
                        });
                    }
                }
                KeyBindings::Complex(entries) => {
                    for entry in entries {
                        let action_name = entry.action.action_name()?;
                        let binding_context = entry.context.as_deref().or(context);
                        specs.push(BindingSpec {
                            keystrokes: entry.key.clone(),
                            action_name,
                            action_params: entry.action.params(),
                            context: binding_context.map(String::from),
                        });
                    }
                }
            }
        }

        Ok(specs)
    }

    /// Convert this collection into boxed actions using a registry
    ///
    /// This is primarily for testing and validation purposes.
    pub fn to_actions(
        &self,
        action_registry: &impl ActionRegistry,
    ) -> Result<Vec<(String, Box<dyn Action>, Option<String>)>> {
        let mut actions = Vec::new();

        for spec in self.get_binding_specs()? {
            if let Some(action) = action_registry.get_action(&spec.action_name, spec.action_params)
            {
                actions.push((spec.keystrokes, action, spec.context));
            } else {
                log::warn!("Unknown action in keymap: {}", spec.action_name);
            }
        }

        Ok(actions)
    }

    /// Get all keymaps in this collection
    pub fn keymaps(&self) -> &[Keymap] {
        &self.keymaps
    }

    /// Clear all keymaps from this collection
    pub fn clear(&mut self) {
        self.keymaps.clear();
    }
}

/// Specification for a key binding that can be used to create actual bindings
#[derive(Debug, Clone)]
pub struct BindingSpec {
    /// The keystroke sequence (e.g., "cmd-s", "ctrl-shift-p")
    pub keystrokes: String,
    /// The action name to trigger
    pub action_name: String,
    /// Optional parameters for the action
    pub action_params: Option<serde_json::Value>,
    /// Optional context where this binding applies
    pub context: Option<String>,
}

/// Trait for registries that can provide Action instances from names
pub trait ActionRegistry {
    /// Get an action by name, optionally with parameters
    fn get_action(&self, name: &str, params: Option<serde_json::Value>) -> Option<Box<dyn Action>>;
}

/// A simple action registry implementation using a HashMap
pub struct SimpleActionRegistry {
    actions: HashMap<String, Box<dyn Fn(Option<serde_json::Value>) -> Box<dyn Action>>>,
}

impl SimpleActionRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            actions: HashMap::new(),
        }
    }

    /// Register an action factory
    pub fn register<F>(&mut self, name: impl Into<String>, factory: F)
    where
        F: Fn(Option<serde_json::Value>) -> Box<dyn Action> + 'static,
    {
        self.actions.insert(name.into(), Box::new(factory));
    }

    /// Register a simple action (no parameters)
    pub fn register_simple<A: Action>(&mut self, name: impl Into<String>, action: A)
    where
        A: Clone + 'static,
    {
        let name = name.into();
        self.register(name, move |_params| Box::new(action.clone()));
    }
}

impl ActionRegistry for SimpleActionRegistry {
    fn get_action(&self, name: &str, params: Option<serde_json::Value>) -> Option<Box<dyn Action>> {
        self.actions.get(name).map(|factory| factory(params))
    }
}

/// Returns the default keymap JSON for the current platform
pub fn default_keymap_json() -> &'static str {
    #[cfg(target_os = "macos")]
    {
        include_str!("../default-keymap.json")
    }

    #[cfg(target_os = "windows")]
    {
        include_str!("../default-keymap-windows.json")
    }

    #[cfg(target_os = "linux")]
    {
        include_str!("../default-keymap-linux.json")
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
    {
        include_str!("../default-keymap.json")
    }
}

/// Helper function to create a KeyBinding from a simple action
pub fn binding(key: impl Into<String>, action: impl Into<String>) -> KeyBindingEntry {
    KeyBindingEntry {
        key: key.into(),
        action: ActionValue::Simple(action.into()),
        context: None,
    }
}

/// Helper function to create a KeyBinding with context
pub fn binding_with_context(
    key: impl Into<String>,
    action: impl Into<String>,
    context: impl Into<String>,
) -> KeyBindingEntry {
    KeyBindingEntry {
        key: key.into(),
        action: ActionValue::Simple(action.into()),
        context: Some(context.into()),
    }
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

        if let KeyBindings::Simple(map) = keymap.bindings {
            assert_eq!(map.len(), 2);
            assert!(matches!(map.get("cmd-s"), Some(ActionValue::Simple(s)) if s == "Save"));
            assert!(matches!(map.get("cmd-z"), Some(ActionValue::Simple(s)) if s == "Undo"));
        } else {
            panic!("Expected simple bindings");
        }
    }

    #[test]
    fn test_parse_complex_keymap() {
        let json = r#"[
            {
                "bindings": [
                    { "key": "cmd-s", "action": "Save" },
                    { "key": "cmd-z", "action": ["Undo", { "count": 1 }] }
                ]
            }
        ]"#;

        let keymaps: Vec<Keymap> = serde_json::from_str(json).unwrap();
        assert_eq!(keymaps.len(), 1);

        let keymap = &keymaps[0];
        if let KeyBindings::Complex(entries) = &keymap.bindings {
            assert_eq!(entries.len(), 2);

            assert_eq!(entries[0].key, "cmd-s");
            assert!(matches!(&entries[0].action, ActionValue::Simple(s) if s == "Save"));

            assert_eq!(entries[1].key, "cmd-z");
            assert!(matches!(&entries[1].action, ActionValue::WithParams(_)));
        } else {
            panic!("Expected complex bindings");
        }
    }

    #[test]
    fn test_action_value_methods() {
        let simple = ActionValue::Simple("Save".to_string());
        assert_eq!(simple.action_name().unwrap(), "Save");
        assert_eq!(simple.params(), None);

        let with_params = ActionValue::WithParams(vec![
            serde_json::json!("Undo"),
            serde_json::json!({ "count": 1 }),
        ]);
        assert_eq!(with_params.action_name().unwrap(), "Undo");
        assert_eq!(
            with_params.params(),
            Some(serde_json::json!({ "count": 1 }))
        );
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
    }

    #[test]
    fn test_serialize_keymap() {
        let mut bindings = HashMap::new();
        bindings.insert("cmd-s".to_string(), ActionValue::Simple("Save".to_string()));
        bindings.insert("cmd-z".to_string(), ActionValue::Simple("Undo".to_string()));

        let keymap = Keymap {
            context: Some("Editor".to_string()),
            use_key_equivalents: false,
            bindings: KeyBindings::Simple(bindings),
        };

        let json = serde_json::to_string_pretty(&keymap).unwrap();
        let parsed: Keymap = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.context, keymap.context);
        assert_eq!(parsed.use_key_equivalents, keymap.use_key_equivalents);
    }
}
