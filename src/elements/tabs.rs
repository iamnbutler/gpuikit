//! Tabs component for organizing content into multiple panels

use crate::layout::h_stack;
use crate::theme::{ActiveTheme, Themeable};
use crate::traits::disableable::Disableable;
use gpui::{
    div, prelude::*, rems, AnyElement, App, Context, ElementId, EventEmitter, InteractiveElement,
    IntoElement, MouseButton, ParentElement, Render, SharedString, StatefulInteractiveElement,
    Styled, Window,
};

/// Event emitted when the selected tab changes
pub struct TabsChanged {
    pub selected_id: SharedString,
}

/// A single tab definition
pub struct Tab {
    id: SharedString,
    label: SharedString,
    disabled: bool,
    content: Box<dyn Fn(&mut Window, &mut App) -> AnyElement + 'static>,
}

impl Tab {
    pub fn new(
        id: impl Into<SharedString>,
        label: impl Into<SharedString>,
        content: impl Fn(&mut Window, &mut App) -> AnyElement + 'static,
    ) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            disabled: false,
            content: Box::new(content),
        }
    }

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
    selected_id: Option<SharedString>,
}

impl EventEmitter<TabsChanged> for Tabs {}

impl Tabs {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            tabs: Vec::new(),
            selected_id: None,
        }
    }

    /// Add a tab to the tabs component
    pub fn tab(mut self, tab: Tab) -> Self {
        // If no tab is selected yet, select the first non-disabled tab
        if self.selected_id.is_none() && !tab.disabled {
            self.selected_id = Some(tab.id.clone());
        }
        self.tabs.push(tab);
        self
    }

    /// Set the selected tab by id
    pub fn selected(mut self, id: impl Into<SharedString>) -> Self {
        self.selected_id = Some(id.into());
        self
    }

    /// Get the currently selected tab id
    pub fn get_selected(&self) -> Option<&SharedString> {
        self.selected_id.as_ref()
    }

    /// Set the selected tab and emit a change event
    pub fn set_selected(&mut self, id: SharedString, cx: &mut Context<Self>) {
        // Check if the tab exists and is not disabled
        let can_select = self
            .tabs
            .iter()
            .find(|t| t.id == id)
            .map(|t| !t.disabled)
            .unwrap_or(false);

        if can_select && self.selected_id.as_ref() != Some(&id) {
            self.selected_id = Some(id.clone());
            cx.emit(TabsChanged { selected_id: id });
            cx.notify();
        }
    }

    fn select_tab(&mut self, id: SharedString, cx: &mut Context<Self>) {
        self.set_selected(id, cx);
    }
}

impl Render for Tabs {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let selected_id = self.selected_id.clone();

        // Tab list (triggers)
        let tab_list = h_stack()
            .gap(rems(0.25))
            .border_b_1()
            .border_color(theme.border())
            .pb(rems(0.5))
            .children(self.tabs.iter().enumerate().map(|(index, tab)| {
                let tab_id = tab.id.clone();
                let is_selected = selected_id.as_ref() == Some(&tab_id);
                let is_disabled = tab.disabled;
                let label = tab.label.clone();

                let text_color = if is_disabled {
                    theme.fg_disabled()
                } else if is_selected {
                    theme.accent()
                } else {
                    theme.fg_muted()
                };

                let bg_color = if is_selected {
                    theme.accent_bg()
                } else {
                    gpui::transparent_black()
                };

                let tab_id_for_click = tab_id.clone();

                div()
                    .id(ElementId::NamedInteger("tab-trigger".into(), index as u64))
                    .px(rems(0.75))
                    .py(rems(0.375))
                    .rounded(rems(0.25))
                    .text_sm()
                    .text_color(text_color)
                    .bg(bg_color)
                    .when(is_selected, |this| {
                        this.border_b_2().border_color(theme.accent())
                    })
                    .when(!is_disabled, |this| {
                        this.cursor_pointer()
                            .hover(|style| style.bg(theme.accent_bg_hover()))
                            .on_mouse_down(MouseButton::Left, |_, window, _| {
                                window.prevent_default()
                            })
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.select_tab(tab_id_for_click.clone(), cx);
                            }))
                    })
                    .when(is_disabled, |this| this.cursor_not_allowed().opacity(0.5))
                    .child(label)
            }));

        // Tab panel (content)
        let tab_panel = div().pt(rems(0.75)).children(
            self.tabs
                .iter()
                .filter(|tab| selected_id.as_ref() == Some(&tab.id))
                .map(|tab| (tab.content)(window, &mut *cx)),
        );

        div()
            .id(self.id.clone())
            .flex()
            .flex_col()
            .child(tab_list)
            .child(tab_panel)
    }
}

/// Convenience function to create a tabs container
pub fn tabs(id: impl Into<ElementId>) -> Tabs {
    Tabs::new(id)
}

/// Convenience function to create a tab
pub fn tab(
    id: impl Into<SharedString>,
    label: impl Into<SharedString>,
    content: impl Fn(&mut Window, &mut App) -> AnyElement + 'static,
) -> Tab {
    Tab::new(id, label, content)
}
