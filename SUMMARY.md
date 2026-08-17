# Loose markdown lists render as lists, and the showcase catches up with the crate

**Loose lists (#136).** A list whose items are separated by a blank line is
*loose*, and CommonMark wraps each item's content in a paragraph.
`handle_end_tag` routed every `TagEnd::Paragraph` to `flush_paragraph`, so a
loose item's text went out as body text — no marker, no indent, no number — and
the item's `TagEnd::Item` then found an empty buffer and emitted nothing; since
the runs joined the accessibility tree, the list was also announced as a
sequence of paragraphs. The renderer now tracks the item currently open
(`ListContext::item`) and routes a paragraph ending inside one to
`flush_list_item`, the way `in_block_quote` already redirects it. Only the
innermost list is asked, because a nested list opens while its parent's item is
still open. An item can hold several blocks, so it remembers the marker its
first block drew: that block paints the marker, takes the list's next ordinal
and reports `Role::ListItem`; every later block lays the same marker out at zero
opacity — reserving its width so the text stays in the item's column — takes no
ordinal, and reports `Role::Paragraph` rather than telling a screen reader the
list has more items than it has. Thirteen tests cover it, including that loose
and tight spellings of five list shapes emit byte-identical rows.

**Showcase drift (#139).** Four element modules had no showcase page —
`slider`, `typography`, `empty`, and `toggle`, which the issue's grep would have
missed because `toggle_group` scores 43 false hits — and `editor` had no example
anywhere. They all have pages now; the Editor page renders a live buffer with
`--features editor` and a placeholder saying how to get one without it, so no
other page has to pay for syntect. `main` now calls `init_code_highlighting`
under `#[cfg(feature = "editor")]`, so the showcase's ` ```rust ` fence is
actually highlighted; it was silently inactive. The Markdown page grew from a
one-liner into a real page: which build you are running, the roles every block
reports, a Selection section with a live readout and a copy button, and a
Streaming section feeding a second document through `append` on a timer — each
naming the standalone example that goes further. `SAMPLE_MARKDOWN` now carries
the nested, nested-ordered and loose list shapes that broke recently, so a
renderer regression is visible to anyone who opens the showcase. So the sweep
does not just reset the clock, an `ELEMENT_COVERAGE` table maps all 38 element
modules to the page that shows them — rendered by a new Coverage page, so it is
live code rather than a constant only a test reads — and two `#[cfg(test)]`
tests in `src/elements.rs` cross-check it against the `pub mod` list and against
the render match, failing the build when an element gains no page or a table row
names a page the nav cannot reach (`("name", "none: <reason>")` opts out). The
convention behind all of this is written down in the new `examples/README.md`.
Both guards were proven live by deleting a row, pointing a row at a bogus page,
and confirming a `none:` row passes; both `cargo check` paths were proven live
by injecting a deliberate error into each.

The GUI itself was not launched — this environment has no display — so the new
pages are verified by compilation and by the element APIs they call, not by
screenshot. Clippy reports nothing new in the changed files; the only warnings
in either build are pre-existing.

Verification: PASSED — cargo fmt --check; cargo test --lib (270 passed); cargo test --doc (2 passed); cargo test --features stitch --lib (274 passed); cargo test --features editor --lib (412 passed); cargo check --all-targets; cargo check --all-targets --features editor
