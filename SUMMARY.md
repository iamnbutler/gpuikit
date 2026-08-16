# Markdown list items and table cells wrap

A markdown list item is a flex row: a `flex_none` marker beside a `flex_1` text
child. That makes the text a flex item, and a flex item carries an automatic
minimum size (CSS `min-width: auto`, which taffy implements). In gpui that
minimum is the width of one unbroken line — a `MinContent` measurement gets no
wrap width, so the text reports its full single-line width — and `flex: 1 1 0%`
could never shrink the item far enough to wrap. A long list item therefore ran
off the edge of the document, while the same text in a paragraph, which is a
plain `w_full` block and never a flex item, wrapped fine. The fix is the classic
one: `min_w_0()` on the text child in both `list_item` and `rich_list_item`.
The `flex_1` cells in `render_table` have the identical shape and had the
identical bug (confirmed by test, not assumed), so they get the same line. Those
three are the only `flex_row` sites under `src/markdown/`; the document
container is a `flex_col` of `w_full` children and is unaffected. No API or
behaviour changes beyond the wrap.

Five layout regression tests were added in a new `src/markdown` test module.
They render markdown into a fixed 240px-wide container and assert on the
container's **height**, since text that refuses to wrap stays one line tall
however long it is, and they compare each case against the same text set as a
paragraph — exactly the comparison the bug report makes. The cases cover an
unordered item, an ordered item, an item carrying `**bold**`/`` `code` ``/
`[link](#)` (the `rich_list_item` path, a different child element in the same
flex container), a nested item at `indent_level > 0`, and a long table cell.
Each was verified to fail with the fix reverted, reporting a 24px — one line —
height. Note for anyone editing them: the measured div must not be the window's
root element, which is stretched to the window and would make every height
assertion pass vacuously; it is wrapped in a `flex_col` parent so it keeps its
content height. One thing found along the way and deliberately left alone, as
it is separate and pre-existing: a nested list swallows its parent item's text,
because `handle_start_tag(Tag::List)` doesn't flush `current_text`. That is
worth its own issue; the nested test is written not to depend on it beyond
needing an indented row.

Verification: PASSED — `cargo test -j 1` (208 lib tests + 2 doctests, all green;
`cargo test` alone OOMs in this container while linking example binaries in
parallel, which is an environment limit, not a code failure), plus
`cargo fmt --check` clean and `cargo clippy --all-targets -j 1` at the same
warning count as before the change with nothing under `src/markdown/`.
