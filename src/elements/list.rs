//! List component backed by `uniform_list` for efficient rendering of large lists.
//!
//! All items (including section headers) have the same height. Headers use
//! bottom-aligned text with top padding to visually separate sections.
//!
//! # Example
//!
//! ```ignore
//! use gpuikit::elements::list::{List, ListEntry};
//!
//! let entries = vec![
//!     ListEntry::header("Section A"),
//!     ListEntry::item("item-1", |_w, _cx| div().child("First item").into_any_element()),
//!     ListEntry::item("item-2", |_w, _cx| div().child("Second item").into_any_element()),
//!     ListEntry::header("Section B"),
//!     ListEntry::item("item-3", |_w, _cx| div().child("Third item").into_any_element()),
//! ];
//!
//! // In your Render impl:
//! List::new("my-list", entries).render(window, cx)
//! ```

use crate::theme::{ActiveTheme, Themeable};
use gpui::{
    div, px, uniform_list, AnyElement, App, ElementId, IntoElement, ParentElement, Pixels,
    SharedString, Styled, UniformListScrollHandle, Window,
};
use std::rc::Rc;

/// Default row height in pixels
const DEFAULT_ITEM_HEIGHT: f32 = 27.0;

/// Default font size in pixels
const DEFAULT_FONT_SIZE: f32 = 13.0;

type ItemRender = Rc<dyn Fn(&mut Window, &mut App) -> AnyElement>;

/// A single entry in the list — either a section header or a content item.
#[derive(Clone)]
pub enum ListEntry {
    /// A section header label, rendered bottom-aligned with space above.
    Header {
        label: SharedString,
    },
    /// A content item rendered by a callback.
    Item {
        id: ElementId,
        render: ItemRender,
    },
}

impl ListEntry {
    /// Create a section header entry.
    pub fn header(label: impl Into<SharedString>) -> Self {
        ListEntry::Header {
            label: label.into(),
        }
    }

    /// Create a content item entry.
    pub fn item(
        id: impl Into<ElementId>,
        render: impl Fn(&mut Window, &mut App) -> AnyElement + 'static,
    ) -> Self {
        ListEntry::Item {
            id: id.into(),
            render: Rc::new(render),
        }
    }
}

/// A virtualized list component using `uniform_list`.
pub struct List {
    id: ElementId,
    entries: Vec<ListEntry>,
    item_height: Pixels,
    font_size: Pixels,
    scroll_handle: Option<UniformListScrollHandle>,
}

impl List {
    pub fn new(id: impl Into<ElementId>, entries: Vec<ListEntry>) -> Self {
        Self {
            id: id.into(),
            entries,
            item_height: px(DEFAULT_ITEM_HEIGHT),
            font_size: px(DEFAULT_FONT_SIZE),
            scroll_handle: None,
        }
    }

    /// Set the row height (applies to both headers and items).
    pub fn item_height(mut self, height: Pixels) -> Self {
        self.item_height = height;
        self
    }

    /// Set the font size for list content.
    pub fn font_size(mut self, size: Pixels) -> Self {
        self.font_size = size;
        self
    }

    /// Attach a scroll handle for programmatic scrolling.
    pub fn track_scroll(mut self, handle: &UniformListScrollHandle) -> Self {
        self.scroll_handle = Some(handle.clone());
        self
    }

    /// Render the list into an element.
    pub fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let item_height = self.item_height;
        let font_size = self.font_size;
        let entries = self.entries.clone();
        let entry_count = entries.len();

        let theme = cx.theme();
        let fg_muted = theme.fg_muted();

        let list = uniform_list(
            self.id,
            entry_count,
            move |range, window, cx| {
                range
                    .map(|ix| {
                        let entry = &entries[ix];
                        match entry {
                            ListEntry::Header { label } => div()
                                .h(item_height)
                                .w_full()
                                .flex()
                                .items_end()
                                .px_2()
                                .pb(px(2.))
                                .child(
                                    div()
                                        .text_size(font_size - px(1.))
                                        .text_color(fg_muted)
                                        .child(label.clone()),
                                )
                                .into_any_element(),
                            ListEntry::Item { render, .. } => {
                                let content = render(window, cx);
                                div()
                                    .h(item_height)
                                    .w_full()
                                    .flex()
                                    .items_center()
                                    .text_size(font_size)
                                    .child(content)
                                    .into_any_element()
                            }
                        }
                    })
                    .collect()
            },
        )
        .size_full();

        match self.scroll_handle {
            Some(ref handle) => list.track_scroll(handle).into_any_element(),
            None => list.into_any_element(),
        }
    }
}
