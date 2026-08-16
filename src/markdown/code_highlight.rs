//! Syntax highlighting for fenced code blocks.
//!
//! The markdown renderer knows a block's info string; the `editor` feature
//! knows how to highlight source. This module is the seam between them, and
//! the only thing [`code_block`](super::elements) has to know about.
//!
//! Highlighting is **opt-in per app**: call
//! [`init_code_highlighting`] once, after `gpuikit::init`. Loading syntect's
//! default syntax and theme sets costs tens of milliseconds and a few
//! megabytes, which a document with no code in it should not pay.
//!
//! Every step degrades to plain monospace rather than failing: no `editor`
//! feature, no [`init_code_highlighting`], no info string, an info string
//! syntect has no grammar for, or a block past
//! [`MAX_HIGHLIGHT_BYTES`](self) all render exactly what they rendered
//! before this module existed.
//!
//! ```ignore
//! Application::new().run(|cx| {
//!     gpuikit::init(cx);
//!     gpuikit::markdown::init_code_highlighting(cx);
//! });
//! ```

use std::ops::Range;

use gpui::{App, HighlightStyle, Hsla};

/// Fence info strings that mean "this is not a language".
const PLAIN_LANGUAGES: &[&str] = &["text", "plaintext", "plain", "none", "txt"];

/// Fence dialects that name a language syntect knows under another token.
///
/// Only aliases that are genuinely the same grammar belong here. `ts` is
/// absent on purpose: syntect's default set has no TypeScript, and pointing it
/// at JavaScript would mis-highlight exactly the type syntax that makes it
/// TypeScript.
const LANGUAGE_ALIASES: &[(&str, &str)] = &[
    ("bash", "bash"),
    ("console", "bash"),
    ("golang", "go"),
    ("htm", "html"),
    ("jsonc", "json"),
    ("objc", "objective-c"),
    ("rs", "rust"),
    ("sh", "bash"),
    ("shell", "bash"),
    ("yml", "yaml"),
    ("zsh", "bash"),
];

/// Reduce a code fence's info string to a language token, or `None` if it
/// names no language.
///
/// Takes the leading word, so ` ```rust,ignore ` and ` ```rust no_run ` both
/// arrive as `rust`, then folds case and applies a small alias table
/// (`golang` → `go`, `yml` → `yaml`, `zsh` → `bash`, …). Returns `None` for an
/// empty string and for the explicit "no language" spellings (`text`,
/// `plaintext`, `plain`, `none`, `txt`).
///
/// The token still has to survive
/// [`SyntaxHighlighter::resolve_language`](crate::editor::SyntaxHighlighter::resolve_language);
/// this only normalizes the *fence* dialect.
pub fn normalize_language(info: &str) -> Option<String> {
    let token = info
        .trim()
        .split([',', ' ', '\t'])
        .next()
        .unwrap_or("")
        .trim();

    if token.is_empty() {
        return None;
    }

    let lowered = token.to_ascii_lowercase();
    if PLAIN_LANGUAGES.contains(&lowered.as_str()) {
        return None;
    }

    Some(
        LANGUAGE_ALIASES
            .iter()
            .find(|(alias, _)| *alias == lowered)
            .map(|(_, canonical)| (*canonical).to_string())
            .unwrap_or(lowered),
    )
}

#[cfg(feature = "editor")]
pub use editor_bridge::{
    code_highlight_themes, init_code_highlighting, set_code_highlight_theme, CodeHighlightTheme,
    DEFAULT_DARK_THEME, DEFAULT_LIGHT_THEME,
};

/// The highlights for one code block, or an empty vector if anything at all is
/// missing — see the module docs for the full list of ways this degrades.
///
/// `background` is the surface the block is painted on; it decides light or
/// dark when the theme is [`CodeHighlightTheme::FollowApp`].
#[cfg(feature = "editor")]
pub(crate) fn code_highlights(
    text: &str,
    language: Option<&str>,
    background: Hsla,
    cx: &App,
) -> Vec<(Range<usize>, HighlightStyle)> {
    editor_bridge::code_highlights(text, language, background, cx)
}

/// Without the `editor` feature there is no highlighter to ask, so every code
/// block renders plain. Same signature as the real one, so `code_block`
/// contains no `cfg` of its own.
#[cfg(not(feature = "editor"))]
pub(crate) fn code_highlights(
    _text: &str,
    _language: Option<&str>,
    _background: Hsla,
    _cx: &App,
) -> Vec<(Range<usize>, HighlightStyle)> {
    Vec::new()
}

#[cfg(feature = "editor")]
mod editor_bridge {
    use std::cell::RefCell;
    use std::collections::HashMap;
    use std::ops::Range;
    use std::rc::Rc;

    use gpui::{App, Global, HighlightStyle, Hsla};

    use crate::editor::SyntaxHighlighter;

    /// The syntect theme used on a dark surface when following the app.
    pub const DEFAULT_DARK_THEME: &str = "base16-ocean.dark";
    /// The syntect theme used on a light surface when following the app.
    pub const DEFAULT_LIGHT_THEME: &str = "InspiredGitHub";

    /// Past this many bytes a block renders plain. A syntect pass is linear,
    /// but a megabyte of pasted log in a chat transcript is not worth a frame.
    const MAX_HIGHLIGHT_BYTES: usize = 256 * 1024;

    /// How many highlighted blocks to remember. Cleared wholesale on
    /// overflow — an LRU would need a recency list for a cache whose whole
    /// purpose is to be cheap.
    const MAX_CACHED_BLOCKS: usize = 256;

    /// Which syntect theme code blocks highlight with.
    #[derive(Clone, Debug, PartialEq, Eq, Default)]
    pub enum CodeHighlightTheme {
        /// Pick [`DEFAULT_DARK_THEME`] or [`DEFAULT_LIGHT_THEME`] from the
        /// lightness of the surface the block is drawn on, so a light/dark
        /// toggle needs no extra call.
        #[default]
        FollowApp,
        /// Always this syntect theme, whatever the app is doing. One of
        /// [`code_highlight_themes`].
        Pinned(String),
    }

    /// (text, resolved syntax name, syntect theme) → that block's highlights.
    /// All three matter: the same text under a different theme is a different
    /// answer, which is what makes an app theme toggle re-highlight for free.
    ///
    /// **Keying on the whole text means a streaming block never hits.** A fence
    /// arriving through `Markdown::append` is a different key on every delta,
    /// so it costs a full syntect pass over the whole block-so-far each time —
    /// quadratic in the block's final length, on the render path. Worse, those
    /// prefixes are all distinct entries, so one long streamed block can fill
    /// [`MAX_CACHED_BLOCKS`] by itself and take every other block's entry with
    /// it when the cache clears. [`MAX_HIGHLIGHT_BYTES`] bounds the individual
    /// pass and is deliberately checked before the key is built, so an
    /// oversized block never lands here at all — but it does not bound the
    /// repetition. Fixing that means an incremental highlighter or a key that
    /// is a prefix rather than an identity; neither is worth doing before
    /// there is a profile that says so.
    type CacheKey = (String, String, String);

    type Highlights = Vec<(Range<usize>, HighlightStyle)>;

    struct CodeHighlighter {
        highlighter: RefCell<SyntaxHighlighter>,
        theme: RefCell<CodeHighlightTheme>,
        cache: RefCell<HashMap<CacheKey, Rc<Highlights>>>,
        /// Fence token → syntax name, memoized. Resolution walks syntect's
        /// whole syntax list, and a document repeats the same few languages.
        resolved: RefCell<HashMap<String, Option<String>>>,
    }

    impl CodeHighlighter {
        fn new() -> Self {
            Self {
                highlighter: RefCell::new(SyntaxHighlighter::new()),
                theme: RefCell::new(CodeHighlightTheme::default()),
                cache: RefCell::new(HashMap::new()),
                resolved: RefCell::new(HashMap::new()),
            }
        }

        /// The syntect theme name to use for a block on `background`.
        fn theme_for(&self, background: Hsla) -> String {
            match &*self.theme.borrow() {
                CodeHighlightTheme::Pinned(name) => name.clone(),
                CodeHighlightTheme::FollowApp => if background.l > 0.5 {
                    DEFAULT_LIGHT_THEME
                } else {
                    DEFAULT_DARK_THEME
                }
                .to_string(),
            }
        }

        fn resolve(&self, token: &str) -> Option<String> {
            if let Some(hit) = self.resolved.borrow().get(token) {
                return hit.clone();
            }
            let resolved = self.highlighter.borrow().resolve_language(token);
            self.resolved
                .borrow_mut()
                .insert(token.to_string(), resolved.clone());
            resolved
        }

        fn highlights(&self, text: &str, token: &str, background: Hsla) -> Rc<Highlights> {
            let empty = || Rc::new(Vec::new());

            if text.len() > MAX_HIGHLIGHT_BYTES {
                return empty();
            }
            let Some(language) = self.resolve(token) else {
                return empty();
            };

            let theme = self.theme_for(background);
            let key = (text.to_string(), language.clone(), theme.clone());
            if let Some(hit) = self.cache.borrow().get(&key) {
                return hit.clone();
            }

            // `set_theme` is a no-op for a theme syntect does not have, which
            // leaves whatever was set before — fine, and still consistent with
            // the cache key, since a miss re-runs this every time.
            {
                let mut highlighter = self.highlighter.borrow_mut();
                if highlighter.current_theme() != theme {
                    highlighter.set_theme(&theme);
                }
            }

            let plain_foreground = self.highlighter.borrow().get_theme_foreground();
            let highlights: Highlights = self
                .highlighter
                .borrow()
                .highlight_block(text, &language)
                .into_iter()
                // A syntect theme's plain foreground is not the app's. Left
                // in, every unremarkable identifier would be repainted in the
                // syntect theme's body color — a dark theme's light grey on a
                // light surface. Dropping those spans lets ordinary code keep
                // the block's own text color, and only *distinctly* colored
                // tokens get restyled.
                .filter(|(_, style)| {
                    style.font_weight.is_some()
                        || style.font_style.is_some()
                        || style.underline.is_some()
                        || style.color.is_some_and(|color| color != plain_foreground)
                })
                .collect();

            let highlights = Rc::new(highlights);

            let mut cache = self.cache.borrow_mut();
            if cache.len() >= MAX_CACHED_BLOCKS {
                cache.clear();
            }
            cache.insert(key, highlights.clone());
            highlights
        }

        fn set_theme(&self, theme: CodeHighlightTheme) {
            *self.theme.borrow_mut() = theme;
        }

        fn available_themes(&self) -> Vec<String> {
            self.highlighter.borrow().available_themes()
        }
    }

    struct GlobalCodeHighlighter(Rc<CodeHighlighter>);

    impl Global for GlobalCodeHighlighter {}

    /// Turn on syntax highlighting for markdown code blocks.
    ///
    /// Call once during app setup, after `gpuikit::init`. Without it, fenced
    /// code blocks render as plain monospace — which is the right default,
    /// since loading syntect's syntax and theme sets is not free and a
    /// document with no code in it should not pay for them.
    ///
    /// Calling it twice replaces the highlighter, discarding the cache.
    pub fn init_code_highlighting(cx: &mut App) {
        cx.set_global(GlobalCodeHighlighter(Rc::new(CodeHighlighter::new())));
    }

    /// Pin code blocks to a specific syntect theme, or put them back on
    /// [`CodeHighlightTheme::FollowApp`].
    ///
    /// Does nothing if [`init_code_highlighting`] has not been called.
    pub fn set_code_highlight_theme(cx: &mut App, theme: CodeHighlightTheme) {
        if let Some(global) = cx.try_global::<GlobalCodeHighlighter>() {
            global.0.set_theme(theme);
        }
    }

    /// The syntect theme names [`CodeHighlightTheme::Pinned`] accepts, or an
    /// empty vector before [`init_code_highlighting`].
    pub fn code_highlight_themes(cx: &App) -> Vec<String> {
        cx.try_global::<GlobalCodeHighlighter>()
            .map(|global| global.0.available_themes())
            .unwrap_or_default()
    }

    pub(super) fn code_highlights(
        text: &str,
        language: Option<&str>,
        background: Hsla,
        cx: &App,
    ) -> Vec<(Range<usize>, HighlightStyle)> {
        let Some(language) = language else {
            return Vec::new();
        };
        let Some(global) = cx.try_global::<GlobalCodeHighlighter>() else {
            return Vec::new();
        };

        // One `Vec` clone per cached block per frame. If that ever shows up in
        // a profile, hand back the `Rc` instead of cloning it.
        (*global.0.highlights(text, language, background)).clone()
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        fn highlighter() -> CodeHighlighter {
            CodeHighlighter::new()
        }

        const DARK: Hsla = Hsla {
            h: 0.0,
            s: 0.0,
            l: 0.1,
            a: 1.0,
        };
        const LIGHT: Hsla = Hsla {
            h: 0.0,
            s: 0.0,
            l: 0.95,
            a: 1.0,
        };

        /// `StyledText::with_highlights` debug-asserts char boundaries and
        /// `compute_runs` assumes sorted, non-overlapping ranges. This is the
        /// guard for both.
        fn assert_well_formed(text: &str, highlights: &[(Range<usize>, HighlightStyle)]) {
            let mut previous_end = 0;
            for (range, _) in highlights {
                assert!(range.start < range.end, "empty range {range:?}");
                assert!(
                    range.start >= previous_end,
                    "range {range:?} overlaps or precedes {previous_end}"
                );
                assert!(range.end <= text.len(), "range {range:?} past end of text");
                assert!(
                    text.is_char_boundary(range.start) && text.is_char_boundary(range.end),
                    "range {range:?} splits a codepoint"
                );
                previous_end = range.end;
            }
        }

        #[test]
        fn highlights_are_well_formed() {
            let highlighter = highlighter();
            let text = "fn main() {\n    println!(\"hello\");\n}\n";
            let highlights = highlighter.highlights(text, "rust", DARK);

            assert!(!highlights.is_empty(), "expected rust to highlight");
            assert_well_formed(text, &highlights);
        }

        #[test]
        fn multibyte_highlights_are_well_formed() {
            let highlighter = highlighter();
            let text = "fn main() {\n    let grüße = \"héllo 🎉 wörld\";\n}\n";
            let highlights = highlighter.highlights(text, "rust", DARK);

            assert_well_formed(text, &highlights);
        }

        /// The reason this path does not reuse `highlight_line`: state has to
        /// carry across the lines of one block, and only those lines.
        #[test]
        fn highlight_block_carries_state_across_lines() {
            let highlighter = highlighter();
            let text = "/* opening\n   still a comment\n   closing */\nfn after() {}\n";
            let highlights = highlighter.highlights(text, "rust", DARK);

            assert_well_formed(text, &highlights);

            let comment_line_two = text.find("still").expect("fixture has the word");
            let covering = highlights
                .iter()
                .find(|(range, _)| range.contains(&comment_line_two));
            assert!(
                covering.is_some(),
                "line 2 of a block comment should still be styled: {highlights:?}"
            );
        }

        #[test]
        fn unknown_language_renders_plain() {
            let highlighter = highlighter();
            let highlights = highlighter.highlights("some text", "definitely-not-a-language", DARK);

            assert!(highlights.is_empty());
        }

        #[test]
        fn empty_and_newlineless_blocks_are_safe() {
            let highlighter = highlighter();

            assert!(highlighter.highlights("", "rust", DARK).is_empty());

            let text = "let x = 1;";
            let highlights = highlighter.highlights(text, "rust", DARK);
            assert_well_formed(text, &highlights);
        }

        /// syntect puts its theme's block background on every span, which
        /// would paint over the code block's own surface.
        #[test]
        fn no_span_carries_a_background() {
            let highlighter = highlighter();
            let text = "fn main() {\n    let x = 1;\n}\n";

            for (_, style) in highlighter.highlights(text, "rust", DARK).iter() {
                assert!(style.background_color.is_none(), "{style:?}");
            }
        }

        #[test]
        fn oversized_blocks_render_plain() {
            let highlighter = highlighter();
            let text = "// x\n".repeat(MAX_HIGHLIGHT_BYTES / 5 + 1);

            assert!(text.len() > MAX_HIGHLIGHT_BYTES);
            assert!(highlighter.highlights(&text, "rust", DARK).is_empty());
        }

        #[test]
        fn a_repeated_block_is_only_highlighted_once() {
            let highlighter = highlighter();
            let text = "fn main() {}\n";

            let first = highlighter.highlights(text, "rust", DARK);
            assert_eq!(highlighter.cache.borrow().len(), 1);

            let second = highlighter.highlights(text, "rust", DARK);
            assert_eq!(highlighter.cache.borrow().len(), 1, "expected a cache hit");
            assert_eq!(*first, *second);
        }

        #[test]
        fn theme_follows_the_block_background_unless_pinned() {
            let highlighter = highlighter();

            assert_eq!(highlighter.theme_for(DARK), DEFAULT_DARK_THEME);
            assert_eq!(highlighter.theme_for(LIGHT), DEFAULT_LIGHT_THEME);

            highlighter.set_theme(CodeHighlightTheme::Pinned("Solarized (dark)".into()));
            assert_eq!(highlighter.theme_for(DARK), "Solarized (dark)");
            assert_eq!(highlighter.theme_for(LIGHT), "Solarized (dark)");
        }

        /// The theme is part of the cache key, so flipping the app between
        /// light and dark re-highlights rather than reusing dark-theme colors
        /// on a light surface.
        #[test]
        fn changing_theme_re_highlights() {
            let highlighter = highlighter();
            let text = "fn main() {}\n";

            let dark = highlighter.highlights(text, "rust", DARK);
            let light = highlighter.highlights(text, "rust", LIGHT);

            assert_eq!(highlighter.cache.borrow().len(), 2);
            assert_ne!(*dark, *light, "expected different colors per theme");
        }

        /// The wiring, end to end through the gpui global: highlighting is
        /// opt-in, and every way of not having it renders plain.
        #[gpui::test]
        fn highlighting_is_opt_in_per_app(cx: &mut gpui::TestAppContext) {
            let text = "fn main() {\n    println!(\"hello\");\n}\n";

            cx.update(|cx| {
                assert!(
                    super::super::code_highlights(text, Some("rust"), DARK, cx).is_empty(),
                    "no highlighting before init_code_highlighting"
                );
                assert!(code_highlight_themes(cx).is_empty());

                init_code_highlighting(cx);

                assert!(
                    !super::super::code_highlights(text, Some("rust"), DARK, cx).is_empty(),
                    "expected highlighting once initialized"
                );
                assert!(code_highlight_themes(cx).contains(&DEFAULT_DARK_THEME.to_string()));

                assert!(
                    super::super::code_highlights(text, None, DARK, cx).is_empty(),
                    "a bare fence has no language to highlight with"
                );
                assert!(
                    super::super::code_highlights(text, Some("not-a-language"), DARK, cx)
                        .is_empty(),
                    "an unknown language renders plain"
                );

                set_code_highlight_theme(cx, CodeHighlightTheme::Pinned("InspiredGitHub".into()));
                assert!(!super::super::code_highlights(text, Some("rust"), DARK, cx).is_empty());
            });
        }

        #[test]
        fn common_fence_labels_resolve() {
            let highlighter = highlighter();

            for token in [
                "rust", "rs", "python", "js", "JS", "json", "yaml", "yml", "html", "css", "c",
                "cpp", "go", "golang", "ruby", "bash", "sh", "shell", "zsh", "sql", "xml",
                "markdown", "java", "php",
            ] {
                let normalized = super::super::normalize_language(token)
                    .unwrap_or_else(|| panic!("{token} should normalize to a language"));
                assert!(
                    highlighter.resolve(&normalized).is_some(),
                    "{token} (normalized to {normalized}) should resolve to a syntax"
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn leading_word_only() {
        assert_eq!(normalize_language("rust"), Some("rust".into()));
        assert_eq!(normalize_language("rust,ignore"), Some("rust".into()));
        assert_eq!(normalize_language("rust no_run"), Some("rust".into()));
        assert_eq!(normalize_language("  rust  "), Some("rust".into()));
        assert_eq!(normalize_language("rust,linenos=1"), Some("rust".into()));
    }

    #[test]
    fn case_is_folded() {
        assert_eq!(normalize_language("Rust"), Some("rust".into()));
        assert_eq!(normalize_language("JSON"), Some("json".into()));
    }

    #[test]
    fn aliases_map_to_one_grammar() {
        assert_eq!(normalize_language("golang"), Some("go".into()));
        assert_eq!(normalize_language("yml"), Some("yaml".into()));
        assert_eq!(normalize_language("jsonc"), Some("json".into()));
        assert_eq!(normalize_language("objc"), Some("objective-c".into()));

        for shell in ["sh", "bash", "zsh", "shell", "console"] {
            assert_eq!(normalize_language(shell), Some("bash".into()), "{shell}");
        }
    }

    #[test]
    fn no_language_is_none() {
        assert_eq!(normalize_language(""), None);
        assert_eq!(normalize_language("   "), None);

        for plain in ["text", "plaintext", "plain", "none", "txt", "TEXT"] {
            assert_eq!(normalize_language(plain), None, "{plain}");
        }
    }

    /// `ts` deliberately has no alias: syntect's default set has no
    /// TypeScript, and aliasing it to JavaScript would mis-highlight the type
    /// syntax. It normalizes fine and then simply fails to resolve.
    #[test]
    fn typescript_is_not_aliased_to_javascript() {
        assert_eq!(normalize_language("ts"), Some("ts".into()));
    }
}
