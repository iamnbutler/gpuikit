//! Markdown parsing utilities.
//!
//! This module provides re-exports and utilities for working with pulldown-cmark.

// Re-export commonly used types from pulldown-cmark
pub use pulldown_cmark::{CodeBlockKind, LinkType, Options, Parser};

/// Default parsing options with GFM support enabled.
///
/// This is the *only* option set the renderer uses — [`Markdown::parse`] calls
/// it rather than assembling the same flags inline, so a change here changes
/// every document.
///
/// The options deliberately left off, and what turning one on would do to
/// documents that render correctly today:
///
/// - `ENABLE_SUBSCRIPT` — takes `~x~` *away from strikethrough* and emits
///   [`pulldown_cmark::Tag::Subscript`] instead, silently restyling existing
///   documents. See
///   `single_tilde_is_strikethrough_not_subscript`.
/// - `ENABLE_SUPERSCRIPT` — claims `^x^`, which is literal text today.
/// - Both also need somewhere to land: `InlineStyle` carries no baseline
///   shift, and neither does `gpui::HighlightStyle`, so enabling them without
///   that work would parse correctly and render identically to plain text.
/// - `ENABLE_WIKILINKS` — turns `[[foo|bar]]` into a link. See
///   `wikilinks_stay_literal_text`.
/// - `ENABLE_MATH` — claims `$x$` and `$$x$$` as `InlineMath`/`DisplayMath`,
///   which the renderer drops on the floor.
/// - `ENABLE_DEFINITION_LIST` — claims `term\n: definition`; the renderer has
///   no element for it and would swallow the text.
/// - `ENABLE_SMART_PUNCTUATION` — rewrites quotes and dashes, which is wrong
///   inside a document quoting code.
/// - `ENABLE_HEADING_ATTRIBUTES`, `ENABLE_*_METADATA_BLOCKS`,
///   `ENABLE_OLD_FOOTNOTES` — no renderer support, or superseded by
///   `ENABLE_GFM`.
///
/// [`Markdown::parse`]: super::Markdown
pub fn default_options() -> Options {
    Options::ENABLE_TABLES
        | Options::ENABLE_FOOTNOTES
        | Options::ENABLE_STRIKETHROUGH
        | Options::ENABLE_TASKLISTS
        | Options::ENABLE_GFM
}

/// Parse markdown source with default options.
pub fn parse(source: &str) -> Parser<'_> {
    Parser::new_ext(source, default_options())
}

/// Parse markdown source with custom options.
pub fn parse_with_options(source: &str, options: Options) -> Parser<'_> {
    Parser::new_ext(source, options)
}

/// Extract the language from a fenced code block kind.
pub fn code_block_language<'a>(kind: &'a CodeBlockKind<'a>) -> Option<&'a str> {
    match kind {
        CodeBlockKind::Fenced(info) => {
            let info = info.trim();
            if info.is_empty() {
                None
            } else {
                // Language is the first word before any space
                Some(info.split_whitespace().next().unwrap_or(info))
            }
        }
        CodeBlockKind::Indented => None,
    }
}

/// Whether `source` ends with a fenced code block that never closed.
///
/// This is the signal that says "the last code block in this document is still
/// arriving", and it exists so that a streaming fence is not re-highlighted
/// from scratch on every delta — see the `CacheKey` note in
/// `super::code_highlight`.
///
/// # Which way it is allowed to be wrong
///
/// A `true` here strips a block's syntax colors, so the two directions of
/// error are not symmetric:
///
/// - Wrong towards **closed** (returning `false` for a fence pulldown-cmark
///   still considers open) costs a streaming block the optimization it would
///   have had. That is today's behaviour, and harmless.
/// - Wrong towards **open** (returning `true` when the block really did close)
///   would take the colors off a *settled* block, and leave them off.
///
/// So the scan is deliberately more eager to close a fence than
/// pulldown-cmark: it accepts any run of the opener's character that reaches
/// the opener's length as a closer, including one carrying an info string —
/// which CommonMark says is content, not a closer. Every disagreement is
/// therefore in the harmless direction. `the_scan_only_ever_errs_towards_closed`
/// pins this.
///
/// # Blind spots
///
/// A fence inside a block quote (`> ```rust`), a fence indented by four or
/// more spaces inside a list item, and a fence nested inside another all read
/// as "no fence here", i.e. as closed. Each falls back to the behaviour this
/// function exists to improve on, which is the safe direction above.
///
/// This is a byte scan over the *raw* source, deliberately not a second
/// pulldown-cmark pass and deliberately not feature-gated: the cost it avoids
/// lands in `editor` builds, which are independent of `stitch`, and whether a
/// block is colored should not depend on an unrelated feature flag.
pub fn has_open_code_fence(source: &str) -> bool {
    let mut open: Option<(u8, usize)> = None;

    for line in source.lines() {
        match (open, fence_at_line_start(line)) {
            (None, Some(fence)) => open = Some(fence),
            (Some((open_char, open_len)), Some((line_char, line_len)))
                if line_char == open_char && line_len >= open_len =>
            {
                open = None;
            }
            _ => {}
        }
    }

    open.is_some()
}

/// The code fence a line opens or closes, as `(fence character, run length)`.
///
/// CommonMark §4.5: at most three leading spaces, then three or more backticks
/// or tildes. A run reached by four spaces is an indented code block, and a run
/// starting anywhere but the line's start is ordinary text.
fn fence_at_line_start(line: &str) -> Option<(u8, usize)> {
    let bytes = line.as_bytes();
    let indent = bytes.iter().take_while(|byte| **byte == b' ').count();
    if indent > 3 {
        return None;
    }

    let fence_char = *bytes.get(indent)?;
    if fence_char != b'`' && fence_char != b'~' {
        return None;
    }

    let len = bytes[indent..]
        .iter()
        .take_while(|byte| **byte == fence_char)
        .count();

    (len >= 3).then_some((fence_char, len))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_basic() {
        let source = "# Hello\n\nWorld";
        let events: Vec<_> = parse(source).collect();

        assert!(!events.is_empty());
    }

    #[test]
    fn test_code_block_language() {
        use pulldown_cmark::CowStr;

        let rust = CodeBlockKind::Fenced(CowStr::from("rust"));
        assert_eq!(code_block_language(&rust), Some("rust"));

        let rust_with_attrs = CodeBlockKind::Fenced(CowStr::from("rust,linenos"));
        assert_eq!(code_block_language(&rust_with_attrs), Some("rust,linenos"));

        let empty = CodeBlockKind::Fenced(CowStr::from(""));
        assert_eq!(code_block_language(&empty), None);

        let indented = CodeBlockKind::Indented;
        assert_eq!(code_block_language(&indented), None);
    }

    // The tests below pin behaviour that a change to `default_options` would
    // alter silently — nothing else in the crate would fail to compile, and the
    // documents would just render differently. They are pure parse tests: no
    // gpui, no window.

    #[test]
    fn default_options_are_exactly_the_gfm_five() {
        let options = default_options();

        for on in [
            Options::ENABLE_TABLES,
            Options::ENABLE_FOOTNOTES,
            Options::ENABLE_STRIKETHROUGH,
            Options::ENABLE_TASKLISTS,
            Options::ENABLE_GFM,
        ] {
            assert!(options.contains(on), "expected {on:?} to be enabled");
        }

        for off in [
            Options::ENABLE_SMART_PUNCTUATION,
            Options::ENABLE_HEADING_ATTRIBUTES,
            Options::ENABLE_YAML_STYLE_METADATA_BLOCKS,
            Options::ENABLE_PLUSES_DELIMITED_METADATA_BLOCKS,
            Options::ENABLE_OLD_FOOTNOTES,
            Options::ENABLE_MATH,
            Options::ENABLE_DEFINITION_LIST,
            Options::ENABLE_SUPERSCRIPT,
            Options::ENABLE_SUBSCRIPT,
            Options::ENABLE_WIKILINKS,
        ] {
            assert!(!options.contains(off), "expected {off:?} to be disabled");
        }
    }

    /// `ENABLE_SUBSCRIPT` is not a free upgrade: pulldown-cmark's own docs say
    /// that with it on, `~x~` parses as subscript *instead of* strikethrough.
    /// Turning it on would restyle every document already using single tildes.
    #[test]
    fn single_tilde_is_strikethrough_not_subscript() {
        use pulldown_cmark::{Event, Tag};

        let events: Vec<_> = parse("~struck~").collect();

        assert!(
            events
                .iter()
                .any(|e| matches!(e, Event::Start(Tag::Strikethrough))),
            "single tildes should still be strikethrough: {events:?}"
        );
        assert!(
            !events
                .iter()
                .any(|e| matches!(e, Event::Start(Tag::Subscript))),
            "subscript must stay off: {events:?}"
        );
    }

    #[test]
    fn wikilinks_stay_literal_text() {
        use pulldown_cmark::{Event, Tag};

        let events: Vec<_> = parse("[[foo|bar]]").collect();

        assert!(
            !events
                .iter()
                .any(|e| matches!(e, Event::Start(Tag::Link { .. }))),
            "wikilinks must stay off: {events:?}"
        );
    }

    /// GFM alerts ride in on the `BlockQuote` tag's payload rather than a tag
    /// of their own, so the renderer's `Tag::BlockQuote(_)` arm sees them.
    #[test]
    fn gfm_alerts_reach_block_quote_kind() {
        use pulldown_cmark::{BlockQuoteKind, Event, Tag};

        let events: Vec<_> = parse("> [!NOTE]\n> Something worth knowing.").collect();

        assert!(
            events
                .iter()
                .any(|e| matches!(e, Event::Start(Tag::BlockQuote(Some(BlockQuoteKind::Note))))),
            "expected a Note alert: {events:?}"
        );
    }

    #[test]
    fn task_list_markers_are_emitted() {
        use pulldown_cmark::Event;

        let events: Vec<_> = parse("- [x] done\n- [ ] todo").collect();
        let markers: Vec<bool> = events
            .iter()
            .filter_map(|e| match e {
                Event::TaskListMarker(checked) => Some(*checked),
                _ => None,
            })
            .collect();

        assert_eq!(markers, vec![true, false]);
    }

    #[test]
    fn table_column_alignments_survive() {
        use pulldown_cmark::{Alignment, Event, Tag};

        let source = "| a | b | c |\n|:--|:-:|--:|\n| 1 | 2 | 3 |";
        let alignments = parse(source).find_map(|e| match e {
            Event::Start(Tag::Table(alignments)) => Some(alignments),
            _ => None,
        });

        assert_eq!(
            alignments,
            Some(vec![Alignment::Left, Alignment::Center, Alignment::Right])
        );
    }

    /// The renderer keeps `into_offset_iter` ranges and slices the source with
    /// them; a range landing mid-codepoint would panic on a non-ASCII document.
    #[test]
    fn offset_ranges_land_on_char_boundaries() {
        let source = "# Überschrift\n\nEin Absatz mit **fettem** Text — und einem Emoji 🎉.\n\n\
                      - Ein Listenpunkt mit `Code`\n";

        for (event, range) in parse(source).into_offset_iter() {
            assert!(
                source.is_char_boundary(range.start) && source.is_char_boundary(range.end),
                "range {range:?} splits a codepoint for {event:?}"
            );
            assert!(range.end <= source.len(), "range {range:?} out of bounds");
        }
    }

    // `has_open_code_fence` is what decides whether a code block is drawn
    // plain, so these tests are about the *decision*, not about colors.

    /// The case the whole thing exists for: a fence arriving one delta at a
    /// time reads as open for every prefix until the closer lands.
    #[test]
    fn a_streamed_fence_reads_as_open_until_its_closer_arrives() {
        let full = "Here you go:\n\n```rust\nfn main() {\n    println!(\"hi\");\n}\n```\n\nDone.";
        let opener_at = full.find("```rust").expect("the opening fence");
        let closer_at = full.find("\n```\n\nDone").expect("the closing fence") + 1;

        for end in 0..full.len() {
            if !full.is_char_boundary(end) {
                continue;
            }
            let prefix = &full[..end];
            // A fence is open from its third backtick, and closed again from
            // the closer's third.
            let opened = end >= opener_at + 3;
            let closed = end >= closer_at + 3;

            assert_eq!(
                has_open_code_fence(prefix),
                opened && !closed,
                "prefix of {end} bytes: {prefix:?}"
            );
        }
    }

    #[test]
    fn a_closer_must_match_the_opener() {
        // A tilde run does not close a backtick fence, and vice versa.
        assert!(has_open_code_fence("```rust\nx\n~~~\n"));
        assert!(has_open_code_fence("~~~rust\nx\n```\n"));

        // Nor does a shorter run of the right character.
        assert!(has_open_code_fence("````\nx\n```\n"));

        // A longer one does.
        assert!(!has_open_code_fence("```\nx\n`````\n"));
    }

    #[test]
    fn indentation_decides_whether_a_run_is_a_fence_at_all() {
        // Up to three spaces is still a fence, opening or closing.
        assert!(has_open_code_fence("   ```rust\nx\n"));
        assert!(!has_open_code_fence("```rust\nx\n   ```\n"));

        // Four is an indented code block, so neither opens nor closes one.
        assert!(!has_open_code_fence("    ```rust\nx\n"));
        assert!(has_open_code_fence("```rust\nx\n    ```\n"));
    }

    #[test]
    fn a_run_that_is_not_at_a_line_start_is_text() {
        assert!(!has_open_code_fence("see ```rust for the fence\n"));
        assert!(has_open_code_fence("```\nsee ``` inline\n"));
    }

    /// Only the *last* block can be the open one: two settled fences followed
    /// by prose leave nothing open, and a third opener does.
    #[test]
    fn only_the_last_fence_can_be_open() {
        assert!(!has_open_code_fence("```\na\n```\n\ntext\n\n```\nb\n```\n"));
        assert!(has_open_code_fence("```\na\n```\n\ntext\n\n```\nb\n"));
        assert!(!has_open_code_fence("no code at all\n"));
        assert!(!has_open_code_fence(""));
    }

    /// Every disagreement with pulldown-cmark must be in the direction that
    /// costs a block the optimization, never in the direction that strips a
    /// settled block's colors.
    #[test]
    fn the_scan_only_ever_errs_towards_closed() {
        use pulldown_cmark::{Event, Tag, TagEnd};

        // The known disagreement: CommonMark says a closing fence carries no
        // info string, so pulldown-cmark reads this as one unfinished block.
        let nested = "```markdown\n```rust\n";
        let events: Vec<_> = parse(nested).collect();
        let starts = events
            .iter()
            .filter(|e| matches!(e, Event::Start(Tag::CodeBlock(_))))
            .count();
        assert_eq!(starts, 1, "pulldown-cmark should see one block: {events:?}");
        assert!(
            !has_open_code_fence(nested),
            "the scan is expected to disagree here, in the harmless direction"
        );

        // And the direction, checked across a spread of documents: whenever
        // the scan says "open", pulldown-cmark must agree that the last code
        // block runs to the end of the source.
        let sources = [
            "```rust\nfn main() {",
            "```\n",
            "~~~\nplain\n",
            "text\n\n```py\nx = 1\n",
            "```rust\nfn main() {}\n```\n",
            "```rust\nfn main() {}\n```\n\ntrailing prose",
            "    indented\n",
            "# heading\n\nno code\n",
            "> ```rust\n> quoted\n",
            "- item\n\n  ```rust\n  fn f() {}\n  ```\n",
        ];

        for source in sources {
            if !has_open_code_fence(source) {
                continue;
            }

            let events: Vec<_> = parse(source).collect();
            assert!(
                events
                    .iter()
                    .any(|e| matches!(e, Event::Start(Tag::CodeBlock(_)))),
                "the scan says {source:?} has an open fence, but there is no code block"
            );

            // pulldown-cmark closes an unterminated block at EOF rather than
            // leaving it open, so "still open" shows up as the code block
            // being the last thing in the document — nothing follows it.
            assert!(
                matches!(events.last(), Some(Event::End(TagEnd::CodeBlock))),
                "the scan says {source:?} has an open fence, but pulldown-cmark put content \
                 after the last code block — that would strip a settled block's colors: \
                 {events:?}"
            );
        }
    }
}
