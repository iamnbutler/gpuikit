//! Example demonstrating file dialog usage with gpuikit's fs module
//!
//! This example shows how to use the operating system's native file dialogs
//! for opening and saving files in a GPUI application.

use anyhow::Result;
use gpui::*;
use gpuikit::fs::{dialog::*, File, FileHandle};

/// Main view that demonstrates file operations with OS dialogs
struct FileDialogExample {
    /// Current file being edited
    file_handle: Entity<FileHandle>,

    /// Text content for editing
    content: String,

    /// Status message to display
    status: String,
}

impl FileDialogExample {
    fn new(cx: &mut App) -> Entity<Self> {
        let file_handle = cx.new(|| FileHandle::new());

        cx.new(|_| Self {
            file_handle,
            content: String::new(),
            status: "Ready. Use Cmd+O to open, Cmd+S to save.".to_string(),
        })
    }

    /// Opens a file using the OS dialog
    fn open_file(&mut self, cx: &mut Context<Self>) {
        self.status = "Opening file...".to_string();
        cx.notify();

        cx.spawn(async move |this, cx| {
            // Use the file dialog to let user choose a file
            let result = open_file(&cx).await;

            match result {
                Ok(Some(file)) => {
                    let content = file.contents().to_string();
                    let path = file.path().display().to_string();

                    this.update(&cx, |this, cx| {
                        // Update the file handle
                        this.file_handle.update(cx, |handle, cx| {
                            handle.file = Some(file);
                            cx.notify();
                        });

                        // Update the content and status
                        this.content = content;
                        this.status = format!("Opened: {}", path);
                        cx.notify();
                    })?;
                }
                Ok(None) => {
                    this.update(&cx, |this, cx| {
                        this.status = "Open cancelled".to_string();
                        cx.notify();
                    })?;
                }
                Err(e) => {
                    this.update(&cx, |this, cx| {
                        this.status = format!("Error opening file: {}", e);
                        cx.notify();
                    })?;
                }
            }

            Ok::<(), anyhow::Error>(())
        })
        .detach_and_log_err(cx);
    }

    /// Saves the current content to a file using the OS dialog
    fn save_file(&mut self, cx: &mut Context<Self>) {
        self.status = "Saving file...".to_string();
        cx.notify();

        let content = self.content.clone();
        let file_handle = self.file_handle.clone();

        cx.spawn(async move |this, cx| {
            // Check if we have an existing file
            let existing_path = file_handle
                .read(&cx)?
                .file()
                .map(|f| f.path().to_path_buf());

            // Prompt for save location
            let path = prompt_for_save(&cx, existing_path.clone()).await?;

            match path {
                Some(path) => {
                    // Create or update the file
                    let mut file = if let Some(existing) = existing_path {
                        File::load(existing).unwrap_or_else(|_| File::new(path.clone()))
                    } else {
                        File::new(path.clone())
                    };

                    file.set_contents(content);
                    file.save_as(&path)?;

                    let path_str = path.display().to_string();

                    this.update(&cx, |this, cx| {
                        // Update the file handle
                        this.file_handle.update(cx, |handle, cx| {
                            handle.file = Some(file);
                            cx.notify();
                        });

                        this.status = format!("Saved: {}", path_str);
                        cx.notify();
                    })?;
                }
                None => {
                    this.update(&cx, |this, cx| {
                        this.status = "Save cancelled".to_string();
                        cx.notify();
                    })?;
                }
            }

            Ok::<(), anyhow::Error>(())
        })
        .detach_and_log_err(cx);
    }

    /// Saves to a new file with a specific extension
    fn save_as_text(&mut self, cx: &mut Context<Self>) {
        self.status = "Saving as text file...".to_string();
        cx.notify();

        let content = self.content.clone();

        cx.spawn(async move |this, cx| {
            // Prompt for save location with .txt extension
            let path = prompt_for_save_with_extension(&cx, None, "txt").await?;

            match path {
                Some(path) => {
                    let mut file = File::new(path.clone());
                    file.set_contents(content);
                    file.save()?;

                    let path_str = path.display().to_string();

                    this.update(&cx, |this, cx| {
                        this.status = format!("Saved as text: {}", path_str);
                        cx.notify();
                    })?;
                }
                None => {
                    this.update(&cx, |this, cx| {
                        this.status = "Save cancelled".to_string();
                        cx.notify();
                    })?;
                }
            }

            Ok::<(), anyhow::Error>(())
        })
        .detach_and_log_err(cx);
    }

    /// Opens multiple files at once
    fn open_multiple(&mut self, cx: &mut Context<Self>) {
        self.status = "Opening multiple files...".to_string();
        cx.notify();

        cx.spawn(async move |this, cx| {
            let options =
                OpenOptions::multiple_files().with_filter("Text Files", vec!["txt", "md", "rs"]);

            let paths = prompt_for_open(&cx, options).await?;

            match paths {
                Some(paths) => {
                    let file_count = paths.len();
                    let mut combined_content = String::new();

                    for path in paths {
                        if let Ok(file) = File::load(&path) {
                            combined_content.push_str(&format!(
                                "=== {} ===\n{}\n\n",
                                path.display(),
                                file.contents()
                            ));
                        }
                    }

                    this.update(&cx, |this, cx| {
                        this.content = combined_content;
                        this.status = format!("Opened {} files", file_count);
                        cx.notify();
                    })?;
                }
                None => {
                    this.update(&cx, |this, cx| {
                        this.status = "Open cancelled".to_string();
                        cx.notify();
                    })?;
                }
            }

            Ok::<(), anyhow::Error>(())
        })
        .detach_and_log_err(cx);
    }

    /// Selects a directory
    fn select_directory(&mut self, cx: &mut Context<Self>) {
        self.status = "Selecting directory...".to_string();
        cx.notify();

        cx.spawn(async move |this, cx| {
            let options = OpenOptions::directory();
            let paths = prompt_for_open(&cx, options).await?;

            match paths.and_then(|p| p.into_iter().next()) {
                Some(path) => {
                    this.update(&cx, |this, cx| {
                        this.status = format!("Selected directory: {}", path.display());
                        this.content = format!("Directory path:\n{}", path.display());
                        cx.notify();
                    })?;
                }
                None => {
                    this.update(&cx, |this, cx| {
                        this.status = "Directory selection cancelled".to_string();
                        cx.notify();
                    })?;
                }
            }

            Ok::<(), anyhow::Error>(())
        })
        .detach_and_log_err(cx);
    }
}

// Define actions for the application
actions!(
    file_dialog_example,
    [
        OpenFile,
        SaveFile,
        SaveAsText,
        OpenMultiple,
        SelectDirectory
    ]
);

impl Render for FileDialogExample {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .size_full()
            .bg(rgb(0x1e1e1e))
            .text_color(rgb(0xffffff))
            .on_action(cx.listener(|this, _: &OpenFile, cx| this.open_file(cx)))
            .on_action(cx.listener(|this, _: &SaveFile, cx| this.save_file(cx)))
            .on_action(cx.listener(|this, _: &SaveAsText, cx| this.save_as_text(cx)))
            .on_action(cx.listener(|this, _: &OpenMultiple, cx| this.open_multiple(cx)))
            .on_action(cx.listener(|this, _: &SelectDirectory, cx| this.select_directory(cx)))
            .child(
                // Header with buttons
                div()
                    .flex()
                    .flex_row()
                    .gap_2()
                    .p_4()
                    .bg(rgb(0x2d2d30))
                    .child(button("Open File", OpenFile))
                    .child(button("Save File", SaveFile))
                    .child(button("Save as .txt", SaveAsText))
                    .child(button("Open Multiple", OpenMultiple))
                    .child(button("Select Directory", SelectDirectory)),
            )
            .child(
                // Text editor area
                div().flex_1().p_4().child(
                    div()
                        .size_full()
                        .bg(rgb(0x1e1e1e))
                        .border_1()
                        .border_color(rgb(0x3e3e42))
                        .rounded_md()
                        .p_4()
                        .overflow_y_scroll()
                        .child(
                            div()
                                .whitespace_pre_wrap()
                                .text_sm()
                                .font_family("monospace")
                                .child(self.content.clone()),
                        ),
                ),
            )
            .child(
                // Status bar
                div()
                    .h_8()
                    .px_4()
                    .flex()
                    .items_center()
                    .bg(rgb(0x007acc))
                    .text_sm()
                    .child(self.status.clone()),
            )
    }
}

/// Helper function to create a button
fn button(label: &str, action: impl Action) -> impl IntoElement {
    div()
        .id(ElementId::Name(label.into()))
        .px_4()
        .py_2()
        .bg(rgb(0x0e639c))
        .hover(|style| style.bg(rgb(0x1177bb)))
        .rounded_md()
        .cursor_pointer()
        .on_click(move |_event, window, cx| {
            window.dispatch_action(action.boxed_clone(), cx);
        })
        .child(label)
}

fn main() {
    let app = Application::new().run(|cx| {
        // Set up window options
        let options = WindowOptions {
            bounds: WindowBounds::Fixed(Bounds {
                origin: Point::new(px(100.0), px(100.0)),
                size: Size {
                    width: px(800.0),
                    height: px(600.0),
                },
            }),
            titlebar: Some(TitlebarOptions {
                title: Some("File Dialog Example".into()),
                appears_transparent: false,
                traffic_light_position: None,
            }),
            kind: WindowKind::Normal,
            is_movable: true,
            display_id: None,
            window_background: WindowBackgroundAppearance::Opaque,
            app_id: Some("com.example.file-dialog".to_string()),
            window_min_size: None,
        };

        cx.open_window(options, |cx| {
            let view = FileDialogExample::new(cx);
            view.clone()
        });

        // Set up keyboard shortcuts
        cx.bind_keys([
            ("cmd-o", OpenFile),
            ("cmd-s", SaveFile),
            ("cmd-shift-s", SaveAsText),
            ("cmd-shift-o", OpenMultiple),
            ("cmd-d", SelectDirectory),
        ]);
    });
}
