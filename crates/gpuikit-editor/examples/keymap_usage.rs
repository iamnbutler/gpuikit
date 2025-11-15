//! Example demonstrating how to use the keymap module
//!
//! This example shows:
//! - Loading keymaps from JSON
//! - Creating an action registry
//! - Programmatically creating keymaps
//! - Converting keymaps to GPUI bindings

use anyhow::Result;
use gpui::{actions, Action};
use gpui_editor::keymap::{
    ActionRegistry, ActionValue, KeyBindingEntry, KeyBindings, Keymap, KeymapCollection,
    SimpleActionRegistry,
};
use serde_json::json;
use std::collections::HashMap;

// Define some example actions
actions!(
    example,
    [Save, Open, Close, Undo, Redo, Cut, Copy, Paste, Find, Replace, SelectAll, ZoomIn, ZoomOut]
);

fn main() -> Result<()> {
    // Example 1: Load keymaps from JSON string
    println!("=== Example 1: Loading from JSON ===");
    load_from_json()?;

    // Example 2: Programmatically create keymaps
    println!("\n=== Example 2: Programmatic Creation ===");
    create_programmatic_keymap()?;

    // Example 3: Use action registry
    println!("\n=== Example 3: Action Registry ===");
    use_action_registry()?;

    // Example 4: Complex keymap with multiple contexts
    println!("\n=== Example 4: Multiple Contexts ===");
    multiple_contexts_example()?;

    // Example 5: Serialize keymaps
    println!("\n=== Example 5: Serialization ===");
    serialize_keymap_example()?;

    Ok(())
}

fn load_from_json() -> Result<()> {
    let json = r#"[
        {
            "context": "Editor",
            "use_key_equivalents": true,
            "bindings": {
                "cmd-s": "example::Save",
                "cmd-o": "example::Open",
                "cmd-z": "example::Undo",
                "cmd-shift-z": "example::Redo"
            }
        }
    ]"#;

    let mut collection = KeymapCollection::new();
    collection.load_json(json)?;

    println!("Loaded {} keymap(s)", collection.keymaps().len());

    // Get binding specifications
    let specs = collection.get_binding_specs()?;
    println!("Found {} key binding(s):", specs.len());

    for spec in specs {
        println!(
            "  {} -> {} (context: {:?})",
            spec.keystrokes, spec.action_name, spec.context
        );
    }

    Ok(())
}

fn create_programmatic_keymap() -> Result<()> {
    // Create a keymap using the builder pattern
    let mut bindings = HashMap::new();
    bindings.insert(
        "cmd-s".to_string(),
        ActionValue::Simple("example::Save".to_string()),
    );
    bindings.insert(
        "cmd-z".to_string(),
        ActionValue::WithParams(vec![
            json!("example::Undo"),
            json!({ "count": 1, "save_point": true }),
        ]),
    );

    let keymap = gpui_editor::keymap::Keymap {
        context: Some("Editor".to_string()),
        use_key_equivalents: true,
        bindings: KeyBindings::Simple(bindings),
    };

    // Add to collection
    let mut collection = KeymapCollection::new();
    collection.load_json(&serde_json::to_string(&keymap)?)?;

    let specs = collection.get_binding_specs()?;
    println!("Created {} binding(s) programmatically:", specs.len());

    for spec in specs {
        println!("  {} -> {}", spec.keystrokes, spec.action_name);
        if let Some(params) = spec.action_params {
            println!(
                "    with params: {}",
                serde_json::to_string_pretty(&params)?
            );
        }
    }

    Ok(())
}

fn use_action_registry() -> Result<()> {
    // Create an action registry
    let mut registry = SimpleActionRegistry::new();

    // Register simple actions
    registry.register_simple("example::Save", Save);
    registry.register_simple("example::Open", Open);

    // Register action with parameter handling
    registry.register("example::Undo", |params| {
        if let Some(params) = params {
            println!("Undo action created with params: {}", params);
        }
        Box::new(Undo)
    });

    // Load a keymap
    let json = r#"{
        "bindings": {
            "cmd-s": "example::Save",
            "cmd-o": "example::Open",
            "cmd-z": ["example::Undo", { "count": 5 }]
        }
    }"#;

    let mut collection = KeymapCollection::new();
    collection.load_json(json)?;

    // Convert to actions using the registry
    let actions = collection.to_actions(&registry)?;
    println!("Created {} action(s) from registry:", actions.len());

    for (keystrokes, _action, context) in actions {
        println!("  {} (context: {:?})", keystrokes, context);
    }

    Ok(())
}

fn multiple_contexts_example() -> Result<()> {
    let json = r#"[
        {
            "context": "Global",
            "bindings": {
                "cmd-q": "application::Quit",
                "cmd-n": "application::NewWindow"
            }
        },
        {
            "context": "Editor",
            "bindings": {
                "cmd-s": "editor::Save",
                "cmd-z": "editor::Undo"
            }
        },
        {
            "context": "Menu",
            "bindings": {
                "up": "menu::SelectPrevious",
                "down": "menu::SelectNext",
                "enter": "menu::Confirm",
                "escape": "menu::Cancel"
            }
        },
        {
            "bindings": [
                {
                    "key": "cmd-f",
                    "action": "editor::Find",
                    "context": "Editor"
                },
                {
                    "key": "cmd-f",
                    "action": "browser::Find",
                    "context": "Browser"
                }
            ]
        }
    ]"#;

    let mut collection = KeymapCollection::new();
    collection.load_json(json)?;

    println!("Loaded {} keymap context(s)", collection.keymaps().len());

    for (i, keymap) in collection.keymaps().iter().enumerate() {
        println!("\nKeymap {}:", i + 1);
        println!("  Context: {:?}", keymap.context);

        match &keymap.bindings {
            KeyBindings::Simple(map) => {
                println!("  Simple bindings: {} entries", map.len());
            }
            KeyBindings::Complex(entries) => {
                println!("  Complex bindings: {} entries", entries.len());
                for entry in entries {
                    println!(
                        "    {} -> {} (override context: {:?})",
                        entry.key,
                        entry
                            .action
                            .action_name()
                            .unwrap_or_else(|_| "unknown".to_string()),
                        entry.context
                    );
                }
            }
        }
    }

    Ok(())
}

fn serialize_keymap_example() -> Result<()> {
    // Create a complex keymap structure
    let entries = vec![
        KeyBindingEntry {
            key: "cmd-s".to_string(),
            action: ActionValue::Simple("Save".to_string()),
            context: None,
        },
        KeyBindingEntry {
            key: "cmd-z".to_string(),
            action: ActionValue::WithParams(vec![json!("Undo"), json!({ "count": 1 })]),
            context: Some("TextEditor".to_string()),
        },
        KeyBindingEntry {
            key: "cmd-k cmd-c".to_string(),
            action: ActionValue::Simple("CommentLine".to_string()),
            context: Some("CodeEditor".to_string()),
        },
    ];

    let keymap = gpui_editor::keymap::Keymap {
        context: Some("Editor".to_string()),
        use_key_equivalents: false,
        bindings: KeyBindings::Complex(entries),
    };

    // Serialize to JSON
    let json = serde_json::to_string_pretty(&keymap)?;
    println!("Serialized keymap:");
    println!("{}", json);

    // Parse it back
    let parsed: gpui_editor::keymap::Keymap = serde_json::from_str(&json)?;
    println!("\nSuccessfully round-tripped keymap");
    println!("Context: {:?}", parsed.context);

    if let KeyBindings::Complex(entries) = parsed.bindings {
        println!("Parsed {} binding entries", entries.len());
    }

    Ok(())
}

// Example of creating a custom action registry that logs all actions
struct LoggingActionRegistry {
    inner: SimpleActionRegistry,
}

impl LoggingActionRegistry {
    fn new() -> Self {
        Self {
            inner: SimpleActionRegistry::new(),
        }
    }

    fn register_simple<A: Action>(&mut self, name: impl Into<String>, action: A)
    where
        A: Clone + 'static,
    {
        let name = name.into();
        println!("Registering action: {}", name);
        self.inner.register_simple(name, action);
    }
}

impl ActionRegistry for LoggingActionRegistry {
    fn get_action(&self, name: &str, params: Option<serde_json::Value>) -> Option<Box<dyn Action>> {
        println!("Retrieving action: {} (params: {:?})", name, params);
        self.inner.get_action(name, params)
    }
}
