//! Gap buffer implementation for efficient text editing.
//!
//! This module provides a gap buffer data structure that efficiently handles
//! text insertions and deletions at arbitrary positions. The gap buffer maintains
//! a gap (unused space) at the cursor position, making insertions and deletions
//! at that position O(1) operations.
//!
//! # Design
//!
//! The gap buffer uses a single contiguous array with a "gap" of unused space.
//! When text is inserted or deleted, the gap moves to that position first,
//! then the operation is performed by adjusting the gap boundaries.
//!
//! # Trade-offs
//!
//! - Insertions/deletions at cursor position: O(1)
//! - Moving the gap: O(n) where n is the distance moved
//! - Memory overhead: The gap size (typically grows as needed)
//! - Line-based operations: Currently O(n) as they require string conversion

use std::cmp::{max, min};

/// A minimal text buffer trait that supports the features we have so far
pub trait TextBuffer {
    /// Get the total number of lines in the buffer
    fn line_count(&self) -> usize;

    /// Get a specific line by index (0-based)
    fn get_line(&self, line_idx: usize) -> Option<String>;

    /// Get the length of a specific line in characters
    fn line_len(&self, line_idx: usize) -> usize {
        self.get_line(line_idx).map(|s| s.len()).unwrap_or(0)
    }

    /// Get all lines (for now, while we're simple)
    fn all_lines(&self) -> Vec<String>;

    /// Insert text at a specific position (row, col)
    fn insert_at(&mut self, row: usize, col: usize, text: &str);

    /// Delete a character at a specific position (row, col)
    fn delete_at(&mut self, row: usize, col: usize);

    /// Delete backwards from a specific position (row, col)
    fn backspace_at(&mut self, row: usize, col: usize);
}

/// Gap buffer implementation for efficient text editing.
///
/// The buffer maintains text as a vector of characters with a gap at the
/// current editing position. This makes insertions and deletions at the
/// cursor position very efficient (O(1)), at the cost of having to move
/// the gap when editing at different positions.
///
/// # Example Structure
///
/// For text "Hello World" with gap after "Hello":
/// ```text
/// ['H', 'e', 'l', 'l', 'o', '\0', '\0', '\0', ' ', 'W', 'o', 'r', 'l', 'd']
///                           ^gap_start      ^gap_end
/// ```
#[derive(Clone)]
pub struct GapBuffer {
    /// The buffer containing the text with a gap for efficient insertion
    buffer: Vec<char>,
    /// Start position of the gap (inclusive)
    gap_start: usize,
    /// End position of the gap (exclusive)
    gap_end: usize,
}

impl GapBuffer {
    /// Create a new empty gap buffer
    pub fn new() -> Self {
        let initial_capacity = 1024;
        let buffer = vec!['\0'; initial_capacity];
        Self {
            buffer,
            gap_start: 0,
            gap_end: initial_capacity,
        }
    }

    /// Create a gap buffer from text
    pub fn from_text(text: &str) -> Self {
        let chars: Vec<char> = text.chars().collect();
        let text_len = chars.len();

        // Add extra capacity for the gap
        let capacity = max(1024, text_len * 2);
        let mut buffer = vec!['\0'; capacity];

        // Copy text to the beginning
        for (i, &ch) in chars.iter().enumerate() {
            buffer[i] = ch;
        }

        Self {
            buffer,
            gap_start: text_len,
            gap_end: capacity,
        }
    }

    /// Create a gap buffer from lines
    pub fn from_lines(lines: Vec<String>) -> Self {
        Self::from_text(&lines.join("\n"))
    }

    /// Get the length of the content (excluding the gap)
    pub fn len(&self) -> usize {
        self.buffer.len() - self.gap_size()
    }

    /// Get the size of the gap
    fn gap_size(&self) -> usize {
        self.gap_end - self.gap_start
    }

    /// Move the gap to a specific position in the text.
    ///
    /// This is the core operation that enables efficient editing. When the gap
    /// is at the cursor position, insertions and deletions become O(1) operations.
    ///
    /// # Complexity
    /// O(n) where n is the distance between the current gap position and the target position
    pub fn move_gap_to(&mut self, position: usize) {
        let position = min(position, self.len());

        if position < self.gap_start {
            // Move gap backward - move characters from before gap to after gap
            let move_count = self.gap_start - position;

            // Move characters right-to-left to avoid overwriting
            for i in (0..move_count).rev() {
                self.buffer[self.gap_end - 1 - i] = self.buffer[self.gap_start - 1 - i];
            }

            self.gap_end -= move_count;
            self.gap_start -= move_count;
        } else if position > self.gap_start {
            // Move gap forward - move characters from after gap to before gap
            let move_count = position - self.gap_start;

            // Move characters left-to-right
            for i in 0..move_count {
                if self.gap_end + i < self.buffer.len() {
                    self.buffer[self.gap_start + i] = self.buffer[self.gap_end + i];
                }
            }

            self.gap_start += move_count;
            self.gap_end += move_count;
        }
    }

    /// Insert a character at the specified position.
    ///
    /// Moves the gap to the position first, then inserts the character
    /// by placing it at gap_start and incrementing gap_start.
    pub fn insert_char(&mut self, position: usize, ch: char) {
        self.move_gap_to(position);
        if self.gap_start >= self.gap_end {
            self.grow_gap();
        }
        self.buffer[self.gap_start] = ch;
        self.gap_start += 1;
    }

    /// Insert text at the specified position
    pub fn insert(&mut self, position: usize, text: &str) {
        self.move_gap_to(position);
        for ch in text.chars() {
            if self.gap_start >= self.gap_end {
                self.grow_gap();
            }
            self.buffer[self.gap_start] = ch;
            self.gap_start += 1;
        }
    }

    /// Delete backward from the specified position
    pub fn delete_backward(&mut self, position: usize) {
        if position > 0 {
            self.move_gap_to(position);
            self.gap_start -= 1;
        }
    }

    /// Delete forward from the specified position
    pub fn delete_forward(&mut self, position: usize) {
        self.move_gap_to(position);
        if self.gap_end < self.buffer.len() {
            self.gap_end += 1;
        }
    }

    /// Delete a range of text
    pub fn delete_range(&mut self, start: usize, end: usize) {
        if start >= end {
            return;
        }

        let start = min(start, self.len());
        let end = min(end, self.len());

        self.move_gap_to(start);
        let delete_count = end - start;
        self.gap_end = min(self.gap_end + delete_count, self.buffer.len());
    }

    /// Grow the gap when it becomes too small
    fn grow_gap(&mut self) {
        let old_capacity = self.buffer.len();
        let new_capacity = old_capacity * 2;
        let additional_capacity = new_capacity - old_capacity;

        let mut new_buffer = vec!['\0'; new_capacity];

        // Copy content before the gap
        for i in 0..self.gap_start {
            new_buffer[i] = self.buffer[i];
        }

        // Copy content after the gap
        let after_gap_start = self.gap_end;
        let after_gap_count = old_capacity - self.gap_end;
        let new_gap_end = self.gap_start + self.gap_size() + additional_capacity;

        for i in 0..after_gap_count {
            new_buffer[new_gap_end + i] = self.buffer[after_gap_start + i];
        }

        self.buffer = new_buffer;
        self.gap_end = new_gap_end;
    }

    /// Convert the buffer to a string.
    ///
    /// Reconstructs the text by concatenating the content before the gap
    /// with the content after the gap, skipping the gap itself.
    ///
    /// # Complexity
    /// O(n) where n is the length of the text
    pub fn to_string(&self) -> String {
        let mut result = String::with_capacity(self.len());

        // Add content before the gap
        for i in 0..self.gap_start {
            result.push(self.buffer[i]);
        }

        // Add content after the gap
        for i in self.gap_end..self.buffer.len() {
            let ch = self.buffer[i];
            if ch != '\0' {
                result.push(ch);
            }
        }

        result
    }

    /// Convert the buffer to lines
    pub fn to_lines(&self) -> Vec<String> {
        let text = self.to_string();
        if text.is_empty() {
            vec![String::new()]
        } else {
            let lines: Vec<String> = text.split('\n').map(|s| s.to_string()).collect();
            if lines.is_empty() {
                vec![String::new()]
            } else {
                lines
            }
        }
    }

    /// Convert cursor position (row, col) to buffer position.
    ///
    /// Translates a 2D cursor position (line and column) to a linear
    /// position in the buffer. This is used to map editor cursor positions
    /// to buffer positions for editing operations.
    ///
    /// # Returns
    /// The character index in the buffer corresponding to the cursor position
    pub fn cursor_to_position(&self, row: usize, col: usize) -> usize {
        let text = self.to_string();
        let mut current_row = 0;
        let mut current_col = 0;

        for (i, ch) in text.char_indices() {
            if current_row == row && current_col == col {
                return i;
            }

            if ch == '\n' {
                if current_row == row {
                    return i; // Return position at end of line if col is beyond line end
                }
                current_row += 1;
                current_col = 0;
            } else {
                current_col += 1;
            }
        }

        // Return end of text if position is beyond
        text.len()
    }

    /// Convert buffer position to cursor position (row, col).
    ///
    /// Translates a linear buffer position to a 2D cursor position
    /// (line and column). This is the inverse of `cursor_to_position`.
    ///
    /// # Returns
    /// A tuple of (row, column) representing the cursor position
    pub fn position_to_cursor(&self, position: usize) -> (usize, usize) {
        let text = self.to_string();
        let position = min(position, text.len());

        let mut row = 0;
        let mut col = 0;

        for (i, ch) in text.char_indices() {
            if i >= position {
                break;
            }
            if ch == '\n' {
                row += 1;
                col = 0;
            } else {
                col += 1;
            }
        }

        (row, col)
    }
}

impl Default for GapBuffer {
    fn default() -> Self {
        Self::new()
    }
}

impl TextBuffer for GapBuffer {
    fn line_count(&self) -> usize {
        let text = self.to_string();
        if text.is_empty() {
            1
        } else {
            text.chars().filter(|&c| c == '\n').count() + 1
        }
    }

    fn get_line(&self, line_idx: usize) -> Option<String> {
        let lines = self.to_lines();
        lines.get(line_idx).cloned()
    }

    fn all_lines(&self) -> Vec<String> {
        self.to_lines()
    }

    fn line_len(&self, line_idx: usize) -> usize {
        let lines = self.to_lines();
        lines.get(line_idx).map(|s| s.len()).unwrap_or(0)
    }

    fn insert_at(&mut self, row: usize, col: usize, text: &str) {
        let position = self.cursor_to_position(row, col);
        self.insert(position, text);
    }

    fn delete_at(&mut self, row: usize, col: usize) {
        let position = self.cursor_to_position(row, col);
        self.delete_forward(position);
    }

    fn backspace_at(&mut self, row: usize, col: usize) {
        let position = self.cursor_to_position(row, col);
        if position > 0 {
            self.delete_backward(position);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_line_splitting_with_newlines() {
        let buffer = GapBuffer::from_text("Hello\nWorld\nTest");
        let lines = buffer.to_lines();

        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0], "Hello");
        assert_eq!(lines[1], "World");
        assert_eq!(lines[2], "Test");
    }

    #[test]
    fn test_cursor_position_conversion() {
        let buffer = GapBuffer::from_text("Hello\nWorld");

        // Test cursor to position
        assert_eq!(buffer.cursor_to_position(0, 0), 0);
        assert_eq!(buffer.cursor_to_position(0, 5), 5);
        assert_eq!(buffer.cursor_to_position(1, 0), 6);
        assert_eq!(buffer.cursor_to_position(1, 5), 11);

        // Test position to cursor
        assert_eq!(buffer.position_to_cursor(0), (0, 0));
        assert_eq!(buffer.position_to_cursor(5), (0, 5));
        assert_eq!(buffer.position_to_cursor(6), (1, 0));
    }

    #[test]
    fn test_newline_insertion() {
        let mut buffer = GapBuffer::from_text("HelloWorld");
        buffer.insert(5, "\n");

        let lines = buffer.to_lines();
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0], "Hello");
        assert_eq!(lines[1], "World");

        assert_eq!(buffer.to_string(), "Hello\nWorld");
    }

    #[test]
    fn test_new_empty_buffer() {
        let buffer = GapBuffer::new();
        assert_eq!(buffer.len(), 0);
        assert_eq!(buffer.to_string(), "");
    }

    #[test]
    fn test_from_text() {
        let buffer = GapBuffer::from_text("Hello, World!");
        assert_eq!(buffer.len(), 13);
        assert_eq!(buffer.to_string(), "Hello, World!");
    }

    #[test]
    fn test_from_lines() {
        let lines = vec!["Hello".to_string(), "World".to_string()];
        let buffer = GapBuffer::from_lines(lines);
        assert_eq!(buffer.to_string(), "Hello\nWorld");
    }

    #[test]
    fn test_insert_char() {
        let mut buffer = GapBuffer::from_text("Hello");
        buffer.insert_char(5, '!');
        assert_eq!(buffer.to_string(), "Hello!");

        buffer.insert_char(0, '>');
        assert_eq!(buffer.to_string(), ">Hello!");

        buffer.insert_char(3, '<');
        assert_eq!(buffer.to_string(), ">He<llo!");
    }

    #[test]
    fn test_insert_text_various_positions() {
        let mut buffer = GapBuffer::from_text("Hello");

        // Insert at end
        buffer.insert(5, " World");
        assert_eq!(buffer.to_string(), "Hello World");

        // Insert at beginning
        buffer.insert(0, "Say ");
        assert_eq!(buffer.to_string(), "Say Hello World");

        // Insert in middle
        buffer.insert(9, " Beautiful");
        assert_eq!(buffer.to_string(), "Say Hello Beautiful World");
    }

    #[test]
    fn test_insert_multiline() {
        let mut buffer = GapBuffer::from_text("Line1");
        buffer.insert(5, "\nLine2\nLine3");
        assert_eq!(buffer.to_string(), "Line1\nLine2\nLine3");

        let lines = buffer.to_lines();
        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0], "Line1");
        assert_eq!(lines[1], "Line2");
        assert_eq!(lines[2], "Line3");
    }

    #[test]
    fn test_delete_backward() {
        let mut buffer = GapBuffer::from_text("Hello World");

        buffer.delete_backward(11);
        assert_eq!(buffer.to_string(), "Hello Worl");

        buffer.delete_backward(6);
        assert_eq!(buffer.to_string(), "HelloWorl");

        buffer.delete_backward(1);
        assert_eq!(buffer.to_string(), "elloWorl");
    }

    #[test]
    fn test_delete_forward() {
        let mut buffer = GapBuffer::from_text("Hello World");

        buffer.delete_forward(0);
        assert_eq!(buffer.to_string(), "ello World");

        buffer.delete_forward(4);
        assert_eq!(buffer.to_string(), "elloWorld");

        buffer.delete_forward(8);
        assert_eq!(buffer.to_string(), "elloWorl");
    }

    #[test]
    fn test_delete_range() {
        let mut buffer = GapBuffer::from_text("Hello World");

        buffer.delete_range(5, 11);
        assert_eq!(buffer.to_string(), "Hello");

        let mut buffer = GapBuffer::from_text("Hello World");
        buffer.delete_range(0, 6);
        assert_eq!(buffer.to_string(), "World");

        let mut buffer = GapBuffer::from_text("Hello World");
        buffer.delete_range(2, 8);
        assert_eq!(buffer.to_string(), "Herld");
    }

    #[test]
    fn test_delete_range_edge_cases() {
        let mut buffer = GapBuffer::from_text("Hello");

        // Delete everything
        buffer.delete_range(0, 5);
        assert_eq!(buffer.to_string(), "");

        // Delete with inverted range (should do nothing)
        let mut buffer = GapBuffer::from_text("Hello");
        buffer.delete_range(3, 2);
        assert_eq!(buffer.to_string(), "Hello");
    }

    #[test]
    fn test_move_gap_to() {
        let mut buffer = GapBuffer::from_text("Hello World");
        let initial_text = buffer.to_string();

        // Moving gap shouldn't change the text
        buffer.move_gap_to(5);
        assert_eq!(buffer.to_string(), initial_text);

        buffer.move_gap_to(0);
        assert_eq!(buffer.to_string(), initial_text);

        buffer.move_gap_to(11);
        assert_eq!(buffer.to_string(), initial_text);
    }

    #[test]
    fn test_gap_movement_with_operations() {
        let mut buffer = GapBuffer::from_text("ABC");

        // Insert at beginning
        buffer.insert(0, "1");
        assert_eq!(buffer.to_string(), "1ABC");

        // Insert at end
        buffer.insert(4, "2");
        assert_eq!(buffer.to_string(), "1ABC2");

        // Insert in middle
        buffer.insert(2, "X");
        assert_eq!(buffer.to_string(), "1AXBC2");
    }

    #[test]
    fn test_cursor_to_position() {
        let buffer = GapBuffer::from_text("Line1\nLine2\nLine3");

        assert_eq!(buffer.cursor_to_position(0, 0), 0);
        assert_eq!(buffer.cursor_to_position(0, 5), 5);
        assert_eq!(buffer.cursor_to_position(1, 0), 6);
        assert_eq!(buffer.cursor_to_position(1, 5), 11);
        assert_eq!(buffer.cursor_to_position(2, 0), 12);

        // Out of bounds column
        assert_eq!(buffer.cursor_to_position(0, 100), 5);
    }

    #[test]
    fn test_position_to_cursor() {
        let buffer = GapBuffer::from_text("Line1\nLine2\nLine3");

        assert_eq!(buffer.position_to_cursor(0), (0, 0));
        assert_eq!(buffer.position_to_cursor(5), (0, 5));
        assert_eq!(buffer.position_to_cursor(6), (1, 0));
        assert_eq!(buffer.position_to_cursor(11), (1, 5));
        assert_eq!(buffer.position_to_cursor(12), (2, 0));

        // Out of bounds position
        assert_eq!(buffer.position_to_cursor(100), (2, 5));
    }

    #[test]
    fn test_cursor_position_roundtrip() {
        let buffer = GapBuffer::from_text("Hello\nWorld\n!");

        for row in 0..3 {
            for col in 0..6 {
                let pos = buffer.cursor_to_position(row, col);
                let (r, c) = buffer.position_to_cursor(pos);
                if col <= buffer.line_len(row) {
                    assert_eq!((r, c), (row, col.min(buffer.line_len(row))));
                }
            }
        }
    }

    #[test]
    fn test_to_lines() {
        let buffer = GapBuffer::from_text("Line1\nLine2\nLine3");
        let lines = buffer.to_lines();
        assert_eq!(lines, vec!["Line1", "Line2", "Line3"]);

        let empty_buffer = GapBuffer::new();
        let empty_lines = empty_buffer.to_lines();
        assert_eq!(empty_lines, vec![""]);
    }

    #[test]
    fn test_line_count() {
        let buffer = GapBuffer::from_text("");
        assert_eq!(buffer.line_count(), 1);

        let buffer = GapBuffer::from_text("Hello");
        assert_eq!(buffer.line_count(), 1);

        let buffer = GapBuffer::from_text("Hello\nWorld");
        assert_eq!(buffer.line_count(), 2);

        let buffer = GapBuffer::from_text("Line1\nLine2\nLine3");
        assert_eq!(buffer.line_count(), 3);

        let buffer = GapBuffer::from_text("Line1\nLine2\n");
        assert_eq!(buffer.line_count(), 3);
    }

    #[test]
    fn test_line_len() {
        let buffer = GapBuffer::from_text("Hello\nWorld!\n");
        assert_eq!(buffer.line_len(0), 5);
        assert_eq!(buffer.line_len(1), 6);
        assert_eq!(buffer.line_len(2), 0);
    }

    #[test]
    fn test_insert_at() {
        let mut buffer = GapBuffer::from_text("Hello\nWorld");

        buffer.insert_at(0, 5, "!");
        assert_eq!(buffer.to_string(), "Hello!\nWorld");

        buffer.insert_at(1, 5, "!");
        assert_eq!(buffer.to_string(), "Hello!\nWorld!");

        buffer.insert_at(1, 0, "> ");
        assert_eq!(buffer.to_string(), "Hello!\n> World!");
    }

    #[test]
    fn test_delete_at() {
        let mut buffer = GapBuffer::from_text("Hello\nWorld");

        buffer.delete_at(0, 4);
        assert_eq!(buffer.to_string(), "Hell\nWorld");

        buffer.delete_at(1, 0);
        assert_eq!(buffer.to_string(), "Hell\norld");
    }

    #[test]
    fn test_backspace_at() {
        let mut buffer = GapBuffer::from_text("Hello\nWorld");

        buffer.backspace_at(0, 5);
        assert_eq!(buffer.to_string(), "Hell\nWorld");

        buffer.backspace_at(1, 1);
        assert_eq!(buffer.to_string(), "Hell\norld");

        // Backspace at beginning of line merges with previous
        buffer.backspace_at(1, 0);
        assert_eq!(buffer.to_string(), "Hellorld");
    }

    #[test]
    fn test_grow_gap() {
        let mut buffer = GapBuffer::from_text("Hi");
        let mut long_text = String::new();
        for _ in 0..1000 {
            long_text.push('x');
        }
        buffer.insert(2, &long_text);
        assert_eq!(buffer.to_string(), format!("Hi{}", long_text));
    }

    #[test]
    fn test_gap_movement_preserves_text() {
        let mut buffer = GapBuffer::from_text("The quick brown fox jumps over the lazy dog");
        let original = buffer.to_string();

        // Move gap to various positions
        for i in 0..original.len() {
            buffer.move_gap_to(i);
            assert_eq!(buffer.to_string(), original);
        }

        // Perform operations at different positions
        buffer.insert(10, "[INSERT]");
        assert_eq!(
            buffer.to_string(),
            "The quick [INSERT]brown fox jumps over the lazy dog"
        );

        buffer.delete_range(10, 18);
        assert_eq!(buffer.to_string(), original);
    }

    #[test]
    fn test_large_buffer_with_gap_movement() {
        let mut text = String::new();
        for i in 0..100 {
            text.push_str(&format!("Line {}\n", i));
        }

        let mut buffer = GapBuffer::from_text(&text);

        // Insert at various positions
        buffer.insert(0, "START\n");
        buffer.insert(buffer.len(), "\nEND");
        buffer.insert(buffer.len() / 2, "\nMIDDLE\n");

        let result = buffer.to_string();
        assert!(result.starts_with("START\n"));
        assert!(result.ends_with("\nEND"));
        assert!(result.contains("\nMIDDLE\n"));
    }

    #[test]
    fn test_empty_buffer_operations() {
        let mut buffer = GapBuffer::new();

        buffer.insert(0, "Hello");
        assert_eq!(buffer.to_string(), "Hello");

        buffer.delete_range(0, 5);
        assert_eq!(buffer.to_string(), "");

        buffer.insert(0, "World");
        assert_eq!(buffer.to_string(), "World");

        buffer.delete_backward(5);
        assert_eq!(buffer.to_string(), "Worl");
    }

    #[test]
    fn test_newline_handling() {
        let mut buffer = GapBuffer::new();

        buffer.insert(0, "Line1");
        buffer.insert(5, "\n");
        buffer.insert(6, "Line2");
        buffer.insert(11, "\n");
        buffer.insert(12, "Line3");

        let lines = buffer.to_lines();
        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0], "Line1");
        assert_eq!(lines[1], "Line2");
        assert_eq!(lines[2], "Line3");

        // Test deleting newlines
        buffer.delete_at(0, 5); // Delete newline after Line1
        assert_eq!(buffer.to_string(), "Line1Line2\nLine3");

        buffer.delete_forward(10); // Delete newline before Line3
        assert_eq!(buffer.to_string(), "Line1Line2Line3");
    }

    #[test]
    fn test_stress_random_operations() {
        let mut buffer = GapBuffer::from_text("Initial");

        // Perform many random operations
        for i in 0..50 {
            let pos = i % (buffer.len() + 1);
            buffer.insert(pos, &format!("{}", i % 10));

            if buffer.len() > 10 && i % 3 == 0 {
                let del_pos = (i * 7) % buffer.len();
                buffer.delete_forward(del_pos);
            }

            if buffer.len() > 5 && i % 5 == 0 {
                let del_pos = (i * 3) % buffer.len();
                buffer.delete_backward(del_pos);
            }
        }

        // Verify buffer integrity
        let text = buffer.to_string();
        let reconstructed = GapBuffer::from_text(&text);
        assert_eq!(reconstructed.to_string(), text);
    }

    #[test]
    fn test_large_text() {
        let mut large_text = String::new();
        for i in 0..1000 {
            large_text.push_str(&format!(
                "Line {}: This is a test line with some content.\n",
                i
            ));
        }

        let buffer = GapBuffer::from_text(&large_text);
        assert_eq!(buffer.to_string(), large_text);

        let lines = buffer.to_lines();
        assert_eq!(lines.len(), 1001); // 1000 lines + 1 empty line at end

        // Verify line operations
        assert_eq!(buffer.line_count(), 1001);
        assert!(buffer.get_line(500).is_some());
    }

    #[test]
    fn test_unicode_characters() {
        let mut buffer = GapBuffer::from_text("Hello 世界");
        assert_eq!(buffer.to_string(), "Hello 世界");

        buffer.insert(6, "🌍 ");
        assert_eq!(buffer.to_string(), "Hello 🌍 世界");

        buffer.insert_char(0, '👋');
        assert_eq!(buffer.to_string(), "👋Hello 🌍 世界");

        // Test with emoji and complex unicode
        let mut buffer = GapBuffer::from_text("🎨🎭🎪");
        buffer.delete_forward(0);
        assert_eq!(buffer.to_string(), "🎭🎪");
    }

    #[test]
    fn test_sequential_edits() {
        let mut buffer = GapBuffer::new();

        // Simulate typing
        let text = "The quick brown fox";
        for ch in text.chars() {
            buffer.insert_char(buffer.len(), ch);
        }
        assert_eq!(buffer.to_string(), text);

        // Simulate corrections
        buffer.delete_backward(19); // Delete 'x'
        buffer.delete_backward(18); // Delete 'o'
        buffer.delete_backward(17); // Delete 'f'
        buffer.insert(16, "cat");
        assert_eq!(buffer.to_string(), "The quick brown cat");

        // Add more text
        buffer.insert(buffer.len(), " jumped over the lazy dog");
        assert_eq!(
            buffer.to_string(),
            "The quick brown cat jumped over the lazy dog"
        );
    }
}
