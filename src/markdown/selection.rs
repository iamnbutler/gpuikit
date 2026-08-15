//! Text selection across a markdown document.
//!
//! A markdown document renders as many text runs (paragraphs, headings,
//! quotes, list items, code blocks), each its own element with its own
//! layout. Selection has to read as one document anyway: the user drags from
//! a heading into the paragraph below it and expects both to highlight, and
//! ⌘C to yield both. This module holds the document-wide state that makes
//! that work.
//!
//! The state lives behind an `Rc<RefCell<…>>` handle ([`MarkdownSelection`])
//! owned by the [`Markdown`](super::Markdown) entity and cloned into every
//! run element. Two kinds of data live in it:
//!
//! - The **selection** itself: an anchor and head, each a `(run, byte offset)`
//!   position. Persistent across frames.
//! - A per-frame **registry** of run layouts and texts, repopulated on every
//!   paint. Mouse handlers hit-test against it (position → run + byte), and
//!   `selected_text` reads run texts out of it. It is a view of the last
//!   painted frame, never authoritative state.
//!
//! Selection is per-document, but coherence across documents mostly falls
//! out: a mouse press inside one document lands outside every run of its
//! siblings, which clears them (see the run-0 handler in
//! [`SelectableText`](super::selectable_text::SelectableText)). An embedding
//! app only needs [`MarkdownSelection::clear`] for programmatic cases.

use std::cell::RefCell;
use std::ops::Range;
use std::rc::Rc;

use gpui::{Bounds, Pixels, Point, SharedString};

use super::selectable_text::RegisteredRun;

/// A position in the document: which run, and a byte offset into its text.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct SelectionPosition {
    pub run: usize,
    pub offset: usize,
}

#[derive(Default)]
pub(crate) struct SelectionState {
    /// Selection endpoints in document order — `anchor` is where the drag
    /// started, `head` where it currently is. Either order.
    anchor: Option<SelectionPosition>,
    head: Option<SelectionPosition>,
    /// A drag is in progress (mouse down, not yet up).
    dragging: bool,
    /// Per-frame registry, indexed by run. Rebuilt every paint; `None` slots
    /// are runs that did not paint this frame.
    runs: Vec<Option<RegisteredRun>>,
}

/// Shared handle to a document's selection state. Cheap to clone; every run
/// element of one document holds the same handle.
#[derive(Clone, Default)]
pub struct MarkdownSelection(Rc<RefCell<SelectionState>>);

impl MarkdownSelection {
    pub fn new() -> Self {
        Self::default()
    }

    /// Start a new frame: forget the previous frame's run registry (layouts
    /// from a dropped frame must not be hit-tested). Called from the
    /// markdown element's render, before any run paints.
    pub(crate) fn begin_frame(&self) {
        self.0.borrow_mut().runs.clear();
    }

    /// Record a run's layout and text for this frame.
    pub(crate) fn register_run(&self, run: usize, entry: RegisteredRun) {
        let mut state = self.0.borrow_mut();
        if state.runs.len() <= run {
            state.runs.resize_with(run + 1, || None);
        }
        state.runs[run] = Some(entry);
    }

    // --- drag lifecycle (mouse handlers) ---

    pub(crate) fn begin_drag(&self, position: SelectionPosition) {
        let mut state = self.0.borrow_mut();
        state.anchor = Some(position);
        state.head = Some(position);
        state.dragging = true;
    }

    pub(crate) fn update_head(&self, position: SelectionPosition) {
        let mut state = self.0.borrow_mut();
        if state.dragging {
            state.head = Some(position);
        }
    }

    pub(crate) fn end_drag(&self) {
        let mut state = self.0.borrow_mut();
        state.dragging = false;
        // A drag that never moved is a click, not a selection.
        if state.anchor == state.head {
            state.anchor = None;
            state.head = None;
        }
    }

    pub(crate) fn is_dragging(&self) -> bool {
        self.0.borrow().dragging
    }

    /// The run the current drag started in, if a drag is in progress. That
    /// run's element owns the document-wide mouse handlers for the drag.
    pub(crate) fn drag_anchor_run(&self) -> Option<usize> {
        let state = self.0.borrow();
        state.dragging.then_some(state.anchor?.run)
    }

    /// Select `range` within one run (double-click word, triple-click run).
    pub(crate) fn select_in_run(&self, run: usize, range: Range<usize>) {
        let mut state = self.0.borrow_mut();
        state.anchor = Some(SelectionPosition {
            run,
            offset: range.start,
        });
        state.head = Some(SelectionPosition {
            run,
            offset: range.end,
        });
        state.dragging = false;
    }

    /// Drop the selection entirely.
    pub fn clear(&self) {
        let mut state = self.0.borrow_mut();
        state.anchor = None;
        state.head = None;
        state.dragging = false;
    }

    /// Whether a non-empty selection exists.
    pub fn is_empty(&self) -> bool {
        let state = self.0.borrow();
        match (state.anchor, state.head) {
            (Some(anchor), Some(head)) => anchor == head,
            _ => true,
        }
    }

    /// The selection's endpoints in document order, if non-empty.
    pub fn range(&self) -> Option<(SelectionPosition, SelectionPosition)> {
        let state = self.0.borrow();
        let (anchor, head) = (state.anchor?, state.head?);
        if anchor == head {
            return None;
        }
        Some((anchor.min(head), anchor.max(head)))
    }

    /// The selected byte range within run `run` of length `len`, if any —
    /// what the renderer highlights.
    pub(crate) fn range_in_run(&self, run: usize, len: usize) -> Option<Range<usize>> {
        let (start, end) = self.range()?;
        if run < start.run || run > end.run {
            return None;
        }
        let from = if run == start.run { start.offset } else { 0 };
        let to = if run == end.run { end.offset } else { len };
        let (from, to) = (from.min(len), to.min(len));
        (from < to).then_some(from..to)
    }

    /// The selected text, assembled across runs in document order, runs
    /// joined with newlines. `None` when nothing is selected.
    pub fn selected_text(&self) -> Option<String> {
        let (start, end) = self.range()?;
        let state = self.0.borrow();
        let mut parts = Vec::new();
        for run in start.run..=end.run {
            let Some(Some(entry)) = state.runs.get(run) else {
                continue;
            };
            let text: &str = &entry.text;
            let from = if run == start.run { start.offset } else { 0 };
            let to = if run == end.run {
                end.offset
            } else {
                text.len()
            };
            let (from, to) = (
                clamp_to_char_boundary(text, from.min(text.len())),
                clamp_to_char_boundary(text, to.min(text.len())),
            );
            if from < to {
                parts.push(text[from..to].to_string());
            }
        }
        (!parts.is_empty()).then(|| parts.join("\n"))
    }

    // --- hit testing against the last painted frame ---

    /// The document position for a window-space point: the run whose bounds
    /// contain (or are vertically nearest to) the point, and the nearest
    /// byte offset within it. `None` when no runs painted.
    pub(crate) fn position_for_point(&self, point: Point<Pixels>) -> Option<SelectionPosition> {
        let state = self.0.borrow();
        let mut best: Option<(Pixels, SelectionPosition)> = None;
        for (run, entry) in state.runs.iter().enumerate() {
            let Some(entry) = entry else { continue };
            let offset = match entry.layout.index_for_position(point) {
                Ok(offset) => {
                    // Inside this run — exact hit, no contest.
                    return Some(SelectionPosition { run, offset });
                }
                Err(nearest) => nearest,
            };
            let distance = vertical_distance(entry.layout.bounds(), point);
            let position = SelectionPosition { run, offset };
            if best.map_or(true, |(best_distance, _)| distance < best_distance) {
                best = Some((distance, position));
            }
        }
        best.map(|(_, position)| position)
    }

    /// Whether the point falls within any registered run's bounds — used to
    /// tell "clicked elsewhere in this document" from "clicked outside it".
    pub(crate) fn point_in_any_run(&self, point: Point<Pixels>) -> bool {
        self.0.borrow().runs.iter().flatten().any(|entry| {
            let bounds = entry.layout.bounds();
            bounds.contains(&point)
        })
    }

    /// The registered text of run `run`, if it painted this frame.
    pub(crate) fn run_text(&self, run: usize) -> Option<SharedString> {
        self.0
            .borrow()
            .runs
            .get(run)?
            .as_ref()
            .map(|entry| entry.text.clone())
    }
}

/// Vertical distance from `point` to `bounds` (zero when inside the band) —
/// the tie-breaker for points between blocks, where every run reports a
/// nearest index and the closest band should win.
fn vertical_distance(bounds: Bounds<Pixels>, point: Point<Pixels>) -> Pixels {
    if point.y < bounds.top() {
        bounds.top() - point.y
    } else if point.y > bounds.bottom() {
        point.y - bounds.bottom()
    } else {
        Pixels::ZERO
    }
}

/// Round `ix` down to a char boundary so slicing can't panic mid-codepoint
/// (offsets come from layout hit-tests, which are glyph-aligned, but clamp
/// defensively — a stale offset against updated text must not crash).
fn clamp_to_char_boundary(text: &str, mut ix: usize) -> usize {
    while ix > 0 && !text.is_char_boundary(ix) {
        ix -= 1;
    }
    ix
}

/// The word range around byte `ix` in `text` — double-click selection.
/// Falls back to the whole text when `ix` lands outside any word.
pub(crate) fn word_range_at(text: &str, ix: usize) -> Range<usize> {
    use unicode_segmentation::UnicodeSegmentation;
    let ix = clamp_to_char_boundary(text, ix.min(text.len()));
    for (start, word) in text.split_word_bound_indices() {
        let end = start + word.len();
        if ix >= start && ix < end {
            return start..end;
        }
    }
    0..text.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pos(run: usize, offset: usize) -> SelectionPosition {
        SelectionPosition { run, offset }
    }

    #[test]
    fn a_drag_that_never_moved_is_not_a_selection() {
        let selection = MarkdownSelection::new();
        selection.begin_drag(pos(0, 4));
        selection.end_drag();
        assert!(selection.is_empty());
        assert!(selection.range().is_none());
    }

    #[test]
    fn range_normalizes_backwards_drags() {
        let selection = MarkdownSelection::new();
        selection.begin_drag(pos(2, 10));
        selection.update_head(pos(0, 3));
        selection.end_drag();
        let (start, end) = selection.range().unwrap();
        assert_eq!(start, pos(0, 3));
        assert_eq!(end, pos(2, 10));
    }

    #[test]
    fn range_in_run_covers_middle_runs_fully_and_ends_partially() {
        let selection = MarkdownSelection::new();
        selection.begin_drag(pos(0, 3));
        selection.update_head(pos(2, 5));
        selection.end_drag();

        assert_eq!(selection.range_in_run(0, 10), Some(3..10));
        assert_eq!(selection.range_in_run(1, 7), Some(0..7));
        assert_eq!(selection.range_in_run(2, 10), Some(0..5));
        assert_eq!(selection.range_in_run(3, 10), None);
    }

    #[test]
    fn range_in_run_clamps_to_run_length() {
        // A head offset past the run's end (text shrank under a live
        // selection) must clamp, not panic or produce an inverted range.
        let selection = MarkdownSelection::new();
        selection.begin_drag(pos(0, 8));
        selection.update_head(pos(0, 20));
        selection.end_drag();
        assert_eq!(selection.range_in_run(0, 10), Some(8..10));
        assert_eq!(selection.range_in_run(0, 4), None);
    }

    #[test]
    fn selected_text_joins_runs_in_document_order() {
        let selection = MarkdownSelection::new();
        selection.register_run(0, RegisteredRun::for_test("alpha beta"));
        selection.register_run(1, RegisteredRun::for_test("gamma"));
        selection.register_run(2, RegisteredRun::for_test("delta epsilon"));

        selection.begin_drag(pos(0, 6));
        selection.update_head(pos(2, 5));
        selection.end_drag();

        assert_eq!(
            selection.selected_text().as_deref(),
            Some("beta\ngamma\ndelta")
        );
    }

    #[test]
    fn selected_text_survives_a_run_that_did_not_paint() {
        let selection = MarkdownSelection::new();
        selection.register_run(0, RegisteredRun::for_test("first"));
        selection.register_run(2, RegisteredRun::for_test("third"));

        selection.begin_drag(pos(0, 0));
        selection.update_head(pos(2, 5));
        selection.end_drag();

        assert_eq!(selection.selected_text().as_deref(), Some("first\nthird"));
    }

    #[test]
    fn selected_text_clamps_offsets_to_char_boundaries() {
        let selection = MarkdownSelection::new();
        selection.register_run(0, RegisteredRun::for_test("héllo"));
        // "é" is two bytes (1..3); an offset of 2 splits it.
        selection.begin_drag(pos(0, 2));
        selection.update_head(pos(0, 5));
        selection.end_drag();
        assert_eq!(selection.selected_text().as_deref(), Some("éll"));
    }

    #[test]
    fn double_click_word_ranges() {
        assert_eq!(word_range_at("alpha beta", 1), 0..5);
        assert_eq!(word_range_at("alpha beta", 7), 6..10);
        // On the space between words: the space is its own "word bound".
        assert_eq!(word_range_at("alpha beta", 5), 5..6);
        // Unicode word.
        assert_eq!(word_range_at("héllo wörld", 2), 0..6);
    }

    #[test]
    fn clearing_forgets_everything() {
        let selection = MarkdownSelection::new();
        selection.begin_drag(pos(0, 0));
        selection.update_head(pos(1, 4));
        selection.end_drag();
        assert!(!selection.is_empty());
        selection.clear();
        assert!(selection.is_empty());
        assert!(selection.selected_text().is_none());
    }
}
