# gpuikit-editor testing todos

## Missing Features

- Undo/redo system
- Proper grapheme cluster support (currently treats each Unicode codepoint as separate, doesn't group multi-codepoint sequences like emoji with modifiers, ZWJ sequences, or combining diacritics)
- Unicode normalization (required for combining diacritics)
- CRLF line ending support
- Consistent character vs byte position handling (mostly fixed, but may have edge cases)

## Broken Behaviors

### Emoji/Unicode

- Emoji cursor movement doesn't respect grapheme boundaries (moves through individual codepoints)
- ZWJ sequences not handled as single units (treated as separate codepoints)
- Skin tone modifiers treated as separate characters (not grouped with base emoji)
- No Unicode normalization (é vs e + combining accent stored differently)
- Mixed width characters (full-width) positioning may have issues (needs verification)
- Combining diacritics not composed (stored as separate codepoints)

### Selection

- Delete with selection might only delete single character
- Selection across emoji boundaries inconsistent
- Empty selection behavior undefined
- Selection with complex emoji (ZWJ) broken

### Text Operations

- Control characters handling undefined (filtered vs preserved)
- Consecutive emoji deletion unpredictable
- Bulk delete with selection has wrong result

## Tests Needing Fixes

- `test_combining_diacritics` - COMMENTED OUT: Requires Unicode normalization feature
- `test_select_emoji` - Should select exact emoji boundaries
- `test_windows_line_endings` - Should handle CRLF properly
- `test_selection_with_zero_width_joiner` - Should handle ZWJ sequences
- `test_mixed_width_characters` - Should handle positioning correctly
- `test_select_across_lines` - Should return exact selection
- `test_selection_with_mixed_line_endings` - Should return exact selection
- `test_empty_selection` - Should have defined behavior
- `test_normalization_inconsistencies` - Should normalize consistently
- `test_repeated_undo_redo` - Needs undo/redo implementation

## Tests with Compromised Assertions

### Tests using vague assertions (contains, !=, >=, <=)

- `test_grapheme_cluster_navigation` - Only checks len > 0
- `test_backspace_multiple_graphemes` - Only checks contains 'A'
- `test_cursor_at_grapheme_boundaries` - Only checks pos2 > pos1

### Tests that only verify no crash / have no real assertions

- `test_syntax_highlighting_language_detection` - No assertions (all commented out)
- `test_syntax_highlighting_state_after_edit` - No assertions, just comments
- `test_undo_after_emoji_insertion` - No actual test, just commented code
- `test_repeated_undo_redo` - Placeholder test, just verifies current state

## Summary of Test Issues

### Tests Still Needing Fixes (7 tests)

- 3 Unicode/character tests with incomplete checks (`test_grapheme_cluster_navigation`, `test_backspace_multiple_graphemes`, `test_cursor_at_grapheme_boundaries`)
- 4 tests with no meaningful assertions (`test_syntax_highlighting_language_detection`, `test_syntax_highlighting_state_after_edit`, `test_undo_after_emoji_insertion`, `test_repeated_undo_redo`)

### Next Priority Fixes

Based on severity and impact:

1. Fix remaining tests with vague assertions:
   - Unicode/character tests: `test_grapheme_cluster_navigation`, `test_backspace_multiple_graphemes`, `test_cursor_at_grapheme_boundaries`
   - No-assertion tests: `test_syntax_highlighting_language_detection`, `test_syntax_highlighting_state_after_edit`
   - Placeholder tests: `test_undo_after_emoji_insertion`, `test_repeated_undo_redo`
2. Undo/redo system - Essential feature (major work, multiple tests depend on it)
3. Grapheme clustering support (would fix several Unicode-related tests)
4. Unicode normalization (required for `test_normalization_inconsistencies`)
5. CRLF line ending support (for `test_windows_line_endings`)
