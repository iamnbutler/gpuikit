# File System Module (`fs`)

The `fs` module in gpuikit provides comprehensive file management utilities for GPUI applications, including native OS file dialogs, async file operations, and integration with GPUI's entity system.

## Features

- **Native OS File Dialogs**: Open and save files using the operating system's native file picker
- **Async File Operations**: Load and save files asynchronously with GPUI's executor
- **Entity Integration**: `FileHandle` for safe file management within GPUI entities
- **State Tracking**: Track unsaved changes, original content, and file metadata
- **Cross-platform**: Works on macOS, Windows, and Linux

## Core Components

### `File` Struct

The main file representation with content and metadata management:

```rust
use gpuikit::fs::File;

// Create a new file
let mut file = File::new("/path/to/file.txt");

// Load an existing file
let file = File::load("/path/to/existing.txt")?;

// Create with initial contents
let file = File::new_with_contents("/path/to/new.txt", "Hello, world!");

// Save to disk
file.save()?;

// Save to a different location
file.save_as("/path/to/another.txt")?;

// Check for unsaved changes
if file.has_changes() {
    println!("File has unsaved changes");
}

// Revert to last saved state
file.revert()?;
```

### File Metadata

Access file information without modifying contents:

```rust
let file = File::load("document.txt")?;

println!("Name: {:?}", file.file_name());        // "document.txt"
println!("Stem: {:?}", file.file_stem());        // "document"
println!("Extension: {:?}", file.extension());   // "txt"
println!("Path: {}", file.path().display());     // Full path
println!("Size: {} bytes", file.size());         // File size
println!("Lines: {}", file.line_count());        // Number of lines
```

## File Dialogs

Use native OS dialogs for file operations:

### Opening Files

```rust
use gpuikit::fs::dialog::*;
use gpui::*;

// Simple file open
async fn open_file_example(cx: &AsyncApp) -> Result<()> {
    if let Some(file) = dialog::open_file(cx).await? {
        println!("Opened: {}", file.path().display());
        println!("Contents: {}", file.contents());
    }
    Ok(())
}

// Open with options
async fn open_with_options(cx: &AsyncApp) -> Result<()> {
    let options = OpenOptions::single_file()
        .with_filter("Text Files", vec!["txt", "md"])
        .with_filter("Source Code", vec!["rs", "js", "ts"]);

    if let Some(paths) = prompt_for_open(cx, options).await? {
        for path in paths {
            println!("Selected: {}", path.display());
        }
    }
    Ok(())
}

// Open multiple files
async fn open_multiple(cx: &AsyncApp) -> Result<()> {
    if let Some(files) = dialog::open_files(cx).await? {
        for file in files {
            println!("Opened: {}", file.path().display());
        }
    }
    Ok(())
}

// Select a directory
async fn select_directory(cx: &AsyncApp) -> Result<()> {
    let options = OpenOptions::directory();
    if let Some(paths) = prompt_for_open(cx, options).await? {
        if let Some(dir) = paths.first() {
            println!("Selected directory: {}", dir.display());
        }
    }
    Ok(())
}
```

### Saving Files

```rust
use gpuikit::fs::dialog::*;

// Save with dialog
async fn save_file_example(cx: &AsyncApp) -> Result<()> {
    let mut file = File::new_with_contents("untitled.txt", "Hello, world!");

    if dialog::save_file(cx, &mut file, None).await? {
        println!("Saved to: {}", file.path().display());
    }
    Ok(())
}

// Save with specific extension
async fn save_with_extension(cx: &AsyncApp) -> Result<()> {
    let path = prompt_for_save_with_extension(cx, None, "json").await?;

    if let Some(path) = path {
        let mut file = File::new(path);
        file.set_contents(r#"{"hello": "world"}"#);
        file.save()?;
    }
    Ok(())
}
```

## Entity Integration

### Using FileHandle

`FileHandle` provides safe async file operations within GPUI entities:

```rust
use gpuikit::fs::{FileHandle, dialog::FileDialogExt};
use gpui::*;

struct MyEditor {
    file_handle: Entity<FileHandle>,
}

impl MyEditor {
    fn new(cx: &mut App) -> Entity<Self> {
        let file_handle = cx.new(|| FileHandle::new());

        cx.new(|_| Self { file_handle })
    }

    fn open_file(&mut self, cx: &mut Context<Self>) {
        // Use the FileDialogExt trait for convenient dialog integration
        self.file_handle.update(cx, |handle, cx| {
            handle.prompt_open_file(cx);
        });
    }

    fn save_file(&mut self, cx: &mut Context<Self>) {
        self.file_handle.update(cx, |handle, cx| {
            handle.prompt_save_file(cx);
        });
    }

    fn load_specific_file(&mut self, path: PathBuf, cx: &mut Context<Self>) {
        self.file_handle.update(cx, |handle, cx| {
            handle.load(path, cx);
        });
    }
}
```

### Custom Entity Integration

Integrate file dialogs into your own entities:

```rust
use gpuikit::fs::dialog::*;
use gpui::*;

struct DocumentEditor {
    content: String,
    file_path: Option<PathBuf>,
}

impl DocumentEditor {
    fn open_document(&mut self, cx: &mut Context<Self>) {
        cx.spawn(async move |this, cx| {
            if let Some(file) = open_file(&cx).await? {
                this.update(&cx, |this, cx| {
                    this.content = file.contents().to_string();
                    this.file_path = Some(file.path().to_path_buf());
                    cx.notify();
                })?;
            }
            Ok::<(), anyhow::Error>(())
        })
        .detach_and_log_err(cx);
    }

    fn save_document(&mut self, cx: &mut Context<Self>) {
        let content = self.content.clone();
        let existing_path = self.file_path.clone();

        cx.spawn(async move |this, cx| {
            let path = prompt_for_save(&cx, existing_path).await?;

            if let Some(path) = path {
                let mut file = File::new(path.clone());
                file.set_contents(content);
                file.save()?;

                this.update(&cx, |this, cx| {
                    this.file_path = Some(path);
                    cx.notify();
                })?;
            }
            Ok::<(), anyhow::Error>(())
        })
        .detach_and_log_err(cx);
    }
}
```

## Async Operations

All file operations support async execution:

```rust
use gpuikit::fs::File;

// Async load
let file = File::load_async("/path/to/file.txt").await?;

// Async save
let mut file = File::new_with_contents("/path/to/file.txt", "content");
file.save_async().await?;

// Async reload
let new_contents = file.reload_async().await?;

// Async create
let file = File::create_async("/path/to/new.txt", "initial content").await?;

// Check existence async
let exists = File::exists_async("/path/to/check.txt").await;
```

## Window Context Support

For views that need file dialogs within a window context:

```rust
use gpuikit::fs::dialog::*;
use gpui::*;

async fn open_in_window(cx: &mut AsyncWindowContext) -> Result<()> {
    let options = OpenOptions::single_file()
        .with_filter("Images", vec!["png", "jpg", "gif"]);

    if let Some(paths) = prompt_for_open_in_window(cx, options).await? {
        // Process selected files
    }
    Ok(())
}

async fn save_in_window(cx: &mut AsyncWindowContext) -> Result<()> {
    if let Some(path) = prompt_for_save_in_window(cx, None).await? {
        // Save to selected path
    }
    Ok(())
}
```

## Complete Example

Here's a complete example showing a simple text editor with file operations:

```rust
use gpuikit::fs::{File, dialog::*};
use gpui::*;

struct SimpleEditor {
    content: String,
    current_file: Option<File>,
    status: String,
}

impl SimpleEditor {
    fn new(cx: &mut App) -> Entity<Self> {
        cx.new(|_| Self {
            content: String::new(),
            current_file: None,
            status: "Ready".to_string(),
        })
    }

    fn open(&mut self, cx: &mut Context<Self>) {
        cx.spawn(async move |this, cx| {
            if let Some(file) = open_file(&cx).await? {
                let content = file.contents().to_string();
                let path = file.path().display().to_string();

                this.update(&cx, |this, cx| {
                    this.content = content;
                    this.current_file = Some(file);
                    this.status = format!("Opened: {}", path);
                    cx.notify();
                })?;
            }
            Ok::<(), anyhow::Error>(())
        })
        .detach_and_log_err(cx);
    }

    fn save(&mut self, cx: &mut Context<Self>) {
        let content = self.content.clone();
        let existing_file = self.current_file.clone();

        cx.spawn(async move |this, cx| {
            let path = if let Some(file) = existing_file {
                Some(file.path().to_path_buf())
            } else {
                prompt_for_save(&cx, None).await?
            };

            if let Some(path) = path {
                let mut file = File::new(path.clone());
                file.set_contents(content);
                file.save()?;

                this.update(&cx, |this, cx| {
                    this.current_file = Some(file);
                    this.status = format!("Saved: {}", path.display());
                    cx.notify();
                })?;
            }
            Ok::<(), anyhow::Error>(())
        })
        .detach_and_log_err(cx);
    }
}

impl Render for SimpleEditor {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .child(
                div()
                    .flex_1()
                    .p_4()
                    .child(self.content.clone())
            )
            .child(
                div()
                    .h_8()
                    .px_4()
                    .child(self.status.clone())
            )
    }
}
```

## Error Handling

All file operations return `Result<T>` for proper error handling:

```rust
use gpuikit::fs::File;

match File::load("document.txt") {
    Ok(file) => {
        println!("Loaded: {}", file.path().display());
    }
    Err(e) => {
        eprintln!("Failed to load file: {}", e);
    }
}

// In async contexts with entities
cx.spawn(async move |this, cx| {
    match open_file(&cx).await {
        Ok(Some(file)) => {
            // Handle successful open
        }
        Ok(None) => {
            // User cancelled
        }
        Err(e) => {
            // Handle error
            log::error!("Failed to open file: {}", e);
        }
    }
    Ok(())
})
.detach_and_log_err(cx);
```

## Platform Notes

- **macOS**: Uses native Cocoa file dialogs
- **Windows**: Uses native Windows file dialogs
- **Linux**: Uses GTK or Qt dialogs depending on the desktop environment

The file dialogs automatically adapt to the host operating system's native style and behavior.

## Performance Considerations

- File operations are async by default to avoid blocking the UI thread
- Large files are handled efficiently with streaming where possible
- File metadata is cached to avoid repeated disk access
- Change tracking is memory-efficient using original content comparison

## License

This module is part of gpuikit and is licensed under the same terms as the parent project.
