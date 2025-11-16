//! Example demonstrating a complete text editor with syntax highlighting
//!
//! Run with: cargo run --example editor_demo

use gpui::*;
use gpui_editor::*;
use gpuikit::fs::{dialog::*, File};
use gpuikit_keymap::KeymapCollection;
use std::path::{Path, PathBuf};

actions!(
    editor,
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
        Paste,
        NewFile,
        OpenFile,
        SaveFile,
        SaveFileAs
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
    current_file: Option<File>,
    file_path: Option<PathBuf>,
}

impl EditorView {
    fn new(cx: &mut Context<Self>) -> Self {
        let focus_handle = cx.focus_handle();

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

        let highlighter = SyntaxHighlighter::new();
        let available_themes = highlighter.available_themes();

        let default_theme_index = available_themes
            .iter()
            .position(|t| t == "base16-ocean.dark")
            .unwrap_or(0);

        editor.set_theme(&available_themes[default_theme_index]);

        let available_languages = vec![
            (
                "Plain Text".to_string(),
                "txt".to_string(),
                get_plain_text_sample(),
            ),
            ("Rust".to_string(), "rs".to_string(), get_rust_sample()),
            (
                "Markdown".to_string(),
                "md".to_string(),
                get_markdown_sample(),
            ),
        ];

        editor.set_language("Plain Text".to_string());

        Self {
            focus_handle,
            editor,
            current_theme_index: default_theme_index,
            available_themes,
            current_language_index: 0,
            available_languages,
            current_file: None,
            file_path: None,
        }
    }

    fn detect_language_from_extension(path: &std::path::Path) -> String {
        path.extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| match ext.to_lowercase().as_str() {
                "rs" => "Rust".to_string(),
                "md" | "markdown" => "Markdown".to_string(),
                _ => "Plain Text".to_string(),
            })
            .unwrap_or_else(|| "Plain Text".to_string())
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

    // Helper method to get the text from the editor
    fn get_editor_text(&self) -> String {
        self.editor.get_buffer().to_string()
    }

    fn new_file(&mut self, _: &NewFile, _window: &mut Window, cx: &mut Context<Self>) {
        // Clear the editor and reset file state
        self.editor = Editor::new(ElementId::Name("editor".into()), vec![]);
        // Preserve the current theme but default to Plain Text
        self.editor
            .set_theme(&self.available_themes[self.current_theme_index]);
        self.editor.set_language("Plain Text".to_string());
        self.current_language_index = 0; // Plain Text is at index 0
        self.current_file = None;
        self.file_path = None;
        cx.notify();
        println!("Created new file");
    }

    fn open_file(&mut self, _: &OpenFile, _window: &mut Window, cx: &mut Context<Self>) {
        // Use the OS file dialog to open a file
        cx.spawn(async move |this, cx| {
            // Show the file dialog with text file filters
            let options = OpenOptions::single_file()
                .with_filter(
                    "Text Files",
                    vec!["txt", "md", "rs", "js", "ts", "jsx", "tsx"],
                )
                .with_filter("All Files", vec!["*"]);

            match prompt_for_open(&cx, options).await {
                Ok(Some(paths)) => {
                    if let Some(path) = paths.first() {
                        // Try to load the selected file
                        match File::load(path) {
                            Ok(file) => {
                                let contents = file.contents().to_string();
                                let lines: Vec<String> =
                                    contents.lines().map(String::from).collect();
                                let file_name = path.display().to_string();

                                // Detect language from file extension
                                let detected_language =
                                    EditorView::detect_language_from_extension(path);

                                this.update(cx, |this, cx| {
                                    this.editor =
                                        Editor::new(ElementId::Name("editor".into()), lines);
                                    // Preserve the current theme
                                    this.editor.set_theme(
                                        &this.available_themes[this.current_theme_index],
                                    );
                                    // Set language based on file extension
                                    this.editor.set_language(detected_language.clone());

                                    // Update the current language index to match
                                    if let Some(index) = this
                                        .available_languages
                                        .iter()
                                        .position(|(lang, _, _)| lang == &detected_language)
                                    {
                                        this.current_language_index = index;
                                    }
                                    this.file_path = Some(path.clone());
                                    this.current_file = Some(file);
                                    cx.notify();
                                    println!("Opened file: {}", file_name);
                                })?;
                            }
                            Err(e) => {
                                println!("Failed to open file: {}", e);
                            }
                        }
                    }
                }
                Ok(None) => {
                    println!("File open cancelled");
                }
                Err(e) => {
                    println!("Error opening file dialog: {}", e);
                }
            }
            Ok::<(), anyhow::Error>(())
        })
        .detach_and_log_err(cx);
    }

    fn save_file(&mut self, _: &SaveFile, window: &mut Window, cx: &mut Context<Self>) {
        let editor_text = self.get_editor_text();
        if let Some(ref mut file) = self.current_file {
            // Update file contents with current editor text
            file.set_contents(editor_text);

            match file.save() {
                Ok(_) => {
                    println!("File saved successfully");
                }
                Err(e) => {
                    println!("Failed to save file: {}", e);
                }
            }
        } else {
            // No current file, treat as save as
            self.save_file_as(&SaveFileAs, window, cx);
        }
    }

    fn save_file_as(&mut self, _: &SaveFileAs, _window: &mut Window, cx: &mut Context<Self>) {
        // Use the OS save dialog
        let editor_text = self.get_editor_text();
        let current_path = self.file_path.clone();

        cx.spawn(async move |this, cx| {
            // Show the save dialog, defaulting to the current path if available
            match prompt_for_save(&cx, current_path).await {
                Ok(Some(path)) => {
                    let mut file = File::new_with_contents(&path, editor_text);

                    match file.save() {
                        Ok(_) => {
                            let file_name = path.display().to_string();

                            this.update(cx, |this, cx| {
                                this.file_path = Some(path.clone());
                                this.current_file = Some(file);
                                cx.notify();
                                println!("File saved as: {}", file_name);
                            })?;
                        }
                        Err(e) => {
                            println!("Failed to save file: {}", e);
                        }
                    }
                }
                Ok(None) => {
                    println!("Save cancelled");
                }
                Err(e) => {
                    println!("Error opening save dialog: {}", e);
                }
            }
            Ok::<(), anyhow::Error>(())
        })
        .detach_and_log_err(cx);
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
            "Markdown" => Language::Markdown,
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

        // Calculate file status display
        let file_status = if let Some(ref file) = self.current_file {
            let filename = self
                .file_path
                .as_ref()
                .and_then(|p| p.file_name())
                .and_then(|n| n.to_str())
                .unwrap_or("Untitled");

            if file.has_changes() {
                format!("{} *", filename)
            } else {
                filename.to_string()
            }
        } else {
            "Untitled".to_string()
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
                    .on_action(cx.listener(Self::new_file))
                    .on_action(cx.listener(Self::open_file))
                    .on_action(cx.listener(Self::save_file))
                    .on_action(cx.listener(Self::save_file_as))
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
            .child(
                div()
                    .flex()
                    .justify_between()
                    .child(MetaLine::new(cursor_point, language, selection))
                    .child(div().px_2().text_color(rgb(0x888888)).child(file_status)),
            )
    }
}

fn load_keymaps(cx: &mut App) {
    // Load keymaps from JSON configuration
    let mut keymap_collection = KeymapCollection::new();

    let keymap_path = Path::new("examples/demo-keymap.json");
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

    if !loaded_from_file {
        let demo_keymap = include_str!("demo-keymap.json");
        keymap_collection
            .load_json(demo_keymap)
            .expect("Failed to load embedded demo keymaps");
        println!("Loaded embedded demo keymaps");
    }

    let specs = keymap_collection.get_binding_specs();

    let mut bindings = Vec::new();

    for spec in specs {
        if !spec.action_name.starts_with("editor::") {
            continue;
        }

        let action_name = spec
            .action_name
            .strip_prefix("editor::")
            .unwrap_or(&spec.action_name);
        let context = spec.context.as_deref();

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
            "NewFile" => bindings.push(KeyBinding::new(&spec.keystrokes, NewFile, context)),
            "OpenFile" => bindings.push(KeyBinding::new(&spec.keystrokes, OpenFile, context)),
            "SaveFile" => bindings.push(KeyBinding::new(&spec.keystrokes, SaveFile, context)),
            "SaveFileAs" => bindings.push(KeyBinding::new(&spec.keystrokes, SaveFileAs, context)),
            unknown => {
                eprintln!("Unknown editor action: {}", unknown);
            }
        }
    }

    println!(
        "Registered {} keybindings from configuration",
        bindings.len()
    );
    cx.bind_keys(bindings);
}

fn main() {
    Application::new().run(move |cx: &mut App| {
        load_keymaps(cx);

        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(Bounds::centered(
                    None,
                    size(px(800.0), px(600.0)),
                    cx,
                ))),
                focus: true,
                ..Default::default()
            },
            |_window, cx| cx.new(EditorView::new),
        )
        .unwrap();

        cx.activate(true)
    });
}

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

fn get_markdown_sample() -> String {
    r#"# Markdown Sample Document

This is a **markdown** document demonstrating various markdown features.

## Text Formatting

- **Bold text** using double asterisks
- *Italic text* using single asterisks
- ~~Strikethrough~~ using double tildes
- `Inline code` using backticks

## Lists

### Unordered List
- First item
- Second item
  - Nested item
  - Another nested item
- Third item

### Ordered List
1. First step
2. Second step
3. Third step

## Code Blocks

```rust
fn main() {
    println!("Hello from Rust!");
}
```

```javascript
console.log("Hello from JavaScript!");
```

## Links and Images

[Visit GitHub](https://github.com)

![Alt text for image](image.png)

## Blockquotes

> This is a blockquote.
> It can span multiple lines.

## Tables

| Column 1 | Column 2 | Column 3 |
|----------|----------|----------|
| Row 1    | Data     | More     |
| Row 2    | Data     | More     |

## Horizontal Rule

---

## Task Lists

- [x] Completed task
- [ ] Incomplete task
- [ ] Another task to do

**Note:** This editor provides syntax highlighting for Markdown files,
making it easier to write and edit documentation."#
        .to_string()
}
