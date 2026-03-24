//! Accordion component for displaying multiple collapsible sections.
//!
//! # Example
//!
//! ```ignore
//! use gpuikit::elements::accordion::{accordion, accordion_item, AccordionState};
//!
//! // Create an accordion with multiple items
//! let accordion_state = cx.new(|_cx| {
//!     AccordionState::new(
//!         accordion("my-accordion")
//!             .item(accordion_item("section-1", "Section 1").content("Content for section 1"))
//!             .item(accordion_item("section-2", "Section 2").content("Content for section 2"))
//!             .item(accordion_item("section-3", "Section 3").content("Content for section 3"))
//!     )
//! });
//!
//! // For single mode (only one item open at a time):
//! accordion("my-accordion").single()
//!
//! // For multiple mode (default, multiple items can be open):
//! accordion("my-accordion").multiple()
//! ```

use crate::icons::Icons;
use crate::theme::{ActiveTheme, Themeable};
use gpui::{
    div, prelude::*, px, rems, Context, ElementId, EventEmitter, IntoElement, ParentElement,
    Render, SharedString, Styled, Window,
};
use std::collections::HashSet;

/// Event emitted when accordion items are expanded or collapsed.
pub struct AccordionChanged {
    /// The ID of the item that was toggled.
    pub item_id: ElementId,
    /// Whether the item is now expanded.
    pub expanded: bool,
}

/// Mode for accordion behavior.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AccordionMode {
    /// Only one item can be open at a time.
    Single,
    /// Multiple items can be open simultaneously.
    #[default]
    Multiple,
}

/// A single item within an accordion.
pub struct AccordionItem {
    id: ElementId,
    header: SharedString,
    content: Option<SharedString>,
    disabled: bool,
}

impl AccordionItem {
    /// Create a new accordion item with a header.
    pub fn new(id: impl Into<ElementId>, header: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            header: header.into(),
            content: None,
            disabled: false,
        }
    }

    /// Set the content of the accordion item.
    pub fn content(mut self, content: impl Into<SharedString>) -> Self {
        self.content = Some(content.into());
        self
    }

    /// Set whether this item is disabled.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

/// Creates a new accordion item.
pub fn accordion_item(id: impl Into<ElementId>, header: impl Into<SharedString>) -> AccordionItem {
    AccordionItem::new(id, header)
}

/// Builder for creating an accordion component.
pub struct Accordion {
    id: ElementId,
    items: Vec<AccordionItem>,
    mode: AccordionMode,
    default_expanded: HashSet<ElementId>,
}

impl Accordion {
    /// Create a new accordion.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            items: Vec::new(),
            mode: AccordionMode::Multiple,
            default_expanded: HashSet::new(),
        }
    }

    /// Add an item to the accordion.
    pub fn item(mut self, item: AccordionItem) -> Self {
        self.items.push(item);
        self
    }

    /// Set the accordion to single mode (only one item open at a time).
    pub fn single(mut self) -> Self {
        self.mode = AccordionMode::Single;
        self
    }

    /// Set the accordion to multiple mode (multiple items can be open).
    pub fn multiple(mut self) -> Self {
        self.mode = AccordionMode::Multiple;
        self
    }

    /// Set the mode of the accordion.
    pub fn mode(mut self, mode: AccordionMode) -> Self {
        self.mode = mode;
        self
    }

    /// Set an item to be expanded by default.
    pub fn default_expanded(mut self, item_id: impl Into<ElementId>) -> Self {
        self.default_expanded.insert(item_id.into());
        self
    }
}

/// Creates a new accordion builder.
pub fn accordion(id: impl Into<ElementId>) -> Accordion {
    Accordion::new(id)
}

/// Stateful accordion component that manages expanded/collapsed state.
pub struct AccordionState {
    id: ElementId,
    items: Vec<AccordionItem>,
    mode: AccordionMode,
    expanded: HashSet<ElementId>,
}

impl EventEmitter<AccordionChanged> for AccordionState {}

impl AccordionState {
    /// Create a new accordion state from an accordion builder.
    pub fn new(accordion: Accordion) -> Self {
        Self {
            id: accordion.id,
            items: accordion.items,
            mode: accordion.mode,
            expanded: accordion.default_expanded,
        }
    }

    /// Check if an item is expanded.
    pub fn is_expanded(&self, item_id: &ElementId) -> bool {
        self.expanded.contains(item_id)
    }

    /// Toggle an item's expanded state.
    pub fn toggle(&mut self, item_id: ElementId, cx: &mut Context<Self>) {
        // Find the item and check if it's disabled
        let item = self.items.iter().find(|i| i.id == item_id);
        if let Some(item) = item {
            if item.disabled {
                return;
            }
        }

        let was_expanded = self.expanded.contains(&item_id);

        if was_expanded {
            self.expanded.remove(&item_id);
        } else {
            if self.mode == AccordionMode::Single {
                self.expanded.clear();
            }
            self.expanded.insert(item_id.clone());
        }

        cx.emit(AccordionChanged {
            item_id,
            expanded: !was_expanded,
        });
        cx.notify();
    }

    /// Expand an item.
    pub fn expand(&mut self, item_id: ElementId, cx: &mut Context<Self>) {
        if !self.expanded.contains(&item_id) {
            if self.mode == AccordionMode::Single {
                self.expanded.clear();
            }
            self.expanded.insert(item_id.clone());
            cx.emit(AccordionChanged {
                item_id,
                expanded: true,
            });
            cx.notify();
        }
    }

    /// Collapse an item.
    pub fn collapse(&mut self, item_id: ElementId, cx: &mut Context<Self>) {
        if self.expanded.remove(&item_id) {
            cx.emit(AccordionChanged {
                item_id,
                expanded: false,
            });
            cx.notify();
        }
    }

    /// Collapse all items.
    pub fn collapse_all(&mut self, cx: &mut Context<Self>) {
        self.expanded.clear();
        cx.notify();
    }

    /// Expand all items (only works in multiple mode).
    pub fn expand_all(&mut self, cx: &mut Context<Self>) {
        if self.mode == AccordionMode::Multiple {
            for item in &self.items {
                if !item.disabled {
                    self.expanded.insert(item.id.clone());
                }
            }
            cx.notify();
        }
    }
}

impl Render for AccordionState {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();

        div()
            .id(self.id.clone())
            .flex()
            .flex_col()
            .w_full()
            .border_1()
            .border_color(theme.border())
            .rounded(rems(0.5))
            .overflow_hidden()
            .children(self.items.iter().enumerate().map(|(index, item)| {
                let is_expanded = self.expanded.contains(&item.id);
                let is_first = index == 0;
                let is_last = index == self.items.len() - 1;
                let item_id = item.id.clone();
                let header = item.header.clone();
                let content = item.content.clone();
                let disabled = item.disabled;
                let theme = cx.theme();

                div()
                    .flex()
                    .flex_col()
                    .when(!is_first, |this| {
                        this.border_t_1().border_color(theme.border_subtle())
                    })
                    .child(
                        // Header
                        div()
                            .id(item_id.clone())
                            .flex()
                            .items_center()
                            .justify_between()
                            .px(rems(0.75))
                            .py(rems(0.5))
                            .bg(theme.surface())
                            .when(!disabled, |this| {
                                this.cursor_pointer()
                                    .hover(|style| style.bg(theme.surface_secondary()))
                                    .on_click(cx.listener(move |this, _, _window, cx| {
                                        this.toggle(item_id.clone(), cx);
                                    }))
                            })
                            .when(disabled, |this| {
                                this.cursor_not_allowed().opacity(0.5)
                            })
                            .child(
                                div()
                                    .text_sm()
                                    .font_weight(gpui::FontWeight::MEDIUM)
                                    .text_color(if disabled {
                                        theme.fg_disabled()
                                    } else {
                                        theme.fg()
                                    })
                                    .child(header),
                            )
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .child(
                                        if is_expanded {
                                            Icons::chevron_down()
                                        } else {
                                            Icons::chevron_right()
                                        }
                                        .size(px(14.))
                                        .text_color(theme.fg_muted()),
                                    ),
                            ),
                    )
                    .when(is_expanded, |this| {
                        this.child(
                            // Content
                            div()
                                .px(rems(0.75))
                                .py(rems(0.5))
                                .bg(theme.surface())
                                .border_t_1()
                                .border_color(theme.border_subtle())
                                .when(!is_last, |this| {
                                    this.border_b_0()
                                })
                                .child(
                                    div()
                                        .text_sm()
                                        .text_color(theme.fg_muted())
                                        .when_some(content, |this, content| {
                                            this.child(content)
                                        }),
                                ),
                        )
                    })
            }))
    }
}
