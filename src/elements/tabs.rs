//! Tabs component for gpuikit
//!
//! A tabbed interface for organizing content into multiple panels.

use crate::layout::h_stack;
use crate::theme::{ActiveTheme, Themeable};
use crate::traits::disableable::Disableable;
use gpui::{
    div, prelude::*, px, rems, Context, ElementId, EventEmitter, InteractiveElement, IntoElement,
    MouseButton, ParentElement, Render, SharedString, StatefulInteractiveElement, Styled, Window,
};

/// Event emitted when the selected tab changes
pub struct TabChanged {
    /// The ID of the newly selected tab
    pub tab_id: SharedString,
}

/// A single tab definition
#[derive(Clone)]
pub struct Tab {
    /// Unique identifier for the tab
    pub id: SharedString,
    /// Display label for the tab
    pub label: SharedString,
    /// Whether this tab is disabled
    pub disabled: bool,
}

impl Tab {
    /// Create a new tab with an ID and label
    pub fn new(id: impl Into<SharedString>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            disabled: false,
        }
    }

    /// Set whether this tab is disabled
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

impl Disableable for Tab {
    fn is_disabled(&self) -> bool {
        self.disabled
    }

    fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

/// A tabs component for organizing content into multiple panels
pub struct Tabs {
    id: ElementId,
    tabs: Vec<Tab>,
    selected: Option<SharedString>,
    disabled: bool,
}

impl EventEmitter<TabChanged> for Tabs {}

impl Tabs {
    /// Create a new tabs component with the given ID
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            tabs: Vec::new(),
            selected: None,
            disabled: false,
        }
    }

    /// Add a tab to the tabs component
    pub fn tab(mut self, tab: Tab) -> Self {
        // If this is the first non-disabled tab and no selection exists, select it
        if self.selected.is_none() && !tab.disabled {
            self.selected = Some(tab.id.clone());
        }
        self.tabs.push(tab);
        self
    }

    /// Set the selected tab by ID
    pub fn selected(mut self, tab_id: impl Into<SharedString>) -> Self {
        self.selected = Some(tab_id.into());
        self
    }

    /// Set the selected tab by ID (optional)
    pub fn selected_option(mut self, tab_id: Option<SharedString>) -> Self {
        self.selected = tab_id;
        self
    }

    /// Disable the entire tabs component
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Get the currently selected tab ID
    pub fn get_selected(&self) -> Option<&SharedString> {
        self.selected.as_ref()
    }

    /// Set the selected tab and emit a change event
    pub fn set_selected(&mut self, tab_id: SharedString, cx: &mut Context<Self>) {
        if self.selected.as_ref() != Some(&tab_id) {
            self.selected = Some(tab_id.clone());
            cx.emit(TabChanged { tab_id });
            cx.notify();
        }
    }

    /// Select a tab by index
    fn select_tab(&mut self, index: usize, cx: &mut Context<Self>) {
        if let Some(tab) = self.tabs.get(index) {
            if !tab.disabled && !self.disabled {
                self.set_selected(tab.id.clone(), cx);
            }
        }
    }
}

impl Render for Tabs {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let group_disabled = self.disabled;
        let selected = self.selected.clone();

        let tab_list = h_stack()
            .gap(rems(0.25))
            .border_b_1()
            .border_color(theme.border())
            .pb(px(1.0));

        div()
            .id(self.id.clone())
            .flex()
            .flex_col()
            .child(
                tab_list.children(
                    self.tabs
                        .iter()
                        .enumerate()
                        .map(|(index, tab)| {
                            let is_selected = selected.as_ref() == Some(&tab.id);
                            let is_disabled = group_disabled || tab.disabled;
                            let label = tab.label.clone();

                            let text_color = if is_disabled {
                                theme.fg_disabled()
                            } else if is_selected {
                                theme.accent()
                            } else {
                                theme.fg_muted()
                            };

                            let bg = if is_selected && !is_disabled {
                                theme.accent_bg()
                            } else {
                                gpui::Hsla::transparent_black()
                            };

                            let border_color = if is_selected && !is_disabled {
                                theme.accent()
                            } else {
                                gpui::Hsla::transparent_black()
                            };

                            div()
                                .id(ElementId::NamedInteger("tab".into(), index as u64))
                                .flex()
                                .items_center()
                                .justify_center()
                                .px(rems(0.75))
                                .py(rems(0.5))
                                .text_sm()
                                .text_color(text_color)
                                .bg(bg)
                                .border_b_2()
                                .border_color(border_color)
                                .mb(px(-1.0)) // Overlap with container border
                                .rounded_t(px(4.0))
                                .when(!is_disabled, |this| {
                                    this.cursor_pointer()
                                        .on_mouse_down(MouseButton::Left, |_, window, _| {
                                            window.prevent_default()
                                        })
                                        .on_click(cx.listener(move |this, _, _, cx| {
                                            this.select_tab(index, cx);
                                        }))
                                        .hover(|style| {
                                            if is_selected {
                                                style
                                            } else {
                                                style
                                                    .text_color(theme.fg())
                                                    .bg(theme.surface_secondary())
                                            }
                                        })
                                })
                                .when(is_disabled, |this| {
                                    this.cursor_not_allowed().opacity(0.65)
                                })
                                .child(label)
                        })
                        .collect::<Vec<_>>(),
                ),
            )
    }
}

/// Convenience function to create a tabs component
pub fn tabs(id: impl Into<ElementId>) -> Tabs {
    Tabs::new(id)
}

/// Convenience function to create a tab
pub fn tab(id: impl Into<SharedString>, label: impl Into<SharedString>) -> Tab {
    Tab::new(id, label)
}

impl Disableable for Tabs {
    fn is_disabled(&self) -> bool {
        self.disabled
    }

    fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}
