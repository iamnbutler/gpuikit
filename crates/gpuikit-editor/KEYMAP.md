# Keymap Module Documentation

The `keymap` module provides JSON-based keyboard shortcut configuration for GPUI applications, allowing keybindings to be loaded from external files rather than hardcoded in the application.

## Features

- **JSON Configuration**: Define keybindings in human-readable JSON files
- **Multiple Contexts**: Support for context-specific keybindings (e.g., Editor, Menu, Global)
- **Action Parameters**: Pass parameters to actions through keybindings
- **Platform-specific**: Automatically loads platform-appropriate default keymaps
- **Flexible Loading**: Load from files, strings, or create programmatically
- **Type-safe**: Compile-time checking when converting to GPUI bindings

## Quick Start

### Basic Usage

```rust
use gpui_editor::keymap::KeymapCollection;

// Create a keymap collection
let mut keymaps = KeymapCollection::new();

// Load default keymaps for the current platform
keymaps.load_defaults()?;

// Or load from a custom file
keymaps.load_file("my-keymaps.json")?;

// Get binding specifications
let specs = keymaps.get_binding_specs()?;

// Convert to GPUI bindings (requires mapping action names to concrete types)
for spec in specs {
    match spec.action_name.as_str() {
        "editor::Save" => {
            cx.bind_keys([KeyBinding::new(&spec.keystrokes, SaveAction, spec.context.as_deref())]);
        }
        // ... handle other actions
    }
}
```

## JSON Format

### Simple Bindings

```json
{
  "context": "Editor",
  "use_key_equivalents": true,
  "bindings": {
    "cmd-s": "editor::Save",
    "cmd-z": "editor::Undo",
    "cmd-shift-z": "editor::Redo"
  }
}
```

### Complex Bindings with Parameters

```json
{
  "bindings": [
    {
      "key": "cmd-z",
      "action": ["editor::Undo", { "count": 1 }]
    },
    {
      "key": "cmd-k cmd-c",
      "action": "editor::CommentLine",
      "context": "CodeEditor"
    }
  ]
}
```

### Multiple Contexts

```json
[
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
  }
]
```

## Keymap Structure

### Fields

- **`context`** (optional): String specifying where these bindings apply
- **`use_key_equivalents`**: Boolean for platform-specific key mappings
- **`bindings`**: Either a map of keystrokes to actions (simple) or an array of binding entries (complex)

### Keystroke Format

Keystrokes follow the pattern: `[modifiers-]key`

**Modifiers:**
- `cmd` / `ctrl`: Command (macOS) or Control (Windows/Linux)
- `alt` / `option`: Alt/Option key
- `shift`: Shift key
- `fn`: Function key

**Examples:**
- `cmd-s`: Command+S (macOS) or Ctrl+S (Windows/Linux)
- `cmd-shift-z`: Command+Shift+Z
- `alt-left`: Alt+Left Arrow
- `cmd-k cmd-c`: Command+K followed by Command+C (chord)

## Action Registry

To convert keymap specifications to actual GPUI actions, you need an action registry:

```rust
use gpui_editor::keymap::{SimpleActionRegistry, ActionRegistry};

let mut registry = SimpleActionRegistry::new();

// Register simple actions
registry.register_simple("editor::Save", SaveAction);
registry.register_simple("editor::Open", OpenAction);

// Register actions with parameter handling
registry.register("editor::Undo", |params| {
    if let Some(params) = params {
        // Handle parameters
        let count = params["count"].as_u64().unwrap_or(1);
        Box::new(UndoAction::with_count(count))
    } else {
        Box::new(UndoAction::default())
    }
});

// Use the registry to convert keymaps to actions
let actions = keymaps.to_actions(&registry)?;
```

## Integration with GPUI

Since GPUI's `KeyBinding::new` requires concrete action types, you need to map action names from the keymap to actual action types:

```rust
// Define your actions
actions!(editor, [Save, Open, Undo, Redo]);

// Load keymaps
let mut keymaps = KeymapCollection::new();
keymaps.load_defaults()?;

// Get binding specifications
let specs = keymaps.get_binding_specs()?;

// Create GPUI bindings
let mut bindings = Vec::new();
for spec in specs {
    let context = spec.context.as_deref();
    
    match spec.action_name.as_str() {
        "editor::Save" => {
            bindings.push(KeyBinding::new(&spec.keystrokes, Save, context));
        }
        "editor::Open" => {
            bindings.push(KeyBinding::new(&spec.keystrokes, Open, context));
        }
        "editor::Undo" => {
            bindings.push(KeyBinding::new(&spec.keystrokes, Undo, context));
        }
        "editor::Redo" => {
            bindings.push(KeyBinding::new(&spec.keystrokes, Redo, context));
        }
        _ => {
            log::warn!("Unknown action: {}", spec.action_name);
        }
    }
}

// Register with GPUI
cx.bind_keys(bindings);
```

## Default Keymaps

The module includes built-in default keymaps for different platforms:

- **macOS**: Uses `cmd` modifier
- **Windows/Linux**: Uses `ctrl` modifier

Load defaults with:

```rust
keymaps.load_defaults()?;
```

## Programmatic Creation

You can also create keymaps programmatically:

```rust
use gpui_editor::keymap::{Keymap, KeyBindings, ActionValue};
use std::collections::HashMap;

let mut bindings = HashMap::new();
bindings.insert(
    "cmd-s".to_string(),
    ActionValue::Simple("editor::Save".to_string())
);
bindings.insert(
    "cmd-z".to_string(),
    ActionValue::WithParams(vec![
        json!("editor::Undo"),
        json!({ "count": 1 })
    ])
);

let keymap = Keymap {
    context: Some("Editor".to_string()),
    use_key_equivalents: true,
    bindings: KeyBindings::Simple(bindings),
};
```

## Helper Functions

The module provides several helper functions:

```rust
use gpui_editor::keymap::{binding, binding_with_context};

// Create a simple binding
let b = binding("cmd-s", "editor::Save");

// Create a binding with context
let b = binding_with_context("enter", "menu::Select", "Menu");
```

## Error Handling

All loading operations return `Result<T, anyhow::Error>`:

```rust
match keymaps.load_file("custom-keymap.json") {
    Ok(_) => println!("Loaded successfully"),
    Err(e) => {
        eprintln!("Failed to load keymap: {}", e);
        // Fall back to defaults
        keymaps.load_defaults()?;
    }
}
```

## Examples

### Loading from Multiple Sources

```rust
let mut keymaps = KeymapCollection::new();

// Load base keymaps
keymaps.load_defaults()?;

// Load user customizations (if they exist)
if Path::new("~/.config/myapp/keymap.json").exists() {
    keymaps.load_file("~/.config/myapp/keymap.json")?;
}

// Load project-specific keymaps
if Path::new(".myapp/keymap.json").exists() {
    keymaps.load_file(".myapp/keymap.json")?;
}
```

### Custom Action Registry

```rust
struct MyActionRegistry {
    actions: HashMap<String, Box<dyn Fn() -> Box<dyn Action>>>,
}

impl ActionRegistry for MyActionRegistry {
    fn get_action(&self, name: &str, params: Option<Value>) -> Option<Box<dyn Action>> {
        self.actions.get(name).map(|factory| factory())
    }
}
```

## API Reference

### `KeymapCollection`

- `new()` - Create a new empty collection
- `load_file(path)` - Load keymaps from a JSON file
- `load_json(json)` - Load keymaps from a JSON string
- `load_defaults()` - Load platform-specific defaults
- `get_binding_specs()` - Get binding specifications
- `to_actions(registry)` - Convert to boxed actions using a registry
- `keymaps()` - Get all keymaps in the collection
- `clear()` - Remove all keymaps

### `Keymap`

- `context: Option<String>` - Optional context for bindings
- `use_key_equivalents: bool` - Use platform-specific keys
- `bindings: KeyBindings` - The actual key bindings

### `KeyBindings`

- `Simple(HashMap<String, ActionValue>)` - Simple keystroke->action map
- `Complex(Vec<KeyBindingEntry>)` - List of binding entries

### `ActionValue`

- `Simple(String)` - Simple action name
- `WithParams(Vec<Value>)` - Action with parameters

### `BindingSpec`

- `keystrokes: String` - The key combination
- `action_name: String` - Name of the action
- `action_params: Option<Value>` - Optional parameters
- `context: Option<String>` - Optional context

## Best Practices

1. **Use Contexts**: Separate keybindings by context to avoid conflicts
2. **Platform Compatibility**: Test keymaps on all target platforms
3. **Document Actions**: Keep a list of all available actions and their parameters
4. **Provide Defaults**: Always have fallback keybindings if loading fails
5. **User Customization**: Allow users to override default keymaps
6. **Validate Early**: Check keymap validity during loading, not at runtime

## Migration from Hardcoded Bindings

Before (hardcoded):
```rust
cx.bind_keys([
    KeyBinding::new("cmd-s", Save, None),
    KeyBinding::new("cmd-o", Open, None),
    // ... many more
]);
```

After (with keymap module):
```rust
let mut keymaps = KeymapCollection::new();
keymaps.load_defaults()?;

for spec in keymaps.get_binding_specs()? {
    // Map spec to concrete action and register
}
```

## Troubleshooting

### Common Issues

1. **"Unknown action" warnings**: Ensure all actions in the keymap are registered
2. **Platform differences**: Use `cmd` for macOS and `ctrl` for Windows/Linux
3. **Context not working**: Verify the context name matches exactly
4. **Parameters not passing**: Check JSON structure for action parameters

### Debug Output

Enable logging to see keymap loading details:
```rust
env::set_var("RUST_LOG", "debug");
env_logger::init();
```
