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
mod selectable_text;
mod selection;
mod stitch;
mod style;

pub use elements::*;
pub use inline_style::*;
pub use parser::*;
pub use selectable_text::{RunRole, SelectableText};
pub use selection::{MarkdownSelection, SelectionPosition};
pub use stitch::preprocessing_available;
pub use style::*;

use crate::theme::{ActiveTheme, Themeable};
use gpui::{
    div, prelude::*, rems, App, Context, ElementId, Entity, EntityId, IntoElement, ParentElement,
    Role, SharedString, Styled, Task, Window,
};
use pulldown_cmark::{Alignment, Event, Tag, TagEnd};

/// The id of the element every run of one document hangs under.
///
/// gpui hashes an element's *whole* id path into an accessibility node id and
/// refuses duplicates, so run ids only need to be unique within a document if
/// the document itself is uniquely identified. Keyed on the entity so it is
/// also stable across frames: assistive technology reads a changed node id as
/// a different element.
fn document_element_id(entity_id: EntityId) -> ElementId {
    ElementId::NamedInteger("md-doc".into(), entity_id.as_u64())
}

/// The id of one text run, in document order. Only unique underneath a
/// [`document_element_id`].
fn run_element_id(index: usize) -> ElementId {
    ElementId::NamedInteger("md-run".into(), index as u64)
}

/// A markdown document that can be rendered as a GPUI element.
///
/// This entity parses and holds markdown content, ready for rendering.
///
/// # Streaming
///
/// Content arriving a piece at a time — an LLM reply, a log tail — goes in
/// through [`append`](Self::append), which extends the source and re-parses on
/// a background thread. The previously parsed events keep rendering until the
/// new parse lands, so the document never blanks, and deltas arriving during a
/// parse coalesce into a single follow-up parse rather than one each.
///
/// ```ignore
/// markdown.update(cx, |markdown, cx| markdown.append(&delta, cx));
/// ```
pub struct Markdown {
    source: SharedString,
    /// The source the current [`events`](Self::events) were parsed from. Lags
    /// [`source`](Self::source) while a parse is in flight.
    parsed_source: SharedString,
    events: Vec<MarkdownEvent>,
    /// Document-wide text selection, shared with every rendered run.
    selection: MarkdownSelection,
    parse_state: ParseState,
    /// Handle to the running parse loop. Held here so that dropping the
    /// document cancels a parse in flight.
    parse_task: Option<Task<()>>,
    preprocess_partial: bool,
}

/// Whether a background parse is running, and whether the source changed again
/// while it ran.
///
/// This pair is the whole coalescing rule: a request that arrives mid-parse
/// sets `dirty` instead of spawning a second parse, and the running loop
/// re-runs once against the newest source when it lands.
enum ParseState {
    Idle,
    Parsing { dirty: bool },
}

/// Parsed markdown event with source range information.
#[derive(Clone, Debug)]
pub struct MarkdownEvent {
    /// The pulldown-cmark event.
    pub event: Event<'static>,
    /// Where the event came from in the text handed to the parser.
    ///
    /// With [partial-syntax preprocessing](Markdown::preprocess_partial) on
    /// and something to close, that text is the *preprocessed* source, so
    /// offsets past the first insertion do not line up with
    /// [`Markdown::source`]. Rendering does not read this field.
    pub source_range: std::ops::Range<usize>,
}

impl Markdown {
    /// Create a new Markdown instance from source text.
    ///
    /// This first parse is synchronous, so a document is never empty on its
    /// first frame; later ones go to a background thread.
    pub fn new(source: impl Into<SharedString>, _cx: &mut Context<Self>) -> Self {
        let source: SharedString = source.into();
        let preprocess_partial = true;
        let events = Self::parse(&source, preprocess_partial);
        Self {
            parsed_source: source.clone(),
            source,
            events,
            selection: MarkdownSelection::new(),
            parse_state: ParseState::Idle,
            parse_task: None,
            preprocess_partial,
        }
    }

    /// Get the source text.
    ///
    /// While a parse is in flight this is ahead of what is rendered — see
    /// [`parsed_source`](Self::parsed_source).
    pub fn source(&self) -> &str {
        &self.source
    }

    /// The source the currently rendered events came from.
    ///
    /// Equal to [`source`](Self::source) once the latest parse has landed.
    pub fn parsed_source(&self) -> &str {
        &self.parsed_source
    }

    /// Update the markdown content. Drops any selection — its positions
    /// belong to the old text.
    ///
    /// The re-parse happens on a background thread: the previous events keep
    /// rendering until it lands, so [`events`](Self::events) still reports the
    /// old parse when read in the same turn. Setting the source it already has
    /// does nothing at all.
    pub fn set_source(&mut self, source: impl Into<SharedString>, cx: &mut Context<Self>) {
        let source = source.into();
        if source == self.source {
            return;
        }

        self.source = source;
        self.selection.clear();
        self.request_parse(cx);
    }

    /// Append to the markdown content — one delta of a streaming document.
    ///
    /// Unlike [`set_source`](Self::set_source) this keeps the selection: a
    /// selection is a pair of `(run, byte offset)` positions, so text arriving
    /// at the end of the document cannot disturb one made earlier in it.
    ///
    /// Appending nothing does nothing.
    pub fn append(&mut self, text: &str, cx: &mut Context<Self>) {
        if text.is_empty() {
            return;
        }

        let mut source = String::with_capacity(self.source.len() + text.len());
        source.push_str(&self.source);
        source.push_str(text);
        self.source = source.into();
        self.request_parse(cx);
    }

    /// Whether a background parse is in flight — i.e. whether what is rendered
    /// is still one source behind.
    pub fn is_parsing(&self) -> bool {
        matches!(self.parse_state, ParseState::Parsing { .. })
    }

    /// Whether syntax a partial document leaves open is closed before parsing.
    /// On by default, and inert unless the crate was built with the `stitch`
    /// feature — see [`preprocessing_available`].
    pub fn preprocess_partial(&self) -> bool {
        self.preprocess_partial
    }

    /// Turn [partial-syntax preprocessing](Self::preprocess_partial) on or
    /// off. Re-parses the current source unless nothing changed.
    pub fn set_preprocess_partial(&mut self, preprocess_partial: bool, cx: &mut Context<Self>) {
        if self.preprocess_partial == preprocess_partial {
            return;
        }

        self.preprocess_partial = preprocess_partial;
        self.request_parse(cx);
    }

    /// The document's selection handle — for clearing it from outside (e.g.
    /// when another document in the same view starts a selection).
    pub fn selection(&self) -> MarkdownSelection {
        self.selection.clone()
    }

    /// The currently selected text, if any — what ⌘C should copy. Routing
    /// the copy binding is the embedding app's job; this is the value.
    pub fn selected_text(&self) -> Option<String> {
        self.selection.selected_text()
    }

    /// Re-parse the current source in the background.
    ///
    /// While a parse is running this only marks the document dirty — the
    /// running loop picks the newest source up when it lands. Ten deltas
    /// during one parse therefore cost one extra parse, not ten.
    fn request_parse(&mut self, cx: &mut Context<Self>) {
        if let ParseState::Parsing { dirty } = &mut self.parse_state {
            *dirty = true;
            return;
        }

        self.parse_state = ParseState::Parsing { dirty: false };

        let mut pending = (self.source.clone(), self.preprocess_partial);
        // One task looping, rather than a task that re-spawns itself: the
        // handle lives on the entity, so a task assigning `parse_task` from
        // inside itself would drop the very task doing the assigning.
        self.parse_task = Some(cx.spawn(async move |this, cx| {
            loop {
                let (source, preprocess_partial) = pending;
                let parse = {
                    let source = source.clone();
                    cx.background_executor()
                        .spawn(async move { Self::parse(&source, preprocess_partial) })
                };
                let events = parse.await;

                match this.update(cx, |this, cx| this.parse_landed(source, events, cx)) {
                    Ok(Some(next)) => pending = next,
                    // Nothing more to parse, or the document is gone.
                    Ok(None) | Err(_) => break,
                }
            }
        }));
    }

    /// Publish a finished parse and decide whether to run another.
    ///
    /// Publishing and deciding happen in this one update, so a delta arriving
    /// around it cannot be lost: it either set `dirty` before this ran and is
    /// picked up here, or it lands afterwards, finds [`ParseState::Idle`], and
    /// starts a parse of its own.
    fn parse_landed(
        &mut self,
        parsed_source: SharedString,
        events: Vec<MarkdownEvent>,
        cx: &mut Context<Self>,
    ) -> Option<(SharedString, bool)> {
        self.events = events;
        self.parsed_source = parsed_source;
        cx.notify();

        match self.parse_state {
            ParseState::Parsing { dirty: true } => {
                self.parse_state = ParseState::Parsing { dirty: false };
                Some((self.source.clone(), self.preprocess_partial))
            }
            _ => {
                self.parse_state = ParseState::Idle;
                None
            }
        }
    }

    fn parse(source: &str, preprocess_partial: bool) -> Vec<MarkdownEvent> {
        let source = if preprocess_partial {
            stitch::close_open_syntax(source)
        } else {
            std::borrow::Cow::Borrowed(source)
        };

        let parser = Parser::new_ext(&source, parser::default_options());

        parser
            .into_offset_iter()
            .map(|(event, range)| MarkdownEvent {
                event: event.into_static(),
                source_range: range,
            })
            .collect()
    }

    /// Get the parsed events.
    ///
    /// These are the events of [`parsed_source`](Self::parsed_source), which
    /// is one source behind while a parse is in flight.
    pub fn events(&self) -> &[MarkdownEvent] {
        &self.events
    }
}

/// Element for rendering markdown content.
#[derive(IntoElement)]
pub struct MarkdownElement {
    markdown: Entity<Markdown>,
    style: MarkdownStyle,
    element_id: Option<ElementId>,
}

/// Create a markdown element from source text.
///
/// This is a convenience function that creates the entity and element in one step.
/// For more control, use `Markdown::new()` and `MarkdownElement::new()` separately.
///
/// Note that this mints a *new* entity on every call. Called from a `render`,
/// the document gets a new element id every frame, which a screen reader reads
/// as the whole document being replaced. Hold an `Entity<Markdown>` — which
/// text selection needs anyway — for anything longer-lived than a one-shot.
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
            element_id: None,
        }
    }

    /// Set a custom style for the markdown.
    pub fn style(mut self, style: MarkdownStyle) -> Self {
        self.style = style;
        self
    }

    /// Override the element id the document — and therefore all of its text
    /// runs — is scoped under.
    ///
    /// The default is derived from the `Markdown` entity, which is unique and
    /// stable already. Set this only when the same entity is rendered more
    /// than once in a frame; each copy then needs its own id. (Note that two
    /// live copies of one entity still share a single selection.)
    pub fn id(mut self, id: impl Into<ElementId>) -> Self {
        self.element_id = Some(id.into());
        self
    }

    /// The id this element renders under — the explicit [`Self::id`] if one
    /// was given, otherwise the entity-derived default.
    pub fn element_id(&self) -> ElementId {
        self.element_id
            .clone()
            .unwrap_or_else(|| document_element_id(self.markdown.entity_id()))
    }
}

impl RenderOnce for MarkdownElement {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let document_id = self.element_id();
        let markdown = self.markdown.read(cx);
        let events = markdown.events.clone();
        let selection = markdown.selection.clone();
        let style = self.style.clone();

        // New frame: the previous frame's run layouts are about to be
        // dropped and must not be hit-tested.
        selection.begin_frame();

        let renderer = MarkdownRenderer::new(style, selection);
        renderer.render_events(&events, document_id, cx)
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
    in_image: Option<ImageContext>,
    list_stack: Vec<ListContext>,
    /// Document-order index for selectable text runs. A run's index is its
    /// element id, its selection identity and its slot in the per-frame
    /// registry all at once, so there is only ever one counter to keep in
    /// step.
    run_counter: usize,
    /// Shared selection state, handed to every run.
    selection: MarkdownSelection,

    // Table state
    in_table: bool,
    table_alignments: Vec<Alignment>,
    table_rows: Vec<Vec<RichText>>,
    current_row: Vec<RichText>,
    in_table_head: bool,

    // Rich text tracking
    current_text: RichText,
    active_style: InlineStyle,
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
    fn new(style: MarkdownStyle, selection: MarkdownSelection) -> Self {
        Self {
            style,
            selection,
            elements: Vec::new(),
            in_heading: None,
            in_code_block: false,
            in_block_quote: false,
            in_image: None,
            list_stack: Vec::new(),
            run_counter: 0,
            in_table: false,
            table_alignments: Vec::new(),
            table_rows: Vec::new(),
            current_row: Vec::new(),
            in_table_head: false,
            current_text: RichText::new(),
            active_style: InlineStyle::default(),
        }
    }

    fn render_events(
        mut self,
        events: &[MarkdownEvent],
        document_id: ElementId,
        cx: &App,
    ) -> impl IntoElement {
        for event in events {
            self.handle_event(&event.event, cx);
        }

        // The id is what scopes the run ids below it; the role is what puts
        // the document into the accessibility tree. `.role()` is only
        // reachable on a div that already has an `.id()`, so the two cannot
        // drift apart. A bare `.id()` adds no hitbox, so selection
        // hit-testing and link clicks are untouched.
        div()
            .id(document_id)
            .role(Role::Document)
            .w_full()
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
            // Agent/LLM output often uses single newlines as real breaks;
            // `soft_break_as_hard_break` opts into honoring them.
            Event::SoftBreak => {
                if self.style.soft_break_as_hard_break {
                    self.current_text.push("\n", self.active_style)
                } else {
                    self.current_text.push(" ", self.active_style)
                }
            }
            Event::HardBreak => self.current_text.push("\n", self.active_style),
            Event::Rule => self.push_divider(cx),
            Event::TaskListMarker(checked) => self.handle_task_marker(*checked),
            Event::Html(_) | Event::InlineHtml(_) => {}
            Event::FootnoteReference(_) | Event::InlineMath(_) | Event::DisplayMath(_) => {}
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
                // Inline: the link is a styled, clickable range of the
                // surrounding text run, not its own block element.
                self.active_style.link = Some(self.current_text.add_link(dest_url.to_string()));
            }
            Tag::Image {
                dest_url, title, ..
            } => {
                self.in_image = Some(ImageContext {
                    url: dest_url.to_string(),
                    alt: title.to_string(),
                });
            }
            Tag::Table(alignments) => {
                self.in_table = true;
                self.table_alignments = alignments.clone();
                self.table_rows.clear();
            }
            Tag::TableHead => {
                self.in_table_head = true;
                self.current_row.clear();
            }
            Tag::TableRow => {
                self.current_row.clear();
            }
            Tag::TableCell => {
                self.current_text.clear();
            }
            // Listed rather than folded into a `_ => {}` wildcard on purpose:
            // an exhaustive match is what turns a pulldown-cmark upgrade into a
            // compile error instead of silently dropped styling.
            //
            // Superscript and Subscript are inert because the options that
            // produce them are off (see `parser::default_options`) *and*
            // because `InlineStyle` has nowhere to put them.
            Tag::FootnoteDefinition(_)
            | Tag::MetadataBlock(_)
            | Tag::DefinitionList
            | Tag::DefinitionListTitle
            | Tag::DefinitionListDefinition
            | Tag::Superscript
            | Tag::Subscript
            | Tag::HtmlBlock => {}
        }
    }

    fn handle_end_tag(&mut self, tag: &TagEnd, cx: &App) {
        match tag {
            TagEnd::Paragraph => {
                if self.in_block_quote {
                    self.flush_block_quote(cx);
                } else {
                    self.flush_paragraph(cx);
                }
            }
            TagEnd::Heading(level) => {
                let heading_level: elements::HeadingLevel = (*level).into();
                self.in_heading = None;
                self.flush_heading(heading_level, cx);
            }
            TagEnd::BlockQuote(_) => {
                self.in_block_quote = false;
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
                self.active_style.link = None;
            }
            TagEnd::Image => {
                self.flush_image(cx);
            }
            TagEnd::Table => {
                self.flush_table(cx);
                self.in_table = false;
            }
            TagEnd::TableHead => {
                self.in_table_head = false;
                if !self.current_row.is_empty() {
                    self.table_rows.push(std::mem::take(&mut self.current_row));
                }
            }
            TagEnd::TableRow => {
                if !self.current_row.is_empty() {
                    self.table_rows.push(std::mem::take(&mut self.current_row));
                }
            }
            TagEnd::TableCell => {
                self.current_row
                    .push(std::mem::take(&mut self.current_text));
            }
            TagEnd::FootnoteDefinition
            | TagEnd::MetadataBlock(_)
            | TagEnd::DefinitionList
            | TagEnd::DefinitionListTitle
            | TagEnd::DefinitionListDefinition
            | TagEnd::Superscript
            | TagEnd::Subscript
            | TagEnd::HtmlBlock => {}
        }
    }

    fn handle_text(&mut self, text: &str) {
        if let Some(ref mut img_ctx) = self.in_image {
            img_ctx.alt = text.to_string();
        } else {
            self.current_text.push(text, self.active_style);
        }
    }

    fn handle_inline_code(&mut self, code: &str) {
        // A styled span (background wash via the palette), not literal
        // backticks in the text.
        let mut style = self.active_style;
        style.code = true;
        self.current_text.push(code, style);
    }

    fn handle_task_marker(&mut self, checked: bool) {
        let marker = if checked { "☑ " } else { "☐ " };
        self.current_text.push(marker, self.active_style);
    }

    /// Theme-resolved colors for inline code and link spans.
    fn palette(&self, cx: &App) -> InlinePalette {
        let theme = cx.theme();
        InlinePalette {
            code_background: Some(self.style.inline_code_bg.unwrap_or(theme.surface())),
            link_color: Some(self.style.link_color.unwrap_or(theme.accent())),
        }
    }

    /// Id and selection context for the next text run, in document order.
    /// The counter must tick once per selectable run and nowhere else — it
    /// is the identity the element id, the per-frame registry and the
    /// highlights all agree on.
    fn next_run(&mut self, cx: &App) -> (ElementId, elements::RunContext) {
        let run = self.run_counter;
        self.run_counter += 1;
        let theme = cx.theme();
        let run_cx = elements::RunContext {
            selection: self.selection.clone(),
            run,
            selection_background: self
                .style
                .selection_background
                .unwrap_or_else(|| theme.accent().opacity(0.25)),
        };
        (run_element_id(run), run_cx)
    }

    fn flush_paragraph(&mut self, cx: &App) {
        if self.current_text.is_empty() {
            return;
        }

        let rich_text = std::mem::take(&mut self.current_text);
        let palette = self.palette(cx);
        let (id, run_cx) = self.next_run(cx);
        let element =
            elements::rich_paragraph(id, &rich_text, &self.style.body, &palette, run_cx, cx);
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
        }
        .clone();

        let palette = self.palette(cx);
        let (id, run_cx) = self.next_run(cx);
        let element =
            elements::rich_heading(id, &rich_text, level, &heading_style, &palette, run_cx, cx);
        self.elements.push(element.into_any_element());
    }

    fn flush_block_quote(&mut self, cx: &App) {
        if self.current_text.is_empty() {
            return;
        }

        let rich_text = std::mem::take(&mut self.current_text);
        let palette = self.palette(cx);
        let (id, run_cx) = self.next_run(cx);
        let element = elements::rich_block_quote(
            id,
            &rich_text,
            &self.style.body,
            self.style.block_quote_border,
            self.style.block_quote_text,
            &palette,
            run_cx,
            cx,
        );
        self.elements.push(element.into_any_element());
    }

    fn flush_code_block(&mut self, cx: &App) {
        if self.current_text.is_empty() {
            return;
        }

        let text = self.current_text.to_plain_text();
        self.current_text.clear();

        let (id, run_cx) = self.next_run(cx);
        let element = elements::code_block(
            id,
            text,
            None,
            &self.style.code,
            &self.style.code_font_family,
            self.style.code_block_bg,
            self.style.code_block_border,
            run_cx,
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
        let palette = self.palette(cx);
        let (id, run_cx) = self.next_run(cx);
        let element = elements::rich_list_item(
            id,
            &rich_text,
            marker,
            indent_level,
            &self.style.body,
            &palette,
            run_cx,
            cx,
        );
        self.elements.push(element.into_any_element());
    }

    fn flush_image(&mut self, cx: &App) {
        let img_ctx = match self.in_image.take() {
            Some(ctx) => ctx,
            None => return,
        };

        self.current_text.clear();

        let alt = if img_ctx.alt.is_empty() {
            None
        } else {
            Some(img_ctx.alt.as_str())
        };

        let element = elements::image(img_ctx.url, alt, cx);
        self.elements.push(element.into_any_element());
    }

    fn flush_table(&mut self, cx: &App) {
        if self.table_rows.is_empty() {
            return;
        }

        let rows = std::mem::take(&mut self.table_rows);
        let alignments = std::mem::take(&mut self.table_alignments);

        let element = self.render_table(rows, alignments, cx);
        self.elements.push(element.into_any_element());
    }

    fn render_table(
        &self,
        rows: Vec<Vec<RichText>>,
        alignments: Vec<Alignment>,
        cx: &App,
    ) -> impl IntoElement {
        let theme = cx.theme();
        let border_color = theme.border();

        div()
            .flex()
            .flex_col()
            .border_1()
            .border_color(border_color)
            .rounded_sm()
            .overflow_hidden()
            .children(rows.into_iter().enumerate().map(|(row_idx, row)| {
                let is_header = row_idx == 0;
                let bg = if is_header {
                    theme.surface()
                } else if row_idx % 2 == 0 {
                    theme.bg()
                } else {
                    theme.surface().opacity(0.5)
                };

                div()
                    .flex()
                    .flex_row()
                    .bg(bg)
                    .when(row_idx > 0, |el| el.border_t_1().border_color(border_color))
                    .children(row.into_iter().enumerate().map(|(col_idx, cell)| {
                        let alignment = alignments.get(col_idx).copied().unwrap_or(Alignment::None);
                        // Styled (code wash, link color) but not clickable —
                        // cells sit in a custom layout without element ids.
                        let (text, highlights) = cell.to_highlights_with(&self.palette(cx));
                        let styled_text: SharedString = text.into();

                        div()
                            .flex_1()
                            // Same shape as a list item: without `min_w_0` the
                            // cell can't shrink below one unbroken line.
                            .min_w_0()
                            .px_2()
                            .py_1()
                            .text_size(rems(self.style.body.size))
                            .when(col_idx > 0, |el| el.border_l_1().border_color(border_color))
                            .when(is_header, |el| el.font_weight(gpui::FontWeight::SEMIBOLD))
                            .map(|el| match alignment {
                                Alignment::Left | Alignment::None => el,
                                Alignment::Center => el.text_center(),
                                Alignment::Right => el.text_right(),
                            })
                            .child(gpui::StyledText::new(styled_text).with_highlights(highlights))
                    }))
            }))
    }

    fn push_divider(&mut self, cx: &App) {
        let element = elements::divider(self.style.rule_color, cx);
        self.elements.push(element.into_any_element());
    }
}

#[cfg(test)]
mod tests {
    use super::selectable_text::recorder::{self, RecordedRun};
    use super::*;
    use gpui::{point, px, size, AnyElement, Pixels, Render, TestAppContext, VisualTestContext};
    use std::cell::Cell;
    use std::collections::HashSet;
    use std::rc::Rc;

    /// Narrow enough that every sample text below has to wrap.
    const WIDTH: Pixels = px(240.);

    /// Long enough to take several lines at that width — and exactly one line
    /// if whatever contains it refuses to wrap.
    const LONG: &str = "This one is deliberately long enough that it has to wrap onto several lines inside a narrow container.";

    struct TestView {
        source: SharedString,
    }

    impl Render for TestView {
        fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            // The root element is stretched to the window, so the div we
            // measure has to be a child of it — inside a column it keeps its
            // content height, which is the signal these tests read.
            div().flex().flex_col().child(
                div()
                    .w(WIDTH)
                    .debug_selector(|| "measured".into())
                    .child(markdown(self.source.clone(), cx)),
            )
        }
    }

    /// Render `source` into a [`WIDTH`]-wide container and report how tall it
    /// got. Text that will not wrap stays one line tall however long it is.
    fn height(cx: &mut TestAppContext, source: &str) -> Pixels {
        cx.update(crate::theme::init);

        let source = SharedString::from(source.to_string());
        let (_view, cx) = cx.add_window_view(move |_window, _cx| TestView { source });

        cx.debug_bounds("measured")
            .expect("the measured container was never drawn")
            .size
            .height
    }

    /// The height of a single line of body text.
    fn line(cx: &mut TestAppContext) -> Pixels {
        height(cx, "x")
    }

    /// A list item's text column is narrower than a paragraph's by the marker
    /// and the gap (and, when nested, the indent), so it is allowed to take a
    /// couple of lines more than the same text set as a paragraph.
    #[track_caller]
    fn assert_wrapped_like_a_paragraph(item: Pixels, paragraph: Pixels, line: Pixels) {
        assert!(
            paragraph > line,
            "the baseline paragraph did not wrap ({paragraph:?}), so this measures nothing"
        );
        assert!(
            item > line,
            "the text stayed on one line ({item:?}) instead of wrapping"
        );
        assert!(
            item <= paragraph + line * 2.,
            "{item:?} is much taller than the same text as a paragraph ({paragraph:?})"
        );
    }

    /// One of every kind of run this renderer emits, in this order: heading,
    /// paragraph, quote, list item, code.
    const EVERY_RUN_KIND: &str = concat!(
        "# Title\n",
        "\n",
        "A paragraph.\n",
        "\n",
        "> A quote.\n",
        "\n",
        "- An item\n",
        "\n",
        "```\n",
        "let x = 1;\n",
        "```\n",
    );

    /// Draw one frame of whatever `build` produces, and report every run the
    /// way gpui's accessibility walk would have seen it.
    fn draw(
        cx: &mut VisualTestContext,
        build: impl FnOnce() -> Vec<MarkdownElement>,
    ) -> Vec<RecordedRun> {
        recorder::clear();
        cx.draw(
            point(px(0.), px(0.)),
            size(px(800.), px(600.)),
            |_window, _cx| -> AnyElement { div().children(build()).into_any_element() },
        );
        recorder::take()
    }

    /// The last two segments of a run's id path: the document it belongs to,
    /// and the run itself.
    fn doc_and_run(run: &RecordedRun) -> (&str, &str) {
        match run.id_segments.as_slice() {
            [.., doc, this] => (doc.as_str(), this.as_str()),
            other => panic!("a run's id path should have at least two segments: {other:?}"),
        }
    }

    fn id_paths(runs: &[RecordedRun]) -> Vec<String> {
        runs.iter().map(|run| run.id_path.clone()).collect()
    }

    #[gpui::test]
    fn two_documents_in_one_frame_get_disjoint_run_ids(cx: &mut TestAppContext) {
        cx.update(crate::theme::init);
        let (first, second) = (document(cx, EVERY_RUN_KIND), document(cx, EVERY_RUN_KIND));
        let cx = cx.add_empty_window();

        let runs = draw(cx, || {
            vec![
                MarkdownElement::new(first.clone()),
                MarkdownElement::new(second.clone()),
            ]
        });

        assert_eq!(runs.len(), 10, "five runs per document, twice over");

        // The whole point: gpui hashes the *whole* path into a node id, and
        // two documents used to mint the same `md-run-N` under the same
        // ancestors.
        let paths = id_paths(&runs);
        let distinct: HashSet<&String> = paths.iter().collect();
        assert_eq!(distinct.len(), 10, "colliding id paths: {paths:?}");

        let mut documents = HashSet::new();
        for run in &runs {
            let (doc, this) = doc_and_run(run);
            assert!(doc.starts_with("md-doc-"), "unscoped run: {}", run.id_path);
            assert!(this.starts_with("md-run-"), "odd run id: {}", run.id_path);
            documents.insert(doc.to_string());
        }
        assert_eq!(
            documents.len(),
            2,
            "two documents, two scopes: {documents:?}"
        );
    }

    #[gpui::test]
    fn a_long_unordered_list_item_wraps(cx: &mut TestAppContext) {
        let line = line(cx);
        let paragraph = height(cx, LONG);
        let item = height(cx, &format!("- {LONG}"));

        assert_wrapped_like_a_paragraph(item, paragraph, line);
    }

    #[gpui::test]
    fn a_long_ordered_list_item_wraps(cx: &mut TestAppContext) {
        let line = line(cx);
        let paragraph = height(cx, LONG);
        let item = height(cx, &format!("1. {LONG}"));

        assert_wrapped_like_a_paragraph(item, paragraph, line);
    }

    #[gpui::test]
    fn a_long_list_item_with_inline_styles_wraps(cx: &mut TestAppContext) {
        // Bold, code and a link put a different element in the same flex
        // container — the `rich_list_item` path rather than `list_item`.
        let styled = format!("**Bold** and `code` and a [link](#) — {LONG}");

        let line = line(cx);
        let paragraph = height(cx, &styled);
        let item = height(cx, &format!("- {styled}"));

        assert_wrapped_like_a_paragraph(item, paragraph, line);
    }

    #[gpui::test]
    fn a_long_nested_list_item_wraps(cx: &mut TestAppContext) {
        // An indented item, so the row is rendered at `indent_level > 0`.
        // (Separately, and not fixed here: the parent item's own text is
        // swallowed by the nested one, so this renders as a single row.)
        let line = line(cx);
        let paragraph = height(cx, LONG);
        let item = height(cx, &format!("- parent\n    - {LONG}"));

        assert_wrapped_like_a_paragraph(item, paragraph, line);
    }

    #[gpui::test]
    fn a_long_table_cell_wraps(cx: &mut TestAppContext) {
        let line = line(cx);
        let short = height(cx, "| A | B |\n| --- | --- |\n| one | two |");
        let long = height(cx, &format!("| A | B |\n| --- | --- |\n| {LONG} | two |"));

        assert!(
            long >= short + line,
            "the cell stayed one line tall ({long:?} against {short:?}) instead of wrapping"
        );
    }

    #[gpui::test]
    fn a_document_keeps_its_run_ids_across_frames(cx: &mut TestAppContext) {
        // Assistive technology reads a changed node id as a different
        // element, so redrawing an unchanged document must not renumber it.
        cx.update(crate::theme::init);
        let doc = document(cx, EVERY_RUN_KIND);
        let cx = cx.add_empty_window();

        let first = draw(cx, || vec![MarkdownElement::new(doc.clone())]);
        let second = draw(cx, || vec![MarkdownElement::new(doc.clone())]);

        assert_eq!(id_paths(&first), id_paths(&second));
        assert!(!first.is_empty());
    }

    #[gpui::test]
    fn an_explicit_id_separates_two_elements_over_one_entity(cx: &mut TestAppContext) {
        cx.update(crate::theme::init);
        let doc = document(cx, EVERY_RUN_KIND);
        let cx = cx.add_empty_window();

        let runs = draw(cx, || {
            vec![
                MarkdownElement::new(doc.clone()).id("left"),
                MarkdownElement::new(doc.clone()).id("right"),
            ]
        });

        let paths = id_paths(&runs);
        let distinct: HashSet<&String> = paths.iter().collect();
        assert_eq!(distinct.len(), 10, "colliding id paths: {paths:?}");

        let documents: HashSet<&str> = runs.iter().map(|run| doc_and_run(run).0).collect();
        assert_eq!(
            documents,
            HashSet::from(["left", "right"]),
            "the override should replace the entity-derived scope"
        );
    }

    // --- streaming ---

    /// Everything the parse produced as text, so a test can say what the
    /// document currently reads as without spelling out an event list.
    fn rendered_text(markdown: &Markdown) -> String {
        markdown
            .events()
            .iter()
            .filter_map(|event| match &event.event {
                Event::Text(text) | Event::Code(text) => Some(text.to_string()),
                _ => None,
            })
            .collect()
    }

    fn document(cx: &mut TestAppContext, source: &str) -> Entity<Markdown> {
        let source = source.to_string();
        cx.new(|cx| Markdown::new(source, cx))
    }

    /// Count parses by counting notifications: a landed parse is the only
    /// thing this entity notifies for.
    fn count_parses(cx: &mut TestAppContext, markdown: &Entity<Markdown>) -> Rc<Cell<usize>> {
        let parses = Rc::new(Cell::new(0));
        let counter = parses.clone();
        cx.update(|cx| {
            cx.observe(markdown, move |_, _| counter.set(counter.get() + 1))
                .detach()
        });
        parses
    }

    #[gpui::test]
    fn the_first_parse_is_synchronous(cx: &mut TestAppContext) {
        // A document must not be empty on its first frame, so `new` parses
        // before it returns rather than scheduling like every later parse.
        let markdown = document(cx, "# Hello");

        markdown.read_with(cx, |markdown, _| {
            assert_eq!(rendered_text(markdown), "Hello");
            assert_eq!(markdown.parsed_source(), "# Hello");
            assert!(!markdown.is_parsing());
        });
    }

    #[gpui::test]
    fn append_extends_the_document(cx: &mut TestAppContext) {
        let markdown = document(cx, "Hello");
        markdown.update(cx, |markdown, cx| markdown.append(", world", cx));
        cx.run_until_parked();

        markdown.read_with(cx, |markdown, _| {
            assert_eq!(markdown.source(), "Hello, world");
            assert_eq!(markdown.parsed_source(), "Hello, world");
            assert_eq!(rendered_text(markdown), "Hello, world");
        });
    }

    #[gpui::test]
    fn appending_nothing_is_inert(cx: &mut TestAppContext) {
        let markdown = document(cx, "Hello");
        let parses = count_parses(cx, &markdown);

        markdown.update(cx, |markdown, cx| markdown.append("", cx));
        cx.run_until_parked();

        assert_eq!(parses.get(), 0, "an empty delta scheduled a parse");
        markdown.read_with(cx, |markdown, _| assert_eq!(markdown.source(), "Hello"));
    }

    #[gpui::test]
    fn the_old_parse_keeps_rendering_until_the_new_one_lands(cx: &mut TestAppContext) {
        // The point of parsing off the UI thread is that the view never goes
        // blank: the previous events stay up while the new parse runs.
        let markdown = document(cx, "before");
        markdown.update(cx, |markdown, cx| markdown.append(" and after", cx));

        markdown.read_with(cx, |markdown, _| {
            assert!(markdown.is_parsing());
            assert_eq!(markdown.source(), "before and after");
            assert_eq!(markdown.parsed_source(), "before");
            assert_eq!(rendered_text(markdown), "before");
        });

        cx.run_until_parked();

        markdown.read_with(cx, |markdown, _| {
            assert!(!markdown.is_parsing());
            assert_eq!(rendered_text(markdown), "before and after");
        });
    }

    #[gpui::test]
    fn deltas_arriving_during_a_parse_coalesce(cx: &mut TestAppContext) {
        let markdown = document(cx, "one");
        let parses = count_parses(cx, &markdown);

        // Five deltas with no chance for the executor to run in between: the
        // first schedules a parse, the other four only mark it dirty.
        markdown.update(cx, |markdown, cx| {
            for delta in [" two", " three", " four", " five", " six"] {
                markdown.append(delta, cx);
            }
        });
        cx.run_until_parked();

        assert_eq!(
            parses.get(),
            2,
            "five deltas during one parse should cost one extra parse, not four"
        );
        markdown.read_with(cx, |markdown, _| {
            assert_eq!(rendered_text(markdown), "one two three four five six")
        });
    }

    #[gpui::test]
    fn a_long_stream_lands_on_the_final_source(cx: &mut TestAppContext) {
        let markdown = document(cx, "");
        let mut expected = String::new();

        for i in 0..200 {
            let delta = format!("{i} ");
            expected.push_str(&delta);
            markdown.update(cx, |markdown, cx| markdown.append(&delta, cx));
            // Let some deltas land mid-parse and others find the document idle.
            if i % 7 == 0 {
                cx.run_until_parked();
            }
        }
        cx.run_until_parked();

        markdown.read_with(cx, |markdown, _| {
            assert_eq!(markdown.source(), expected);
            assert_eq!(markdown.parsed_source(), expected);
            assert!(!markdown.is_parsing());
            assert_eq!(rendered_text(markdown), expected.trim_end());
        });
    }

    #[gpui::test]
    fn setting_the_same_source_does_not_reparse(cx: &mut TestAppContext) {
        let markdown = document(cx, "# Same");
        let parses = count_parses(cx, &markdown);

        markdown.update(cx, |markdown, cx| markdown.set_source("# Same", cx));
        cx.run_until_parked();
        assert_eq!(parses.get(), 0, "an unchanged source scheduled a parse");

        markdown.update(cx, |markdown, cx| markdown.set_source("# Different", cx));
        cx.run_until_parked();
        assert_eq!(parses.get(), 1);
    }

    #[gpui::test]
    fn append_keeps_the_selection(cx: &mut TestAppContext) {
        // Selection positions are `(run, byte offset within the run)`, so text
        // arriving at the end of the document cannot disturb one made earlier.
        let markdown = document(cx, "First block\n\nSecond block");
        let selection = markdown.read_with(cx, |markdown, _| markdown.selection());
        selection.select_in_run(0, 0..5);

        markdown.update(cx, |markdown, cx| markdown.append("\n\nThird block", cx));
        cx.run_until_parked();

        assert!(!selection.is_empty(), "append dropped the selection");
        let (start, end) = selection.range().expect("the selection went away");
        assert_eq!((start.run, start.offset), (0, 0));
        assert_eq!((end.run, end.offset), (0, 5));
    }

    #[gpui::test]
    fn set_source_drops_the_selection(cx: &mut TestAppContext) {
        let markdown = document(cx, "First block\n\nSecond block");
        let selection = markdown.read_with(cx, |markdown, _| markdown.selection());
        selection.select_in_run(0, 0..5);

        markdown.update(cx, |markdown, cx| markdown.set_source("Something else", cx));

        assert!(
            selection.is_empty(),
            "the selection survived a source it no longer indexes"
        );
    }

    #[gpui::test]
    fn the_default_document_id_follows_the_entity(cx: &mut TestAppContext) {
        cx.update(crate::theme::init);
        let doc = document(cx, "Hello.");
        let expected = document_element_id(doc.entity_id());

        assert_eq!(MarkdownElement::new(doc.clone()).element_id(), expected);
        assert_eq!(
            MarkdownElement::new(doc).id("mine").element_id(),
            ElementId::Name("mine".into())
        );
    }

    #[gpui::test]
    fn every_run_kind_reports_a_role_and_its_text(cx: &mut TestAppContext) {
        cx.update(crate::theme::init);
        let doc = document(cx, EVERY_RUN_KIND);
        let cx = cx.add_empty_window();

        let runs = draw(cx, || vec![MarkdownElement::new(doc.clone())]);

        let reported: Vec<(Option<Role>, Option<&str>, Option<usize>)> = runs
            .iter()
            .map(|run| (run.role, run.label.as_deref(), run.level))
            .collect();

        assert_eq!(
            reported,
            vec![
                (Some(Role::Heading), Some("Title"), Some(1)),
                (Some(Role::Paragraph), Some("A paragraph."), None),
                (Some(Role::Blockquote), Some("A quote."), None),
                (Some(Role::ListItem), Some("An item"), None),
                (Some(Role::Code), Some("let x = 1;\n"), None),
            ]
        );
    }

    #[gpui::test]
    fn headings_report_their_level(cx: &mut TestAppContext) {
        cx.update(crate::theme::init);
        let doc = document(cx, "# One\n\n### Three\n");
        let cx = cx.add_empty_window();

        let runs = draw(cx, || vec![MarkdownElement::new(doc.clone())]);

        let levels: Vec<_> = runs.iter().map(|run| run.level).collect();
        assert_eq!(levels, vec![Some(1), Some(3)]);
    }

    #[test]
    fn heading_levels_are_numbered_one_through_six() {
        let levels: Vec<u8> = [
            HeadingLevel::H1,
            HeadingLevel::H2,
            HeadingLevel::H3,
            HeadingLevel::H4,
            HeadingLevel::H5,
            HeadingLevel::H6,
        ]
        .into_iter()
        .map(HeadingLevel::level)
        .collect();

        assert_eq!(levels, vec![1, 2, 3, 4, 5, 6]);
    }

    #[gpui::test]
    fn dropping_the_document_mid_parse_is_harmless(cx: &mut TestAppContext) {
        let markdown = document(cx, "start");
        markdown.update(cx, |markdown, cx| markdown.append(" more", cx));
        drop(markdown);

        // The parse task is owned by the entity, so this drives an in-flight
        // parse whose document is already gone.
        cx.run_until_parked();
    }

    #[cfg(feature = "stitch")]
    #[gpui::test]
    fn partial_emphasis_renders_as_emphasis(cx: &mut TestAppContext) {
        // Mid-stream `**bold` has no closer yet. Parsed as-is it draws literal
        // asterisks that turn into bold one delta later — the flicker.
        let markdown = document(cx, "A **partially written");

        markdown.read_with(cx, |markdown, _| {
            assert_eq!(rendered_text(markdown), "A partially written");
            assert!(markdown
                .events()
                .iter()
                .any(|event| matches!(event.event, Event::Start(Tag::Strong))));
        });
    }

    #[gpui::test]
    #[cfg(feature = "stitch")]
    fn a_partial_link_is_not_a_link_yet(cx: &mut TestAppContext) {
        // The label reads as plain text until the URL completes — a live link
        // to a placeholder URL would be worse than no link.
        let markdown = document(cx, "See [the docs](htt");

        markdown.read_with(cx, |markdown, _| {
            assert_eq!(rendered_text(markdown), "See the docs");
            assert!(
                !markdown
                    .events()
                    .iter()
                    .any(|event| matches!(event.event, Event::Start(Tag::Link { .. }))),
                "an incomplete URL became a clickable link"
            );
        });
    }

    #[cfg(feature = "stitch")]
    #[gpui::test]
    fn a_complete_document_parses_the_same_either_way(cx: &mut TestAppContext) {
        let source = "# Title\n\nA **bold** word, `code`, and a [link](https://example.com).\n";
        let markdown = document(cx, source);
        let with_preprocessing = markdown.read_with(cx, |markdown, _| rendered_text(markdown));

        markdown.update(cx, |markdown, cx| {
            markdown.set_preprocess_partial(false, cx)
        });
        cx.run_until_parked();

        markdown.read_with(cx, |markdown, _| {
            assert_eq!(rendered_text(markdown), with_preprocessing)
        });
    }

    #[cfg(feature = "stitch")]
    #[gpui::test]
    fn preprocessing_can_be_turned_off(cx: &mut TestAppContext) {
        let markdown = document(cx, "A **partially written");
        markdown.update(cx, |markdown, cx| {
            assert!(markdown.preprocess_partial());
            markdown.set_preprocess_partial(false, cx);
        });
        cx.run_until_parked();

        markdown.read_with(cx, |markdown, _| {
            assert!(!markdown.preprocess_partial());
            assert_eq!(rendered_text(markdown), "A **partially written");
        });
    }
}
