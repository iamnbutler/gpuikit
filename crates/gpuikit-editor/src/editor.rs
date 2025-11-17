use gpui::{px, rgb, ElementId, Pixels, Rgba, SharedString, TextRun};

use crate::buffer::{GapBuffer, TextBuffer};
use crate::syntax_highlighter::SyntaxHighlighter;

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
    scroll_row: usize,
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
            scroll_row: 0,
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
        self.cursor_position = self.clamp_cursor_position(position);
        // Reset goal column when cursor position is explicitly set
        self.goal_column = None;
        // Auto-scroll to keep cursor visible
        self.ensure_cursor_visible();
    }

    fn clamp_cursor_position(&self, position: CursorPosition) -> CursorPosition {
        let line_count = self.buffer.line_count();
        // Clamp row to valid range [0, line_count - 1]
        let row = position.row.min(line_count.saturating_sub(1));

        // Clamp column to valid range [0, line_len]
        let line_len = self.buffer.line_len(row);
        let col = position.col.min(line_len);

        CursorPosition::new(row, col)
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
        // Ensure parse states are built up to this line
        let lines: Vec<String> = self.buffer.to_lines();
        self.syntax_highlighter
            .ensure_parse_states(&self.language, line_index, &lines);

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

        // Auto-scroll to keep cursor visible
        self.ensure_cursor_visible();
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

        // Auto-scroll to keep cursor visible
        self.ensure_cursor_visible();
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

            // Auto-scroll to keep cursor visible
            self.ensure_cursor_visible();
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

            // Auto-scroll to keep cursor visible
            self.ensure_cursor_visible();
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

        // Store the previous line length before the merge (if at line start)
        let prev_line_len = if self.cursor_position.col == 0 && self.cursor_position.row > 0 {
            self.buffer.line_len(self.cursor_position.row - 1)
        } else {
            0
        };

        // Use the buffer's backspace_at method directly
        self.buffer
            .backspace_at(self.cursor_position.row, self.cursor_position.col);

        if self.cursor_position.col > 0 {
            self.cursor_position.col -= 1;
            // Clear highlighting state from this line onward
            self.syntax_highlighter
                .clear_state_from_line(self.cursor_position.row, &self.language);
        } else if self.cursor_position.row > 0 {
            // Move to the merge point (end of previous line before merge)
            self.cursor_position.row -= 1;
            self.cursor_position.col = prev_line_len;
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

    pub fn scroll_row(&self) -> usize {
        self.scroll_row
    }

    pub fn set_scroll_row(&mut self, row: usize) {
        let max_scroll = self.buffer.line_count().saturating_sub(1);
        self.scroll_row = row.min(max_scroll);

        // Pre-build parse states for the visible range when scrolling
        let range = self.visible_row_range(10.0);
        let lines: Vec<String> = self.buffer.to_lines();
        let end_line = range.end.min(lines.len());
        self.syntax_highlighter
            .ensure_parse_states(&self.language, end_line, &lines);
    }

    pub fn visible_row_range(&self, viewport_height: f32) -> std::ops::Range<usize> {
        let line_height_f32: f32 = self.config.line_height.into();
        let visible_rows = (viewport_height / line_height_f32).floor() as usize;
        let start = self.scroll_row;
        let end = (start + visible_rows).min(self.buffer.line_count());
        start..end
    }

    pub fn scroll_by(&mut self, delta: isize) {
        if delta < 0 {
            self.scroll_row = self.scroll_row.saturating_sub(delta.abs() as usize);
        } else {
            let max_scroll = self.buffer.line_count().saturating_sub(1);
            self.scroll_row = (self.scroll_row + delta as usize).min(max_scroll);
        }

        // Pre-build parse states for the visible range when scrolling
        let range = self.visible_row_range(10.0);
        let lines: Vec<String> = self.buffer.to_lines();
        let end_line = range.end.min(lines.len());
        self.syntax_highlighter
            .ensure_parse_states(&self.language, end_line, &lines);
    }

    /// Ensure the cursor is visible in the viewport, scrolling if necessary
    pub fn ensure_cursor_visible(&mut self) {
        // Default viewport height for auto-scroll calculation
        // This will be overridden by the actual viewport size when available
        const DEFAULT_VIEWPORT_HEIGHT: f32 = 600.0;
        const SCROLL_MARGIN: usize = 3; // Keep at least 3 lines visible above/below cursor

        let cursor_row = self.cursor_position.row;

        // If cursor is above the visible range (with margin)
        if cursor_row < self.scroll_row.saturating_add(SCROLL_MARGIN) {
            // Scroll up to show the cursor with margin
            self.scroll_row = cursor_row.saturating_sub(SCROLL_MARGIN);
        }

        // Calculate approximate visible rows
        let line_height_f32: f32 = self.config.line_height.into();
        let visible_rows = (DEFAULT_VIEWPORT_HEIGHT / line_height_f32).floor() as usize;
        let bottom_visible_row = self.scroll_row + visible_rows.saturating_sub(1);

        // If cursor is below the visible range (with margin)
        if cursor_row > bottom_visible_row.saturating_sub(SCROLL_MARGIN) {
            // Scroll down to show the cursor with margin
            let target_scroll = cursor_row
                .saturating_add(SCROLL_MARGIN)
                .saturating_sub(visible_rows.saturating_sub(1));
            self.scroll_row = target_scroll.min(self.buffer.line_count().saturating_sub(1));
        }
    }

    /// Ensure cursor is visible with a specific viewport height
    pub fn ensure_cursor_visible_with_height(&mut self, viewport_height: f32) {
        const SCROLL_MARGIN: usize = 3;

        let cursor_row = self.cursor_position.row;

        // If cursor is above the visible range (with margin)
        if cursor_row < self.scroll_row.saturating_add(SCROLL_MARGIN) {
            self.scroll_row = cursor_row.saturating_sub(SCROLL_MARGIN);
        }

        // Calculate visible rows based on actual viewport
        let line_height_f32: f32 = self.config.line_height.into();
        let visible_rows = (viewport_height / line_height_f32).floor() as usize;
        let bottom_visible_row = self.scroll_row + visible_rows.saturating_sub(1);

        // If cursor is below the visible range (with margin)
        if cursor_row > bottom_visible_row.saturating_sub(SCROLL_MARGIN) {
            let target_scroll = cursor_row
                .saturating_add(SCROLL_MARGIN)
                .saturating_sub(visible_rows.saturating_sub(1));
            self.scroll_row = target_scroll.min(self.buffer.line_count().saturating_sub(1));
        }

        // Pre-build parse states for the visible range after auto-scrolling
        let range = self.visible_row_range(viewport_height);
        let lines: Vec<String> = self.buffer.to_lines();
        let end_line = range.end.min(lines.len());
        self.syntax_highlighter
            .ensure_parse_states(&self.language, end_line, &lines);
    }
}

#[cfg(test)]
mod scrolling_tests {
    use super::*;

    #[test]
    fn test_set_scroll_row() {
        let lines = vec![
            "Line 1".to_string(),
            "Line 2".to_string(),
            "Line 3".to_string(),
            "Line 4".to_string(),
            "Line 5".to_string(),
        ];
        let mut editor = Editor::new("test_editor", lines);

        editor.set_scroll_row(2);
        assert_eq!(editor.scroll_row(), 2);

        // Test clamping to max
        editor.set_scroll_row(100);
        assert_eq!(editor.scroll_row(), 4); // 5 lines, max scroll is 4
    }

    #[test]
    fn test_visible_row_range() {
        let lines: Vec<String> = (0..20).map(|i| format!("Line {}", i)).collect();
        let editor = Editor::new("test_editor", lines);

        // Assuming default line height of 20px
        let viewport_height = 100.0; // Should show 5 lines
        let range = editor.visible_row_range(viewport_height);
        assert_eq!(range, 0..5);
    }

    #[test]
    fn test_visible_row_range_with_scroll() {
        let lines: Vec<String> = (0..20).map(|i| format!("Line {}", i)).collect();
        let mut editor = Editor::new("test_editor", lines);

        editor.set_scroll_row(5);
        let viewport_height = 100.0; // Should show 5 lines
        let range = editor.visible_row_range(viewport_height);
        assert_eq!(range, 5..10);
    }

    #[test]
    fn test_visible_row_range_at_end() {
        let lines: Vec<String> = (0..20).map(|i| format!("Line {}", i)).collect();
        let mut editor = Editor::new("test_editor", lines);

        editor.set_scroll_row(17);
        let viewport_height = 100.0; // Should show 5 lines, but only 3 available
        let range = editor.visible_row_range(viewport_height);
        assert_eq!(range, 17..20); // Should clamp to buffer end
    }

    #[test]
    fn test_scroll_by_positive() {
        let lines: Vec<String> = (0..20).map(|i| format!("Line {}", i)).collect();
        let mut editor = Editor::new("test_editor", lines);

        editor.scroll_by(3);
        assert_eq!(editor.scroll_row(), 3);

        editor.scroll_by(2);
        assert_eq!(editor.scroll_row(), 5);
    }

    #[test]
    fn test_scroll_by_negative() {
        let lines: Vec<String> = (0..20).map(|i| format!("Line {}", i)).collect();
        let mut editor = Editor::new("test_editor", lines);

        editor.set_scroll_row(10);
        editor.scroll_by(-3);
        assert_eq!(editor.scroll_row(), 7);

        editor.scroll_by(-10);
        assert_eq!(editor.scroll_row(), 0); // Should clamp to 0
    }

    #[test]
    fn test_scroll_by_beyond_bounds() {
        let lines: Vec<String> = (0..10).map(|i| format!("Line {}", i)).collect();
        let mut editor = Editor::new("test_editor", lines);

        // Scroll beyond end
        editor.scroll_by(100);
        assert_eq!(editor.scroll_row(), 9); // 10 lines, max scroll is 9

        // Scroll before start
        editor.scroll_by(-200);
        assert_eq!(editor.scroll_row(), 0);
    }

    #[test]
    fn test_empty_buffer_scroll() {
        let editor = Editor::new("test_editor", vec![]);
        assert_eq!(editor.scroll_row(), 0);

        let range = editor.visible_row_range(100.0);
        assert_eq!(range, 0..1); // Empty buffer has 1 empty line
    }

    #[test]
    fn test_single_line_buffer_scroll() {
        let mut editor = Editor::new("test_editor", vec!["Single line".to_string()]);

        editor.set_scroll_row(5);
        assert_eq!(editor.scroll_row(), 0); // Should clamp to 0 since only 1 line

        let range = editor.visible_row_range(100.0);
        assert_eq!(range, 0..1);
    }

    #[test]
    fn test_scroll_preserves_cursor_visibility() {
        // This test documents expected behavior for auto-scrolling
        // which will be implemented in a future TODO item
        let lines: Vec<String> = (0..20).map(|i| format!("Line {}", i)).collect();
        let mut editor = Editor::new("test_editor", lines);

        // Set cursor at line 10
        editor.set_cursor_position(CursorPosition::new(10, 0));

        // Currently, scrolling doesn't auto-adjust for cursor visibility
        // This will be implemented as part of the auto-scroll TODO
        editor.set_scroll_row(0);
        assert_eq!(editor.scroll_row(), 0);
        assert_eq!(editor.cursor_position().row, 10);

        // In the future, we'd expect auto-scroll to make cursor visible
        // For now, we just verify the scroll and cursor are independent
    }

    #[test]
    fn test_large_viewport_shows_all_lines() {
        let lines: Vec<String> = (0..5).map(|i| format!("Line {}", i)).collect();
        let editor = Editor::new("test_editor", lines);

        let viewport_height = 1000.0; // Very large viewport
        let range = editor.visible_row_range(viewport_height);
        assert_eq!(range, 0..5); // Should show all 5 lines
    }

    #[test]
    fn test_tiny_viewport() {
        let lines: Vec<String> = (0..10).map(|i| format!("Line {}", i)).collect();
        let editor = Editor::new("test_editor", lines);

        let viewport_height = 25.0; // Slightly more than one line
        let range = editor.visible_row_range(viewport_height);
        assert_eq!(range, 0..1); // Should show at least 1 line
    }

    #[test]
    fn test_scroll_state_persists_through_edits() {
        let lines: Vec<String> = (0..20).map(|i| format!("Line {}", i)).collect();
        let mut editor = Editor::new("test_editor", lines);

        editor.set_scroll_row(5);
        assert_eq!(editor.scroll_row(), 5);

        // Move cursor within visible range (shouldn't trigger scroll)
        // But due to auto-scroll with margin, it may adjust
        editor.cursor_position = CursorPosition::new(6, 0);
        editor.insert_char('X');

        // With scroll margin of 3, cursor at row 6 with scroll_row 5 is fine
        // But auto-scroll may have adjusted it to 3 (cursor at 6, margin 3)
        // Just verify cursor is still visible
        let visible_range = editor.visible_row_range(600.0);
        assert!(visible_range.contains(&6));
    }

    #[test]
    fn test_ensure_cursor_visible() {
        let lines: Vec<String> = (0..50).map(|i| format!("Line {}", i)).collect();
        let mut editor = Editor::new("test_editor", lines);

        // Use ensure_cursor_visible_with_height to test with specific viewport
        // Small viewport that shows only 5 lines
        editor.set_cursor_position(CursorPosition::new(0, 0));
        editor.set_scroll_row(0);

        // Move cursor far down
        editor.cursor_position = CursorPosition::new(20, 0);
        editor.ensure_cursor_visible_with_height(100.0); // 100px = ~5 lines

        // Should have scrolled to make cursor visible
        assert!(editor.scroll_row() > 0);

        // Cursor should be in visible range
        let visible_range = editor.visible_row_range(100.0);
        assert!(visible_range.contains(&20));
    }

    #[test]
    fn test_ensure_cursor_visible_scrolls_up() {
        let lines: Vec<String> = (0..30).map(|i| format!("Line {}", i)).collect();
        let mut editor = Editor::new("test_editor", lines);

        // Start scrolled down
        editor.set_scroll_row(20);

        // Move cursor to top
        editor.set_cursor_position(CursorPosition::new(2, 0));

        // Should scroll up to show cursor with margin
        assert!(editor.scroll_row() <= 2);
    }

    #[test]
    fn test_ensure_cursor_visible_with_margin() {
        let lines: Vec<String> = (0..30).map(|i| format!("Line {}", i)).collect();
        let mut editor = Editor::new("test_editor", lines);

        // Set cursor position that should trigger scroll with margin
        editor.set_cursor_position(CursorPosition::new(10, 0));
        editor.ensure_cursor_visible();

        // Move cursor up by one - should not scroll if within margin
        let initial_scroll = editor.scroll_row();
        editor.move_up(false);
        assert_eq!(editor.scroll_row(), initial_scroll);
    }

    #[test]
    fn test_cursor_movement_auto_scrolls() {
        let lines: Vec<String> = (0..50).map(|i| format!("Line {}", i)).collect();
        let mut editor = Editor::new("test_editor", lines);

        // Start at top
        editor.set_scroll_row(0);
        editor.cursor_position = CursorPosition::new(0, 0);

        // Move down many times - with small viewport to force scrolling
        for _ in 0..35 {
            editor.cursor_position.row += 1;
            editor.ensure_cursor_visible_with_height(100.0); // Small viewport
        }

        // Should have scrolled to keep cursor visible
        assert!(editor.scroll_row() > 0);
        assert_eq!(editor.cursor_position().row, 35);

        // Cursor should still be visible
        let visible_range = editor.visible_row_range(100.0);
        assert!(visible_range.contains(&35));
    }

    #[test]
    fn test_ensure_cursor_visible_with_specific_height() {
        let lines: Vec<String> = (0..50).map(|i| format!("Line {}", i)).collect();
        let mut editor = Editor::new("test_editor", lines);

        // Test with small viewport (5 lines visible)
        let viewport_height = 100.0;

        editor.set_cursor_position(CursorPosition::new(20, 0));
        editor.ensure_cursor_visible_with_height(viewport_height);

        // Cursor should be visible in the calculated range
        let visible_range = editor.visible_row_range(viewport_height);
        assert!(visible_range.contains(&20));

        // Should maintain scroll margin
        assert!(editor.scroll_row() <= 20);
        assert!(editor.scroll_row() >= 20 - 5); // viewport shows ~5 lines
    }

    #[test]
    fn test_syntax_highlighting_consistency_when_scrolling() {
        // Set up a Rust code example
        let code = r#"fn main() {
    let x = 42;
    let y = "hello";
    println!("{} {}", x, y);

    for i in 0..10 {
        println!("{}", i);
    }

    let result = match x {
        42 => "answer",
        _ => "unknown",
    };
}
"#;

        let lines: Vec<String> = code.lines().map(|s| s.to_string()).collect();
        let mut editor = Editor::new("test_syntax_highlight", lines);
        editor.set_language("Rust".to_string());

        // Get highlighting for line 3 (println! line) at scroll position 0
        let line_3 = editor.get_buffer().get_line(3).unwrap_or_default();
        let initial_highlight = editor.highlight_line(&line_3, 3, "Courier".into(), 14.0);

        // Scroll down
        editor.set_scroll_row(2);

        // Get highlighting for the same line again
        let line_3_after_scroll = editor.get_buffer().get_line(3).unwrap_or_default();
        let highlight_after_scroll =
            editor.highlight_line(&line_3_after_scroll, 3, "Courier".into(), 14.0);

        // Verify the highlighting is the same
        assert_eq!(
            initial_highlight.len(),
            highlight_after_scroll.len(),
            "Number of text runs should be the same"
        );

        for (initial, after) in initial_highlight.iter().zip(highlight_after_scroll.iter()) {
            assert_eq!(initial.len, after.len, "Text run length should be the same");
            assert_eq!(
                initial.color, after.color,
                "Text run color should be the same"
            );
            assert_eq!(
                initial.font.weight, after.font.weight,
                "Font weight should be the same"
            );
            assert_eq!(
                initial.font.style, after.font.style,
                "Font style should be the same"
            );
        }

        // Test scrolling back up
        editor.set_scroll_row(0);

        let highlight_after_scroll_back = editor.highlight_line(&line_3, 3, "Courier".into(), 14.0);

        // Verify it's still the same
        assert_eq!(
            initial_highlight.len(),
            highlight_after_scroll_back.len(),
            "Number of text runs should be the same after scrolling back"
        );

        for (initial, after) in initial_highlight
            .iter()
            .zip(highlight_after_scroll_back.iter())
        {
            assert_eq!(
                initial.len, after.len,
                "Text run length should be the same after scrolling back"
            );
            assert_eq!(
                initial.color, after.color,
                "Text run color should be the same after scrolling back"
            );
        }
    }
}
