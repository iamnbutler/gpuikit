//! Toggle group component for gpuikit
//!
//! A toggle group allows selecting one or multiple options from a group of toggle buttons.

use crate::theme::{ActiveTheme, Themeable};
use crate::traits::disableable::Disableable;
use crate::traits::orientable::{Orientable, Orientation};
use gpui::{
    div, prelude::*, rems, Context, Div, ElementId, EventEmitter, FontWeight, Hsla,
    InteractiveElement, IntoElement, MouseButton, ParentElement, Render, SharedString, Stateful,
    StatefulInteractiveElement, Styled, Window,
};

/// Selection mode for the toggle group
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ToggleGroupMode {
    /// Only one item can be selected at a time (like radio buttons)
    #[default]
    Single,
    /// Multiple items can be selected simultaneously
    Multiple,
}

/// Event emitted when the toggle group selection changes
pub struct ToggleGroupChanged<T: Clone> {
    /// Currently selected values
    pub selected: Vec<T>,
}

/// A single toggle option in the group
#[derive(Clone)]
pub struct ToggleOption<T: Clone> {
    /// The value associated with this option
    pub value: T,
    /// Display label for the option
    pub label: SharedString,
    /// Whether this option is disabled
    pub disabled: bool,
}

impl<T: Clone> ToggleOption<T> {
    /// Create a new toggle option with a value and label
    pub fn new(value: T, label: impl Into<SharedString>) -> Self {
        Self {
            value,
            label: label.into(),
            disabled: false,
        }
    }

    /// Set whether this option is disabled
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

impl<T: Clone> Disableable for ToggleOption<T> {
    fn is_disabled(&self) -> bool {
        self.disabled
    }

    fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

/// A toggle group component for selecting one or multiple options
///
/// # Example
///
/// ```ignore
/// // Single-select mode (default)
/// let single = toggle_group(
///     "alignment",
///     vec![
///         toggle_option("left", "Left"),
///         toggle_option("center", "Center"),
///         toggle_option("right", "Right"),
///     ],
/// ).selected(vec!["center"]);
///
/// // Multi-select mode
/// let multi = toggle_group(
///     "features",
///     vec![
///         toggle_option("bold", "B"),
///         toggle_option("italic", "I"),
///         toggle_option("underline", "U"),
///     ],
/// ).mode(ToggleGroupMode::Multiple)
///  .selected(vec!["bold", "italic"]);
/// ```
pub struct ToggleGroup<T: Clone + PartialEq + 'static> {
    id: ElementId,
    options: Vec<ToggleOption<T>>,
    selected: Vec<T>,
    mode: ToggleGroupMode,
    disabled: bool,
    orientation: Orientation,
}

impl<T: Clone + PartialEq + 'static> EventEmitter<ToggleGroupChanged<T>> for ToggleGroup<T> {}

impl<T: Clone + PartialEq + 'static> ToggleGroup<T> {
    /// Create a new toggle group with an ID and options
    pub fn new(id: impl Into<ElementId>, options: Vec<ToggleOption<T>>) -> Self {
        Self {
            id: id.into(),
            options,
            selected: Vec::new(),
            mode: ToggleGroupMode::default(),
            disabled: false,
            orientation: Orientation::Horizontal,
        }
    }

    /// Set the selection mode (single or multiple)
    pub fn mode(mut self, mode: ToggleGroupMode) -> Self {
        self.mode = mode;
        self
    }

    /// Set the selected values
    pub fn selected(mut self, values: Vec<T>) -> Self {
        self.selected = values;
        self
    }

    /// Set a single selected value (convenience for single-select mode)
    pub fn selected_value(mut self, value: T) -> Self {
        self.selected = vec![value];
        self
    }

    /// Get the currently selected values
    pub fn get_selected(&self) -> &[T] {
        &self.selected
    }

    /// Check if a value is selected
    pub fn is_value_selected(&self, value: &T) -> bool {
        self.selected.contains(value)
    }

    /// Set the selected values programmatically
    pub fn set_selected(&mut self, values: Vec<T>, cx: &mut Context<Self>) {
        if self.selected != values {
            self.selected = values.clone();
            cx.emit(ToggleGroupChanged { selected: values });
            cx.notify();
        }
    }

    fn toggle_option(&mut self, index: usize, cx: &mut Context<Self>) {
        if let Some(option) = self.options.get(index) {
            if option.disabled || self.disabled {
                return;
            }

            let value = option.value.clone();
            let is_selected = self.selected.contains(&value);

            match self.mode {
                ToggleGroupMode::Single => {
                    // In single mode, always select the clicked option (unless it's already selected)
                    if !is_selected {
                        self.selected = vec![value.clone()];
                        cx.emit(ToggleGroupChanged {
                            selected: vec![value],
                        });
                        cx.notify();
                    }
                }
                ToggleGroupMode::Multiple => {
                    // In multiple mode, toggle the selection
                    if is_selected {
                        self.selected.retain(|v| v != &value);
                    } else {
                        self.selected.push(value);
                    }
                    cx.emit(ToggleGroupChanged {
                        selected: self.selected.clone(),
                    });
                    cx.notify();
                }
            }
        }
    }
}

impl<T: Clone + PartialEq + 'static> Render for ToggleGroup<T> {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let group_disabled = self.disabled;
        let selected = self.selected.clone();
        let orientation = self.orientation;
        let num_options = self.options.len();

        let container = if orientation == Orientation::Vertical {
            div().flex().flex_col()
        } else {
            div().flex().flex_row()
        };

        container
            .id(self.id.clone())
            .bg(theme.surface_secondary())
            .border_1()
            .border_color(theme.border())
            .rounded_md()
            .p(rems(0.125))
            .gap(rems(0.125))
            .children(
                self.options
                    .iter()
                    .enumerate()
                    .map(|(index, option)| {
                        let is_selected = selected.contains(&option.value);
                        let is_disabled = group_disabled || option.disabled;
                        let label = option.label.clone();
                        let is_first = index == 0;
                        let is_last = index == num_options - 1;

                        let bg: Option<Hsla> = if is_disabled {
                            Some(theme.surface_tertiary())
                        } else if is_selected {
                            Some(theme.surface())
                        } else {
                            None
                        };

                        let text_color = if is_disabled {
                            theme.fg_disabled()
                        } else if is_selected {
                            theme.fg()
                        } else {
                            theme.fg_muted()
                        };

                        div()
                            .id(ElementId::NamedInteger("toggle-option".into(), index as u64))
                            .flex()
                            .items_center()
                            .justify_center()
                            .px(rems(0.75))
                            .py(rems(0.375))
                            .text_sm()
                            .font_weight(FontWeight::MEDIUM)
                            .when_some(bg, |this, bg| this.bg(bg))
                            .text_color(text_color)
                            .when(is_selected && !is_disabled, |this: Stateful<Div>| {
                                this.shadow_sm()
                            })
                            // Apply rounded corners based on orientation and position
                            .when(orientation == Orientation::Horizontal, |this| {
                                this.when(is_first, |t| t.rounded_l_sm())
                                    .when(is_last, |t| t.rounded_r_sm())
                                    .when(!is_first && !is_last, |t| t.rounded_none())
                            })
                            .when(orientation == Orientation::Vertical, |this| {
                                this.when(is_first, |t| t.rounded_t_sm())
                                    .when(is_last, |t| t.rounded_b_sm())
                                    .when(!is_first && !is_last, |t| t.rounded_none())
                            })
                            .when(!is_disabled, |this| {
                                this.cursor_pointer()
                                    .hover(|style| {
                                        if is_selected {
                                            style
                                        } else {
                                            style.bg(theme.surface_tertiary())
                                        }
                                    })
                                    .on_mouse_down(MouseButton::Left, |_, window, _| {
                                        window.prevent_default()
                                    })
                                    .on_click(cx.listener(move |this, _, _, cx| {
                                        this.toggle_option(index, cx);
                                    }))
                            })
                            .when(is_disabled, |this| this.cursor_not_allowed().opacity(0.65))
                            .child(label)
                    })
                    .collect::<Vec<_>>(),
            )
    }
}

/// Convenience function to create a toggle group
pub fn toggle_group<T: Clone + PartialEq + 'static>(
    id: impl Into<ElementId>,
    options: Vec<ToggleOption<T>>,
) -> ToggleGroup<T> {
    ToggleGroup::new(id, options)
}

/// Convenience function to create a toggle option
pub fn toggle_option<T: Clone>(value: T, label: impl Into<SharedString>) -> ToggleOption<T> {
    ToggleOption::new(value, label)
}

impl<T: Clone + PartialEq + 'static> Disableable for ToggleGroup<T> {
    fn is_disabled(&self) -> bool {
        self.disabled
    }

    fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

impl<T: Clone + PartialEq + 'static> Orientable for ToggleGroup<T> {
    fn orientation(mut self, orientation: Orientation) -> Self {
        self.orientation = orientation;
        self
    }
}
