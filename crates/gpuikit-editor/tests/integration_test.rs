use gpui_editor::{CursorPosition, Editor, GapBuffer, TextBuffer};

#[test]
fn test_editor_with_gap_buffer() {
    // Create an editor with some initial lines
    let initial_lines = vec![
        "fn main() {".to_string(),
        "    println!(\"Hello, world!\");".to_string(),
        "}".to_string(),
    ];

    let mut editor = Editor::new("test_editor", initial_lines.clone());

    // Verify initial state
    assert_eq!(editor.get_buffer().line_count(), 3);
    assert_eq!(
        editor.get_buffer().get_line(0),
        Some("fn main() {".to_string())
    );
    assert_eq!(
        editor.get_buffer().get_line(1),
        Some("    println!(\"Hello, world!\");".to_string())
    );
    assert_eq!(editor.get_buffer().get_line(2), Some("}".to_string()));

    // Test cursor movement
    editor.move_right(false);
    assert_eq!(editor.cursor_position().col, 1);

    editor.move_down(false);
    assert_eq!(editor.cursor_position().row, 1);
    assert_eq!(editor.cursor_position().col, 1);

    // Test character insertion
    editor.insert_char('x');
    assert_eq!(
        editor.get_buffer().get_line(1),
        Some(" x   println!(\"Hello, world!\");".to_string())
    );

    // Test backspace
    editor.backspace();
    assert_eq!(
        editor.get_buffer().get_line(1),
        Some("    println!(\"Hello, world!\");".to_string())
    );

    // Test newline insertion
    editor.set_cursor_position(CursorPosition::new(0, 11)); // End of first line
    editor.insert_newline();
    assert_eq!(editor.get_buffer().line_count(), 4);
    assert_eq!(
        editor.get_buffer().get_line(0),
        Some("fn main() {".to_string())
    );
    assert_eq!(editor.get_buffer().get_line(1), Some("".to_string()));

    // Test delete forward
    editor.set_cursor_position(CursorPosition::new(1, 0));
    editor.delete();
    assert_eq!(editor.get_buffer().line_count(), 3);
    assert_eq!(
        editor.get_buffer().get_line(1),
        Some("    println!(\"Hello, world!\");".to_string())
    );
}

#[test]
fn test_gap_buffer_directly() {
    let mut buffer = GapBuffer::from_text("Hello\nWorld");

    // Test basic operations
    assert_eq!(buffer.line_count(), 2);
    assert_eq!(buffer.get_line(0), Some("Hello".to_string()));
    assert_eq!(buffer.get_line(1), Some("World".to_string()));

    // Insert at specific row/col
    buffer.insert_at(0, 5, "!");
    assert_eq!(buffer.get_line(0), Some("Hello!".to_string()));

    // Delete at specific position
    buffer.delete_at(0, 5);
    assert_eq!(buffer.get_line(0), Some("Hello".to_string()));

    // Test newline handling
    buffer.insert_at(0, 5, "\nNew Line");
    assert_eq!(buffer.line_count(), 3);
    assert_eq!(buffer.get_line(0), Some("Hello".to_string()));
    assert_eq!(buffer.get_line(1), Some("New Line".to_string()));
    assert_eq!(buffer.get_line(2), Some("World".to_string()));
}

#[test]
fn test_editor_selection_with_gap_buffer() {
    let lines = vec![
        "Line 1".to_string(),
        "Line 2".to_string(),
        "Line 3".to_string(),
    ];

    let mut editor = Editor::new("test", lines);

    // Select all
    editor.select_all();
    assert!(editor.has_selection());

    let selected = editor.get_selected_text();
    assert_eq!(selected, "Line 1\nLine 2\nLine 3");

    // Delete selection
    editor.delete_selection();
    assert_eq!(editor.get_buffer().line_count(), 1);
    assert_eq!(editor.get_buffer().get_line(0), Some("".to_string()));

    // Type new text
    editor.insert_char('N');
    editor.insert_char('e');
    editor.insert_char('w');
    assert_eq!(editor.get_buffer().get_line(0), Some("New".to_string()));
}

#[test]
fn test_large_text_editing() {
    // Create a large document
    let mut lines = Vec::new();
    for i in 0..100 {
        lines.push(format!(
            "Line {}: This is a test line with some content.",
            i
        ));
    }

    let mut editor = Editor::new("large", lines);

    // Verify initial state
    assert_eq!(editor.get_buffer().line_count(), 100);

    // Move to middle of document
    editor.set_cursor_position(CursorPosition::new(50, 0));
    editor.insert_char('X');

    assert_eq!(
        editor.get_buffer().get_line(50),
        Some("XLine 50: This is a test line with some content.".to_string())
    );

    // Move to end and add new line
    editor.set_cursor_position(CursorPosition::new(99, editor.get_buffer().line_len(99)));
    editor.insert_newline();
    editor.insert_char('E');
    editor.insert_char('N');
    editor.insert_char('D');

    assert_eq!(editor.get_buffer().line_count(), 101);
    assert_eq!(editor.get_buffer().get_line(100), Some("END".to_string()));
}

#[test]
fn test_unicode_handling() {
    let lines = vec![
        "Hello 世界".to_string(),
        "🎨 Emoji test 🎭".to_string(),
        "Café ☕".to_string(),
    ];

    let mut editor = Editor::new("unicode", lines);

    // Test that unicode is preserved
    assert_eq!(
        editor.get_buffer().get_line(0),
        Some("Hello 世界".to_string())
    );
    assert_eq!(
        editor.get_buffer().get_line(1),
        Some("🎨 Emoji test 🎭".to_string())
    );

    // Insert unicode characters - insert after all characters
    editor.set_cursor_position(CursorPosition::new(2, 6));
    editor.insert_char('🍰');

    assert_eq!(
        editor.get_buffer().get_line(2),
        Some("Café ☕🍰".to_string())
    );
}
