use anyhow::{anyhow, Result};
use gpui::Context;
use std::path::{Path, PathBuf};

/// Represents a file with its contents and metadata
#[derive(Debug, Clone)]
pub struct File {
    /// The full path to the file
    path: PathBuf,

    /// Current contents of the file (in memory)
    contents: String,

    /// Original contents when the file was loaded (for reverting)
    original_contents: Option<String>,

    /// Whether the file exists on disk
    exists_on_disk: bool,
}

impl File {
    /// Creates a new file instance without loading from disk
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self {
            path: path.into(),
            contents: String::new(),
            original_contents: None,
            exists_on_disk: false,
        }
    }

    /// Loads a file from disk
    pub fn load(path: impl Into<PathBuf>) -> Result<Self> {
        let path = path.into();

        // Check if file exists
        let metadata = std::fs::metadata(&path)?;

        if metadata.is_dir() {
            return Err(anyhow!("Path is a directory, not a file"));
        }

        // Read the file contents
        let contents = std::fs::read(&path)?;

        // Try to convert to string - panic if not a text file for now
        let contents = String::from_utf8(contents)
            .map_err(|_| anyhow!("File is not a valid UTF-8 text file"))?;

        Ok(Self {
            path,
            contents: contents.clone(),
            original_contents: Some(contents),
            exists_on_disk: true,
        })
    }

    /// Creates a new file with the given contents
    pub fn new_with_contents(path: impl Into<PathBuf>, contents: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            contents: contents.into(),
            original_contents: None,
            exists_on_disk: false,
        }
    }

    /// Saves the current contents to disk
    pub fn save(&mut self) -> Result<()> {
        // Create parent directories if they don't exist
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // Write the contents to disk
        std::fs::write(&self.path, &self.contents)?;

        // Update state after successful save
        self.original_contents = Some(self.contents.clone());
        self.exists_on_disk = true;

        Ok(())
    }

    /// Saves the file to a different path
    pub fn save_as(&mut self, new_path: impl Into<PathBuf>) -> Result<()> {
        self.path = new_path.into();
        self.save()
    }

    /// Loads a file from disk asynchronously using GPUI's background executor
    pub fn load_async(path: impl Into<PathBuf>) -> impl std::future::Future<Output = Result<Self>> {
        let path = path.into();

        async move { File::load(path) }
    }

    /// Saves the file asynchronously
    pub fn save_async(&mut self) -> impl std::future::Future<Output = Result<()>> {
        let path = self.path.clone();
        let contents = self.contents.clone();

        // Update state optimistically
        self.original_contents = Some(self.contents.clone());
        self.exists_on_disk = true;

        async move {
            // Create parent directories if they don't exist
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)?;
            }

            // Write the contents to disk
            std::fs::write(&path, &contents)?;
            Ok(())
        }
    }

    /// Reloads the file from disk asynchronously
    pub fn reload_async(&self) -> impl std::future::Future<Output = Result<String>> {
        let path = self.path.clone();
        let exists = self.exists_on_disk;

        async move {
            if !exists {
                return Err(anyhow!("File does not exist on disk"));
            }

            let contents = std::fs::read(&path)?;
            let contents = String::from_utf8(contents)
                .map_err(|_| anyhow!("File is not a valid UTF-8 text file"))?;

            Ok(contents)
        }
    }

    /// Reverts the contents to the last saved state
    pub fn revert(&mut self) -> Result<()> {
        match &self.original_contents {
            Some(original) => {
                self.contents = original.clone();
                Ok(())
            }
            None => Err(anyhow!("No original contents to revert to")),
        }
    }

    /// Reloads the file from disk
    pub fn reload(&mut self) -> Result<()> {
        if !self.exists_on_disk {
            return Err(anyhow!("File does not exist on disk"));
        }

        let contents = std::fs::read(&self.path)?;
        let contents = String::from_utf8(contents)
            .map_err(|_| anyhow!("File is not a valid UTF-8 text file"))?;

        self.contents = contents.clone();
        self.original_contents = Some(contents);

        Ok(())
    }

    // Metadata accessors

    /// Returns the file name (without path)
    pub fn file_name(&self) -> Option<&str> {
        self.path.file_name().and_then(|name| name.to_str())
    }

    /// Returns the file stem (filename without extension)
    pub fn file_stem(&self) -> Option<&str> {
        self.path.file_stem().and_then(|stem| stem.to_str())
    }

    /// Returns the file extension
    pub fn extension(&self) -> Option<&str> {
        self.path.extension().and_then(|ext| ext.to_str())
    }

    /// Returns the full path to the file
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Returns the parent directory of the file
    pub fn parent(&self) -> Option<&Path> {
        self.path.parent()
    }

    /// Returns a mutable reference to the contents
    pub fn contents_mut(&mut self) -> &mut String {
        &mut self.contents
    }

    /// Returns the contents as a string slice
    pub fn contents(&self) -> &str {
        &self.contents
    }

    /// Sets the contents
    pub fn set_contents(&mut self, contents: impl Into<String>) {
        self.contents = contents.into();
    }

    /// Checks if the file has unsaved changes
    pub fn has_changes(&self) -> bool {
        match &self.original_contents {
            Some(original) => &self.contents != original,
            None => !self.contents.is_empty(),
        }
    }

    /// Checks if the file exists on disk
    pub fn exists_on_disk(&self) -> bool {
        self.exists_on_disk
    }

    /// Checks if this is a new file (never saved)
    pub fn is_new(&self) -> bool {
        !self.exists_on_disk
    }

    /// Checks if a file exists at the given path
    pub fn exists_at_path(path: impl AsRef<Path>) -> bool {
        path.as_ref().exists() && path.as_ref().is_file()
    }

    /// Gets the byte size of the current contents
    pub fn size(&self) -> usize {
        self.contents.len()
    }

    /// Gets the number of lines in the file
    pub fn line_count(&self) -> usize {
        if self.contents.is_empty() {
            0
        } else {
            self.contents.lines().count()
        }
    }

    /// Checks if the file is empty
    pub fn is_empty(&self) -> bool {
        self.contents.is_empty()
    }

    /// Creates a new file asynchronously
    pub fn create_async(
        path: impl Into<PathBuf>,
        contents: impl Into<String>,
    ) -> impl std::future::Future<Output = Result<Self>> {
        let path = path.into();
        let contents = contents.into();

        async move {
            let mut file = File::new_with_contents(path, contents);
            file.save()?;
            Ok(file)
        }
    }

    /// Checks if a file exists at the given path asynchronously
    pub fn exists_async(path: impl Into<PathBuf>) -> impl std::future::Future<Output = bool> {
        let path = path.into();

        async move { path.exists() && path.is_file() }
    }
}

/// Helper struct for managing file operations within a GPUI entity
/// This provides safe async file operations when used with Entity<FileHandle>
pub struct FileHandle {
    file: Option<File>,
}

impl FileHandle {
    /// Creates a new empty file handle
    pub fn new() -> Self {
        Self { file: None }
    }

    /// Loads a file into this handle
    pub fn load(&mut self, path: impl Into<PathBuf>, cx: &mut Context<Self>) {
        let path = path.into();

        cx.spawn(async move |this, cx| {
            let file = cx
                .background_executor()
                .spawn(File::load_async(path))
                .await?;

            this.update(cx, |this, cx| {
                this.file = Some(file);
                cx.notify();
            })?;

            Ok::<(), anyhow::Error>(())
        })
        .detach_and_log_err(cx);
    }

    /// Saves the current file
    pub fn save(&mut self, cx: &mut Context<Self>) {
        if let Some(file) = &mut self.file {
            let save_future = file.save_async();

            cx.spawn(async move |_this, cx| {
                cx.background_executor().spawn(save_future).await?;
                Ok::<(), anyhow::Error>(())
            })
            .detach_and_log_err(cx);
        }
    }

    /// Gets a reference to the file if loaded
    pub fn file(&self) -> Option<&File> {
        self.file.as_ref()
    }

    /// Gets a mutable reference to the file if loaded
    pub fn file_mut(&mut self) -> Option<&mut File> {
        self.file.as_mut()
    }

    /// Reloads the file if it exists
    pub fn reload(&mut self, cx: &mut Context<Self>) {
        if let Some(file) = &self.file {
            let reload_future = file.reload_async();

            cx.spawn(async move |this, cx| {
                let new_contents = cx.background_executor().spawn(reload_future).await?;

                this.update(cx, |this, cx| {
                    if let Some(file) = &mut this.file {
                        file.contents = new_contents.clone();
                        file.original_contents = Some(new_contents);
                    }
                    cx.notify();
                })?;

                Ok::<(), anyhow::Error>(())
            })
            .detach_and_log_err(cx);
        }
    }
}

impl Default for FileHandle {
    fn default() -> Self {
        Self::new()
    }
}

/// File dialog utilities for opening and saving files using the operating system's native dialogs
pub mod dialog {
    use super::*;
    use gpui::{AsyncApp, AsyncWindowContext, PathPromptOptions};

    /// Options for opening files via system dialog
    #[derive(Debug, Clone)]
    pub struct OpenOptions {
        /// Whether to allow selecting files
        pub files: bool,
        /// Whether to allow selecting directories
        pub directories: bool,
        /// Whether to allow selecting multiple items
        pub multiple: bool,
        /// Optional file type filters (e.g., vec![("Text Files", vec!["txt", "md"])])
        pub filters: Vec<(&'static str, Vec<&'static str>)>,
    }

    impl Default for OpenOptions {
        fn default() -> Self {
            Self {
                files: true,
                directories: false,
                multiple: false,
                filters: Vec::new(),
            }
        }
    }

    impl OpenOptions {
        /// Creates options for opening a single file
        pub fn single_file() -> Self {
            Self::default()
        }

        /// Creates options for opening multiple files
        pub fn multiple_files() -> Self {
            Self {
                files: true,
                directories: false,
                multiple: true,
                filters: Vec::new(),
            }
        }

        /// Creates options for selecting a directory
        pub fn directory() -> Self {
            Self {
                files: false,
                directories: true,
                multiple: false,
                filters: Vec::new(),
            }
        }

        /// Adds a file type filter
        pub fn with_filter(mut self, name: &'static str, extensions: Vec<&'static str>) -> Self {
            self.filters.push((name, extensions));
            self
        }
    }

    /// Prompts the user to open one or more files using the OS file dialog
    pub async fn prompt_for_open(
        cx: &AsyncApp,
        options: OpenOptions,
    ) -> Result<Option<Vec<PathBuf>>> {
        let paths = cx
            .update(|cx| {
                cx.prompt_for_paths(PathPromptOptions {
                    files: options.files,
                    directories: options.directories,
                    multiple: options.multiple,
                    prompt: None,
                })
            })?
            .await??;

        Ok(paths)
    }

    /// Prompts the user to save a file using the OS save dialog
    pub async fn prompt_for_save(
        cx: &AsyncApp,
        default_path: Option<PathBuf>,
    ) -> Result<Option<PathBuf>> {
        let default_path = default_path.unwrap_or_else(|| {
            std::env::var("HOME")
                .ok()
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from("/"))
        });

        let path = cx
            .update(|cx| cx.prompt_for_new_path(&default_path, None))?
            .await??;

        Ok(path)
    }

    /// Prompts the user to save a file with a specific extension
    pub async fn prompt_for_save_with_extension(
        cx: &AsyncApp,
        default_path: Option<PathBuf>,
        extension: &str,
    ) -> Result<Option<PathBuf>> {
        let path = prompt_for_save(cx, default_path).await?;

        Ok(path.map(|mut p| {
            // Ensure the file has the specified extension
            if p.extension().is_none() || p.extension() != Some(std::ffi::OsStr::new(extension)) {
                p.set_extension(extension);
            }
            p
        }))
    }

    /// Prompts the user to open a file and loads it as a File instance
    pub async fn open_file(cx: &AsyncApp) -> Result<Option<File>> {
        let paths = prompt_for_open(cx, OpenOptions::single_file()).await?;

        match paths.and_then(|p| p.into_iter().next()) {
            Some(path) => {
                let file = File::load(path)?;
                Ok(Some(file))
            }
            None => Ok(None),
        }
    }

    /// Prompts the user to open multiple files and loads them
    pub async fn open_files(cx: &AsyncApp) -> Result<Option<Vec<File>>> {
        let paths = prompt_for_open(cx, OpenOptions::multiple_files()).await?;

        match paths {
            Some(paths) => {
                let mut files = Vec::new();
                for path in paths {
                    files.push(File::load(path)?);
                }
                Ok(Some(files))
            }
            None => Ok(None),
        }
    }

    /// Prompts the user to save a file and returns the chosen path
    pub async fn save_file(
        cx: &AsyncApp,
        file: &mut File,
        default_path: Option<PathBuf>,
    ) -> Result<bool> {
        let path =
            prompt_for_save(cx, default_path.or_else(|| Some(file.path().to_path_buf()))).await?;

        match path {
            Some(path) => {
                file.save_as(path)?;
                Ok(true)
            }
            None => Ok(false),
        }
    }

    /// Helper trait for entities that want to use file dialogs
    pub trait FileDialogExt {
        /// Opens a file dialog to load a file
        fn prompt_open_file(&mut self, cx: &mut Context<Self>)
        where
            Self: Sized;

        /// Opens a file dialog to save a file
        fn prompt_save_file(&mut self, cx: &mut Context<Self>)
        where
            Self: Sized;
    }

    /// Implementation example for FileHandle
    impl FileDialogExt for FileHandle {
        fn prompt_open_file(&mut self, cx: &mut Context<Self>) {
            cx.spawn(async move |this, cx| {
                if let Some(file) = open_file(&cx).await? {
                    this.update(cx, |this, cx| {
                        this.file = Some(file);
                        cx.notify();
                    })?;
                }
                Ok::<(), anyhow::Error>(())
            })
            .detach_and_log_err(cx);
        }

        fn prompt_save_file(&mut self, cx: &mut Context<Self>) {
            if let Some(file) = self.file.as_ref() {
                let current_path = file.path().to_path_buf();

                cx.spawn(async move |this, cx| {
                    let path = prompt_for_save(&cx, Some(current_path)).await?;

                    if let Some(path) = path {
                        this.update(cx, |this, cx| {
                            if let Some(file) = &mut this.file {
                                if let Err(e) = file.save_as(path) {
                                    log::error!("Failed to save file: {}", e);
                                }
                                cx.notify();
                            }
                        })?;
                    }

                    Ok::<(), anyhow::Error>(())
                })
                .detach_and_log_err(cx);
            }
        }
    }

    /// Window context version for use in views
    pub async fn prompt_for_open_in_window(
        cx: &mut AsyncWindowContext,
        options: OpenOptions,
    ) -> Result<Option<Vec<PathBuf>>> {
        let paths = cx
            .update(|_window, cx| {
                cx.prompt_for_paths(PathPromptOptions {
                    files: options.files,
                    directories: options.directories,
                    multiple: options.multiple,
                    prompt: None,
                })
            })?
            .await??;

        Ok(paths)
    }

    /// Window context version for save dialog
    pub async fn prompt_for_save_in_window(
        cx: &mut AsyncWindowContext,
        default_path: Option<PathBuf>,
    ) -> Result<Option<PathBuf>> {
        let default_path = default_path.unwrap_or_else(|| {
            std::env::var("HOME")
                .ok()
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from("/"))
        });

        let path = cx
            .update(|_window, cx| cx.prompt_for_new_path(&default_path, None))?
            .await??;

        Ok(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_metadata() {
        let file = File::new("/path/to/test.txt");
        assert_eq!(file.file_name(), Some("test.txt"));
        assert_eq!(file.file_stem(), Some("test"));
        assert_eq!(file.extension(), Some("txt"));
    }

    #[test]
    fn test_file_changes() {
        let mut file = File::new("/path/to/test.txt");
        assert!(!file.has_changes());

        file.set_contents("Hello, world!");
        assert!(file.has_changes());

        file.original_contents = Some("Hello, world!".to_string());
        assert!(!file.has_changes());

        file.set_contents("Modified content");
        assert!(file.has_changes());
    }

    #[test]
    fn test_revert() {
        let mut file = File::new("/path/to/test.txt");
        file.original_contents = Some("Original content".to_string());
        file.set_contents("Modified content");

        assert_eq!(file.contents(), "Modified content");
        file.revert().unwrap();
        assert_eq!(file.contents(), "Original content");
    }

    #[test]
    fn test_new_file() {
        let file = File::new("/path/to/new.txt");
        assert!(file.is_new());
        assert!(!file.exists_on_disk());
        assert_eq!(file.contents(), "");
    }

    #[test]
    fn test_file_with_contents() {
        let file = File::new_with_contents("/path/to/test.txt", "Initial content");
        assert_eq!(file.contents(), "Initial content");
        assert!(file.is_new());
        assert!(file.has_changes());
    }

    #[test]
    fn test_line_count() {
        let mut file = File::new("/path/to/test.txt");
        assert_eq!(file.line_count(), 0);

        file.set_contents("Line 1");
        assert_eq!(file.line_count(), 1);

        file.set_contents("Line 1\nLine 2\nLine 3");
        assert_eq!(file.line_count(), 3);
    }
}
