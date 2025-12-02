//! Markdown parsing utilities.
//!
//! This module provides re-exports and utilities for working with pulldown-cmark.

// Re-export commonly used types from pulldown-cmark
pub use pulldown_cmark::{Alignment, CodeBlockKind, LinkType, Options, Parser};

/// Default parsing options with GFM support enabled.
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
}
