#[allow(unused)]
use super::super::*;

#[test]
fn test_insert_basic_emoji() {
    let mut editor = Editor::new("test", vec![]);
    editor.insert_char('😀');
    assert_eq!(editor.get_buffer().get_line(0), Some("😀".to_string()));
    // Emoji takes up one logical position despite being multiple bytes
    assert_eq!(editor.cursor_position(), CursorPosition::new(0, 1));
}

#[test]
fn test_insert_multiple_emoji() {
    let mut editor = Editor::new("test", vec![]);
    editor.insert_char('👍');
    editor.insert_char('🎉');
    editor.insert_char('🚀');
    assert_eq!(editor.get_buffer().get_line(0), Some("👍🎉🚀".to_string()));
    assert_eq!(editor.cursor_position(), CursorPosition::new(0, 3));
}

#[test]
fn test_emoji_with_skin_tone_modifier() {
    let mut editor = Editor::new("test", vec![]);
    // This is a combined character: waving hand + skin tone modifier
    let emoji_with_modifier = "👋🏽";
    for ch in emoji_with_modifier.chars() {
        editor.insert_char(ch);
    }
    assert_eq!(
        editor.get_buffer().get_line(0),
        Some(emoji_with_modifier.to_string())
    );
}

#[test]
fn test_emoji_zwj_sequence() {
    let mut editor = Editor::new("test", vec![]);
    // Family emoji: man + ZWJ + woman + ZWJ + boy
    let family_emoji = "👨‍👩‍👦";
    for ch in family_emoji.chars() {
        editor.insert_char(ch);
    }
    assert_eq!(
        editor.get_buffer().get_line(0),
        Some(family_emoji.to_string())
    );
}

#[test]
fn test_delete_emoji() {
    let mut editor = Editor::new("test", vec!["Hello 😀 World".to_string()]);

    // Position 8 is at 'W', backspace should delete the space before it
    editor.set_cursor_position(CursorPosition::new(0, 8));
    editor.backspace();

    assert_eq!(
        editor.get_buffer().get_line(0),
        Some("Hello 😀World".to_string())
    );
    assert_eq!(editor.cursor_position(), CursorPosition::new(0, 7));

    // Now test deleting the emoji itself - position 7 is right after the emoji
    editor.set_cursor_position(CursorPosition::new(0, 7));
    editor.backspace();

    // Should delete the entire emoji as one grapheme cluster
    assert_eq!(
        editor.get_buffer().get_line(0),
        Some("Hello World".to_string())
    );
    assert_eq!(editor.cursor_position(), CursorPosition::new(0, 6));
}

#[test]
fn test_cursor_movement_over_emoji() {
    // Test 1: Simple emoji - should work with current implementation
    let mut editor = Editor::new("test", vec!["A😀B".to_string()]);
    editor.set_cursor_position(CursorPosition::new(0, 0));

    editor.move_right(false);
    assert_eq!(
        editor.cursor_position(),
        CursorPosition::new(0, 1),
        "Should be after 'A'"
    );

    editor.move_right(false);
    assert_eq!(
        editor.cursor_position(),
        CursorPosition::new(0, 2),
        "Should be after emoji"
    );

    editor.move_right(false);
    assert_eq!(
        editor.cursor_position(),
        CursorPosition::new(0, 3),
        "Should be after 'B'"
    );

    // Test 2: Emoji with skin tone modifier - should treat as single grapheme cluster
    let mut editor2 = Editor::new("test", vec!["A👋🏽B".to_string()]);
    editor2.set_cursor_position(CursorPosition::new(0, 0));

    editor2.move_right(false);
    assert_eq!(
        editor2.cursor_position(),
        CursorPosition::new(0, 1),
        "Should be after 'A'"
    );

    editor2.move_right(false);
    // Without grapheme clustering, this should move through each char separately
    // So we should be at position 2 (after 👋)
    assert_eq!(
        editor2.cursor_position(),
        CursorPosition::new(0, 2),
        "Should be after waving hand emoji (before skin tone modifier)"
    );

    editor2.move_right(false);
    // Now should be at position 3 (after skin tone modifier)
    assert_eq!(
        editor2.cursor_position(),
        CursorPosition::new(0, 3),
        "Should be after skin tone modifier"
    );

    editor2.move_right(false);
    assert_eq!(
        editor2.cursor_position(),
        CursorPosition::new(0, 4),
        "Should be after 'B'"
    );

    // Test 3: ZWJ sequence - should treat as single grapheme cluster
    let mut editor3 = Editor::new("test", vec!["A👨‍👩‍👦B".to_string()]);
    editor3.set_cursor_position(CursorPosition::new(0, 0));

    editor3.move_right(false);
    assert_eq!(
        editor3.cursor_position(),
        CursorPosition::new(0, 1),
        "Should be after 'A'"
    );

    editor3.move_right(false);
    // Without grapheme clustering, should be at position 2 (after 👨)
    assert_eq!(
        editor3.cursor_position(),
        CursorPosition::new(0, 2),
        "Should be after man emoji"
    );

    // Need to move through ZWJ, woman, ZWJ, boy to get to B
    editor3.move_right(false); // After ZWJ
    assert_eq!(editor3.cursor_position(), CursorPosition::new(0, 3));

    editor3.move_right(false); // After 👩
    assert_eq!(editor3.cursor_position(), CursorPosition::new(0, 4));

    editor3.move_right(false); // After ZWJ
    assert_eq!(editor3.cursor_position(), CursorPosition::new(0, 5));

    editor3.move_right(false); // After 👦
    assert_eq!(editor3.cursor_position(), CursorPosition::new(0, 6));

    editor3.move_right(false);
    assert_eq!(
        editor3.cursor_position(),
        CursorPosition::new(0, 7),
        "Should be after 'B'"
    );

    // Test 4: Combining diacritics - should treat as single grapheme cluster
    let mut editor4 = Editor::new("test", vec!["Ae\u{0301}B".to_string()]); // e + combining acute
    editor4.set_cursor_position(CursorPosition::new(0, 0));

    editor4.move_right(false);
    assert_eq!(
        editor4.cursor_position(),
        CursorPosition::new(0, 1),
        "Should be after 'A'"
    );

    editor4.move_right(false);
    // Without grapheme clustering, should be at position 2 (after 'e')
    assert_eq!(
        editor4.cursor_position(),
        CursorPosition::new(0, 2),
        "Should be after 'e'"
    );

    editor4.move_right(false);
    // Now should be at position 3 (after combining accent)
    assert_eq!(
        editor4.cursor_position(),
        CursorPosition::new(0, 3),
        "Should be after combining accent"
    );

    editor4.move_right(false);
    assert_eq!(
        editor4.cursor_position(),
        CursorPosition::new(0, 4),
        "Should be after 'B'"
    );
}

#[test]
fn test_mixed_ascii_and_emoji() {
    let mut editor = Editor::new("test", vec![]);
    editor.insert_char('H');
    editor.insert_char('i');
    editor.insert_char('👋');
    editor.insert_char('!');
    assert_eq!(editor.get_buffer().get_line(0), Some("Hi👋!".to_string()));
    assert_eq!(editor.cursor_position(), CursorPosition::new(0, 4));
}
