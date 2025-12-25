//! Breadcrumb component for gpuikit

use crate::layout::h_stack;
use crate::theme::{ActiveTheme, Themeable};
use gpui::{
    div, prelude::*, rems, App, ClickEvent, ElementId, InteractiveElement, IntoElement,
    MouseButton, ParentElement, RenderOnce, SharedString, StatefulInteractiveElement, Styled,
    Window,
};

/// A single breadcrumb item
pub struct BreadcrumbItem {
    label: SharedString,
    on_click: Option<Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>>,
}

impl BreadcrumbItem {
    pub fn new(label: impl Into<SharedString>) -> Self {
        Self {
            label: label.into(),
            on_click: None,
        }
    }

    pub fn on_click(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_click = Some(Box::new(handler));
        self
    }
}

/// Separator style for breadcrumbs
#[derive(Clone, Copy, Default)]
pub enum BreadcrumbSeparator {
    #[default]
    Slash,
    Chevron,
    Arrow,
    Dot,
}

impl BreadcrumbSeparator {
    fn as_str(&self) -> &'static str {
        match self {
            BreadcrumbSeparator::Slash => "/",
            BreadcrumbSeparator::Chevron => "›",
            BreadcrumbSeparator::Arrow => "→",
            BreadcrumbSeparator::Dot => "•",
        }
    }
}

/// A breadcrumb navigation component
#[derive(IntoElement)]
pub struct Breadcrumb {
    id: ElementId,
    items: Vec<BreadcrumbItem>,
    separator: BreadcrumbSeparator,
}

impl Breadcrumb {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            items: Vec::new(),
            separator: BreadcrumbSeparator::default(),
        }
    }

    pub fn item(mut self, item: BreadcrumbItem) -> Self {
        self.items.push(item);
        self
    }

    pub fn items(mut self, items: impl IntoIterator<Item = BreadcrumbItem>) -> Self {
        self.items.extend(items);
        self
    }

    pub fn separator(mut self, separator: BreadcrumbSeparator) -> Self {
        self.separator = separator;
        self
    }
}

impl RenderOnce for Breadcrumb {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let separator_str = self.separator.as_str();
        let item_count = self.items.len();

        h_stack()
            .id(self.id)
            .gap(rems(0.375))
            .items_center()
            .text_sm()
            .children(
                self.items
                    .into_iter()
                    .enumerate()
                    .flat_map(|(index, item)| {
                        let is_last = index == item_count - 1;
                        let is_clickable = item.on_click.is_some();

                        let item_element = div()
                            .id(ElementId::NamedInteger(
                                "breadcrumb-item".into(),
                                index as u64,
                            ))
                            .text_color(if is_last {
                                theme.fg()
                            } else {
                                theme.fg_muted()
                            })
                            .when(is_clickable && !is_last, |this| {
                                this.cursor_pointer()
                                    .hover(|style| style.text_color(theme.fg()))
                            })
                            .when(!is_clickable || is_last, |this| this.cursor_default())
                            .when_some(item.on_click.filter(|_| !is_last), |this, handler| {
                                this.on_mouse_down(MouseButton::Left, |_, window, _| {
                                    window.prevent_default()
                                })
                                .on_click(
                                    move |event, window, cx| {
                                        handler(event, window, cx);
                                    },
                                )
                            })
                            .child(item.label);

                        let separator_element = if !is_last {
                            Some(
                                div()
                                    .text_color(theme.fg_muted())
                                    .flex_none()
                                    .child(separator_str),
                            )
                        } else {
                            None
                        };

                        std::iter::once(item_element.into_any_element())
                            .chain(separator_element.map(|s| s.into_any_element()))
                    }),
            )
    }
}

/// Convenience function to create a breadcrumb
pub fn breadcrumb(id: impl Into<ElementId>) -> Breadcrumb {
    Breadcrumb::new(id)
}

/// Convenience function to create a breadcrumb item
pub fn breadcrumb_item(label: impl Into<SharedString>) -> BreadcrumbItem {
    BreadcrumbItem::new(label)
}
