use gpui::{Font, FontStyle, FontWeight, Hsla, SharedString, TextRun};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use syntect::highlighting::{HighlightIterator, HighlightState, Highlighter, Style, ThemeSet};
use syntect::parsing::{ParseState, ScopeStack, SyntaxSet};

struct SyntaxHighlighterInner {
    syntax_set: SyntaxSet,
    theme_set: ThemeSet,
    current_theme: String,
    /// Map of (language, line_number) -> the [`ParseState`] *after* parsing
    /// that line, i.e. the state the next line starts from.
    ///
    /// Two known limitations, neither fixed here: the key carries no document
    /// identity, so two documents in the same language overwrite each other's
    /// line 0 (this is why [`SyntaxHighlighter::highlight_block`] keeps its
    /// state local instead of reusing [`SyntaxHighlighter::highlight_line`]);
    /// and the map is only ever pruned by `reset_state` /
    /// `clear_state_from_line`, so it grows with the largest file highlighted.
    parse_states: HashMap<(String, usize), ParseState>,
    /// Map of (language, line_number) -> the [`HighlightState`] after that
    /// line. Same two limitations as `parse_states`.
    highlight_states: HashMap<(String, usize), HighlightState>,
}

#[derive(Clone)]
pub struct SyntaxHighlighter {
    inner: Rc<RefCell<SyntaxHighlighterInner>>,
}

impl SyntaxHighlighter {
    pub fn new() -> Self {
        let syntax_set = SyntaxSet::load_defaults_newlines();
        let theme_set = ThemeSet::load_defaults();

        // Get the first available theme as default, or use a fallback
        let current_theme = theme_set
            .themes
            .keys()
            .next()
            .cloned()
            .unwrap_or_else(|| "Default".to_string());

        Self {
            inner: Rc::new(RefCell::new(SyntaxHighlighterInner {
                syntax_set,
                theme_set,
                current_theme,
                parse_states: HashMap::new(),
                highlight_states: HashMap::new(),
            })),
        }
    }

    pub fn set_theme(&mut self, theme_name: &str) {
        let mut inner = self.inner.borrow_mut();
        if inner.theme_set.themes.contains_key(theme_name) {
            inner.current_theme = theme_name.to_string();
            inner.highlight_states.clear();
        }
    }

    /// The theme highlighting currently uses.
    pub fn current_theme(&self) -> String {
        self.inner.borrow().current_theme.clone()
    }

    /// Resolve a language token — a code fence's info string, a file
    /// extension, a syntax name — to the canonical syntax name
    /// [`highlight_block`](Self::highlight_block) expects, or `None` if
    /// nothing in the syntax set matches.
    ///
    /// `"rs"`, `"rust"` and `"Rust"` all resolve to `"Rust"`. The lowercased
    /// retry is not redundant: syntect's `find_syntax_by_token` matches
    /// *names* case-insensitively but *extensions* exactly, so `"JS"` misses
    /// on both passes without it.
    pub fn resolve_language(&self, token: &str) -> Option<String> {
        let token = token.trim();
        if token.is_empty() {
            return None;
        }

        let inner = self.inner.borrow();
        let lowered = token.to_ascii_lowercase();
        inner
            .syntax_set
            .find_syntax_by_token(token)
            .or_else(|| inner.syntax_set.find_syntax_by_token(&lowered))
            .map(|syntax| syntax.name.clone())
    }

    /// Highlight a whole block of text in one stateless pass.
    ///
    /// Unlike [`highlight_line`](Self::highlight_line), this keeps its parse
    /// and highlight state local to the call, so two blocks of the same
    /// language cannot contaminate each other (or a live editor) by both
    /// starting at line 0. Multi-line constructs — a block comment, a raw
    /// string — carry across lines *within* the block, which is the point.
    ///
    /// Returns sorted, disjoint byte ranges over `text`, each on a char
    /// boundary. Ranges syntect gave no opinion about are simply absent, so
    /// the caller's own text color shows through. Background colors are
    /// dropped: they are the syntect theme's block background, which would
    /// paint over the surface the code block already draws.
    ///
    /// `language` must be a canonical syntax name — see
    /// [`resolve_language`](Self::resolve_language). An unknown one yields no
    /// highlights rather than an error, so the caller renders plain.
    pub fn highlight_block(
        &self,
        text: &str,
        language: &str,
    ) -> Vec<(std::ops::Range<usize>, gpui::HighlightStyle)> {
        use syntect::util::LinesWithEndings;

        let inner = self.inner.borrow();

        let Some(syntax) = inner.syntax_set.find_syntax_by_name(language) else {
            return Vec::new();
        };
        let Some(theme) = inner
            .theme_set
            .themes
            .get(&inner.current_theme)
            .or_else(|| inner.theme_set.themes.values().next())
        else {
            return Vec::new();
        };

        let highlighter = Highlighter::new(theme);
        let mut parse_state = ParseState::new(syntax);
        let mut highlight_state = HighlightState::new(&highlighter, ScopeStack::new());

        let mut highlights: Vec<(std::ops::Range<usize>, gpui::HighlightStyle)> = Vec::new();
        let mut offset = 0usize;

        for line in LinesWithEndings::from(text) {
            // Parse once and highlight from that same op list. Parsing a line
            // twice would make it look to `parse_state` like it occurred twice
            // and corrupt every following line; doing it once is what keeps the
            // cross-line state honest.
            let ops = parse_state
                .parse_line(line, &inner.syntax_set)
                .unwrap_or_default();

            for (style, piece) in
                HighlightIterator::new(&mut highlight_state, &ops, line, &highlighter)
            {
                let start = offset;
                offset += piece.len();
                if piece.is_empty() {
                    continue;
                }

                let highlight = style_to_highlight(style);
                match highlights.last_mut() {
                    // Merge with the immediately preceding span when it is
                    // contiguous and identical, so the range list stays short
                    // and trivially sorted and disjoint.
                    Some((range, previous)) if range.end == start && *previous == highlight => {
                        range.end = offset;
                    }
                    _ => highlights.push((start..offset, highlight)),
                }
            }
        }

        highlights
    }

    pub fn available_themes(&self) -> Vec<String> {
        self.inner
            .borrow()
            .theme_set
            .themes
            .keys()
            .cloned()
            .collect()
    }

    pub fn detect_language(&self, text: &str, file_extension: Option<&str>) -> Option<String> {
        let inner = self.inner.borrow();
        if let Some(ext) = file_extension {
            if let Some(syntax) = inner.syntax_set.find_syntax_by_extension(ext) {
                return Some(syntax.name.clone());
            }
        }

        inner
            .syntax_set
            .find_syntax_by_first_line(text)
            .map(|s| s.name.clone())
    }

    /// Clear cached highlighting state from a specific line onward.
    /// This is useful for incremental re-highlighting when text changes.
    pub fn clear_state_from_line(&mut self, line_number: usize, language: &str) {
        let mut inner = self.inner.borrow_mut();

        // Clear parse states for this language from this line onward
        let states_to_remove: Vec<_> = inner
            .parse_states
            .keys()
            .filter(|(lang, line)| lang == language && *line >= line_number)
            .cloned()
            .collect();

        for key in states_to_remove {
            inner.parse_states.remove(&key);
        }

        // Clear highlight states that might be affected
        let highlight_states_to_remove: Vec<_> = inner
            .highlight_states
            .keys()
            .filter(|(lang, line)| lang == language && *line >= line_number)
            .cloned()
            .collect();

        for key in highlight_states_to_remove {
            inner.highlight_states.remove(&key);
        }
    }

    /// Reset all cached highlighting state.
    /// Call this when the buffer content has significantly changed.
    pub fn reset_state(&mut self) {
        let mut inner = self.inner.borrow_mut();
        inner.parse_states.clear();
        inner.highlight_states.clear();
    }

    /// Highlight one line, continuing from the state cached for the line
    /// before it.
    ///
    /// `line` is expected to carry its `\n`, the same convention
    /// [`highlight_block`](Self::highlight_block) gets from `LinesWithEndings`
    /// — the syntax set is `load_defaults_newlines`, whose grammars anchor
    /// rules to end of line. The one exception is a final line of text that
    /// does not end in one. [`GapBuffer::to_lines_with_endings`] produces
    /// exactly this shape; `to_lines` does not.
    ///
    /// The returned runs sum to `line.len()` — including the newline, so a
    /// caller shaping the line without it must trim them back (see
    /// [`Editor::highlight_line`](crate::editor::Editor::highlight_line)).
    ///
    /// [`GapBuffer::to_lines_with_endings`]: crate::editor::GapBuffer::to_lines_with_endings
    pub fn highlight_line(
        &mut self,
        line: &str,
        language: &str,
        line_number: usize,
        font_family: SharedString,
        _font_size: f32,
    ) -> Vec<TextRun> {
        // For multi-line parsing, we need to handle line content carefully
        // Some lines might just be empty or need to be parsed with previous context
        self.highlight_line_with_context(line, language, line_number, font_family, None)
    }

    fn highlight_line_with_context(
        &mut self,
        line: &str,
        language: &str,
        line_number: usize,
        font_family: SharedString,
        lines_context: Option<&[String]>,
    ) -> Vec<TextRun> {
        let mut inner = self.inner.borrow_mut();

        // First, check if we have the syntax
        let has_syntax = inner.syntax_set.find_syntax_by_name(language).is_some();
        if !has_syntax {
            // Fallback to plain text
            return vec![TextRun {
                len: line.len(),
                font: Font {
                    family: font_family,
                    features: Default::default(),
                    weight: FontWeight::NORMAL,
                    style: FontStyle::Normal,
                    fallbacks: Default::default(),
                },
                color: gpui::rgb(0xcccccc).into(),
                background_color: None,
                underline: None,
                strikethrough: None,
            }];
        }

        // Get or create parse state - we already checked syntax exists above
        let syntax = inner
            .syntax_set
            .find_syntax_by_name(language)
            .expect("syntax should exist after check above");

        // Get the parse state for the previous line, or build it up if needed
        let mut parse_state = if line_number == 0 {
            ParseState::new(syntax)
        } else {
            // Check if we have the state for the previous line
            let prev_line_key = (language.to_string(), line_number - 1);

            if let Some(state) = inner.parse_states.get(&prev_line_key) {
                state.clone()
            } else {
                // We need to build up the state from the beginning or from the last cached state
                let mut last_cached_line = None;
                for i in (0..line_number).rev() {
                    if inner.parse_states.contains_key(&(language.to_string(), i)) {
                        last_cached_line = Some(i);
                        break;
                    }
                }

                let mut state = if let Some(cached_line) = last_cached_line {
                    inner
                        .parse_states
                        .get(&(language.to_string(), cached_line))
                        .cloned()
                        .unwrap_or_else(|| ParseState::new(syntax))
                } else {
                    ParseState::new(syntax)
                };

                // Parse lines from last_cached_line+1 to line_number-1 if we have context
                if let Some(lines) = lines_context {
                    let start_line = last_cached_line.map_or(0, |l| l + 1);
                    for i in start_line..line_number {
                        if i < lines.len() {
                            let ops = state.parse_line(&lines[i], &inner.syntax_set);
                            if ops.is_ok() {
                                // Store intermediate states
                                inner
                                    .parse_states
                                    .insert((language.to_string(), i), state.clone());
                            }
                        }
                    }
                }

                state
            }
        };

        // Parse this line exactly once: the ops rendered below and the state
        // cached for the next line come out of the same call. Parsing a second
        // time "to update state" made the cached state the state after seeing
        // the line twice, so every multi-line construct handed the following
        // line a corrupt scope stack.
        //
        // This block sits above the theme lookup for a borrow reason, not a
        // stylistic one: `Highlighter::new(theme)` holds an immutable borrow of
        // `inner` across the rest of the function, so inserting into
        // `inner.parse_states` below it would need a `ParseState::clone` per
        // line to compile. Keep the parse and the insert adjacent and up here.
        let parse_result = parse_state.parse_line(line, &inner.syntax_set);
        let ops = match parse_result {
            Ok(ops) => {
                inner
                    .parse_states
                    .insert((language.to_string(), line_number), parse_state);
                ops
            }
            // A failed parse is not trusted: render nothing from it and leave
            // the cache untouched, as before.
            Err(_) => Vec::new(),
        };

        // Get the theme, with fallback to default colors if theme not found
        let theme = inner
            .theme_set
            .themes
            .get(&inner.current_theme)
            .or_else(|| inner.theme_set.themes.values().next());

        if theme.is_none() {
            // No themes available at all, return plain text
            return vec![TextRun {
                len: line.len(),
                font: Font {
                    family: font_family,
                    features: Default::default(),
                    weight: FontWeight::NORMAL,
                    style: FontStyle::Normal,
                    fallbacks: Default::default(),
                },
                color: gpui::rgb(0xcccccc).into(),
                background_color: None,
                underline: None,
                strikethrough: None,
            }];
        }

        let theme = theme.expect("theme should exist after check above");
        let highlighter = Highlighter::new(theme);

        let mut highlight_state = if line_number == 0 {
            HighlightState::new(&highlighter, ScopeStack::new())
        } else if let Some(state) = inner
            .highlight_states
            .get(&(language.to_string(), line_number - 1))
        {
            state.clone()
        } else {
            HighlightState::new(&highlighter, ScopeStack::new())
        };

        let mut text_runs = Vec::new();
        let mut current_pos = 0;

        let ranges: Vec<(Style, usize, usize)> =
            HighlightIterator::new(&mut highlight_state, &ops, line, &highlighter)
                .map(|(style, text)| {
                    let start = current_pos;
                    let end = current_pos + text.len();
                    current_pos = end;
                    (style, start, end)
                })
                .collect();

        for (style, start, end) in ranges {
            let len = end - start;
            if len == 0 {
                continue;
            }

            let color = style_to_hsla(style);
            let (weight, font_style) = get_font_style(style);

            text_runs.push(TextRun {
                len,
                font: Font {
                    family: font_family.clone(),
                    features: Default::default(),
                    weight,
                    style: font_style,
                    fallbacks: Default::default(),
                },
                color,
                background_color: if style.background != style.foreground {
                    Some(style_color_to_hsla(style.background))
                } else {
                    None
                },
                underline: if style
                    .font_style
                    .contains(syntect::highlighting::FontStyle::UNDERLINE)
                {
                    Some(Default::default())
                } else {
                    None
                },
                strikethrough: None,
            });
        }

        if text_runs.is_empty() {
            text_runs.push(TextRun {
                len: line.len(),
                font: Font {
                    family: font_family,
                    features: Default::default(),
                    weight: FontWeight::NORMAL,
                    style: FontStyle::Normal,
                    fallbacks: Default::default(),
                },
                color: gpui::rgb(0xcccccc).into(),
                background_color: None,
                underline: None,
                strikethrough: None,
            });
        }

        // Store highlight state for this line
        inner
            .highlight_states
            .insert((language.to_string(), line_number), highlight_state);

        text_runs
    }

    /// Ensure parse states exist up to a given line by parsing from the
    /// beginning if needed.
    ///
    /// `lines` must be on the same newline convention as
    /// [`highlight_line`](Self::highlight_line): every line carries its `\n`,
    /// except a final line of text that does not end in one. Feed it
    /// [`GapBuffer::to_lines_with_endings`], not `to_lines` — a state built
    /// from separator-less lines is the state of a *different* document, and
    /// the two disagree wherever a grammar rule is anchored to end of line.
    ///
    /// [`GapBuffer::to_lines_with_endings`]: crate::editor::GapBuffer::to_lines_with_endings
    pub fn ensure_parse_states(&mut self, language: &str, up_to_line: usize, lines: &[String]) {
        let mut inner = self.inner.borrow_mut();

        let syntax = match inner.syntax_set.find_syntax_by_name(language) {
            Some(s) => s,
            None => return,
        };

        // Find the last cached state before up_to_line
        let mut last_cached_line = None;
        for i in (0..=up_to_line).rev() {
            if inner.parse_states.contains_key(&(language.to_string(), i)) {
                last_cached_line = Some(i);
                break;
            }
        }

        // Build up states from the last cached line (or from the beginning)
        let start_line = last_cached_line.map_or(0, |l| l + 1);
        let mut parse_state = if let Some(cached_line) = last_cached_line {
            inner
                .parse_states
                .get(&(language.to_string(), cached_line))
                .cloned()
                .unwrap_or_else(|| ParseState::new(syntax))
        } else {
            ParseState::new(syntax)
        };

        for i in start_line..=up_to_line {
            if i >= lines.len() {
                break;
            }

            let ops = parse_state.parse_line(&lines[i], &inner.syntax_set);
            if ops.is_ok() {
                inner
                    .parse_states
                    .insert((language.to_string(), i), parse_state.clone());
            }
        }
    }

    pub fn get_theme_background(&self) -> Hsla {
        let inner = self.inner.borrow();
        inner
            .theme_set
            .themes
            .get(&inner.current_theme)
            .and_then(|theme| theme.settings.background)
            .map(style_color_to_hsla)
            .unwrap_or_else(|| gpui::rgb(0x1e1e1e).into())
    }

    pub fn get_theme_foreground(&self) -> Hsla {
        let inner = self.inner.borrow();
        inner
            .theme_set
            .themes
            .get(&inner.current_theme)
            .and_then(|theme| theme.settings.foreground)
            .map(style_color_to_hsla)
            .unwrap_or_else(|| gpui::rgb(0xcccccc).into())
    }

    pub fn get_theme_gutter_background(&self) -> Hsla {
        let inner = self.inner.borrow();
        inner
            .theme_set
            .themes
            .get(&inner.current_theme)
            .and_then(|theme| {
                theme.settings.gutter.map(style_color_to_hsla).or_else(|| {
                    theme.settings.background.map(|bg| {
                        // Darken background slightly for gutter
                        let mut hsla: Hsla = style_color_to_hsla(bg);
                        hsla.l = (hsla.l * 0.95).max(0.0);
                        hsla
                    })
                })
            })
            .unwrap_or_else(|| gpui::rgb(0x252525).into())
    }

    pub fn get_theme_line_highlight(&self) -> Hsla {
        let inner = self.inner.borrow();
        inner
            .theme_set
            .themes
            .get(&inner.current_theme)
            .and_then(|theme| theme.settings.line_highlight)
            .map(|color| {
                let mut hsla = style_color_to_hsla(color);
                hsla.a = hsla.a.min(0.3); // Make semi-transparent
                hsla
            })
            .unwrap_or_else(|| gpui::rgba(0x2a2a2aff).into())
    }

    pub fn get_theme_selection(&self) -> Hsla {
        let inner = self.inner.borrow();
        inner
            .theme_set
            .themes
            .get(&inner.current_theme)
            .and_then(|theme| theme.settings.selection)
            .map(|color| {
                let mut hsla = style_color_to_hsla(color);
                hsla.a = hsla.a.min(0.5); // Make semi-transparent
                hsla
            })
            .unwrap_or_else(|| gpui::rgba(0x3e4451aa).into())
    }

    // Load custom themes from a directory
    // Example: highlighter.load_theme_from_file("./themes/my-theme.tmTheme")
    #[allow(dead_code)]
    pub fn load_theme_from_file(&mut self, path: &str) -> Result<(), String> {
        use std::fs::File;
        use std::io::BufReader;

        let file = File::open(path).map_err(|e| format!("Failed to open theme file: {}", e))?;
        let reader = BufReader::new(file);

        let theme = syntect::highlighting::ThemeSet::load_from_reader(&mut BufReader::new(reader))
            .map_err(|e| format!("Failed to parse theme: {}", e))?;

        let theme_name = std::path::Path::new(path)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("custom")
            .to_string();

        let mut inner = self.inner.borrow_mut();
        inner.theme_set.themes.insert(theme_name.clone(), theme);
        inner.current_theme = theme_name;

        Ok(())
    }

    // Load custom syntax definitions
    // Example: highlighter.load_syntax_from_file("./syntaxes/mylang.sublime-syntax")
    #[allow(dead_code)]
    pub fn load_syntax_from_file(&mut self, path: &str) -> Result<(), String> {
        let mut inner = self.inner.borrow_mut();
        let mut builder = syntect::parsing::SyntaxSetBuilder::new();
        builder
            .add_from_folder(path, true)
            .map_err(|e| format!("Failed to load syntax: {}", e))?;

        // Merge with existing syntaxes
        for _syntax in inner.syntax_set.syntaxes() {
            builder.add_plain_text_syntax();
        }

        inner.syntax_set = builder.build();
        inner.parse_states.clear();
        inner.highlight_states.clear();

        Ok(())
    }
}

fn style_color_to_hsla(color: syntect::highlighting::Color) -> Hsla {
    gpui::rgba(
        ((color.r as u32) << 24)
            | ((color.g as u32) << 16)
            | ((color.b as u32) << 8)
            | (color.a as u32),
    )
    .into()
}

fn style_to_hsla(style: Style) -> Hsla {
    style_color_to_hsla(style.foreground)
}

/// A syntect span style as a gpui [`HighlightStyle`](gpui::HighlightStyle).
///
/// Every field stays `None` unless syntect actually asked for it, so a span
/// the theme has no opinion about inherits the surrounding text's style.
/// `background_color` is deliberately never set: syntect puts the theme's own
/// block background on *every* span, which would paint over the code block's
/// surface and fight the selection highlight.
fn style_to_highlight(style: Style) -> gpui::HighlightStyle {
    use syntect::highlighting::FontStyle as SyntectFontStyle;

    gpui::HighlightStyle {
        color: Some(style_to_hsla(style)),
        font_weight: style
            .font_style
            .contains(SyntectFontStyle::BOLD)
            .then_some(FontWeight::BOLD),
        font_style: style
            .font_style
            .contains(SyntectFontStyle::ITALIC)
            .then_some(FontStyle::Italic),
        underline: style
            .font_style
            .contains(SyntectFontStyle::UNDERLINE)
            .then(Default::default),
        ..Default::default()
    }
}

fn get_font_style(style: Style) -> (FontWeight, FontStyle) {
    let weight = if style
        .font_style
        .contains(syntect::highlighting::FontStyle::BOLD)
    {
        FontWeight::BOLD
    } else {
        FontWeight::NORMAL
    };

    let font_style = if style
        .font_style
        .contains(syntect::highlighting::FontStyle::ITALIC)
    {
        FontStyle::Italic
    } else {
        FontStyle::Normal
    };

    (weight, font_style)
}

impl Default for SyntaxHighlighter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syntect::util::LinesWithEndings;

    /// The colour each byte of a line ends up rendered in.
    fn run_colors(runs: &[TextRun]) -> Vec<Option<Hsla>> {
        runs.iter()
            .flat_map(|run| std::iter::repeat(Some(run.color)).take(run.len))
            .collect()
    }

    /// The colour each byte of `text` gets from the stateless block pass, which
    /// parses every line exactly once. This is the oracle: it is the same
    /// grammar, the same theme and the same [`HighlightIterator`], differing
    /// only in how the cross-line state is carried.
    fn block_colors(
        highlighter: &SyntaxHighlighter,
        text: &str,
        language: &str,
    ) -> Vec<Option<Hsla>> {
        let mut colors = vec![None; text.len()];
        for (range, style) in highlighter.highlight_block(text, language) {
            for slot in &mut colors[range] {
                *slot = style.color;
            }
        }
        colors
    }

    /// The colour each byte of `text` gets from feeding it to
    /// [`SyntaxHighlighter::highlight_line`] one line at a time, exactly as the
    /// editor's paint loop does. Lines keep their newline: the syntax set is
    /// `load_defaults_newlines`.
    fn line_by_line_colors(text: &str, language: &str) -> Vec<Option<Hsla>> {
        let mut highlighter = SyntaxHighlighter::new();
        LinesWithEndings::from(text)
            .enumerate()
            .flat_map(|(index, line)| {
                let runs = highlighter.highlight_line(line, language, index, "test".into(), 14.0);
                run_colors(&runs)
            })
            .collect()
    }

    /// Where the two passes first disagree, as a message a human can act on.
    fn describe_mismatch(text: &str, expected: &[Option<Hsla>], actual: &[Option<Hsla>]) -> String {
        let at = expected
            .iter()
            .zip(actual)
            .position(|(a, b)| a != b)
            .unwrap_or_else(|| expected.len().min(actual.len()));
        let line = text[..at.min(text.len())].lines().count();
        format!(
            "first difference at byte {at} (line {line}), \
             block pass says {:?} but line-by-line says {:?}",
            expected.get(at),
            actual.get(at),
        )
    }

    /// The regression from #135, in the language the issue named. A block
    /// comment opens on one line, carries across another and closes; the code
    /// after it must be highlighted as code.
    ///
    /// The doubled parse did *not* leave the trailing line comment-coloured —
    /// it flattened it to one plain colour — so "is not comment-coloured" is
    /// not a sufficient assertion on its own. The distinct-colour count is what
    /// actually discriminates.
    #[test]
    fn block_comment_opens_carries_and_closes() {
        let text = "const before = 1;\n\
                    /* opening a block comment\n\
                    still inside the comment */\n\
                    function after() { return 2; }\n";

        let colors = line_by_line_colors(text, "JavaScript");
        let per_line: Vec<Vec<Option<Hsla>>> = text
            .lines()
            .zip(offsets_by_line(text))
            .map(|(line, start)| colors[start..start + line.len()].to_vec())
            .collect();

        // Line 1 opens the comment, line 2 closes it, line 3 is code again.
        let comment_color = per_line[1][0];
        let code = &per_line[3];
        assert!(
            code.iter().all(|color| *color != comment_color),
            "the line after `*/` is still painted in the comment colour"
        );

        let distinct: std::collections::HashSet<_> =
            code.iter().map(|color| format!("{color:?}")).collect();
        assert!(
            distinct.len() >= 2,
            "the line after `*/` collapsed to {} colour(s); `function` and \
             `after` should not share the plain-text colour",
            distinct.len()
        );
    }

    /// Byte offset of the start of each line of `text`.
    fn offsets_by_line(text: &str) -> Vec<usize> {
        let mut offsets = Vec::new();
        let mut offset = 0;
        for line in LinesWithEndings::from(text) {
            offsets.push(offset);
            offset += line.len();
        }
        offsets
    }

    /// The general form of the bug: highlighting a document line by line must
    /// agree, byte for byte, with highlighting it in one stateless pass.
    ///
    /// The fixtures are not interchangeable. A plain Rust `/* … */` does *not*
    /// diverge: Rust's block comments nest, so the doubled parse just doubles
    /// the depth, and a balanced comment still returns to depth zero on the
    /// same line the honest parse does. The Rust fixture below is deliberately
    /// *unbalanced* — two opens and one close on one line — which is what makes
    /// the doubled depth outlive the closing `*/`. Each fixture here was
    /// checked to diverge (or, for the last two, to keep agreeing) with the
    /// second parse temporarily put back.
    #[test]
    fn line_by_line_matches_the_stateless_block_pass() {
        let fixtures: &[(&str, &str)] = &[
            (
                "JavaScript",
                "const before = 1;\n/* comment\nstill comment */\nfunction after() {}\n",
            ),
            (
                "Rust",
                "fn before() {}\n/* a /* b */\nstill commented */ fn after() {}\n",
            ),
            (
                "Rust",
                "fn main() {\n    let s = r#\"a raw\nstring across lines\"#;\n    let n = 1;\n}\n",
            ),
            (
                "Python",
                "x = 1\ndoc = '''a triple\nquoted string'''\ny = 2\n",
            ),
            // Single-line-only fixtures: these agreed before the fix and must
            // keep agreeing after it.
            ("Rust", "// just a line comment\nfn after() {}\n"),
            ("C", "int before = 1;\nint after = 2;\n"),
        ];

        let highlighter = SyntaxHighlighter::new();
        let mut disagreements = Vec::new();
        for (language, text) in fixtures {
            let expected = block_colors(&highlighter, text, language);
            let actual = line_by_line_colors(text, language);
            if actual != expected {
                disagreements.push(format!(
                    "{language} fixture {text:?}: {}",
                    describe_mismatch(text, &expected, &actual)
                ));
            }
        }

        assert!(
            disagreements.is_empty(),
            "line-by-line highlighting diverged from the block pass:\n{}",
            disagreements.join("\n")
        );
    }

    /// Rendering a single line on its own is unchanged by the fix — the ops the
    /// runs are built from come from the first parse either way.
    #[test]
    fn a_lone_line_is_unaffected() {
        let text = "fn main() { let x = 42; }\n";
        let highlighter = SyntaxHighlighter::new();
        assert_eq!(
            line_by_line_colors(text, "Rust"),
            block_colors(&highlighter, text, "Rust")
        );
    }

    /// The early return above the parse: an unknown language renders one plain
    /// run and caches nothing.
    #[test]
    fn unknown_language_falls_back_to_plain_text() {
        let mut highlighter = SyntaxHighlighter::new();
        let line = "some text in no language at all\n";
        let runs = highlighter.highlight_line(line, "Nonexistent Language", 0, "test".into(), 14.0);

        assert_eq!(runs.len(), 1);
        assert_eq!(runs[0].len, line.len());
        assert_eq!(runs[0].color, gpui::rgb(0xcccccc).into());
        assert!(highlighter.inner.borrow().parse_states.is_empty());
    }
}

// HOW TO ADD CUSTOM GRAMMARS AND THEMES:
//
// 1. THEMES:
//    Themes use the TextMate .tmTheme format (XML plist files).
//    You can get themes from:
//    - https://github.com/textmate/themes
//    - VSCode themes (extract from .vsix)
//    - Sublime Text packages
//
//    To use a custom theme:
//    highlighter.load_theme_from_file("./my-theme.tmTheme").ok();
//
// 2. SYNTAX DEFINITIONS:
//    Syntaxes use Sublime Text's .sublime-syntax format (YAML).
//    You can get syntax definitions from:
//    - https://github.com/sublimehq/Packages
//    - Convert TextMate grammars (.tmLanguage) to Sublime syntax
//
//    To use custom syntax:
//    highlighter.load_syntax_from_file("./syntaxes/").ok();
//
// 3. BUNDLED SYNTAXES:
//    Syntect includes these by default:
//    - Rust, Python, JavaScript, TypeScript, Java, C, C++, C#
//    - Go, Ruby, PHP, Swift, Kotlin, Scala, Haskell
//    - HTML, CSS, JSON, XML, YAML, Markdown
//    - Shell scripts, Dockerfile, SQL, and many more
//
// 4. BUNDLED THEMES:
//    Default themes from syntect include:
//    - base16-ocean.dark, base16-ocean.light
//    - base16-mocha.dark, base16-eighties.dark
//    - InspiredGitHub, Solarized (dark), Solarized (light)
