//! Generic text input field component for GPUI applications.

use gpui::{
    div, prelude::FluentBuilder, px, App, ClickEvent, Context, ElementId, FocusHandle, Focusable,
    InteractiveElement, IntoElement, KeyDownEvent, ParentElement, Render, SharedString,
    StatefulInteractiveElement, Styled, Window,
};
use std::ops::Range;
use theme::ActiveTheme;

/// A generic text input field component with focus management and editing capabilities.
pub struct TextInput {
    id: ElementId,
    value: String,
    placeholder: Option<SharedString>,
    focus_handle: FocusHandle,
    selected_range: Range<usize>,
    width: Option<f32>,
    was_focused: bool,
}

impl TextInput {
    pub fn new(id: impl Into<ElementId>, value: impl Into<String>, cx: &mut Context<Self>) -> Self {
        Self {
            id: id.into(),
            value: value.into(),
            placeholder: None,
            focus_handle: cx.focus_handle(),
            selected_range: 0..0,
            width: None,
            was_focused: false,
        }
    }

    pub fn placeholder(mut self, placeholder: impl Into<SharedString>) -> Self {
        self.placeholder = Some(placeholder.into());
        self
    }

    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    pub fn update_value(&mut self, value: impl Into<String>, cx: &mut Context<Self>) {
        self.value = value.into();
        self.selected_range = 0..0;
        cx.notify();
    }

    pub fn get_value(&self) -> &str {
        &self.value
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

    fn insert_text(&mut self, text: &str, cx: &mut Context<Self>) {
        self.value.replace_range(self.selected_range.clone(), text);
        let new_cursor = self.selected_range.start + text.len();
        self.selected_range = new_cursor..new_cursor;
        cx.notify();
    }

    fn backspace(&mut self, cx: &mut Context<Self>) {
        if self.selected_range.is_empty() && self.selected_range.start > 0 {
            let start = self.selected_range.start - 1;
            self.value.remove(start);
            self.selected_range = start..start;
        } else if !self.selected_range.is_empty() {
            self.value.replace_range(self.selected_range.clone(), "");
            let cursor = self.selected_range.start;
            self.selected_range = cursor..cursor;
        }
        cx.notify();
    }

    fn select_all(&mut self) {
        self.selected_range = 0..self.value.len();
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
        } else if self.selected_range.end < self.value.len() {
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

impl Focusable for TextInput {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for TextInput {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let is_focused = self.focus_handle.is_focused(window);
        let is_empty = self.value.is_empty();

        // Check if focus state changed
        if self.was_focused && !is_focused {
            // Lost focus
            self.selected_range = 0..0;
        } else if !self.was_focused && is_focused {
            // Gained focus - select all
            self.selected_range = 0..self.value.len();
        }
        self.was_focused = is_focused;

        div()
            .id(self.id.clone())
            .key_context("TextInput")
            .track_focus(&self.focus_handle)
            .on_click(cx.listener(|this, event: &ClickEvent, window, cx| {
                this.on_click(event, window, cx);
            }))
            .on_key_down(cx.listener(|this, event: &KeyDownEvent, window, cx| {
                match event.keystroke.key.as_str() {
                    "enter" => {
                        window.blur();
                    }
                    "escape" => {
                        window.blur();
                    }
                    "backspace" => this.backspace(cx),
                    "left" => this.move_cursor_left(),
                    "right" => this.move_cursor_right(),
                    "a" if event.keystroke.modifiers.platform => this.select_all(),
                    _ => {
                        // Handle regular character input
                        if let Some(key_char) = &event.keystroke.key_char {
                            if !event.keystroke.modifiers.control
                                && !event.keystroke.modifiers.platform
                            {
                                this.insert_text(key_char, cx);
                            }
                        }
                    }
                }
                cx.notify();
            }))
            .flex()
            .items_center()
            .pl(px(8.))
            .pr(px(8.))
            .h(px(28.))
            .when_some(self.width, |d, w| d.w(px(w)))
            .rounded(px(4.))
            .bg(theme.surface)
            .border_1()
            .border_color(if is_focused {
                theme.outline
            } else {
                theme.border
            })
            .text_color(theme.fg)
            .text_size(px(13.))
            .child(
                div()
                    .flex_1()
                    .overflow_hidden()
                    .relative()
                    .when(is_empty && !is_focused, |d| {
                        d.when_some(self.placeholder.clone(), |d, placeholder| {
                            d.child(
                                div()
                                    .absolute()
                                    .left_0()
                                    .text_color(theme.fg.alpha(0.5))
                                    .child(placeholder),
                            )
                        })
                    })
                    .when(!is_empty || is_focused, |d| {
                        d.child(
                            div()
                                .when(is_focused && !self.selected_range.is_empty(), |d| {
                                    d.bg(theme.outline.alpha(0.3))
                                })
                                .child(self.value.clone()),
                        )
                    })
                    .when(is_focused && self.selected_range.is_empty(), |d| {
                        // Show cursor
                        let cursor_offset = self.selected_range.start;
                        let text_before = &self.value[..cursor_offset];
                        d.child(
                            div()
                                .absolute()
                                .left(px(text_before.len() as f32 * 7.0)) // Approximate char width
                                .w(px(1.))
                                .h(px(16.))
                                .bg(theme.fg),
                        )
                    }),
            )
    }
}
