//! An interactive, focusable editor view.
//!
//! [`EditorElement`] draws an [`Editor`] but handles no keyboard input of its
//! own — it is the picture, not the typewriter. [`EditorView`] is the other
//! half: a focusable [`Render`] that owns an `Editor`, inserts the characters
//! the platform reports as typed, and turns a fixed set of editing `actions`
//! into edits.
//!
//! What it deliberately does *not* own is the map from keys to those actions.
//! Which key selects a word or pastes the clipboard is platform- and
//! preference-specific, and gpui dispatches actions globally, so the binding
//! is the application's call — the same division `input` draws. gpuikit ships
//! a sensible default in [`EditorBindings`], installed by [`bind_editor_keys`]
//! (which `gpuikit::init` calls for you), and a consumer is free to replace,
//! extend, or ignore it. The actions themselves are public, so an application
//! can bind its own keys to them or dispatch them from a menu.
//!
//! ```
//! use gpuikit::editor::{EditorView, Language};
//! use gpui::AppContext as _;
//!
//! # fn build(cx: &mut gpui::App) {
//! let lines = "fn main() {}".lines().map(str::to_string).collect();
//! let view = cx.new(|cx| EditorView::new("my-editor", lines, Language::Rust, cx));
//! # let _ = view;
//! # }
//! ```

use gpui::{
    App, ClipboardItem, Context, FocusHandle, Focusable, InteractiveElement, IntoElement,
    KeyBinding, KeyDownEvent, ParentElement, Point, Render, Styled, Window, actions, div,
};

use super::{Editor, EditorElement, Language, MetaLine, Selection};

/// The key context an [`EditorView`] renders in, and the context
/// [`EditorBindings`] scopes its keys to. A binding in this context fires only
/// while an `EditorView` holds focus, so an editor's arrows do not move a
/// selection in some other focused element and vice versa.
pub const EDITOR_CONTEXT: &str = "Editor";

actions!(
    editor,
    [
        /// Delete the character before the cursor, or the selection.
        Backspace,
        /// Delete the character after the cursor, or the selection.
        Delete,
        /// Insert a newline at the cursor.
        InsertNewline,
        /// Move the cursor one character left.
        MoveLeft,
        /// Move the cursor one character right.
        MoveRight,
        /// Move the cursor up one line.
        MoveUp,
        /// Move the cursor down one line.
        MoveDown,
        /// Extend the selection one character left.
        SelectLeft,
        /// Extend the selection one character right.
        SelectRight,
        /// Extend the selection up one line.
        SelectUp,
        /// Extend the selection down one line.
        SelectDown,
        /// Select the whole buffer.
        SelectAll,
        /// Drop the selection, keeping the cursor where it is.
        ClearSelection,
        /// Copy the selection to the clipboard.
        Copy,
        /// Copy the selection to the clipboard and delete it.
        Cut,
        /// Insert the clipboard's text at the cursor.
        Paste,
    ]
);

/// The default key bindings for an [`EditorView`], one `Option` per editing
/// action so a consumer can drop or replace any single one and keep the rest.
///
/// [`Default`] fills every field with the platform's usual chord — `cmd-` on
/// macOS, `ctrl-` elsewhere — all scoped to [`EDITOR_CONTEXT`]. Build the set,
/// adjust the fields you disagree with, and hand it to [`bind_editor_keys`];
/// or start from [`EditorBindings::empty`] and set only the few you want.
pub struct EditorBindings {
    /// Binding for [`Backspace`]. Default: `backspace`.
    pub backspace: Option<KeyBinding>,
    /// Binding for [`Delete`]. Default: `delete`.
    pub delete: Option<KeyBinding>,
    /// Binding for [`InsertNewline`]. Default: `enter`.
    pub insert_newline: Option<KeyBinding>,
    /// Binding for [`MoveLeft`]. Default: `left`.
    pub move_left: Option<KeyBinding>,
    /// Binding for [`MoveRight`]. Default: `right`.
    pub move_right: Option<KeyBinding>,
    /// Binding for [`MoveUp`]. Default: `up`.
    pub move_up: Option<KeyBinding>,
    /// Binding for [`MoveDown`]. Default: `down`.
    pub move_down: Option<KeyBinding>,
    /// Binding for [`SelectLeft`]. Default: `shift-left`.
    pub select_left: Option<KeyBinding>,
    /// Binding for [`SelectRight`]. Default: `shift-right`.
    pub select_right: Option<KeyBinding>,
    /// Binding for [`SelectUp`]. Default: `shift-up`.
    pub select_up: Option<KeyBinding>,
    /// Binding for [`SelectDown`]. Default: `shift-down`.
    pub select_down: Option<KeyBinding>,
    /// Binding for [`SelectAll`]. Default: `cmd-a` (macOS) / `ctrl-a`.
    pub select_all: Option<KeyBinding>,
    /// Binding for [`ClearSelection`]. Default: `escape`.
    pub clear_selection: Option<KeyBinding>,
    /// Binding for [`struct@Copy`]. Default: `cmd-c` (macOS) / `ctrl-c`.
    pub copy: Option<KeyBinding>,
    /// Binding for [`Cut`]. Default: `cmd-x` (macOS) / `ctrl-x`.
    pub cut: Option<KeyBinding>,
    /// Binding for [`Paste`]. Default: `cmd-v` (macOS) / `ctrl-v`.
    pub paste: Option<KeyBinding>,
}

impl Default for EditorBindings {
    fn default() -> Self {
        let context = Some(EDITOR_CONTEXT);

        // The clipboard and select-all chords differ by platform; everything
        // else — arrows, backspace, enter — is the same key everywhere.
        #[cfg(target_os = "macos")]
        let (select_all, copy, cut, paste) = ("cmd-a", "cmd-c", "cmd-x", "cmd-v");
        #[cfg(not(target_os = "macos"))]
        let (select_all, copy, cut, paste) = ("ctrl-a", "ctrl-c", "ctrl-x", "ctrl-v");

        Self {
            backspace: Some(KeyBinding::new("backspace", Backspace, context)),
            delete: Some(KeyBinding::new("delete", Delete, context)),
            insert_newline: Some(KeyBinding::new("enter", InsertNewline, context)),
            move_left: Some(KeyBinding::new("left", MoveLeft, context)),
            move_right: Some(KeyBinding::new("right", MoveRight, context)),
            move_up: Some(KeyBinding::new("up", MoveUp, context)),
            move_down: Some(KeyBinding::new("down", MoveDown, context)),
            select_left: Some(KeyBinding::new("shift-left", SelectLeft, context)),
            select_right: Some(KeyBinding::new("shift-right", SelectRight, context)),
            select_up: Some(KeyBinding::new("shift-up", SelectUp, context)),
            select_down: Some(KeyBinding::new("shift-down", SelectDown, context)),
            select_all: Some(KeyBinding::new(select_all, SelectAll, context)),
            clear_selection: Some(KeyBinding::new("escape", ClearSelection, context)),
            copy: Some(KeyBinding::new(copy, Copy, context)),
            cut: Some(KeyBinding::new(cut, Cut, context)),
            paste: Some(KeyBinding::new(paste, Paste, context)),
        }
    }
}

impl EditorBindings {
    /// An `EditorBindings` with every field `None`. Set only the bindings you
    /// want, so no default chord is installed for the rest.
    pub fn empty() -> Self {
        Self {
            backspace: None,
            delete: None,
            insert_newline: None,
            move_left: None,
            move_right: None,
            move_up: None,
            move_down: None,
            select_left: None,
            select_right: None,
            select_up: None,
            select_down: None,
            select_all: None,
            clear_selection: None,
            copy: None,
            cut: None,
            paste: None,
        }
    }

    /// Collects every `Some` binding into a `Vec<KeyBinding>`.
    pub fn into_bindings(self) -> Vec<KeyBinding> {
        [
            self.backspace,
            self.delete,
            self.insert_newline,
            self.move_left,
            self.move_right,
            self.move_up,
            self.move_down,
            self.select_left,
            self.select_right,
            self.select_up,
            self.select_down,
            self.select_all,
            self.clear_selection,
            self.copy,
            self.cut,
            self.paste,
        ]
        .into_iter()
        .flatten()
        .collect()
    }
}

/// Installs [`EditorView`]'s key bindings. Pass `None` for the
/// platform-default [`EditorBindings`], or a set of your own. `gpuikit::init`
/// calls this with `None`; call it yourself only to override the defaults.
pub fn bind_editor_keys(cx: &mut App, bindings: impl Into<Option<EditorBindings>>) {
    let bindings = bindings.into().unwrap_or_default();
    cx.bind_keys(bindings.into_bindings());
}

/// A focusable, editable view over an [`Editor`].
///
/// Hold one as an `Entity<EditorView>`, render it, and — once it has focus —
/// it accepts typed characters and the editing `actions` this module defines.
/// See the [module docs](self) for the binding contract.
pub struct EditorView {
    focus_handle: FocusHandle,
    editor: Editor,
    language: Language,
}

impl EditorView {
    /// Builds a view over a new [`Editor`] holding `lines`, highlighted as
    /// `language`. Construct it inside a `cx.new(…)` closure, which is where
    /// the [`FocusHandle`] comes from.
    pub fn new(
        id: impl Into<gpui::ElementId>,
        lines: Vec<String>,
        language: Language,
        cx: &mut Context<Self>,
    ) -> Self {
        let mut editor = Editor::new(id, lines);
        editor.set_language(syntax_name(language).to_string());
        Self {
            focus_handle: cx.focus_handle(),
            editor,
            language,
        }
    }

    /// The wrapped editor, for reading its content or cursor.
    pub fn editor(&self) -> &Editor {
        &self.editor
    }

    /// The wrapped editor, mutably. Call [`Context::notify`] after an edit made
    /// through this so the view repaints.
    pub fn editor_mut(&mut self) -> &mut Editor {
        &mut self.editor
    }

    /// The language the meta line reports and the editor highlights as.
    pub fn language(&self) -> Language {
        self.language
    }

    /// Sets the language: what the meta line reports and what the editor
    /// highlights as. Does not change the buffer's contents.
    pub fn set_language(&mut self, language: Language, cx: &mut Context<Self>) {
        self.language = language;
        self.editor.set_language(syntax_name(language).to_string());
        cx.notify();
    }

    fn backspace(&mut self, _: &Backspace, _: &mut Window, cx: &mut Context<Self>) {
        self.editor.backspace();
        cx.notify();
    }

    fn delete(&mut self, _: &Delete, _: &mut Window, cx: &mut Context<Self>) {
        self.editor.delete();
        cx.notify();
    }

    fn insert_newline(&mut self, _: &InsertNewline, _: &mut Window, cx: &mut Context<Self>) {
        self.editor.insert_newline();
        cx.notify();
    }

    fn move_left(&mut self, _: &MoveLeft, _: &mut Window, cx: &mut Context<Self>) {
        self.editor.move_left(false);
        cx.notify();
    }

    fn move_right(&mut self, _: &MoveRight, _: &mut Window, cx: &mut Context<Self>) {
        self.editor.move_right(false);
        cx.notify();
    }

    fn move_up(&mut self, _: &MoveUp, _: &mut Window, cx: &mut Context<Self>) {
        self.editor.move_up(false);
        cx.notify();
    }

    fn move_down(&mut self, _: &MoveDown, _: &mut Window, cx: &mut Context<Self>) {
        self.editor.move_down(false);
        cx.notify();
    }

    fn select_left(&mut self, _: &SelectLeft, _: &mut Window, cx: &mut Context<Self>) {
        self.editor.move_left(true);
        cx.notify();
    }

    fn select_right(&mut self, _: &SelectRight, _: &mut Window, cx: &mut Context<Self>) {
        self.editor.move_right(true);
        cx.notify();
    }

    fn select_up(&mut self, _: &SelectUp, _: &mut Window, cx: &mut Context<Self>) {
        self.editor.move_up(true);
        cx.notify();
    }

    fn select_down(&mut self, _: &SelectDown, _: &mut Window, cx: &mut Context<Self>) {
        self.editor.move_down(true);
        cx.notify();
    }

    fn select_all(&mut self, _: &SelectAll, _: &mut Window, cx: &mut Context<Self>) {
        self.editor.select_all();
        cx.notify();
    }

    fn clear_selection(&mut self, _: &ClearSelection, _: &mut Window, cx: &mut Context<Self>) {
        self.editor.clear_selection();
        cx.notify();
    }

    fn copy(&mut self, _: &Copy, _: &mut Window, cx: &mut Context<Self>) {
        let text = self.editor.get_selected_text();
        if !text.is_empty() {
            cx.write_to_clipboard(ClipboardItem::new_string(text));
        }
    }

    fn cut(&mut self, _: &Cut, _: &mut Window, cx: &mut Context<Self>) {
        let text = self.editor.get_selected_text();
        if !text.is_empty() {
            cx.write_to_clipboard(ClipboardItem::new_string(text));
            self.editor.delete_selection();
            cx.notify();
        }
    }

    fn paste(&mut self, _: &Paste, _: &mut Window, cx: &mut Context<Self>) {
        let Some(text) = cx.read_from_clipboard().and_then(|item| item.text()) else {
            return;
        };
        self.editor.delete_selection();
        for ch in text.chars() {
            match ch {
                '\n' => self.editor.insert_newline(),
                '\r' => {}
                ch => self.editor.insert_char(ch),
            }
        }
        cx.notify();
    }

    fn on_key_down(&mut self, event: &KeyDownEvent, _: &mut Window, cx: &mut Context<Self>) {
        let Some(text) = event.keystroke.key_char.as_ref() else {
            return;
        };
        // A chord (cmd/ctrl/fn) is an action's business, not text; and control
        // characters — enter, backspace, tab — arrive as actions too, so
        // inserting them here would type them twice.
        let modifiers = event.keystroke.modifiers;
        if modifiers.platform || modifiers.control || modifiers.function {
            return;
        }
        let mut typed = false;
        for ch in text.chars().filter(|ch| !ch.is_control()) {
            self.editor.insert_char(ch);
            typed = true;
        }
        if typed {
            cx.notify();
        }
    }
}

impl Focusable for EditorView {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for EditorView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let cursor = self.editor.cursor_position();
        let selection = self.editor.has_selection().then(|| {
            let text = self.editor.get_selected_text();
            Selection {
                lines: text.matches('\n').count(),
                chars: text.chars().count(),
            }
        });

        div()
            .key_context(EDITOR_CONTEXT)
            .size_full()
            .flex()
            .flex_col()
            .child(
                div()
                    .flex_1()
                    .track_focus(&self.focus_handle)
                    .on_action(cx.listener(Self::backspace))
                    .on_action(cx.listener(Self::delete))
                    .on_action(cx.listener(Self::insert_newline))
                    .on_action(cx.listener(Self::move_left))
                    .on_action(cx.listener(Self::move_right))
                    .on_action(cx.listener(Self::move_up))
                    .on_action(cx.listener(Self::move_down))
                    .on_action(cx.listener(Self::select_left))
                    .on_action(cx.listener(Self::select_right))
                    .on_action(cx.listener(Self::select_up))
                    .on_action(cx.listener(Self::select_down))
                    .on_action(cx.listener(Self::select_all))
                    .on_action(cx.listener(Self::clear_selection))
                    .on_action(cx.listener(Self::copy))
                    .on_action(cx.listener(Self::cut))
                    .on_action(cx.listener(Self::paste))
                    .on_key_down(cx.listener(Self::on_key_down))
                    .child(EditorElement::new(self.editor.clone())),
            )
            .child(MetaLine::new(
                Point::new(cursor.col, cursor.row),
                self.language,
                selection,
            ))
    }
}

/// The syntect syntax name the editor highlights a [`Language`] as.
fn syntax_name(language: Language) -> &'static str {
    match language {
        Language::Rust => "rust",
        Language::Markdown => "markdown",
        Language::PlainText => "txt",
    }
}
