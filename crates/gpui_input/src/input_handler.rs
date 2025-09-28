//! Common input handling trait for GPUI input components.

use std::ops::Range;

/// Common input handling behavior for text-based input fields.
///
/// This trait defines shared behavior and utilities for input components
/// without requiring Context or notification handling.
pub trait InputHandler {
    /// Get the current text content of the input.
    fn content(&self) -> &str;

    /// Get the current selection range.
    fn selection_range(&self) -> Range<usize>;

    /// Check if the input is currently disabled.
    fn is_disabled(&self) -> bool {
        false
    }

    /// Get the cursor position (end of selection range).
    fn cursor_offset(&self) -> usize {
        self.selection_range().end
    }

    /// Filter text before insertion. Override this to implement input validation.
    /// For example, numeric inputs can filter out non-numeric characters.
    fn filter_text(&self, text: &str) -> String {
        text.to_string()
    }

    /// Check if there is a text selection (non-empty range).
    fn has_selection(&self) -> bool {
        let range = self.selection_range();
        range.start != range.end
    }

    /// Get the selected text.
    fn selected_text(&self) -> &str {
        let range = self.selection_range();
        let content = self.content();
        if range.end <= content.len() {
            &content[range.clone()]
        } else {
            ""
        }
    }

    /// Calculate the previous grapheme boundary from the given offset.
    fn previous_boundary(&self, offset: usize) -> usize {
        use unicode_segmentation::UnicodeSegmentation;

        self.content()
            .grapheme_indices(true)
            .rev()
            .find_map(|(idx, _)| (idx < offset).then_some(idx))
            .unwrap_or(0)
    }

    /// Calculate the next grapheme boundary from the given offset.
    fn next_boundary(&self, offset: usize) -> usize {
        use unicode_segmentation::UnicodeSegmentation;

        self.content()
            .grapheme_indices(true)
            .find_map(|(idx, _)| (idx > offset).then_some(idx))
            .unwrap_or(self.content().len())
    }

    /// Check if the cursor is at the beginning of the input.
    fn is_cursor_at_start(&self) -> bool {
        self.cursor_offset() == 0 && !self.has_selection()
    }

    /// Check if the cursor is at the end of the input.
    fn is_cursor_at_end(&self) -> bool {
        self.cursor_offset() == self.content().len() && !self.has_selection()
    }

    /// Calculate where the cursor should move when pressing left.
    fn calculate_left_movement(&self) -> usize {
        let range = self.selection_range();
        if range.start != range.end {
            // If there's a selection, move to the start
            range.start
        } else if range.start > 0 {
            // Otherwise move to previous boundary
            self.previous_boundary(range.start)
        } else {
            0
        }
    }

    /// Calculate where the cursor should move when pressing right.
    fn calculate_right_movement(&self) -> usize {
        let range = self.selection_range();
        if range.start != range.end {
            // If there's a selection, move to the end
            range.end
        } else if range.end < self.content().len() {
            // Otherwise move to next boundary
            self.next_boundary(range.end)
        } else {
            self.content().len()
        }
    }

    /// Calculate the range to delete when backspace is pressed.
    fn calculate_backspace_range(&self) -> Option<Range<usize>> {
        let range = self.selection_range();
        if range.start != range.end {
            // Delete selection
            Some(range)
        } else if range.start > 0 {
            // Delete previous character
            let prev = self.previous_boundary(range.start);
            Some(prev..range.start)
        } else {
            None
        }
    }

    /// Calculate the range to delete when delete key is pressed.
    fn calculate_delete_range(&self) -> Option<Range<usize>> {
        let range = self.selection_range();
        if range.start != range.end {
            // Delete selection
            Some(range)
        } else if range.end < self.content().len() {
            // Delete next character
            let next = self.next_boundary(range.end);
            Some(range.end..next)
        } else {
            None
        }
    }
}
