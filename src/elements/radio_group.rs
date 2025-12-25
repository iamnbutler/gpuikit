//! Radio group component for gpuikit

use crate::theme::{ActiveTheme, Themeable};
use gpui::{
    div, prelude::*, rems, Context, ElementId, EventEmitter, InteractiveElement, IntoElement,
    MouseButton, ParentElement, Render, SharedString, StatefulInteractiveElement, Styled, Window,
};

/// Event emitted when the radio group selection changes
pub struct RadioGroupChanged<T: Clone> {
    pub value: T,
}

/// A single radio option
#[derive(Clone)]
pub struct RadioOption<T: Clone> {
    pub value: T,
    pub label: SharedString,
    pub disabled: bool,
}

impl<T: Clone> RadioOption<T> {
    pub fn new(value: T, label: impl Into<SharedString>) -> Self {
        Self {
            value,
            label: label.into(),
            disabled: false,
        }
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

/// A radio group component for selecting one option from a list
pub struct RadioGroup<T: Clone + PartialEq + 'static> {
    id: ElementId,
    options: Vec<RadioOption<T>>,
    selected: Option<T>,
    disabled: bool,
    orientation: Orientation,
}

#[derive(Clone, Copy, Default, PartialEq)]
pub enum Orientation {
    #[default]
    Vertical,
    Horizontal,
}

impl<T: Clone + PartialEq + 'static> EventEmitter<RadioGroupChanged<T>> for RadioGroup<T> {}

impl<T: Clone + PartialEq + 'static> RadioGroup<T> {
    pub fn new(id: impl Into<ElementId>, options: Vec<RadioOption<T>>) -> Self {
        Self {
            id: id.into(),
            options,
            selected: None,
            disabled: false,
            orientation: Orientation::default(),
        }
    }

    pub fn selected(mut self, value: T) -> Self {
        self.selected = Some(value);
        self
    }

    pub fn selected_option(mut self, value: Option<T>) -> Self {
        self.selected = value;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn orientation(mut self, orientation: Orientation) -> Self {
        self.orientation = orientation;
        self
    }

    pub fn horizontal(mut self) -> Self {
        self.orientation = Orientation::Horizontal;
        self
    }

    pub fn vertical(mut self) -> Self {
        self.orientation = Orientation::Vertical;
        self
    }

    pub fn get_selected(&self) -> Option<&T> {
        self.selected.as_ref()
    }

    pub fn set_selected(&mut self, value: T, cx: &mut Context<Self>) {
        if self.selected.as_ref() != Some(&value) {
            self.selected = Some(value.clone());
            cx.emit(RadioGroupChanged { value });
            cx.notify();
        }
    }

    fn select_option(&mut self, index: usize, cx: &mut Context<Self>) {
        if let Some(option) = self.options.get(index) {
            if !option.disabled && !self.disabled {
                self.set_selected(option.value.clone(), cx);
            }
        }
    }
}

impl<T: Clone + PartialEq + 'static> Render for RadioGroup<T> {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let group_disabled = self.disabled;
        let selected = self.selected.clone();
        let orientation = self.orientation;

        let radio_size = rems(1.0);
        let dot_size = rems(0.5);

        let container = if orientation == Orientation::Vertical {
            div().flex().flex_col().gap(rems(0.5))
        } else {
            div().flex().flex_row().gap(rems(1.0)).items_center()
        };

        container.id(self.id.clone()).children(
            self.options
                .iter()
                .enumerate()
                .map(|(index, option)| {
                    let is_selected = selected.as_ref() == Some(&option.value);
                    let is_disabled = group_disabled || option.disabled;
                    let label = option.label.clone();

                    let radio_bg = if is_disabled {
                        theme.surface_tertiary()
                    } else {
                        theme.surface()
                    };

                    let radio_border = if is_disabled {
                        theme.border_subtle()
                    } else if is_selected {
                        theme.accent()
                    } else {
                        theme.border()
                    };

                    let dot_color = if is_disabled {
                        theme.fg_disabled()
                    } else {
                        theme.accent()
                    };

                    let text_color = if is_disabled {
                        theme.fg_disabled()
                    } else {
                        theme.fg()
                    };

                    div()
                        .id(ElementId::NamedInteger("radio-option".into(), index as u64))
                        .flex()
                        .flex_row()
                        .gap(rems(0.5))
                        .items_center()
                        .when(!is_disabled, |this| {
                            this.cursor_pointer()
                                .on_mouse_down(MouseButton::Left, |_, window, _| {
                                    window.prevent_default()
                                })
                                .on_click(cx.listener(move |this, _, _, cx| {
                                    this.select_option(index, cx);
                                }))
                        })
                        .when(is_disabled, |this| this.cursor_not_allowed().opacity(0.65))
                        .child(
                            div()
                                .size(radio_size)
                                .flex_none()
                                .flex()
                                .items_center()
                                .justify_center()
                                .bg(radio_bg)
                                .border_1()
                                .border_color(radio_border)
                                .rounded_full()
                                .when(!is_disabled, |this| {
                                    this.hover(|style| {
                                        style.border_color(if is_selected {
                                            theme.accent()
                                        } else {
                                            theme.border_secondary()
                                        })
                                    })
                                })
                                .when(is_selected, |this| {
                                    this.child(div().size(dot_size).bg(dot_color).rounded_full())
                                }),
                        )
                        .child(div().text_sm().text_color(text_color).child(label))
                })
                .collect::<Vec<_>>(),
        )
    }
}

/// Convenience function to create a radio group
pub fn radio_group<T: Clone + PartialEq + 'static>(
    id: impl Into<ElementId>,
    options: Vec<RadioOption<T>>,
) -> RadioGroup<T> {
    RadioGroup::new(id, options)
}

/// Convenience function to create a radio option
pub fn radio_option<T: Clone>(value: T, label: impl Into<SharedString>) -> RadioOption<T> {
    RadioOption::new(value, label)
}
