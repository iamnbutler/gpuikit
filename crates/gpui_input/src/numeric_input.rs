//! Numeric input field component for GPUI applications.

use gpui::{
    div, prelude::FluentBuilder, px, App, ClickEvent, Context, ElementId, FocusHandle, Focusable,
    InteractiveElement, IntoElement, KeyDownEvent, ParentElement, Render, SharedString,
    StatefulInteractiveElement, Styled, Window,
};
use std::ops::Range;
use theme::ActiveTheme;

/// A numeric input field that only accepts numeric values.
pub struct NumericInput {
    id: ElementId,
    value: Option<f32>,
    placeholder: Option<SharedString>,
    icon: Option<SharedString>,
    focus_handle: FocusHandle,
    content: String,
    selected_range: Range<usize>,
    min: Option<f32>,
    max: Option<f32>,
    width: Option<f32>,
    was_focused: bool,
}

impl NumericInput {
    pub fn new(id: impl Into<ElementId>, value: Option<f32>, cx: &mut Context<Self>) -> Self {
        Self {
            id: id.into(),
            value,
            placeholder: None,
            icon: None,
            focus_handle: cx.focus_handle(),
            content: value.map(|v| v.to_string()).unwrap_or_default(),
            selected_range: 0..0,
            min: None,
            max: None,
            width: None,
            was_focused: false,
        }
    }

    pub fn placeholder(mut self, placeholder: impl Into<SharedString>) -> Self {
        self.placeholder = Some(placeholder.into());
        self
    }

    pub fn icon(mut self, icon: impl Into<SharedString>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    pub fn min(mut self, min: f32) -> Self {
        self.min = Some(min);
        self
    }

    pub fn max(mut self, max: f32) -> Self {
        self.max = Some(max);
        self
    }

    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    pub fn update_value(&mut self, value: Option<f32>, cx: &mut Context<Self>) {
        self.value = value;
        self.content = value.map(|v| v.to_string()).unwrap_or_default();
        self.selected_range = 0..0;
        cx.notify();
    }

    pub fn get_value(&self) -> Option<f32> {
        self.value
    }

    pub fn is_focused(&self, window: &Window) -> bool {
        self.focus_handle.is_focused(window)
    }

    pub fn focus(&self, window: &mut Window) {
        window.focus(&self.focus_handle);
    }

    pub fn blur(&self, window: &mut Window) {
        window.blur();
    }

    fn apply_value(&mut self, cx: &mut Context<Self>) {
        if let Ok(mut new_value) = self.content.parse::<f32>() {
            // Apply min/max constraints
            if let Some(min) = self.min {
                new_value = new_value.max(min);
            }
            if let Some(max) = self.max {
                new_value = new_value.min(max);
            }

            self.value = Some(new_value);
            self.content = new_value.to_string();
        } else if self.content.is_empty() {
            self.value = None;
        }

        // Notify parent that value has changed
        cx.notify();
    }

    fn insert_text(&mut self, text: &str) {
        // Only allow numeric input (digits, decimal point, minus sign)
        let filtered: String = text
            .chars()
            .filter(|c| c.is_ascii_digit() || *c == '.' || *c == '-')
            .collect();

        if !filtered.is_empty() {
            self.content
                .replace_range(self.selected_range.clone(), &filtered);
            let new_cursor = self.selected_range.start + filtered.len();
            self.selected_range = new_cursor..new_cursor;
        }
    }

    fn backspace(&mut self) {
        if self.selected_range.is_empty() && self.selected_range.start > 0 {
            let start = self.selected_range.start - 1;
            self.content.remove(start);
            self.selected_range = start..start;
        } else if !self.selected_range.is_empty() {
            self.content.replace_range(self.selected_range.clone(), "");
            let cursor = self.selected_range.start;
            self.selected_range = cursor..cursor;
        }
    }

    fn select_all(&mut self) {
        self.selected_range = 0..self.content.len();
    }

    fn move_cursor_left(&mut self) {
        if !self.selected_range.is_empty() {
            self.selected_range = self.selected_range.start..self.selected_range.start;
        } else if self.selected_range.start > 0 {
            let new_pos = self.selected_range.start - 1;
            self.selected_range = new_pos..new_pos;
        }
    }

    fn move_cursor_right(&mut self) {
        if !self.selected_range.is_empty() {
            self.selected_range = self.selected_range.end..self.selected_range.end;
        } else if self.selected_range.end < self.content.len() {
            let new_pos = self.selected_range.end + 1;
            self.selected_range = new_pos..new_pos;
        }
    }

    fn on_click(&mut self, _event: &ClickEvent, window: &mut Window, cx: &mut Context<Self>) {
        if !self.focus_handle.is_focused(window) {
            window.focus(&self.focus_handle);
            self.select_all();
        }
        cx.notify();
    }
}

impl Focusable for NumericInput {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for NumericInput {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let is_focused = self.focus_handle.is_focused(window);
        let is_empty = self.content.is_empty();

        // Clone theme values we need before mutable borrows
        let bg_color = theme.surface;
        let outline_color = theme.outline;
        let border_color = theme.border;
        let fg_color = theme.fg;

        // Check if focus state changed
        if self.was_focused && !is_focused {
            // Lost focus - apply value
            self.apply_value(cx);
            self.selected_range = 0..0;
        } else if !self.was_focused && is_focused {
            // Gained focus - select all
            self.selected_range = 0..self.content.len();
        }
        self.was_focused = is_focused;

        div()
            .id(self.id.clone())
            .key_context("NumericInput")
            .track_focus(&self.focus_handle)
            .on_click(cx.listener(|this, event: &ClickEvent, window, cx| {
                this.on_click(event, window, cx);
            }))
            .on_key_down(cx.listener(|this, event: &KeyDownEvent, window, cx| {
                match event.keystroke.key.as_str() {
                    "enter" => {
                        this.apply_value(cx);
                        window.blur();
                    }
                    "escape" => {
                        // Reset to original value
                        this.content = this.value.map(|v| v.to_string()).unwrap_or_default();
                        this.selected_range = 0..0;
                        window.blur();
                    }
                    "backspace" => this.backspace(),
                    "left" => this.move_cursor_left(),
                    "right" => this.move_cursor_right(),
                    "a" if event.keystroke.modifiers.platform => this.select_all(),
                    _ => {
                        // Handle regular character input
                        if let Some(key_char) = &event.keystroke.key_char {
                            if !event.keystroke.modifiers.control
                                && !event.keystroke.modifiers.platform
                            {
                                this.insert_text(key_char);
                            }
                        }
                    }
                }
                cx.notify();
            }))
            .flex()
            .flex_row()
            .items_center()
            .pl(px(6.))
            .pr(px(4.))
            .when_some(self.width, |d, w| d.w(px(w)))
            .when(self.width.is_none(), |d| d.w(px(84.)))
            .h(px(24.))
            .rounded(px(4.))
            .bg(bg_color)
            .border_1()
            .border_color(if is_focused {
                outline_color
            } else {
                border_color
            })
            .text_color(fg_color)
            .when(is_empty, |this| this.text_color(fg_color.alpha(0.5)))
            .text_size(px(11.))
            .child(
                div()
                    .flex_1()
                    .overflow_hidden()
                    .relative()
                    .when(is_empty && !is_focused, |d| {
                        d.when_some(self.placeholder.clone(), |d, placeholder| {
                            d.child(div().text_color(fg_color.alpha(0.5)).child(placeholder))
                        })
                    })
                    .when(!is_empty || is_focused, |d| {
                        d.child(
                            div()
                                .when(is_focused && !self.selected_range.is_empty(), |d| {
                                    d.bg(outline_color.alpha(0.3))
                                })
                                .child(self.content.clone()),
                        )
                    })
                    .when(
                        is_focused && self.selected_range.is_empty() && !is_empty,
                        |d| {
                            // Show cursor
                            let cursor_offset = self.selected_range.start;
                            let text_before = &self.content[..cursor_offset];
                            d.child(
                                div()
                                    .absolute()
                                    .left(px(text_before.len() as f32 * 6.0)) // Approximate char width for smaller text
                                    .w(px(1.))
                                    .h(px(12.))
                                    .bg(fg_color),
                            )
                        },
                    ),
            )
            .when_some(self.icon.clone(), |d, icon| {
                d.child(
                    div()
                        .flex()
                        .justify_center()
                        .flex_none()
                        .overflow_hidden()
                        .w(px(11.))
                        .h_full()
                        .child(icon),
                )
            })
    }
}
