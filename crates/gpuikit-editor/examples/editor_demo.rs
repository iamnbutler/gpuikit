//! Example demonstrating a complete text editor with syntax highlighting
//!
//! Run with: cargo run --example editor_demo

use gpui::*;
use gpui_editor::*;
use gpuikit_keymap::KeymapCollection;
use std::path::Path;

actions!(
    editor_demo,
    [
        MoveUp,
        MoveDown,
        MoveLeft,
        MoveRight,
        MoveUpWithShift,
        MoveDownWithShift,
        MoveLeftWithShift,
        MoveRightWithShift,
        Backspace,
        Delete,
        InsertNewline,
        NextTheme,
        PreviousTheme,
        NextLanguage,
        PreviousLanguage,
        SelectAll,
        Escape,
        Copy,
        Cut,
        Paste
    ]
);

/// A complete editor view with keyboard handling and state management
struct EditorView {
    focus_handle: FocusHandle,
    editor: Editor,
    current_theme_index: usize,
    available_themes: Vec<String>,
    current_language_index: usize,
    available_languages: Vec<(String, String, String)>, // (name, extension, sample_code)
}

impl EditorView {
    fn new(cx: &mut Context<Self>) -> Self {
        let focus_handle = cx.focus_handle();

        // Initialize with sample Rust code
        let initial_code = vec![
            "// Rust sample code".to_string(),
            "use std::collections::HashMap;".to_string(),
            "".to_string(),
            "fn main() {".to_string(),
            "    let mut count = 0;".to_string(),
            "    ".to_string(),
            "    // Count from 1 to 10".to_string(),
            "    for i in 1..=10 {".to_string(),
            "        count += i;".to_string(),
            "    }".to_string(),
            "    ".to_string(),
            "    // HashMap example".to_string(),
            "    let mut scores = HashMap::new();".to_string(),
            "    scores.insert(\"Blue\", 10);".to_string(),
            "    scores.insert(\"Yellow\", 50);".to_string(),
            "    ".to_string(),
            "    println!(\"Final count: {}\", count);".to_string(),
            "}".to_string(),
        ];

        let mut editor = Editor::new("editor", initial_code);

        // Get available themes from syntax highlighter
        let highlighter = SyntaxHighlighter::new();
        let available_themes = highlighter.available_themes();

        // Find and set a default theme
        let default_theme_index = available_themes
            .iter()
            .position(|t| t == "base16-ocean.dark")
            .unwrap_or(0);

        editor.set_theme(&available_themes[default_theme_index]);

        let available_languages = vec![
            ("Rust".to_string(), "rs".to_string(), get_rust_sample()),
            (
                "Plain Text".to_string(),
                "txt".to_string(),
                get_plain_text_sample(),
            ),
        ];

        editor.set_language("Rust".to_string());

        Self {
            focus_handle,
            editor,
            current_theme_index: default_theme_index,
            available_themes,
            current_language_index: 0,
            available_languages,
        }
    }

    fn get_selected_text(&self) -> String {
        self.editor.get_selected_text()
    }

    // Action handlers
    fn move_up(&mut self, _: &MoveUp, _window: &mut Window, cx: &mut Context<Self>) {
        self.editor.move_up(false);
        cx.notify();
    }

    fn move_down(&mut self, _: &MoveDown, _window: &mut Window, cx: &mut Context<Self>) {
        self.editor.move_down(false);
        cx.notify();
    }

    fn move_left(&mut self, _: &MoveLeft, _window: &mut Window, cx: &mut Context<Self>) {
        self.editor.move_left(false);
        cx.notify();
    }

    fn move_right(&mut self, _: &MoveRight, _window: &mut Window, cx: &mut Context<Self>) {
        self.editor.move_right(false);
        cx.notify();
    }

    fn move_up_with_shift(
        &mut self,
        _: &MoveUpWithShift,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.editor.move_up(true);
        cx.notify();
    }

    fn move_down_with_shift(
        &mut self,
        _: &MoveDownWithShift,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.editor.move_down(true);
        cx.notify();
    }

    fn move_left_with_shift(
        &mut self,
        _: &MoveLeftWithShift,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.editor.move_left(true);
        cx.notify();
    }

    fn move_right_with_shift(
        &mut self,
        _: &MoveRightWithShift,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.editor.move_right(true);
        cx.notify();
    }

    fn backspace(&mut self, _: &Backspace, _window: &mut Window, cx: &mut Context<Self>) {
        self.editor.backspace();
        cx.notify();
    }

    fn delete(&mut self, _: &Delete, _window: &mut Window, cx: &mut Context<Self>) {
        self.editor.delete();
        cx.notify();
    }

    fn insert_newline(&mut self, _: &InsertNewline, _window: &mut Window, cx: &mut Context<Self>) {
        self.editor.insert_newline();
        cx.notify();
    }

    fn select_all(&mut self, _: &SelectAll, _window: &mut Window, cx: &mut Context<Self>) {
        self.editor.select_all();
        cx.notify();
    }

    fn escape(&mut self, _: &Escape, _window: &mut Window, cx: &mut Context<Self>) {
        self.editor.clear_selection();
        cx.notify();
    }

    fn copy(&mut self, _: &Copy, _window: &mut Window, cx: &mut Context<Self>) {
        let selected_text = self.get_selected_text();
        if !selected_text.is_empty() {
            cx.write_to_clipboard(ClipboardItem::new_string(selected_text));
        }
    }

    fn cut(&mut self, _: &Cut, _window: &mut Window, cx: &mut Context<Self>) {
        let selected_text = self.get_selected_text();
        if !selected_text.is_empty() {
            cx.write_to_clipboard(ClipboardItem::new_string(selected_text));
            self.editor.delete_selection();
            cx.notify();
        }
    }

    fn paste(&mut self, _: &Paste, _window: &mut Window, cx: &mut Context<Self>) {
        if let Some(clipboard) = cx.read_from_clipboard() {
            if let Some(text) = clipboard.text() {
                // Delete selection if exists
                self.editor.delete_selection();

                // Insert text character by character (simplified)
                for ch in text.chars() {
                    if ch == '\n' {
                        self.editor.insert_newline();
                    } else if ch != '\r' {
                        self.editor.insert_char(ch);
                    }
                }
                cx.notify();
            }
        }
    }

    fn next_theme(&mut self, _: &NextTheme, _window: &mut Window, cx: &mut Context<Self>) {
        self.current_theme_index = (self.current_theme_index + 1) % self.available_themes.len();
        self.editor
            .set_theme(&self.available_themes[self.current_theme_index]);
        cx.notify();
    }

    fn previous_theme(&mut self, _: &PreviousTheme, _window: &mut Window, cx: &mut Context<Self>) {
        self.current_theme_index = if self.current_theme_index == 0 {
            self.available_themes.len() - 1
        } else {
            self.current_theme_index - 1
        };
        self.editor
            .set_theme(&self.available_themes[self.current_theme_index]);
        cx.notify();
    }

    fn next_language(&mut self, _: &NextLanguage, _window: &mut Window, cx: &mut Context<Self>) {
        self.current_language_index =
            (self.current_language_index + 1) % self.available_languages.len();
        let (language, _, sample_code) = &self.available_languages[self.current_language_index];
        self.editor.set_language(language.clone());
        self.editor
            .update_buffer(sample_code.lines().map(|s| s.to_string()).collect());
        cx.notify();
    }

    fn previous_language(
        &mut self,
        _: &PreviousLanguage,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.current_language_index = if self.current_language_index == 0 {
            self.available_languages.len() - 1
        } else {
            self.current_language_index - 1
        };
        let (language, _, sample_code) = &self.available_languages[self.current_language_index];
        self.editor.set_language(language.clone());
        self.editor
            .update_buffer(sample_code.lines().map(|s| s.to_string()).collect());
        cx.notify();
    }
}

impl Render for EditorView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let _current_theme = &self.available_themes[self.current_theme_index];
        let (current_language, _, _) = &self.available_languages[self.current_language_index];

        let language = match current_language.as_str() {
            "Rust" => Language::Rust,
            _ => Language::PlainText,
        };

        let cursor_position = self.editor.cursor_position();
        let cursor_point = Point::new(cursor_position.col, cursor_position.row);

        let selection = if self.editor.has_selection() {
            let selected_text = self.get_selected_text();
            Some(Selection {
                lines: selected_text.matches('\n').count(),
                chars: selected_text.len(),
            })
        } else {
            None
        };

        div()
            .key_context("editor")
            .size_full()
            .flex()
            .flex_col()
            .child(
                div()
                    .flex_grow()
                    .track_focus(&self.focus_handle)
                    .on_action(cx.listener(Self::move_up))
                    .on_action(cx.listener(Self::move_down))
                    .on_action(cx.listener(Self::move_left))
                    .on_action(cx.listener(Self::move_right))
                    .on_action(cx.listener(Self::move_up_with_shift))
                    .on_action(cx.listener(Self::move_down_with_shift))
                    .on_action(cx.listener(Self::move_left_with_shift))
                    .on_action(cx.listener(Self::move_right_with_shift))
                    .on_action(cx.listener(Self::backspace))
                    .on_action(cx.listener(Self::delete))
                    .on_action(cx.listener(Self::insert_newline))
                    .on_action(cx.listener(Self::select_all))
                    .on_action(cx.listener(Self::escape))
                    .on_action(cx.listener(Self::copy))
                    .on_action(cx.listener(Self::cut))
                    .on_action(cx.listener(Self::paste))
                    .on_action(cx.listener(Self::next_theme))
                    .on_action(cx.listener(Self::previous_theme))
                    .on_action(cx.listener(Self::next_language))
                    .on_action(cx.listener(Self::previous_language))
                    .on_key_down(cx.listener(
                        |this: &mut Self, event: &KeyDownEvent, _window, cx| {
                            // Handle character input
                            if let Some(text) = &event.keystroke.key_char {
                                if !event.keystroke.modifiers.platform
                                    && !event.keystroke.modifiers.control
                                    && !event.keystroke.modifiers.function
                                {
                                    for ch in text.chars() {
                                        this.editor.insert_char(ch);
                                    }
                                    cx.notify();
                                }
                            }
                        },
                    ))
                    .child(EditorElement::new(self.editor.clone())),
            )
            .child(MetaLine::new(cursor_point, language, selection))
    }
}

fn load_keymaps(cx: &mut App) {
    // Load keymaps from JSON configuration
    let mut keymap_collection = KeymapCollection::new();

    // Try to load from file first, fall back to defaults if not found
    let keymap_path = Path::new("default-keymap.json");
    let loaded_from_file = if keymap_path.exists() {
        match keymap_collection.load_file(keymap_path) {
            Ok(_) => {
                println!("Loaded keymaps from file: {}", keymap_path.display());
                true
            }
            Err(e) => {
                eprintln!("Failed to load keymap file: {}", e);
                false
            }
        }
    } else {
        false
    };

    // If we didn't load from file, try built-in defaults
    if !loaded_from_file {
        if let Err(e) = keymap_collection.load_defaults() {
            eprintln!("Failed to load default keymaps: {}", e);
            eprintln!("Falling back to hardcoded keybindings");

            // Fallback to minimal hardcoded bindings
            load_fallback_keymaps(cx);
            return;
        }
        println!("Loaded built-in default keymaps");
    }

    // Get binding specifications from the loaded keymaps
    let specs = keymap_collection.get_binding_specs();

    // Create GPUI key bindings from the specifications
    // Since GPUI requires concrete action types, we need to map action names to actions
    let mut bindings = Vec::new();

    for spec in specs {
        // Only process editor_demo actions for this example
        if !spec.action_name.starts_with("editor_demo::") {
            continue;
        }

        let action_name = spec
            .action_name
            .strip_prefix("editor_demo::")
            .unwrap_or(&spec.action_name);
        let context = spec.context.as_deref();

        // Map action names to concrete actions
        // This is where the keymap system connects with GPUI's type system
        match action_name {
            "MoveUp" => bindings.push(KeyBinding::new(&spec.keystrokes, MoveUp, context)),
            "MoveDown" => bindings.push(KeyBinding::new(&spec.keystrokes, MoveDown, context)),
            "MoveLeft" => bindings.push(KeyBinding::new(&spec.keystrokes, MoveLeft, context)),
            "MoveRight" => bindings.push(KeyBinding::new(&spec.keystrokes, MoveRight, context)),
            "MoveUpWithShift" => {
                bindings.push(KeyBinding::new(&spec.keystrokes, MoveUpWithShift, context))
            }
            "MoveDownWithShift" => bindings.push(KeyBinding::new(
                &spec.keystrokes,
                MoveDownWithShift,
                context,
            )),
            "MoveLeftWithShift" => bindings.push(KeyBinding::new(
                &spec.keystrokes,
                MoveLeftWithShift,
                context,
            )),
            "MoveRightWithShift" => bindings.push(KeyBinding::new(
                &spec.keystrokes,
                MoveRightWithShift,
                context,
            )),
            "Backspace" => bindings.push(KeyBinding::new(&spec.keystrokes, Backspace, context)),
            "Delete" => bindings.push(KeyBinding::new(&spec.keystrokes, Delete, context)),
            "InsertNewline" => {
                bindings.push(KeyBinding::new(&spec.keystrokes, InsertNewline, context))
            }
            "SelectAll" => bindings.push(KeyBinding::new(&spec.keystrokes, SelectAll, context)),
            "Escape" => bindings.push(KeyBinding::new(&spec.keystrokes, Escape, context)),
            "Copy" => bindings.push(KeyBinding::new(&spec.keystrokes, Copy, context)),
            "Cut" => bindings.push(KeyBinding::new(&spec.keystrokes, Cut, context)),
            "Paste" => bindings.push(KeyBinding::new(&spec.keystrokes, Paste, context)),
            "NextTheme" => bindings.push(KeyBinding::new(&spec.keystrokes, NextTheme, context)),
            "PreviousTheme" => {
                bindings.push(KeyBinding::new(&spec.keystrokes, PreviousTheme, context))
            }
            "NextLanguage" => {
                bindings.push(KeyBinding::new(&spec.keystrokes, NextLanguage, context))
            }
            "PreviousLanguage" => {
                bindings.push(KeyBinding::new(&spec.keystrokes, PreviousLanguage, context))
            }
            unknown => {
                eprintln!("Unknown editor_demo action: {}", unknown);
            }
        }
    }

    if bindings.is_empty() {
        eprintln!("No valid keybindings found, using fallback");
        load_fallback_keymaps(cx);
    } else {
        println!(
            "Registered {} keybindings from configuration",
            bindings.len()
        );
        cx.bind_keys(bindings);
    }
}

fn load_fallback_keymaps(cx: &mut App) {
    // Minimal fallback keybindings for when the keymap system fails
    cx.bind_keys([
        KeyBinding::new("up", MoveUp, None),
        KeyBinding::new("down", MoveDown, None),
        KeyBinding::new("left", MoveLeft, None),
        KeyBinding::new("right", MoveRight, None),
        KeyBinding::new("backspace", Backspace, None),
        KeyBinding::new("delete", Delete, None),
        KeyBinding::new("enter", InsertNewline, None),
        KeyBinding::new("escape", Escape, None),
        // Platform-specific modifier keys
        #[cfg(target_os = "macos")]
        KeyBinding::new("cmd-c", Copy, None),
        #[cfg(not(target_os = "macos"))]
        KeyBinding::new("ctrl-c", Copy, None),
        #[cfg(target_os = "macos")]
        KeyBinding::new("cmd-x", Cut, None),
        #[cfg(not(target_os = "macos"))]
        KeyBinding::new("ctrl-x", Cut, None),
        #[cfg(target_os = "macos")]
        KeyBinding::new("cmd-v", Paste, None),
        #[cfg(not(target_os = "macos"))]
        KeyBinding::new("ctrl-v", Paste, None),
    ]);
}

fn main() {
    // Main application entry point
    Application::new().run(move |cx: &mut App| {
        // Load keymaps using the new keymap system
        load_keymaps(cx);

        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(Bounds::centered(
                    None,
                    size(px(800.0), px(600.0)),
                    cx,
                ))),
                ..Default::default()
            },
            |_window, cx| cx.new(EditorView::new),
        )
        .unwrap();
    });
}

// Sample code generators
fn get_rust_sample() -> String {
    r#"// Rust sample code
use std::collections::HashMap;

fn main() {
    let mut count = 0;

    // Count from 1 to 10
    for i in 1..=10 {
        count += i;
    }

    // HashMap example
    let mut scores = HashMap::new();
    scores.insert("Blue", 10);
    scores.insert("Yellow", 50);

    println!("Final count: {}", count);
}"#
    .to_string()
}

fn get_plain_text_sample() -> String {
    r#"This is a plain text document.

No syntax highlighting is applied to plain text files.
You can write anything here without worrying about code formatting.

Features of this editor:
- Syntax highlighting for multiple languages
- Theme switching with Cmd+[ and Cmd+]
- Language switching with Cmd+Shift+[ and Cmd+Shift+]
- Text selection with Shift+Arrow keys
- Copy, Cut, and Paste support
- Line numbers
- Active line highlighting

The editor uses the syntect library for syntax highlighting,
which provides TextMate-compatible syntax definitions and themes.

Try switching between different languages and themes to see
how the editor adapts to different file types!"#
        .to_string()
}
