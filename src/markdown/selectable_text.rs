//! A text run that participates in document-wide selection.
//!
//! Wraps [`StyledText`], modeled on gpui's `InteractiveText`, and adds the
//! two behaviours markdown needs from every run:
//!
//! - **Selection**: mouse down anchors a drag; while dragging, the anchor
//!   run's element registers window-wide move/up handlers and hit-tests the
//!   pointer against the *whole document's* registered runs (via the shared
//!   [`MarkdownSelection`]), so a drag flows across paragraphs, headings and
//!   code blocks. Double-click selects a word, triple-click the run.
//! - **Link clicks**: clickable ranges still open on click — but only when
//!   the mouse didn't move between down and up. A drag that starts on a link
//!   selects; a click on one follows it.
//!
//! Rendering the selection is not this element's job: the renderer injects
//! the selected range as one more background highlight before construction,
//! so painting stays plain `StyledText`.

use std::mem;
use std::ops::Range;
use std::rc::Rc;

use gpui::{
    App, Bounds, CursorStyle, DispatchPhase, Element, ElementId, GlobalElementId, Hitbox,
    HitboxBehavior, IntoElement, LayoutId, MouseDownEvent, MouseMoveEvent, MouseUpEvent, Pixels,
    SharedString, StyledText, TextLayout, Window,
};

use super::selection::{word_range_at, MarkdownSelection, SelectionPosition};

/// One run's layout and text, registered into the document's selection state
/// for the current frame.
pub(crate) struct RegisteredRun {
    pub layout: TextLayout,
    pub text: SharedString,
}

impl RegisteredRun {
    /// A registry entry with no layout behind it — selection-state tests
    /// exercise text assembly, which never touches the layout.
    #[cfg(test)]
    pub(crate) fn for_test(text: &str) -> Self {
        Self {
            layout: TextLayout::default(),
            text: SharedString::from(text.to_string()),
        }
    }
}

type ClickListener = Rc<dyn Fn(usize, &mut Window, &mut App)>;

/// A selectable (and optionally link-bearing) text run.
pub struct SelectableText {
    element_id: ElementId,
    text: StyledText,
    /// This run's index in document order — its identity in the selection.
    run: usize,
    selection: MarkdownSelection,
    clickable_ranges: Vec<Range<usize>>,
    click_listener: Option<ClickListener>,
}

impl SelectableText {
    pub fn new(
        id: impl Into<ElementId>,
        text: StyledText,
        run: usize,
        selection: MarkdownSelection,
    ) -> Self {
        Self {
            element_id: id.into(),
            text,
            run,
            selection,
            clickable_ranges: Vec::new(),
            click_listener: None,
        }
    }

    /// `listener` is called with the index of the clicked range — same
    /// contract as `InteractiveText::on_click`, except a click is only a
    /// click when the pointer didn't drag between down and up.
    pub fn on_click(
        mut self,
        ranges: Vec<Range<usize>>,
        listener: impl Fn(usize, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.clickable_ranges = ranges;
        self.click_listener = Some(Rc::new(listener));
        self
    }
}

impl IntoElement for SelectableText {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

impl Element for SelectableText {
    type RequestLayoutState = ();
    type PrepaintState = Hitbox;

    fn id(&self) -> Option<ElementId> {
        Some(self.element_id.clone())
    }

    fn source_location(&self) -> Option<&'static core::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _id: Option<&GlobalElementId>,
        inspector_id: Option<&gpui::InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        self.text.request_layout(None, inspector_id, window, cx)
    }

    fn prepaint(
        &mut self,
        _global_id: Option<&GlobalElementId>,
        inspector_id: Option<&gpui::InspectorElementId>,
        bounds: Bounds<Pixels>,
        state: &mut Self::RequestLayoutState,
        window: &mut Window,
        cx: &mut App,
    ) -> Hitbox {
        self.text
            .prepaint(None, inspector_id, bounds, state, window, cx);
        window.insert_hitbox(bounds, HitboxBehavior::Normal)
    }

    fn paint(
        &mut self,
        _global_id: Option<&GlobalElementId>,
        inspector_id: Option<&gpui::InspectorElementId>,
        bounds: Bounds<Pixels>,
        _: &mut Self::RequestLayoutState,
        hitbox: &mut Hitbox,
        window: &mut Window,
        cx: &mut App,
    ) {
        let text_layout = self.text.layout().clone();
        let selection = self.selection.clone();
        let run = self.run;

        // This frame's registry entry — hit-testing and copy read from it.
        selection.register_run(
            run,
            RegisteredRun {
                layout: text_layout.clone(),
                text: SharedString::from(text_layout.text()),
            },
        );

        // Cursor: ibeam over text; pointing hand over a link.
        let over_link = text_layout
            .index_for_position(window.mouse_position())
            .is_ok_and(|ix| {
                self.clickable_ranges
                    .iter()
                    .any(|range| range.contains(&ix))
            });
        window.set_cursor_style(
            if over_link {
                CursorStyle::PointingHand
            } else {
                CursorStyle::IBeam
            },
            hitbox,
        );

        // Mouse down in this run: anchor a drag (single click), select a
        // word (double), or the whole run (triple). Also the place a click
        // anywhere *outside* the document clears its selection.
        {
            let selection = selection.clone();
            let text_layout = text_layout.clone();
            let hitbox = hitbox.clone();
            window.on_mouse_event(move |event: &MouseDownEvent, phase, window, _cx| {
                if phase != DispatchPhase::Bubble {
                    return;
                }
                if hitbox.is_hovered(window) {
                    let offset = match text_layout.index_for_position(event.position) {
                        Ok(offset) => offset,
                        Err(nearest) => nearest,
                    };
                    match event.click_count {
                        1 => selection.begin_drag(SelectionPosition { run, offset }),
                        2 => {
                            if let Some(text) = selection.run_text(run) {
                                selection.select_in_run(run, word_range_at(&text, offset));
                            }
                        }
                        _ => {
                            let len = selection.run_text(run).map_or(0, |text| text.len());
                            selection.select_in_run(run, 0..len);
                        }
                    }
                    window.refresh();
                } else if run == 0
                    && !selection.is_empty()
                    && !selection.point_in_any_run(event.position)
                {
                    // Run 0 speaks for the document: a press outside every
                    // run drops the selection. (Guarded to one run so the
                    // clear doesn't run once per block.)
                    selection.clear();
                    window.refresh();
                }
            });
        }

        // While a drag that started in this run is live, this element owns
        // the document-wide move/up handlers. Deliberately not hitbox-gated:
        // the pointer outruns the run immediately, and crossing into another
        // block is the point.
        if selection.drag_anchor_run() == Some(run) {
            {
                let selection = selection.clone();
                window.on_mouse_event(move |event: &MouseMoveEvent, phase, window, _cx| {
                    if phase != DispatchPhase::Bubble || !selection.is_dragging() {
                        return;
                    }
                    if let Some(position) = selection.position_for_point(event.position) {
                        selection.update_head(position);
                        window.refresh();
                    }
                });
            }
            {
                let selection = selection.clone();
                let text_layout = text_layout.clone();
                let hitbox = hitbox.clone();
                let clickable_ranges = mem::take(&mut self.clickable_ranges);
                let click_listener = self.click_listener.clone();
                window.on_mouse_event(move |event: &MouseUpEvent, phase, window, cx| {
                    if phase != DispatchPhase::Bubble || !selection.is_dragging() {
                        return;
                    }
                    let was_click = selection.range().is_none();
                    selection.end_drag();
                    // A press-and-release with no movement on a link opens
                    // it; with movement, the selection wins and the link
                    // stays put.
                    if was_click && hitbox.is_hovered(window) {
                        if let (Some(listener), Ok(ix)) = (
                            click_listener.as_ref(),
                            text_layout.index_for_position(event.position),
                        ) {
                            if let Some(range_ix) = clickable_ranges
                                .iter()
                                .position(|range| range.contains(&ix))
                            {
                                listener(range_ix, window, cx);
                            }
                        }
                    }
                    window.refresh();
                });
            }
        }

        self.text
            .paint(None, inspector_id, bounds, &mut (), &mut (), window, cx);
    }
}
