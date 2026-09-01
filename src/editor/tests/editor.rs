#[allow(unused)]
use super::super::*;

#[test]
fn test_empty_buffer_has_one_line() {
    let editor = Editor::new("test", vec![]);
    assert_eq!(editor.get_buffer().line_count(), 1);
    assert_eq!(editor.get_buffer().get_line(0), Some("".to_string()));
}

#[test]
fn test_empty_buffer_cursor_starts_at_origin() {
    let editor = Editor::new("test", vec![]);
    assert_eq!(editor.cursor_position(), CursorPosition::new(0, 0));
}

#[test]
fn test_empty_buffer_backspace_does_nothing() {
    let mut editor = Editor::new("test", vec![]);
    editor.backspace();
    assert_eq!(editor.get_buffer().line_count(), 1);
    assert_eq!(editor.cursor_position(), CursorPosition::new(0, 0));
}

#[test]
fn test_empty_buffer_delete_does_nothing() {
    let mut editor = Editor::new("test", vec![]);
    editor.delete();
    assert_eq!(editor.get_buffer().line_count(), 1);
    assert_eq!(editor.cursor_position(), CursorPosition::new(0, 0));
}

#[test]
fn test_empty_buffer_movement_stays_at_origin() {
    let mut editor = Editor::new("test", vec![]);

    editor.move_up(false);
    assert_eq!(editor.cursor_position(), CursorPosition::new(0, 0));

    editor.move_down(false);
    assert_eq!(editor.cursor_position(), CursorPosition::new(0, 0));

    editor.move_left(false);
    assert_eq!(editor.cursor_position(), CursorPosition::new(0, 0));

    editor.move_right(false);
    assert_eq!(editor.cursor_position(), CursorPosition::new(0, 0));
}

#[test]
fn test_empty_buffer_insert_character() {
    let mut editor = Editor::new("test", vec![]);
    editor.insert_char('A');
    assert_eq!(editor.get_buffer().line_count(), 1);
    assert_eq!(editor.get_buffer().get_line(0), Some("A".to_string()));
    assert_eq!(editor.cursor_position(), CursorPosition::new(0, 1));
}

#[test]
fn test_empty_buffer_select_all() {
    let mut editor = Editor::new("test", vec![]);
    editor.select_all();
    assert_eq!(editor.get_selected_text(), "");
}

#[test]
fn test_single_char_delete_at_end_does_nothing() {
    let mut editor = Editor::new("test", vec!["X".to_string()]);
    editor.set_cursor_position(CursorPosition::new(0, 1));
    editor.delete();
    assert_eq!(editor.get_buffer().get_line(0), Some("X".to_string()));
}

#[test]
fn test_single_char_backspace_deletes_character() {
    let mut editor = Editor::new("test", vec!["X".to_string()]);
    editor.set_cursor_position(CursorPosition::new(0, 1));
    editor.backspace();
    assert_eq!(editor.get_buffer().get_line(0), Some("".to_string()));
}

#[test]
fn test_chinese_characters() {
    let mut editor = Editor::new("test", vec![]);
    editor.insert_char('你');
    editor.insert_char('好');
    assert_eq!(editor.get_buffer().get_line(0), Some("你好".to_string()));
    assert_eq!(editor.cursor_position(), CursorPosition::new(0, 2));
}

#[test]
fn test_arabic_text() {
    let mut editor = Editor::new("test", vec![]);
    let arabic = "مرحبا";
    for ch in arabic.chars() {
        editor.insert_char(ch);
    }
    assert_eq!(editor.get_buffer().get_line(0), Some(arabic.to_string()));
}

// TODO: This test requires Unicode normalization feature
// #[test]
// fn test_combining_diacritics() {
//     let mut editor = Editor::new("test", vec![]);
//     // e + combining acute accent
//     editor.insert_char('e');
//     editor.insert_char('\u{0301}'); // combining acute accent
//
//     // The buffer should normalize to the composed form
//     assert_eq!(
//         editor.get_buffer().get_line(0),
//         Some("é".to_string()), // U+00E9 - composed form
//         "Combining characters should be normalized to their composed form"
//     );
//
//     // Cursor should be at position 1 since it's one grapheme cluster
//     assert_eq!(
//         editor.cursor_position(),
//         CursorPosition::new(0, 1),
//         "Cursor should treat normalized character as single position"
//     );
// }

#[test]
fn test_cursor_at_start_of_line() {
    let mut editor = Editor::new("test", vec!["Hello".to_string()]);
    editor.set_cursor_position(CursorPosition::new(0, 0));

    editor.move_left(false);
    assert_eq!(editor.cursor_position(), CursorPosition::new(0, 0));

    editor.backspace();
    assert_eq!(editor.get_buffer().get_line(0), Some("Hello".to_string()));
}

#[test]
fn test_cursor_at_end_of_line() {
    let mut editor = Editor::new("test", vec!["Hello".to_string()]);
    editor.set_cursor_position(CursorPosition::new(0, 5));

    editor.move_right(false);
    assert_eq!(editor.cursor_position(), CursorPosition::new(0, 5));

    editor.delete();
    assert_eq!(editor.get_buffer().get_line(0), Some("Hello".to_string()));
}

#[test]
fn test_cursor_wrapping_to_next_line() {
    let mut editor = Editor::new("test", vec!["First".to_string(), "Second".to_string()]);
    editor.set_cursor_position(CursorPosition::new(0, 5)); // End of first line

    // Moving right from end of line should wrap to beginning of next line
    editor.move_right(false);
    assert_eq!(editor.cursor_position(), CursorPosition::new(1, 0));
}

#[test]
fn test_cursor_wrapping_to_previous_line() {
    let mut editor = Editor::new("test", vec!["First".to_string(), "Second".to_string()]);
    editor.set_cursor_position(CursorPosition::new(1, 0)); // Beginning of second line

    // Moving left from beginning of line should wrap to end of previous line
    editor.move_left(false);
    assert_eq!(editor.cursor_position(), CursorPosition::new(0, 5));
}

#[test]
fn test_select_single_character() {
    let mut editor = Editor::new("test", vec!["Hello".to_string()]);
    editor.set_cursor_position(CursorPosition::new(0, 0));
    editor.move_right(true);
    assert_eq!(editor.get_selected_text(), "H");
}

#[test]
fn test_select_word() {
    let mut editor = Editor::new("test", vec!["Hello World".to_string()]);
    editor.set_cursor_position(CursorPosition::new(0, 0));
    for _ in 0..5 {
        editor.move_right(true);
    }
    assert_eq!(editor.get_selected_text(), "Hello");
}

#[test]
fn test_select_across_lines() {
    let mut editor = Editor::new("test", vec!["First".to_string(), "Second".to_string()]);
    editor.set_cursor_position(CursorPosition::new(0, 3)); // After "Fir"
    editor.move_down(true); // Selects to (1, 3) - includes "st\nSec"
    editor.move_right(true); // Selects to (1, 4)
    editor.move_right(true); // Selects to (1, 5)

    // Selection from (0, 3) to (1, 5) should be "st\nSecon"
    assert_eq!(editor.get_selected_text(), "st\nSecon");
}

#[test]
fn test_selection_replacement() {
    let mut editor = Editor::new("test", vec!["Hello World".to_string()]);
    editor.set_cursor_position(CursorPosition::new(0, 0));
    for _ in 0..5 {
        editor.move_right(true);
    }
    editor.insert_char('G');
    assert_eq!(editor.get_buffer().get_line(0), Some("G World".to_string()));
    assert_eq!(editor.cursor_position(), CursorPosition::new(0, 1));
}

#[test]
fn test_select_emoji() {
    let mut editor = Editor::new("test", vec!["Hello 😀 World".to_string()]);
    editor.set_cursor_position(CursorPosition::new(0, 6)); // After "Hello "
    editor.move_right(true); // Select the emoji

    // Should have selected just the emoji
    assert_eq!(editor.get_selected_text(), "😀");
    assert_eq!(editor.cursor_position(), CursorPosition::new(0, 7));

    editor.move_right(true); // Select the space after emoji
    assert_eq!(editor.get_selected_text(), "😀 ");
    assert_eq!(editor.cursor_position(), CursorPosition::new(0, 8));
}

// ==================== Goal Column Tests ====================

#[test]
fn test_goal_column_maintains_on_shorter_line() {
    let mut editor = Editor::new(
        "test",
        vec![
            "Long line here".to_string(),
            "Short".to_string(),
            "Another long line".to_string(),
        ],
    );

    editor.set_cursor_position(CursorPosition::new(0, 10));
    editor.move_down(false);
    assert_eq!(editor.cursor_position(), CursorPosition::new(1, 5)); // End of short line

    editor.move_down(false);
    assert_eq!(editor.cursor_position(), CursorPosition::new(2, 10)); // Back to column 10
}

#[test]
fn test_goal_column_resets_on_horizontal_movement() {
    let mut editor = Editor::new(
        "test",
        vec!["Long line here".to_string(), "Short".to_string()],
    );

    editor.set_cursor_position(CursorPosition::new(0, 10));
    editor.move_down(false);
    editor.move_left(false); // Horizontal movement should reset goal column
    editor.move_up(false);
    assert_eq!(editor.cursor_position().col, 4); // Should be at column 4, not 10
}

#[test]
fn test_backspace_at_line_start_merges_lines() {
    let mut editor = Editor::new("test", vec!["First".to_string(), "Second".to_string()]);
    editor.set_cursor_position(CursorPosition::new(1, 0));
    editor.backspace();
    assert_eq!(editor.get_buffer().line_count(), 1);
    assert_eq!(
        editor.get_buffer().get_line(0),
        Some("FirstSecond".to_string())
    );
    // Cursor should be at merge point (0, 5) where the lines were joined
    assert_eq!(editor.cursor_position(), CursorPosition::new(0, 5));
}

#[test]
fn test_delete_at_line_end_merges_lines() {
    let mut editor = Editor::new("test", vec!["First".to_string(), "Second".to_string()]);
    editor.set_cursor_position(CursorPosition::new(0, 5));
    editor.delete();
    assert_eq!(editor.get_buffer().line_count(), 1);
    assert_eq!(
        editor.get_buffer().get_line(0),
        Some("FirstSecond".to_string())
    );
}

#[test]
fn test_insert_newline_splits_line() {
    let mut editor = Editor::new("test", vec!["HelloWorld".to_string()]);
    editor.set_cursor_position(CursorPosition::new(0, 5));
    editor.insert_newline();
    assert_eq!(editor.get_buffer().line_count(), 2);
    assert_eq!(editor.get_buffer().get_line(0), Some("Hello".to_string()));
    assert_eq!(editor.get_buffer().get_line(1), Some("World".to_string()));
}

#[test]
fn test_very_long_line() {
    let long_line = "x".repeat(10000);
    let mut editor = Editor::new("test", vec![long_line.clone()]);

    editor.set_cursor_position(CursorPosition::new(0, 10000));
    assert_eq!(editor.cursor_position(), CursorPosition::new(0, 10000));

    editor.insert_char('!');
    assert_eq!(editor.cursor_position(), CursorPosition::new(0, 10001));
}

#[test]
fn test_tab_character_handling() {
    let mut editor = Editor::new("test", vec![]);
    editor.insert_char('\t');
    assert_eq!(editor.get_buffer().get_line(0), Some("\t".to_string()));
    assert_eq!(editor.cursor_position(), CursorPosition::new(0, 1));
}

#[test]
fn test_windows_line_endings() {
    // Test CRLF handling - buffer treats \n as line separator but preserves \r
    let editor = Editor::new("test", vec!["Line1\r\n".to_string(), "Line2".to_string()]);

    // The buffer splits on \n, so "Line1\r\n" becomes two lines: "Line1\r" and ""
    assert_eq!(editor.get_buffer().line_count(), 3);

    // First line keeps the \r
    assert_eq!(editor.get_buffer().get_line(0), Some("Line1\r".to_string()));

    // Second line is empty (the part after \n)
    assert_eq!(editor.get_buffer().get_line(1), Some("".to_string()));

    // Third line is the second input string
    assert_eq!(editor.get_buffer().get_line(2), Some("Line2".to_string()));
}

#[test]
fn test_undo_after_emoji_insertion() {
    let mut editor = Editor::new("test", vec![]);
    editor.insert_char('A');
    editor.insert_char('😀');
    editor.insert_char('B');

    // If undo is implemented
    // editor.undo();
    // assert_eq!(editor.get_buffer().get_line(0), Some("A😀".to_string()));
}

#[test]
fn test_zero_width_characters() {
    let mut editor = Editor::new("test", vec![]);
    editor.insert_char('a');
    editor.insert_char('\u{200B}'); // Zero-width space
    editor.insert_char('b');

    // Should have all three characters in order
    let line = editor.get_buffer().get_line(0).unwrap();
    assert_eq!(line, "a\u{200B}b");

    // Cursor should be after all 3 characters
    assert_eq!(editor.cursor_position(), CursorPosition::new(0, 3));
}

#[test]
fn test_surrogate_pairs() {
    let mut editor = Editor::new("test", vec![]);
    // Mathematical bold capital A (U+1D400)
    let surrogate = "𝐀";
    for ch in surrogate.chars() {
        editor.insert_char(ch);
    }
    assert_eq!(editor.get_buffer().get_line(0), Some(surrogate.to_string()));
}

#[test]
fn test_right_to_left_text() {
    let mut editor = Editor::new("test", vec![]);
    let hebrew = "שלום";
    for ch in hebrew.chars() {
        editor.insert_char(ch);
    }
    assert_eq!(editor.get_buffer().get_line(0), Some(hebrew.to_string()));
}

#[gpui::test]
async fn test_cursor_position_after_paste(cx: &mut gpui::TestAppContext) {
    use gpui::ClipboardItem;

    // Create an editor with initial content
    let mut editor = Editor::new("test", vec!["Hello".to_string()]);

    editor.set_cursor_position(CursorPosition::new(0, 5));

    // Write to clipboard
    cx.write_to_clipboard(ClipboardItem::new_string(" World".to_string()));

    // Simulate paste by reading from clipboard and inserting
    let clipboard_content = cx.read_from_clipboard();
    if let Some(item) = clipboard_content {
        if let Some(text) = item.text() {
            for ch in text.chars() {
                editor.insert_char(ch);
            }
        }
    }

    assert_eq!(
        editor.get_buffer().get_line(0),
        Some("Hello World".to_string())
    );
    assert_eq!(editor.cursor_position(), CursorPosition::new(0, 11));
}

#[gpui::test]
async fn test_copy_and_paste_selection(cx: &mut gpui::TestAppContext) {
    use gpui::ClipboardItem;

    // Create an editor with initial content
    let mut editor = Editor::new("test", vec!["Hello World".to_string()]);

    // Select "World"
    editor.set_cursor_position(CursorPosition::new(0, 6));
    for _ in 0..5 {
        editor.move_right(true);
    }

    // Copy the selection to clipboard
    let selected_text = editor.get_selected_text();
    cx.write_to_clipboard(ClipboardItem::new_string(selected_text));

    // Clear selection and move cursor to beginning
    editor.clear_selection();
    editor.set_cursor_position(CursorPosition::new(0, 0));

    // Read from clipboard and insert
    let clipboard_content = cx.read_from_clipboard();
    if let Some(item) = clipboard_content {
        if let Some(text) = item.text() {
            for ch in text.chars() {
                editor.insert_char(ch);
            }
        }
    }

    // Should have "World" pasted at the beginning
    assert_eq!(
        editor.get_buffer().get_line(0),
        Some("WorldHello World".to_string())
    );
    assert_eq!(editor.cursor_position(), CursorPosition::new(0, 5));
}

#[gpui::test]
async fn test_cut_and_paste(cx: &mut gpui::TestAppContext) {
    use gpui::ClipboardItem;

    // Create an editor with initial content
    let mut editor = Editor::new("test", vec!["Hello World".to_string()]);

    // Select "Hello "
    editor.set_cursor_position(CursorPosition::new(0, 0));
    for _ in 0..6 {
        editor.move_right(true);
    }

    // Cut the selection (copy to clipboard and delete)
    let selected_text = editor.get_selected_text();
    cx.write_to_clipboard(ClipboardItem::new_string(selected_text));
    editor.delete_selection();

    // Should have only "World" left
    assert_eq!(editor.get_buffer().get_line(0), Some("World".to_string()));

    // Move to end and paste
    editor.set_cursor_position(CursorPosition::new(0, 5));

    // Read from clipboard and insert
    let clipboard_content = cx.read_from_clipboard();
    if let Some(item) = clipboard_content {
        if let Some(text) = item.text() {
            for ch in text.chars() {
                editor.insert_char(ch);
            }
        }
    }

    // Should have "WorldHello "
    assert_eq!(
        editor.get_buffer().get_line(0),
        Some("WorldHello ".to_string())
    );
    assert_eq!(editor.cursor_position(), CursorPosition::new(0, 11));
}

#[test]
fn test_selection_with_zero_width_joiner() {
    let mut editor = Editor::new("test", vec![]);
    // Emoji with ZWJ: man (👨) + ZWJ + computer (💻) = man technologist
    let emoji = "👨‍💻"; // Man technologist (with ZWJ)
    for ch in emoji.chars() {
        editor.insert_char(ch);
    }

    // Should have inserted all characters (man, ZWJ, computer)
    let line = editor.get_buffer().get_line(0).unwrap();
    assert_eq!(line, emoji);

    // Without grapheme clustering, cursor should be at position 3 (3 chars: 👨 + ZWJ + 💻)
    assert_eq!(editor.cursor_position(), CursorPosition::new(0, 3));

    // Select all should select the entire emoji sequence
    editor.select_all();
    let selected = editor.get_selected_text();
    assert_eq!(selected, emoji);
}

#[test]
fn test_syntax_highlighting_language_detection() {
    let _editor = Editor::new("test.rs", vec!["fn main() {}".to_string()]);
    // Test that language is detected from file extension
    // assert_eq!(editor.get_language(), Some("rust"));
}

#[test]
fn test_syntax_highlighting_state_after_edit() {
    let mut editor = Editor::new("test.rs", vec!["fn main() {".to_string()]);
    editor.set_cursor_position(CursorPosition::new(0, 11));
    editor.insert_char('}');
    // Verify highlighting state is updated
}

#[test]
fn test_navigate_through_empty_lines() {
    let mut editor = Editor::new("test", vec!["".to_string(), "".to_string(), "".to_string()]);
    assert_eq!(editor.get_buffer().line_count(), 3);

    editor.move_down(false);
    assert_eq!(editor.cursor_position(), CursorPosition::new(1, 0));

    editor.move_down(false);
    assert_eq!(editor.cursor_position(), CursorPosition::new(2, 0));
}

#[test]
fn test_delete_empty_line() {
    let mut editor = Editor::new("test", vec!["".to_string(), "".to_string(), "".to_string()]);
    editor.set_cursor_position(CursorPosition::new(1, 0));
    editor.backspace();
    assert_eq!(editor.get_buffer().line_count(), 2);
}

#[test]
fn test_cursor_clamping_to_valid_position() {
    let mut editor = Editor::new("test", vec!["Hello".to_string()]);
    editor.set_cursor_position(CursorPosition::new(100, 100));
    // Cursor should be clamped to the last valid position: end of "Hello" (row 0, col 5)
    let pos = editor.cursor_position();
    assert_eq!(pos, CursorPosition::new(0, 5));
}

// Note: This test is commented out because CursorPosition uses usize for row/col,
// which cannot represent negative values. The Rust type system prevents this issue.
// If the API changes to accept signed integers, this test should verify clamping to (0, 0).
// #[test]
// fn test_cursor_clamping_negative_values() {
//     let mut editor = Editor::new("test", vec!["Hello".to_string()]);
//     // If this were possible with signed integers:
//     // editor.set_cursor_position(CursorPosition::new(-1, -1));
//     // assert_eq!(editor.cursor_position(), CursorPosition::new(0, 0));
// }

#[test]
fn test_consecutive_emoji_deletion() {
    let mut editor = Editor::new("test", vec!["👍👎👌".to_string()]);
    editor.set_cursor_position(CursorPosition::new(0, 3)); // After all three emoji

    // Delete the last emoji (👌)
    editor.backspace();
    let line = editor.get_buffer().get_line(0).unwrap();
    assert_eq!(line, "👍👎");
    assert_eq!(editor.cursor_position(), CursorPosition::new(0, 2));
}

#[test]
fn test_mixed_width_characters() {
    let mut editor = Editor::new("test", vec![]);
    // Mix of ASCII, full-width, and emoji
    editor.insert_char('A');
    editor.insert_char('Ａ'); // Full-width A
    editor.insert_char('😀');
    editor.insert_char('B');

    let line = editor.get_buffer().get_line(0).unwrap();
    // Should have all characters in order
    assert_eq!(line, "AＡ😀B");
    // Cursor should be after all 4 characters
    assert_eq!(editor.cursor_position(), CursorPosition::new(0, 4));
}

#[test]
fn test_consecutive_newlines() {
    let mut editor = Editor::new("test", vec![]);
    editor.insert_newline();
    editor.insert_newline();
    editor.insert_newline();

    assert_eq!(editor.get_buffer().line_count(), 4);
    assert_eq!(editor.cursor_position(), CursorPosition::new(3, 0));
}

#[test]
fn test_rapid_insertions() {
    let mut editor = Editor::new("test", vec![]);

    // Simulate rapid typing
    for c in "The quick brown fox jumps over the lazy dog".chars() {
        editor.insert_char(c);
    }

    let line = editor.get_buffer().get_line(0).unwrap();
    assert_eq!(line, "The quick brown fox jumps over the lazy dog");
}

#[test]
fn test_alternating_insert_delete() {
    let mut editor = Editor::new("test", vec![]);

    editor.insert_char('A');
    editor.insert_char('B');
    editor.backspace();
    editor.insert_char('C');
    editor.insert_char('D');
    editor.backspace();

    assert_eq!(editor.get_buffer().get_line(0), Some("AC".to_string()));
}

#[test]
fn test_selection_past_end_of_line() {
    let mut editor = Editor::new("test", vec!["Hello".to_string()]);
    editor.set_cursor_position(CursorPosition::new(0, 0));

    // Try to select past end of line
    for _ in 0..10 {
        editor.move_right(true);
    }

    let selected = editor.get_selected_text();
    assert_eq!(selected, "Hello");
}

#[test]
fn test_delete_across_empty_lines() {
    let mut editor = Editor::new(
        "test",
        vec![
            "First".to_string(),
            "".to_string(),
            "".to_string(),
            "Last".to_string(),
        ],
    );

    // Delete from end of first line
    editor.set_cursor_position(CursorPosition::new(0, 5));
    editor.delete();
    editor.delete();
    editor.delete();

    assert_eq!(editor.get_buffer().line_count(), 1);
    assert_eq!(
        editor.get_buffer().get_line(0),
        Some("FirstLast".to_string())
    );
}

#[test]
fn test_invisible_characters() {
    let mut editor = Editor::new("test", vec![]);

    // Insert various invisible characters
    editor.insert_char('\u{00A0}'); // Non-breaking space
    editor.insert_char('\u{2000}'); // En quad
    editor.insert_char('\u{2001}'); // Em quad
    editor.insert_char('\u{200B}'); // Zero-width space
    editor.insert_char('\u{FEFF}'); // Zero-width no-break space

    let line = editor.get_buffer().get_line(0).unwrap();
    assert_eq!(line.chars().count(), 5);
}

#[test]
fn test_grapheme_cluster_navigation() {
    let mut editor = Editor::new("test", vec![]);

    // Insert grapheme clusters
    let clusters = "é"; // e + combining accent
    for ch in clusters.chars() {
        editor.insert_char(ch);
    }
    editor.insert_char(' ');

    let flag = "🇺🇸"; // Regional indicator symbols
    for ch in flag.chars() {
        editor.insert_char(ch);
    }

    let line = editor.get_buffer().get_line(0).unwrap();
    assert!(!line.is_empty());
}

#[test]
fn test_selection_collapse_on_typing() {
    let mut editor = Editor::new("test", vec!["Hello World".to_string()]);

    // Create selection
    editor.set_cursor_position(CursorPosition::new(0, 0));
    for _ in 0..5 {
        editor.move_right(true);
    }
    assert!(editor.has_selection());

    // Type should replace selection
    editor.insert_char('X');
    assert!(!editor.has_selection());
    assert_eq!(editor.get_buffer().get_line(0), Some("X World".to_string()));
}

#[test]
fn test_backspace_multiple_graphemes() {
    let mut editor = Editor::new("test", vec![]);

    // Insert multiple complex characters
    editor.insert_char('A');
    editor.insert_char('👨');
    editor.insert_char('\u{200D}'); // ZWJ
    editor.insert_char('💻'); // Forms "man technologist"
    editor.insert_char('B');

    // Backspace from end
    editor.backspace();
    let line = editor.get_buffer().get_line(0).unwrap();
    assert!(line.contains('A'));
}

#[test]
fn test_cursor_at_grapheme_boundaries() {
    let mut editor = Editor::new("test", vec!["A👨‍👩‍👦B".to_string()]);

    // Navigate through complex emoji
    editor.set_cursor_position(CursorPosition::new(0, 0));
    editor.move_right(false); // Past 'A'
    let pos1 = editor.cursor_position().col;

    editor.move_right(false); // Past emoji
    let pos2 = editor.cursor_position().col;

    // Should have moved past the complex emoji
    assert!(pos2 > pos1);
}

#[test]
fn test_line_wrapping_behavior() {
    let very_long_word = "a".repeat(200);
    let mut editor = Editor::new("test", vec![very_long_word.clone()]);

    // Navigate to middle
    editor.set_cursor_position(CursorPosition::new(0, 100));

    // Insert newline to break the line
    editor.insert_newline();

    assert_eq!(editor.get_buffer().line_count(), 2);
    assert_eq!(editor.get_buffer().get_line(0), Some("a".repeat(100)));
    assert_eq!(editor.get_buffer().get_line(1), Some("a".repeat(100)));
}

#[test]
fn test_selection_with_mixed_line_endings() {
    // Test selection across lines with different ending styles
    let mut editor = Editor::new("test", vec!["Line 1".to_string(), "Line 2".to_string()]);

    editor.set_cursor_position(CursorPosition::new(0, 4));
    editor.move_down(true);
    editor.move_right(true);
    editor.move_right(true);

    // Selection from (0, 4) to (1, 6) should be " 1\nLine 2"
    let selected = editor.get_selected_text();
    assert_eq!(selected, " 1\nLine 2");
}

#[test]
fn test_empty_selection() {
    let mut editor = Editor::new("test", vec!["Hello".to_string()]);

    // Create zero-width selection
    editor.set_cursor_position(CursorPosition::new(0, 2)); // After "He"
    editor.move_right(true); // Select "l" (position 2 to 3)
    editor.move_left(true); // Move back to position 2, collapsing selection

    // Selection should be empty after moving back to start position
    let selected = editor.get_selected_text();
    assert_eq!(selected, "");
    assert_eq!(editor.cursor_position(), CursorPosition::new(0, 2));
}

#[test]
fn test_normalization_inconsistencies() {
    let mut editor = Editor::new("test", vec![]);

    // Insert pre-composed character
    editor.insert_char('é'); // U+00E9
    editor.insert_char(' ');

    // Insert decomposed equivalent
    editor.insert_char('e');
    editor.insert_char('\u{0301}'); // combining acute

    let line = editor.get_buffer().get_line(0).unwrap();
    // Without normalization, both forms are preserved exactly as entered
    assert_eq!(line, "é e\u{0301}");

    // Verify they are indeed different representations
    let chars: Vec<char> = line.chars().collect();
    assert_eq!(chars.len(), 4); // é(1) + space(1) + e(1) + combining(1) = 4
    assert_eq!(chars[0], 'é'); // Pre-composed
    assert_eq!(chars[2], 'e'); // Base character
    assert_eq!(chars[3], '\u{0301}'); // Combining accent
}

#[test]
fn test_cursor_position_after_bulk_delete() {
    let mut editor = Editor::new("test", vec!["1234567890".to_string()]);

    // Select middle portion (positions 3-7, characters "4567")
    editor.set_cursor_position(CursorPosition::new(0, 3));
    for _ in 0..4 {
        editor.move_right(true);
    }

    // Delete selection should remove "4567"
    editor.delete();

    // Should have removed "4567", leaving "123890"
    let result = editor.get_buffer().get_line(0).unwrap();
    assert_eq!(result, "123890");

    // Cursor should be at the deletion point (position 3)
    assert_eq!(editor.cursor_position(), CursorPosition::new(0, 3));
}

// #[test]
// fn test_repeated_undo_redo() {
//     // Test that repeated undo/redo operations don't corrupt state
//     // This is a placeholder since undo might not be implemented
//     let mut editor = Editor::new("test", vec![]);

//     editor.insert_char('A');
//     editor.insert_char('B');
//     editor.insert_char('C');

//     // If undo were implemented:
//     // editor.undo();
//     // editor.undo();
//     // editor.redo();
//     // editor.undo();

//     // For now, just verify current state
//     assert_eq!(editor.get_buffer().get_line(0), Some("ABC".to_string()));
// }

#[test]
fn test_control_characters() {
    let mut editor = Editor::new("test", vec![]);

    // Insert various control characters
    editor.insert_char('\x00'); // NULL
    editor.insert_char('\x01'); // SOH
    editor.insert_char('\x1F'); // Unit separator
    editor.insert_char('\x7F'); // DEL

    let line = editor.get_buffer().get_line(0).unwrap();
    // All control characters should be preserved
    assert_eq!(line.len(), 4);
    assert_eq!(line.chars().next(), Some('\x00'));
    assert_eq!(line.chars().nth(1), Some('\x01'));
    assert_eq!(line.chars().nth(2), Some('\x1F'));
    assert_eq!(line.chars().nth(3), Some('\x7F'));

    // Cursor should be after all 4 characters
    assert_eq!(editor.cursor_position(), CursorPosition::new(0, 4));
}

/// Paint every line through `Editor::highlight_line`, in order, exactly as
/// `EditorElement::paint_line` does, and return the colour of each *painted*
/// byte.
///
/// It also asserts the invariant the paint loop depends on: the runs must sum
/// to exactly the display line, because that same string is what
/// `shape_line` is handed. `Editor::highlight_line` parses the line with the
/// newline the grammar wants and trims back to here, and this is what catches
/// it if the trim is ever dropped.
#[cfg(test)]
fn painted_colors(editor: &mut Editor, lines: &[String]) -> Vec<Vec<Option<gpui::Hsla>>> {
    lines
        .iter()
        .enumerate()
        .map(|(index, line)| {
            let runs = editor.highlight_line(line, index, "Courier".into(), 14.0);
            let painted: usize = runs.iter().map(|run| run.len).sum();
            assert_eq!(
                painted,
                line.len(),
                "line {index} ({line:?}) got runs covering {painted} bytes, but \
                 `shape_line` is given the {} bytes of the line itself",
                line.len(),
            );
            runs.iter()
                .flat_map(|run| std::iter::repeat_n(Some(run.color), run.len))
                .collect()
        })
        .collect()
}

/// The same per-byte colours from the stateless whole-document pass, split back
/// up by display line.
///
/// This is the oracle: same grammar, same theme, same `HighlightIterator`,
/// differing only in how the cross-line state is carried. It is only a valid
/// comparison because both sides now feed syntect lines that carry their `\n`.
#[cfg(test)]
fn block_colors_per_line(text: &str, language: &str) -> Vec<Vec<Option<gpui::Hsla>>> {
    let highlighter = SyntaxHighlighter::new();
    let mut colors: Vec<Option<gpui::Hsla>> = vec![None; text.len()];
    for (range, style) in highlighter.highlight_block(text, language) {
        for slot in &mut colors[range] {
            *slot = style.color;
        }
    }

    let mut per_line = Vec::new();
    let mut offset = 0;
    for line in text.split('\n') {
        per_line.push(colors[offset..offset + line.len()].to_vec());
        offset += line.len() + 1; // the separator this line was split on
    }
    per_line
}

/// The #135 regression through the path the paint loop actually uses:
/// `Editor::highlight_line`, which runs `ensure_parse_states` first. The second
/// parse used to overwrite the correct state that call had just cached, so the
/// next `ensure_parse_states` adopted the corrupt entry as its starting point.
///
/// Now that the editor feeds syntect the same newline-carrying lines
/// `highlight_block` gets, this can assert the strong form the issue asked for:
/// the painted colours must match the stateless block pass byte for byte. The
/// structural assertions are kept underneath as documentation of what that
/// comparison stands for.
#[test]
fn test_line_after_block_comment_is_highlighted_as_code() {
    let lines: Vec<String> = [
        "const before = 1;",
        "/* opening a block comment",
        "still inside the comment */",
        "function after() { return 2; }",
    ]
    .iter()
    .map(|line| line.to_string())
    .collect();

    let mut editor = Editor::new("test_block_comment", lines.clone());
    editor.set_language("JavaScript".to_string());

    let painted = painted_colors(&mut editor, &lines);
    assert_eq!(
        painted,
        block_colors_per_line(&lines.join("\n"), "JavaScript"),
        "painting line by line disagreed with the stateless block pass"
    );

    let comment_color = painted[1][0];
    let code = &painted[3];

    assert!(
        code.iter().all(|color| *color != comment_color),
        "the line after `*/` is still painted in the comment colour"
    );

    let distinct: std::collections::HashSet<String> =
        code.iter().map(|color| format!("{color:?}")).collect();
    assert!(
        distinct.len() >= 2,
        "the line after `*/` collapsed to {} colour(s); `function` and `after` \
         should not share the plain-text colour",
        distinct.len()
    );
}

/// #142: the syntax set is `load_defaults_newlines`, so a `//` comment only
/// closes when the parser is shown the newline that closes it. Stripping the
/// separator left the comment open forever and painted every following line as
/// comment.
///
/// The language matters. Rust's `//` does *not* diverge in syntect 5.3's
/// default newline grammars — which is why the existing `("Rust", "// just a
/// line comment\n…")` fixture never caught this. JavaScript and C do.
#[test]
fn test_line_comment_terminates_at_end_of_line() {
    let lines: Vec<String> = [
        "// a line comment",
        "function after() { return 2; }",
        "const tail = 3;",
    ]
    .iter()
    .map(|line| line.to_string())
    .collect();

    let mut editor = Editor::new("test_line_comment", lines.clone());
    editor.set_language("JavaScript".to_string());

    let painted = painted_colors(&mut editor, &lines);
    assert_eq!(
        painted,
        block_colors_per_line(&lines.join("\n"), "JavaScript"),
        "the `//` comment leaked past its newline"
    );

    let comment_color = painted[0][0];
    assert!(
        painted[1].iter().all(|color| *color != comment_color),
        "the line after a `//` comment is still painted in the comment colour"
    );
}

/// The other shape of #142: a string the grammar closes at end of line. Without
/// the newline the Python grammar never terminates it, so the following line is
/// painted as string.
#[test]
fn test_unterminated_python_string_ends_at_its_newline() {
    let lines: Vec<String> = ["x = 'unterminated", "y = 2", "z = 3"]
        .iter()
        .map(|line| line.to_string())
        .collect();

    let mut editor = Editor::new("test_unterminated_string", lines.clone());
    editor.set_language("Python".to_string());

    let painted = painted_colors(&mut editor, &lines);
    assert_eq!(
        painted,
        block_colors_per_line(&lines.join("\n"), "Python"),
        "the unterminated string ran on past its newline"
    );

    let string_color = painted[0][4]; // the opening quote
    assert!(
        painted[1].iter().all(|color| *color != string_color),
        "the line after an unterminated string is still painted as string"
    );
}

/// The buffer decides which lines are newline-terminated, and the last one is
/// not — matching what `LinesWithEndings` gives `highlight_block` for the same
/// text. This guards the run-length invariant the fix could have broken: an
/// empty line must still come back with exactly one zero-length run, not an
/// empty `Vec`.
#[test]
fn test_last_line_is_not_given_a_separator_it_does_not_have() {
    let lines: Vec<String> = ["fn main() {}", "", "// trailing"]
        .iter()
        .map(|line| line.to_string())
        .collect();

    let mut editor = Editor::new("test_last_line", lines.clone());
    editor.set_language("Rust".to_string());

    // `painted_colors` asserts runs sum to the line on every line, including
    // the unterminated last one.
    let painted = painted_colors(&mut editor, &lines);
    assert_eq!(
        painted,
        block_colors_per_line(&lines.join("\n"), "Rust"),
        "the last line was parsed as if it had a separator"
    );

    let empty_line_runs = editor.highlight_line("", 1, "Courier".into(), 14.0);
    assert_eq!(
        empty_line_runs.len(),
        1,
        "an empty line must keep exactly one zero-length run"
    );
    assert_eq!(empty_line_runs[0].len, 0);
}
