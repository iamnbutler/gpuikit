//! Extension traits and helpers for GPUI keymap integration

use gpui::{Action, KeyBinding};

/// Helper to create a list of key bindings from action-keystroke pairs
pub fn create_bindings<A: Action + Clone>(
    mappings: &[(&str, A)],
    context: Option<&str>,
) -> Vec<KeyBinding> {
    mappings
        .iter()
        .map(|(keystrokes, action)| KeyBinding::new(keystrokes, action.clone(), context))
        .collect()
}

/// Helper to create a single binding
pub fn bind<A: Action>(keystrokes: &str, action: A, context: Option<&str>) -> KeyBinding {
    KeyBinding::new(keystrokes, action, context)
}

/// Helper to create multiple bindings with the same context
pub struct BindingBuilder {
    context: Option<String>,
    bindings: Vec<KeyBinding>,
}

impl BindingBuilder {
    /// Create a new binding builder
    pub fn new() -> Self {
        Self {
            context: None,
            bindings: Vec::new(),
        }
    }

    /// Create a new binding builder with a context
    pub fn with_context(context: impl Into<String>) -> Self {
        Self {
            context: Some(context.into()),
            bindings: Vec::new(),
        }
    }

    /// Add a binding to the builder
    pub fn bind<A: Action>(mut self, keystrokes: &str, action: A) -> Self {
        let binding = KeyBinding::new(keystrokes, action, self.context.as_deref());
        self.bindings.push(binding);
        self
    }

    /// Build the list of bindings
    pub fn build(self) -> Vec<KeyBinding> {
        self.bindings
    }
}

impl Default for BindingBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Macro to simplify creating multiple bindings
#[macro_export]
macro_rules! bindings {
    // With context
    (context: $context:expr, $($key:expr => $action:expr),* $(,)?) => {
        {
            vec![
                $(
                    $crate::keymap_ext::bind($key, $action, Some($context))
                ),*
            ]
        }
    };

    // Without context
    ($($key:expr => $action:expr),* $(,)?) => {
        {
            vec![
                $(
                    $crate::keymap_ext::bind($key, $action, None)
                ),*
            ]
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::actions;

    actions!(test, [TestAction, AnotherAction]);

    #[test]
    fn test_binding_builder() {
        let bindings = BindingBuilder::new()
            .bind("cmd-s", TestAction)
            .bind("cmd-z", AnotherAction)
            .build();

        assert_eq!(bindings.len(), 2);
    }

    #[test]
    fn test_binding_builder_with_context() {
        let builder = BindingBuilder::with_context("Editor");
        assert_eq!(builder.context, Some("Editor".to_string()));
    }

    #[test]
    fn test_create_bindings() {
        let mappings = [
            ("cmd-s", TestAction),
            ("cmd-z", TestAction),
            ("cmd-y", TestAction),
        ];

        let bindings = create_bindings(&mappings, Some("Editor"));
        assert_eq!(bindings.len(), 3);
    }

    #[test]
    fn test_bind() {
        let _binding = bind("cmd-s", TestAction, Some("Editor"));
        // Just checking it compiles
    }
}
