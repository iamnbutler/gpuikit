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
    parse_states: HashMap<String, ParseState>,
    highlight_states: HashMap<String, HighlightState>,
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
        // Since we don't track line numbers in parse_states directly,
        // we need to clear it entirely for now
        // TODO: Improve this to track line-specific states
        if line_number == 0 {
            inner.parse_states.remove(language);
        }

        // Clear highlight states that might be affected
        let cache_key = format!("{}-{}", language, inner.current_theme);
        if line_number == 0 {
            inner.highlight_states.remove(&cache_key);
        }
    }

    /// Reset all cached highlighting state.
    /// Call this when the buffer content has significantly changed.
    pub fn reset_state(&mut self) {
        let mut inner = self.inner.borrow_mut();
        inner.parse_states.clear();
        inner.highlight_states.clear();
    }

    pub fn highlight_line(
        &mut self,
        line: &str,
        language: &str,
        line_number: usize,
        font_family: SharedString,
        font_size: f32,
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

        let cache_key = format!("{}-{}", language, inner.current_theme);
        let parse_state_key = language.to_string();

        // Clear states if starting fresh
        if line_number == 0 {
            inner.parse_states.remove(&parse_state_key);
            inner.highlight_states.remove(&cache_key);
        }

        // Get or create parse state - we already checked syntax exists above
        let syntax = inner
            .syntax_set
            .find_syntax_by_name(language)
            .expect("syntax should exist after check above");

        let mut parse_state = if line_number == 0 {
            ParseState::new(syntax)
        } else if let Some(state) = inner.parse_states.get(&parse_state_key) {
            state.clone()
        } else {
            ParseState::new(syntax)
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

        let ops = parse_state
            .parse_line(line, &inner.syntax_set)
            .unwrap_or_default();

        let mut highlight_state = if line_number == 0 {
            HighlightState::new(&highlighter, ScopeStack::new())
        } else if let Some(state) = inner.highlight_states.get(&cache_key) {
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

        // Store parse state for next line
        let new_parse_state = parse_state
            .parse_line(line, &inner.syntax_set)
            .map(|_| parse_state.clone())
            .unwrap_or_else(|_| ParseState::new(syntax));
        inner.parse_states.insert(parse_state_key, new_parse_state);

        // Store highlight state for next line - it was already mutated by the iterator
        inner.highlight_states.insert(cache_key, highlight_state);

        text_runs
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
        for syntax in inner.syntax_set.syntaxes() {
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
