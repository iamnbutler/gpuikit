# A nested list no longer swallows its parent item's text (#130)

A list nested under an item consumed that item's text: `- x` with an indented
`- y` under it rendered as a single row. The nested list's `Start(List)` event
arrives *before* the parent item's `End(Item)`, so the parent's text was still
buffered in `current_text` when the child list opened — the child's first
`flush_list_item` picked it up merged with its own, and the parent's `End(Item)`
then found an empty buffer and emitted nothing at all. The fix is one guarded
call: `handle_start_tag(Tag::List)` flushes the pending item text when the list
stack is non-empty, before pushing the new `ListContext`. Flushing *before* the
push is what gives the parent's row its own indent, its own marker and — in an
ordered list — its own ordinal, so `1. x` / nested child / `2. z` no longer
silently renumbers; the stack guard keeps a top-level list, which has no parent
item to attribute a row to, from inventing a row. `handle_start_tag` gained a
`cx: &App` parameter, since flushing builds an element. That, plus a CHANGELOG
entry, is the whole production change.

The rest is tests. A `#[cfg(test)] emitted_list_items` field on
`MarkdownRenderer` records each emitted row's marker, indent level and text: the
existing tests can only measure rendered height, which cannot see a marker, an
indent level, or which row a piece of text landed in — exactly the things this
bug scrambles. Eleven tests cover unordered-in-unordered, ordered-in-unordered,
unordered-in-ordered, three levels deep, a parent carrying bold/code/link
(the `rich_list_item` path), ordinal continuity across a nested child, task-list
checkboxes, a parent with no text of its own (must not emit a blank row), a list
following a paragraph, and two end-to-end height checks that the rows actually
reach layout. Nine of the eleven were confirmed to fail with the flush disabled;
the other two are guards against placing the flush wrongly and pass either way.
`a_long_nested_list_item_wraps` (from #131) was written against the buggy
single-row output, so it is re-based to subtract the parent's row before
checking the nested one's wrap. Loose lists — items separated by blank lines —
are separately broken and untouched here: pulldown-cmark wraps each item's text
in a `Paragraph`, which routes to `flush_paragraph`, so those render with no
marker and no indent. That deserves its own issue.

Verification: PASSED — `cargo test --lib` (249 passed, 0 failed), plus `cargo fmt --check`, `cargo check --all-targets` (one pre-existing `unused_mut` warning in `src/input/bindings.rs`), and `cargo clippy --all-targets` (zero findings in `src/markdown/`). Unqualified `cargo test` is not usable here — linking the examples gets the linker OOM-killed, which is pre-existing and unrelated.
