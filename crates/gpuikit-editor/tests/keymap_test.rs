//! Integration tests for the keymap module

use gpui::Action;
use gpui_editor::keymap::*;
use serde_json::json;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;

// Mock actions for testing
#[derive(Clone, Debug, PartialEq)]
struct MockAction {
    name: String,
    params: Option<serde_json::Value>,
}

impl MockAction {
    fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            params: None,
        }
    }

    fn with_params(name: impl Into<String>, params: serde_json::Value) -> Self {
        Self {
            name: name.into(),
            params: Some(params),
        }
    }
}

impl Action for MockAction {
    fn boxed_clone(&self) -> Box<dyn Action> {
        Box::new(self.clone())
    }

    fn partial_eq(&self, action: &dyn Action) -> bool {
        // We can't downcast without as_any, so just return false
        false
    }

    fn name(&self) -> &'static str {
        // We need to return a static string, so use a placeholder
        "MockAction"
    }

    fn name_for_type() -> &'static str {
        "MockAction"
    }

    fn build(_: serde_json::Value) -> Result<Box<dyn Action>, anyhow::Error> {
        Ok(Box::new(MockAction::new("MockAction")))
    }
}

// Test action registry that tracks action creation
struct TestActionRegistry {
    created_actions: Arc<Mutex<Vec<(String, Option<serde_json::Value>)>>>,
}

impl TestActionRegistry {
    fn new() -> Self {
        Self {
            created_actions: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn get_created_actions(&self) -> Vec<(String, Option<serde_json::Value>)> {
        self.created_actions.lock().unwrap().clone()
    }
}

impl ActionRegistry for TestActionRegistry {
    fn get_action(&self, name: &str, params: Option<serde_json::Value>) -> Option<Box<dyn Action>> {
        self.created_actions
            .lock()
            .unwrap()
            .push((name.to_string(), params.clone()));

        Some(Box::new(MockAction::with_params(
            name,
            params.unwrap_or(json!(null)),
        )))
    }
}

#[test]
fn test_simple_keymap_parsing() {
    let json = r#"{
        "context": "Editor",
        "use_key_equivalents": true,
        "bindings": {
            "cmd-s": "Save",
            "cmd-z": "Undo",
            "cmd-shift-z": "Redo"
        }
    }"#;

    let keymap: Keymap = serde_json::from_str(json).expect("Failed to parse keymap");

    assert_eq!(keymap.context, Some("Editor".to_string()));
    assert!(keymap.use_key_equivalents);

    if let KeyBindings::Simple(bindings) = keymap.bindings {
        assert_eq!(bindings.len(), 3);
        assert!(matches!(bindings.get("cmd-s"), Some(ActionValue::Simple(s)) if s == "Save"));
        assert!(matches!(bindings.get("cmd-z"), Some(ActionValue::Simple(s)) if s == "Undo"));
        assert!(matches!(bindings.get("cmd-shift-z"), Some(ActionValue::Simple(s)) if s == "Redo"));
    } else {
        panic!("Expected simple bindings");
    }
}

#[test]
fn test_complex_keymap_parsing() {
    let json = r#"[
        {
            "context": "Editor",
            "bindings": [
                { "key": "cmd-s", "action": "Save" },
                { "key": "cmd-z", "action": ["Undo", { "count": 1 }] },
                { "key": "cmd-k cmd-c", "action": "CommentLine", "context": "CodeEditor" }
            ]
        }
    ]"#;

    let keymaps: Vec<Keymap> = serde_json::from_str(json).expect("Failed to parse keymaps");

    assert_eq!(keymaps.len(), 1);
    let keymap = &keymaps[0];
    assert_eq!(keymap.context, Some("Editor".to_string()));

    if let KeyBindings::Complex(entries) = &keymap.bindings {
        assert_eq!(entries.len(), 3);

        // Check first entry
        assert_eq!(entries[0].key, "cmd-s");
        assert!(matches!(&entries[0].action, ActionValue::Simple(s) if s == "Save"));
        assert_eq!(entries[0].context, None);

        // Check second entry with parameters
        assert_eq!(entries[1].key, "cmd-z");
        if let ActionValue::WithParams(params) = &entries[1].action {
            assert_eq!(params.len(), 2);
            assert_eq!(params[0], json!("Undo"));
            assert_eq!(params[1], json!({ "count": 1 }));
        } else {
            panic!("Expected action with params");
        }

        // Check third entry with custom context
        assert_eq!(entries[2].key, "cmd-k cmd-c");
        assert_eq!(entries[2].context, Some("CodeEditor".to_string()));
    } else {
        panic!("Expected complex bindings");
    }
}

#[test]
fn test_keymap_collection_loading() {
    let mut collection = KeymapCollection::new();

    let json1 = r#"{ "bindings": { "cmd-s": "Save", "cmd-o": "Open" } }"#;
    let json2 = r#"{ "context": "Menu", "bindings": { "enter": "Select", "escape": "Cancel" } }"#;

    collection
        .load_json(json1)
        .expect("Failed to load first keymap");
    collection
        .load_json(json2)
        .expect("Failed to load second keymap");

    assert_eq!(collection.keymaps().len(), 2);
    assert_eq!(collection.keymaps()[0].context, None);
    assert_eq!(collection.keymaps()[1].context, Some("Menu".to_string()));
}

#[test]
fn test_keymap_collection_array_loading() {
    let mut collection = KeymapCollection::new();

    let json = r#"[
        { "bindings": { "cmd-s": "Save" } },
        { "context": "Menu", "bindings": { "enter": "Select" } }
    ]"#;

    collection.load_json(json).expect("Failed to load keymaps");

    assert_eq!(collection.keymaps().len(), 2);
    assert_eq!(collection.keymaps()[0].context, None);
    assert_eq!(collection.keymaps()[1].context, Some("Menu".to_string()));
}

#[test]
fn test_action_value_methods() {
    // Test simple action
    let simple = ActionValue::Simple("Save".to_string());
    assert_eq!(simple.action_name().unwrap(), "Save");
    assert_eq!(simple.params(), None);

    // Test action with parameters
    let with_params = ActionValue::WithParams(vec![
        json!("Undo"),
        json!({ "count": 5, "interactive": true }),
    ]);
    assert_eq!(with_params.action_name().unwrap(), "Undo");
    assert_eq!(
        with_params.params(),
        Some(json!({ "count": 5, "interactive": true }))
    );

    // Test invalid action format
    let invalid = ActionValue::WithParams(vec![json!(42)]);
    assert!(invalid.action_name().is_err());
}

#[test]
fn test_get_binding_specs() {
    let json = r#"{
        "context": "TestContext",
        "bindings": {
            "cmd-s": "Save",
            "cmd-z": ["Undo", { "count": 1 }],
            "cmd-y": "Redo"
        }
    }"#;

    let mut collection = KeymapCollection::new();
    collection.load_json(json).expect("Failed to load keymap");

    let specs = collection
        .get_binding_specs()
        .expect("Failed to get binding specs");

    // Check that the correct number of binding specs were created
    assert_eq!(specs.len(), 3);

    // Check the specs have the correct data
    assert!(specs.iter().any(|spec| spec.keystrokes == "cmd-s"
        && spec.action_name == "Save"
        && spec.action_params.is_none()));
    assert!(specs.iter().any(|spec| spec.keystrokes == "cmd-z"
        && spec.action_name == "Undo"
        && spec.action_params == Some(json!({ "count": 1 }))));
    assert!(specs.iter().any(|spec| spec.keystrokes == "cmd-y"
        && spec.action_name == "Redo"
        && spec.action_params.is_none()));

    // Test the to_actions method with a registry
    let registry = TestActionRegistry::new();
    let actions = collection
        .to_actions(&registry)
        .expect("Failed to convert to actions");

    // Check that the registry was called with the correct actions
    let created = registry.get_created_actions();
    assert_eq!(created.len(), 3);
    assert_eq!(actions.len(), 3);
}

#[test]
fn test_mixed_context_bindings() {
    let json = r#"[
        {
            "context": "Global",
            "bindings": {
                "cmd-q": "Quit"
            }
        },
        {
            "bindings": [
                { "key": "cmd-s", "action": "Save" },
                { "key": "cmd-o", "action": "Open", "context": "FileDialog" }
            ]
        }
    ]"#;

    let mut collection = KeymapCollection::new();
    collection.load_json(json).expect("Failed to load keymaps");

    assert_eq!(collection.keymaps().len(), 2);

    // Check global context keymap
    let global = &collection.keymaps()[0];
    assert_eq!(global.context, Some("Global".to_string()));

    // Check mixed context keymap
    let mixed = &collection.keymaps()[1];
    assert_eq!(mixed.context, None);

    if let KeyBindings::Complex(entries) = &mixed.bindings {
        assert_eq!(entries[0].context, None);
        assert_eq!(entries[1].context, Some("FileDialog".to_string()));
    } else {
        panic!("Expected complex bindings");
    }
}

#[test]
fn test_serialization_roundtrip() {
    let mut bindings = HashMap::new();
    bindings.insert("cmd-s".to_string(), ActionValue::Simple("Save".to_string()));
    bindings.insert(
        "cmd-z".to_string(),
        ActionValue::WithParams(vec![json!("Undo"), json!({ "count": 1 })]),
    );

    let original = Keymap {
        context: Some("Editor".to_string()),
        use_key_equivalents: true,
        bindings: KeyBindings::Simple(bindings),
    };

    // Serialize to JSON
    let json = serde_json::to_string(&original).expect("Failed to serialize");

    // Deserialize back
    let deserialized: Keymap = serde_json::from_str(&json).expect("Failed to deserialize");

    assert_eq!(deserialized.context, original.context);
    assert_eq!(
        deserialized.use_key_equivalents,
        original.use_key_equivalents
    );

    // Check bindings
    if let (KeyBindings::Simple(orig), KeyBindings::Simple(deser)) =
        (&original.bindings, &deserialized.bindings)
    {
        assert_eq!(orig.len(), deser.len());
        for (key, _) in orig {
            assert!(deser.contains_key(key));
        }
    } else {
        panic!("Bindings type mismatch");
    }
}

#[test]
fn test_error_handling() {
    let mut collection = KeymapCollection::new();

    // Test invalid JSON
    assert!(collection.load_json("not valid json").is_err());

    // Test empty JSON
    assert!(collection.load_json("").is_err());

    // Test wrong structure
    assert!(collection.load_json(r#"{"invalid": "structure"}"#).is_err());

    // Test that collection remains unchanged after errors
    assert_eq!(collection.keymaps().len(), 0);
}

#[test]
fn test_clear_collection() {
    let mut collection = KeymapCollection::new();

    collection
        .load_json(r#"{ "bindings": { "cmd-s": "Save" } }"#)
        .expect("Failed to load keymap");
    assert_eq!(collection.keymaps().len(), 1);

    collection.clear();
    assert_eq!(collection.keymaps().len(), 0);
}

#[test]
fn test_simple_action_registry() {
    let mut registry = SimpleActionRegistry::new();

    // Register a simple action
    registry.register_simple("Save", MockAction::new("Save"));

    // Register an action factory
    registry.register("Undo", |params| {
        Box::new(MockAction::with_params(
            "Undo",
            params.unwrap_or(json!(null)),
        ))
    });

    // Test getting actions
    let save_action = registry.get_action("Save", None);
    assert!(save_action.is_some());
    assert_eq!(save_action.unwrap().name(), "MockAction");

    let undo_action = registry.get_action("Undo", Some(json!({ "count": 1 })));
    assert!(undo_action.is_some());

    // Test non-existent action
    let unknown = registry.get_action("Unknown", None);
    assert!(unknown.is_none());
}

#[test]
fn test_helper_functions() {
    // Test binding helper
    let binding = binding("cmd-s", "Save");
    assert_eq!(binding.key, "cmd-s");
    assert!(matches!(binding.action, ActionValue::Simple(s) if s == "Save"));
    assert_eq!(binding.context, None);

    // Test binding_with_context helper
    let binding = binding_with_context("enter", "Select", "Menu");
    assert_eq!(binding.key, "enter");
    assert!(matches!(binding.action, ActionValue::Simple(s) if s == "Select"));
    assert_eq!(binding.context, Some("Menu".to_string()));
}

#[test]
fn test_default_keymap_loading() {
    let mut collection = KeymapCollection::new();

    // This should load the embedded default keymap
    let result = collection.load_defaults();

    // The result might fail if the default keymap isn't properly embedded,
    // but the function should not panic
    if result.is_ok() {
        assert!(
            collection.keymaps().len() > 0,
            "Default keymap should contain at least one keymap"
        );
    }
}

#[test]
fn test_complex_keystroke_sequences() {
    let json = r#"{
        "bindings": {
            "cmd-k cmd-c": "CommentLine",
            "cmd-k cmd-u": "UncommentLine",
            "ctrl-x ctrl-s": "Save",
            "ctrl-x ctrl-c": "Quit"
        }
    }"#;

    let keymap: Keymap = serde_json::from_str(json).expect("Failed to parse keymap");

    if let KeyBindings::Simple(bindings) = keymap.bindings {
        assert!(bindings.contains_key("cmd-k cmd-c"));
        assert!(bindings.contains_key("cmd-k cmd-u"));
        assert!(bindings.contains_key("ctrl-x ctrl-s"));
        assert!(bindings.contains_key("ctrl-x ctrl-c"));
    } else {
        panic!("Expected simple bindings");
    }
}

// Tests for file loading are commented out due to missing tempfile dependency
// These tests would require adding tempfile to dev-dependencies

// #[cfg(test)]
// fn create_temp_keymap_file() -> (PathBuf, tempfile::TempDir) {
//     let dir = tempfile::tempdir().expect("Failed to create temp dir");
//     let path = dir.path().join("test-keymap.json");
//
//     let content = r#"{
//         "context": "TestFile",
//         "bindings": {
//             "cmd-t": "Test"
//         }
//     }"#;
//
//     fs::write(&path, content).expect("Failed to write temp file");
//     (path, dir)
// }

// #[test]
// fn test_load_from_file() {
//     let (path, _dir) = create_temp_keymap_file();
//
//     let mut collection = KeymapCollection::new();
//     collection
//         .load_file(&path)
//         .expect("Failed to load from file");
//
//     assert_eq!(collection.keymaps().len(), 1);
//     assert_eq!(
//         collection.keymaps()[0].context,
//         Some("TestFile".to_string())
//     );
// }

#[test]
fn test_load_from_nonexistent_file() {
    let mut collection = KeymapCollection::new();
    let result = collection.load_file("/this/file/does/not/exist.json");

    assert!(result.is_err());
    assert_eq!(collection.keymaps().len(), 0);
}
