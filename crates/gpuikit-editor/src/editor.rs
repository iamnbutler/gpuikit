use crate::buffer::{GapBuffer, TextBuffer};
use crate::syntax_highlighter::SyntaxHighlighter;
use gpui::*;

#[derive(Clone)]
pub struct EditorConfig {
    pub line_height: Pixels,
    pub font_size: Pixels,
    pub gutter_width: Pixels,
    pub gutter_padding: Pixels,
    pub text_color: Rgba,
    pub line_number_color: Rgba,
    pub gutter_bg_color: Rgba,
    pub editor_bg_color: Rgba,
    pub active_line_bg_color: Rgba,
    pub font_family: SharedString,
}

impl Default for EditorConfig {
    fn default() -> Self {
        Self {
            line_height: px(20.0),
            font_size: px(14.0),
            gutter_width: px(50.0),
            gutter_padding: px(10.0),
            text_color: rgb(0xcccccc),
            line_number_color: rgb(0x666666),
            gutter_bg_color: rgb(0x252525),
            editor_bg_color: rgb(0x1e1e1e),
            active_line_bg_color: rgb(0x2a2a2a),
            font_family: "Monaco".into(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CursorPosition {
    pub row: usize,
    pub col: usize,
}

impl CursorPosition {
    pub fn new(row: usize, col: usize) -> Self {
        Self { row, col }
    }
}

#[derive(Clone)]
pub struct Editor {
    id: ElementId,
    buffer: GapBuffer,
    config: EditorConfig,
    cursor_position: CursorPosition,
    goal_column: Option<usize>,
    selection_anchor: Option<CursorPosition>,
    syntax_highlighter: SyntaxHighlighter,
    language: String,
    current_theme: String,
}

impl Editor {
    pub fn new(id: impl Into<ElementId>, lines: Vec<String>) -> Self {
        let id = id.into();
        let syntax_highlighter = SyntaxHighlighter::new();

        // Auto-detect language from content
        let full_text = lines.join("\n");
        let language = syntax_highlighter
            .detect_language(&full_text, Some("rs"))
            .unwrap_or_else(|| "Rust".to_string());

        Self {
            id,
            buffer: GapBuffer::from_lines(lines),
            config: EditorConfig::default(),
            cursor_position: CursorPosition { row: 0, col: 0 },
            goal_column: None,
            selection_anchor: None,
            syntax_highlighter,
            language,
            current_theme: String::new(),
        }
    }

    pub fn id(&self) -> &ElementId {
        &self.id
    }

    pub fn config(&self) -> &EditorConfig {
        &self.config
    }

    pub fn config_mut(&mut self) -> &mut EditorConfig {
        &mut self.config
    }

    pub fn set_config(&mut self, config: EditorConfig) {
        self.config = config;
    }

    pub fn cursor_position(&self) -> CursorPosition {
        self.cursor_position
    }

    pub fn set_cursor_position(&mut self, position: CursorPosition) {
        self.cursor_position = position;
        // Reset goal column when cursor position is explicitly set
        self.goal_column = None;
    }

    pub fn get_cursor_position(&self) -> CursorPosition {
        self.cursor_position
    }

    pub fn clear_selection(&mut self) {
        self.selection_anchor = None;
        // Reset goal column when clearing selection
        self.goal_column = None;
    }

    pub fn get_buffer(&self) -> &GapBuffer {
        &self.buffer
    }

    pub fn get_buffer_mut(&mut self) -> &mut GapBuffer {
        &mut self.buffer
    }

    pub fn language(&self) -> &str {
        &self.language
    }

    pub fn set_language(&mut self, language: String) {
        self.language = language;
    }

    pub fn current_theme(&self) -> &str {
        &self.current_theme
    }

    pub fn set_theme(&mut self, theme: &str) {
        self.current_theme = theme.to_string();
        self.syntax_highlighter.set_theme(theme);
        // Update colors from theme
        self.config.editor_bg_color = self.syntax_highlighter.get_theme_background().into();
        self.config.text_color = self.syntax_highlighter.get_theme_foreground().into();
        self.config.gutter_bg_color = self.syntax_highlighter.get_theme_gutter_background().into();
        self.config.active_line_bg_color =
            self.syntax_highlighter.get_theme_line_highlight().into();
    }

    pub fn update_buffer(&mut self, lines: Vec<String>) {
        self.buffer = GapBuffer::from_lines(lines);
        // Reset highlighting state to force complete re-highlighting
        self.syntax_highlighter.reset_state();
    }

    /// Update buffer content at a specific line (for future incremental updates)
    pub fn update_line(&mut self, line_index: usize, new_content: String) {
        // Delete the old line and insert the new content
        if line_index < self.buffer.line_count() {
            // Get the current line to find its length
            let line_len = self.buffer.line_len(line_index);

            // Delete the entire line content
            let start_pos = self.buffer.cursor_to_position(line_index, 0);
            let end_pos = self.buffer.cursor_to_position(line_index, line_len);
            self.buffer.delete_range(start_pos, end_pos);

            // Insert the new content
            self.buffer.insert_at(line_index, 0, &new_content);

            // Clear highlighting state from this line onward
            self.syntax_highlighter
                .clear_state_from_line(line_index, &self.language);
        }
    }

    /// Get syntax highlighting for a line
    pub fn highlight_line(
        &mut self,
        line: &str,
        line_index: usize,
        font_family: SharedString,
        font_size: f32,
    ) -> Vec<TextRun> {
        self.syntax_highlighter.highlight_line(
            line,
            &self.language,
            line_index,
            font_family,
            font_size,
        )
    }

    // Movement methods
    pub fn move_left(&mut self, shift_held: bool) {
        if shift_held && self.selection_anchor.is_none() {
            self.selection_anchor = Some(self.cursor_position);
        } else if !shift_held {
            self.selection_anchor = None;
        }

        // Reset goal column when moving horizontally
        self.goal_column = None;

        if self.cursor_position.col > 0 {
            self.cursor_position.col -= 1;
        } else if self.cursor_position.row > 0 {
            self.cursor_position.row -= 1;
            self.cursor_position.col = self.buffer.line_len(self.cursor_position.row);
        }
    }

    pub fn move_right(&mut self, shift_held: bool) {
        if shift_held && self.selection_anchor.is_none() {
            self.selection_anchor = Some(self.cursor_position);
        } else if !shift_held {
            self.selection_anchor = None;
        }

        // Reset goal column when moving horizontally
        self.goal_column = None;

        let current_line_len = self.buffer.line_len(self.cursor_position.row);

        if self.cursor_position.col < current_line_len {
            self.cursor_position.col += 1;
        } else if self.cursor_position.row < self.buffer.line_count().saturating_sub(1) {
            // Move to start of next line
            self.cursor_position.row += 1;
            self.cursor_position.col = 0;
        }
    }

    pub fn move_up(&mut self, shift_held: bool) {
        if shift_held && self.selection_anchor.is_none() {
            self.selection_anchor = Some(self.cursor_position);
        } else if !shift_held {
            self.selection_anchor = None;
        }

        if self.cursor_position.row > 0 {
            // Set goal column if not already set
            if self.goal_column.is_none() {
                self.goal_column = Some(self.cursor_position.col);
            }

            self.cursor_position.row -= 1;

            // Try to use goal column, but clamp to line length
            let line_len = self.buffer.line_len(self.cursor_position.row);
            self.cursor_position.col = self
                .goal_column
                .unwrap_or(self.cursor_position.col)
                .min(line_len);
        }
    }

    pub fn move_down(&mut self, shift_held: bool) {
        if shift_held && self.selection_anchor.is_none() {
            self.selection_anchor = Some(self.cursor_position);
        } else if !shift_held {
            self.selection_anchor = None;
        }

        if self.cursor_position.row < self.buffer.line_count().saturating_sub(1) {
            // Set goal column if not already set
            if self.goal_column.is_none() {
                self.goal_column = Some(self.cursor_position.col);
            }

            self.cursor_position.row += 1;

            // Try to use goal column, but clamp to line length
            let line_len = self.buffer.line_len(self.cursor_position.row);
            self.cursor_position.col = self
                .goal_column
                .unwrap_or(self.cursor_position.col)
                .min(line_len);
        }
    }

    pub fn select_all(&mut self) {
        // Reset goal column when selecting all
        self.goal_column = None;

        // Set anchor at beginning
        self.selection_anchor = Some(CursorPosition { row: 0, col: 0 });

        // Move cursor to end
        let last_row = self.buffer.line_count().saturating_sub(1);
        let last_col = self.buffer.line_len(last_row);
        self.cursor_position = CursorPosition {
            row: last_row,
            col: last_col,
        };
    }

    pub fn has_selection(&self) -> bool {
        self.selection_anchor.is_some()
    }

    pub fn get_selection_range(&self) -> Option<(CursorPosition, CursorPosition)> {
        self.selection_anchor.map(|anchor| {
            // Return (start, end) positions in document order
            if anchor.row < self.cursor_position.row
                || (anchor.row == self.cursor_position.row && anchor.col < self.cursor_position.col)
            {
                (anchor, self.cursor_position)
            } else {
                (self.cursor_position, anchor)
            }
        })
    }

    pub fn delete_selection(&mut self) -> bool {
        if let Some((start, end)) = self.get_selection_range() {
            // Convert cursor positions to buffer positions
            let start_pos = self.buffer.cursor_to_position(start.row, start.col);
            let end_pos = self.buffer.cursor_to_position(end.row, end.col);

            // Delete the range
            self.buffer.delete_range(start_pos, end_pos);

            // Update cursor position
            self.cursor_position = start;
            self.selection_anchor = None;
            self.goal_column = None;

            // Reset highlighting state from the changed line onward
            self.syntax_highlighter
                .clear_state_from_line(start.row, &self.language);

            true
        } else {
            false
        }
    }

    pub fn get_selected_text(&self) -> String {
        if let Some((start, end)) = self.get_selection_range() {
            // Convert cursor positions to buffer positions
            let start_pos = self.buffer.cursor_to_position(start.row, start.col);
            let end_pos = self.buffer.cursor_to_position(end.row, end.col);

            // Get the full text and extract the selection
            let text = self.buffer.to_string();

            // The positions are character indices, so we can slice the chars directly
            let chars: Vec<char> = text.chars().collect();
            if start_pos <= chars.len() && end_pos <= chars.len() && start_pos <= end_pos {
                chars[start_pos..end_pos].iter().collect()
            } else {
                String::new()
            }
        } else {
            String::new()
        }
    }

    pub fn insert_char(&mut self, ch: char) {
        // Delete selection first if there is one
        self.delete_selection();

        // Use the buffer's insert_at method directly
        self.buffer.insert_at(
            self.cursor_position.row,
            self.cursor_position.col,
            &ch.to_string(),
        );
        self.cursor_position.col += 1;
        self.goal_column = None;

        // Clear highlighting state from this line onward
        self.syntax_highlighter
            .clear_state_from_line(self.cursor_position.row, &self.language);
    }

    pub fn insert_newline(&mut self) {
        // Delete selection first if there is one
        self.delete_selection();

        // Use the buffer's insert_at method directly
        self.buffer
            .insert_at(self.cursor_position.row, self.cursor_position.col, "\n");
        self.cursor_position.row += 1;
        self.cursor_position.col = 0;
        self.goal_column = None;

        // Clear highlighting state from this line onward
        self.syntax_highlighter
            .clear_state_from_line(self.cursor_position.row - 1, &self.language);
    }

    pub fn backspace(&mut self) {
        // If there's a selection, delete it instead
        if self.selection_anchor.is_some() {
            self.delete_selection();
            return;
        }

        // Use the buffer's backspace_at method directly
        self.buffer
            .backspace_at(self.cursor_position.row, self.cursor_position.col);

        if self.cursor_position.col > 0 {
            self.cursor_position.col -= 1;
            // Clear highlighting state from this line onward
            self.syntax_highlighter
                .clear_state_from_line(self.cursor_position.row, &self.language);
        } else if self.cursor_position.row > 0 {
            // Move to end of previous line
            self.cursor_position.row -= 1;
            let line_len = self.buffer.line_len(self.cursor_position.row);
            self.cursor_position.col = line_len;
            // Clear highlighting state from the previous line onward
            self.syntax_highlighter
                .clear_state_from_line(self.cursor_position.row, &self.language);
        }

        self.goal_column = None;
    }

    pub fn delete(&mut self) {
        // If there's a selection, delete it instead
        if self.selection_anchor.is_some() {
            self.delete_selection();
            return;
        }

        // Use the buffer's delete_at method directly
        self.buffer
            .delete_at(self.cursor_position.row, self.cursor_position.col);

        // Clear highlighting state from this line onward
        self.syntax_highlighter
            .clear_state_from_line(self.cursor_position.row, &self.language);

        self.goal_column = None;
    }
}
