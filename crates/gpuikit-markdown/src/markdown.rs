//! Markdown rendering for gpuikit
//!
//! This crate provides markdown parsing and rendering using GPUI elements.
//! It supports CommonMark and GitHub Flavored Markdown.
//!
//! # Example
//!
//! ```ignore
//! use gpuikit_markdown::{markdown, MarkdownStyle};
//!
//! // Simple usage - create markdown element inline
//! div().child(markdown("# Hello\n\nThis is **bold** text.", cx))
//!
//! // With custom style
//! div().child(
//!     markdown("# Hello", cx)
//!         .style(MarkdownStyle::new().code_font("Monaco"))
//! )
//! ```

mod elements;
mod inline_style;
mod parser;
mod style;

pub use elements::*;
pub use inline_style::*;
pub use parser::*;
pub use style::*;

use gpui::{
    div, prelude::*, rems, App, Context, Entity, IntoElement, ParentElement, SharedString, Styled,
    Window,
};
use pulldown_cmark::{Event, Tag, TagEnd};

/// A markdown document that can be rendered as a GPUI element.
///
/// This entity parses and holds markdown content, ready for rendering.
pub struct Markdown {
    source: SharedString,
    events: Vec<MarkdownEvent>,
}

/// Parsed markdown event with source range information.
#[derive(Clone, Debug)]
pub struct MarkdownEvent {
    pub event: Event<'static>,
    pub source_range: std::ops::Range<usize>,
}

impl Markdown {
    /// Create a new Markdown instance from source text.
    pub fn new(source: impl Into<SharedString>, _cx: &mut Context<Self>) -> Self {
        let source: SharedString = source.into();
        let events = Self::parse(&source);
        Self { source, events }
    }

    /// Get the source text.
    pub fn source(&self) -> &str {
        &self.source
    }

    /// Update the markdown content.
    pub fn set_source(&mut self, source: impl Into<SharedString>, cx: &mut Context<Self>) {
        self.source = source.into();
        self.events = Self::parse(&self.source);
        cx.notify();
    }

    fn parse(source: &str) -> Vec<MarkdownEvent> {
        let options = Options::ENABLE_TABLES
            | Options::ENABLE_FOOTNOTES
            | Options::ENABLE_STRIKETHROUGH
            | Options::ENABLE_TASKLISTS
            | Options::ENABLE_GFM;

        let parser = Parser::new_ext(source, options);

        parser
            .into_offset_iter()
            .map(|(event, range)| MarkdownEvent {
                event: event.into_static(),
                source_range: range,
            })
            .collect()
    }

    /// Get the parsed events.
    pub fn events(&self) -> &[MarkdownEvent] {
        &self.events
    }
}

/// Element for rendering markdown content.
#[derive(IntoElement)]
pub struct MarkdownElement {
    markdown: Entity<Markdown>,
    style: MarkdownStyle,
}

/// Create a markdown element from source text.
///
/// This is a convenience function that creates the entity and element in one step.
/// For more control, use `Markdown::new()` and `MarkdownElement::new()` separately.
pub fn markdown(source: impl Into<SharedString>, cx: &mut App) -> MarkdownElement {
    let entity = cx.new(|cx| Markdown::new(source, cx));
    MarkdownElement::new(entity)
}

impl MarkdownElement {
    /// Create a new markdown element with default styling.
    pub fn new(markdown: Entity<Markdown>) -> Self {
        Self {
            markdown,
            style: MarkdownStyle::default(),
        }
    }

    /// Set a custom style for the markdown.
    pub fn style(mut self, style: MarkdownStyle) -> Self {
        self.style = style;
        self
    }
}

impl RenderOnce for MarkdownElement {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let markdown = self.markdown.read(cx);
        let events = markdown.events.clone();
        let style = self.style.clone();

        let renderer = MarkdownRenderer::new(style);
        renderer.render_events(&events, cx)
    }
}

/// Internal renderer that builds the element tree from markdown events.
struct MarkdownRenderer {
    style: MarkdownStyle,
    elements: Vec<gpui::AnyElement>,

    // State tracking
    in_heading: Option<HeadingLevel>,
    in_code_block: bool,
    in_block_quote: bool,
    in_link: Option<LinkContext>,
    in_image: Option<ImageContext>,
    list_stack: Vec<ListContext>,
    link_counter: usize,

    // Rich text tracking
    current_text: RichText,
    active_style: InlineStyle,
}

#[derive(Clone, Debug)]
struct LinkContext {
    url: String,
}

#[derive(Clone, Debug)]
struct ImageContext {
    url: String,
    alt: String,
}

#[derive(Clone, Debug)]
struct ListContext {
    ordered: bool,
    current_index: u64,
}

impl MarkdownRenderer {
    fn new(style: MarkdownStyle) -> Self {
        Self {
            style,
            elements: Vec::new(),
            in_heading: None,
            in_code_block: false,
            in_block_quote: false,
            in_link: None,
            in_image: None,
            list_stack: Vec::new(),
            link_counter: 0,
            current_text: RichText::new(),
            active_style: InlineStyle::default(),
        }
    }

    fn render_events(mut self, events: &[MarkdownEvent], cx: &App) -> impl IntoElement {
        for event in events {
            self.handle_event(&event.event, cx);
        }

        div()
            .flex()
            .flex_col()
            .gap(rems(self.style.block_spacing))
            .children(self.elements)
    }

    fn handle_event(&mut self, event: &Event<'static>, cx: &App) {
        match event {
            Event::Start(tag) => self.handle_start_tag(tag),
            Event::End(tag) => self.handle_end_tag(tag, cx),
            Event::Text(text) => self.handle_text(text),
            Event::Code(code) => self.handle_inline_code(code),
            Event::SoftBreak => self.current_text.push(" ", self.active_style),
            Event::HardBreak => self.current_text.push("\n", self.active_style),
            Event::Rule => self.push_divider(cx),
            Event::TaskListMarker(checked) => self.handle_task_marker(*checked),
            Event::Html(_) | Event::InlineHtml(_) => {
                // TODO: HTML rendering
            }
            Event::FootnoteReference(_) | Event::InlineMath(_) | Event::DisplayMath(_) => {
                // TODO: Footnotes and math
            }
        }
    }

    fn handle_start_tag(&mut self, tag: &Tag<'static>) {
        match tag {
            Tag::Paragraph => {}
            Tag::Heading { level, .. } => {
                self.in_heading = Some((*level).into());
            }
            Tag::BlockQuote(_) => {
                self.in_block_quote = true;
            }
            Tag::CodeBlock(_kind) => {
                self.in_code_block = true;
            }
            Tag::List(start) => {
                self.list_stack.push(ListContext {
                    ordered: start.is_some(),
                    current_index: start.unwrap_or(1),
                });
            }
            Tag::Item => {}
            Tag::Emphasis => {
                self.active_style.italic = true;
            }
            Tag::Strong => {
                self.active_style.bold = true;
            }
            Tag::Strikethrough => {
                self.active_style.strikethrough = true;
            }
            Tag::Link { dest_url, .. } => {
                self.in_link = Some(LinkContext {
                    url: dest_url.to_string(),
                });
            }
            Tag::Image {
                dest_url, title, ..
            } => {
                self.in_image = Some(ImageContext {
                    url: dest_url.to_string(),
                    alt: title.to_string(),
                });
            }
            Tag::Table(_) | Tag::TableHead | Tag::TableRow | Tag::TableCell => {
                // TODO: Table rendering
            }
            Tag::FootnoteDefinition(_)
            | Tag::MetadataBlock(_)
            | Tag::DefinitionList
            | Tag::DefinitionListTitle
            | Tag::DefinitionListDefinition
            | Tag::HtmlBlock => {
                // TODO: These elements
            }
        }
    }

    fn handle_end_tag(&mut self, tag: &TagEnd, cx: &App) {
        match tag {
            TagEnd::Paragraph => {
                self.flush_paragraph(cx);
            }
            TagEnd::Heading(level) => {
                let heading_level: elements::HeadingLevel = (*level).into();
                self.in_heading = None;
                self.flush_heading(heading_level, cx);
            }
            TagEnd::BlockQuote(_) => {
                self.in_block_quote = false;
                self.flush_block_quote(cx);
            }
            TagEnd::CodeBlock => {
                self.in_code_block = false;
                self.flush_code_block(cx);
            }
            TagEnd::List(_) => {
                self.list_stack.pop();
            }
            TagEnd::Item => {
                self.flush_list_item(cx);
            }
            TagEnd::Emphasis => {
                self.active_style.italic = false;
            }
            TagEnd::Strong => {
                self.active_style.bold = false;
            }
            TagEnd::Strikethrough => {
                self.active_style.strikethrough = false;
            }
            TagEnd::Link => {
                self.flush_link(cx);
            }
            TagEnd::Image => {
                self.flush_image(cx);
            }
            TagEnd::Table | TagEnd::TableHead | TagEnd::TableRow | TagEnd::TableCell => {
                // TODO: Table rendering
            }
            TagEnd::FootnoteDefinition
            | TagEnd::MetadataBlock(_)
            | TagEnd::DefinitionList
            | TagEnd::DefinitionListTitle
            | TagEnd::DefinitionListDefinition
            | TagEnd::HtmlBlock => {
                // TODO: These elements
            }
        }
    }

    fn handle_text(&mut self, text: &str) {
        // If we're in an image tag, this is the alt text
        if let Some(ref mut img_ctx) = self.in_image {
            img_ctx.alt = text.to_string();
        } else {
            self.current_text.push(text, self.active_style);
        }
    }

    fn handle_inline_code(&mut self, code: &str) {
        // For now, just add as text with backticks
        // TODO: Use inline_code element when we support inline element mixing
        self.current_text.push("`", self.active_style);
        self.current_text.push(code, self.active_style);
        self.current_text.push("`", self.active_style);
    }

    fn handle_task_marker(&mut self, checked: bool) {
        let marker = if checked { "☑ " } else { "☐ " };
        self.current_text.push(marker, self.active_style);
    }

    fn flush_paragraph(&mut self, cx: &App) {
        if self.current_text.is_empty() {
            return;
        }

        let rich_text = std::mem::take(&mut self.current_text);
        let element = elements::rich_paragraph(&rich_text, &self.style.body, cx);
        self.elements.push(element.into_any_element());
    }

    fn flush_heading(&mut self, level: HeadingLevel, cx: &App) {
        if self.current_text.is_empty() {
            return;
        }

        let rich_text = std::mem::take(&mut self.current_text);
        let heading_style = match level {
            elements::HeadingLevel::H1 => &self.style.h1,
            elements::HeadingLevel::H2 => &self.style.h2,
            elements::HeadingLevel::H3 => &self.style.h3,
            elements::HeadingLevel::H4 => &self.style.h4,
            elements::HeadingLevel::H5 => &self.style.h5,
            elements::HeadingLevel::H6 => &self.style.h6,
        };

        let element = elements::rich_heading(&rich_text, heading_style, cx);
        self.elements.push(element.into_any_element());
    }

    fn flush_block_quote(&mut self, cx: &App) {
        if self.current_text.is_empty() {
            return;
        }

        let rich_text = std::mem::take(&mut self.current_text);
        let element = elements::rich_block_quote(
            &rich_text,
            &self.style.body,
            self.style.block_quote_border,
            self.style.block_quote_text,
            cx,
        );
        self.elements.push(element.into_any_element());
    }

    fn flush_code_block(&mut self, cx: &App) {
        if self.current_text.is_empty() {
            return;
        }

        // Code blocks don't use rich text - they're monospace plain text
        let text = self.current_text.to_plain_text();
        self.current_text.clear();

        let element = elements::code_block(
            text,
            None, // TODO: Track language from CodeBlock tag
            &self.style.code,
            &self.style.code_font_family,
            self.style.code_block_bg,
            self.style.code_block_border,
            cx,
        );
        self.elements.push(element.into_any_element());
    }

    fn flush_list_item(&mut self, cx: &App) {
        if self.current_text.is_empty() {
            return;
        }

        let rich_text = std::mem::take(&mut self.current_text);

        let marker = if let Some(list_ctx) = self.list_stack.last_mut() {
            if list_ctx.ordered {
                let marker = elements::ordered_marker(list_ctx.current_index);
                list_ctx.current_index += 1;
                marker
            } else {
                elements::unordered_marker()
            }
        } else {
            elements::unordered_marker()
        };

        let indent_level = self.list_stack.len().saturating_sub(1);
        let element =
            elements::rich_list_item(&rich_text, marker, indent_level, &self.style.body, cx);
        self.elements.push(element.into_any_element());
    }

    fn flush_link(&mut self, cx: &App) {
        let link_ctx = match self.in_link.take() {
            Some(ctx) => ctx,
            None => return,
        };

        if self.current_text.is_empty() {
            return;
        }

        // Links use plain text for now (the link styling takes precedence)
        let text = self.current_text.to_plain_text();
        self.current_text.clear();

        let id: SharedString = format!("md-link-{}", self.link_counter).into();
        self.link_counter += 1;

        let element = elements::link(id, text, link_ctx.url.into(), self.style.link_color, cx);
        self.elements.push(element.into_any_element());
    }

    fn flush_image(&mut self, cx: &App) {
        let img_ctx = match self.in_image.take() {
            Some(ctx) => ctx,
            None => return,
        };

        // Clear any collected text (alt text is stored in img_ctx)
        self.current_text.clear();

        let alt = if img_ctx.alt.is_empty() {
            None
        } else {
            Some(img_ctx.alt.as_str())
        };

        let element = elements::image(img_ctx.url, alt, cx);
        self.elements.push(element.into_any_element());
    }

    fn push_divider(&mut self, cx: &App) {
        let element = elements::divider(self.style.rule_color, cx);
        self.elements.push(element.into_any_element());
    }
}
