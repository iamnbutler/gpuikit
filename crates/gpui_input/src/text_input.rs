//! Generic text input field component for GPUI applications.

use crate::InputHandler;
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
    disabled: bool,
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
            disabled: false,
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

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
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

    // Mutation methods that handle Context

    fn move_cursor_left(&mut self, cx: &mut Context<Self>) {
        let new_pos = self.calculate_left_movement();
        self.selected_range = new_pos..new_pos;
        cx.notify();
    }

    fn move_cursor_right(&mut self, cx: &mut Context<Self>) {
        let new_pos = self.calculate_right_movement();
        self.selected_range = new_pos..new_pos;
        cx.notify();
    }

    fn select_all(&mut self, cx: &mut Context<Self>) {
        self.selected_range = 0..self.value.len();
        cx.notify();
    }

    fn insert_text(&mut self, text: &str, cx: &mut Context<Self>) {
        let filtered = self.filter_text(text);
        if !filtered.is_empty() {
            self.value
                .replace_range(self.selected_range.clone(), &filtered);
            let new_cursor = self.selected_range.start + filtered.len();
            self.selected_range = new_cursor..new_cursor;
            cx.notify();
        }
    }

    fn backspace(&mut self, cx: &mut Context<Self>) {
        if let Some(range) = self.calculate_backspace_range() {
            self.value.replace_range(range.clone(), "");
            self.selected_range = range.start..range.start;
            cx.notify();
        }
    }

    fn delete(&mut self, cx: &mut Context<Self>) {
        if let Some(range) = self.calculate_delete_range() {
            self.value.replace_range(range.clone(), "");
            self.selected_range = range.start..range.start;
            cx.notify();
        }
    }

    fn on_click(&mut self, _event: &ClickEvent, window: &mut Window, cx: &mut Context<Self>) {
        if !self.disabled && !self.focus_handle.is_focused(window) {
            window.focus(&self.focus_handle);
            self.select_all(cx);
        }
    }
}

impl InputHandler for TextInput {
    fn content(&self) -> &str {
        &self.value
    }

    fn selection_range(&self) -> Range<usize> {
        self.selected_range.clone()
    }

    fn is_disabled(&self) -> bool {
        self.disabled
    }
}

impl Focusable for TextInput {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for TextInput {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let is_focused = self.focus_handle.is_focused(window);
        let is_empty = self.value.is_empty();

        // Check if focus state changed
        if self.was_focused && !is_focused {
            // Lost focus
            self.selected_range = 0..0;
        } else if !self.was_focused && is_focused {
            // Gained focus - select all (inline to avoid cx borrow issue)
            self.selected_range = 0..self.value.len();
            cx.notify();
        }
        self.was_focused = is_focused;

        let theme = cx.theme();

        div()
            .id(self.id.clone())
            .key_context("TextInput")
            .track_focus(&self.focus_handle)
            .on_click(cx.listener(|this, event: &ClickEvent, window, cx| {
                this.on_click(event, window, cx);
            }))
            .on_key_down(cx.listener(|this, event: &KeyDownEvent, window, cx| {
                if this.disabled {
                    return;
                }

                match event.keystroke.key.as_str() {
                    "enter" | "escape" => {
                        window.blur();
                    }
                    "backspace" => this.backspace(cx),
                    "delete" => this.delete(cx),
                    "left" => this.move_cursor_left(cx),
                    "right" => this.move_cursor_right(cx),
                    "home" => {
                        this.selected_range = 0..0;
                        cx.notify();
                    }
                    "end" => {
                        let len = this.value.len();
                        this.selected_range = len..len;
                        cx.notify();
                    }
                    "a" if event.keystroke.modifiers.platform => this.select_all(cx),
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
            .border_color(if is_focused && !self.disabled {
                theme.outline
            } else {
                theme.border
            })
            .text_color(if self.disabled {
                theme.fg.alpha(0.3)
            } else {
                theme.fg
            })
            .text_size(px(13.))
            .cursor_pointer()
            .when(self.disabled, |d| d.cursor_not_allowed())
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
                                .when(is_focused && self.has_selection(), |d| {
                                    d.bg(theme.outline.alpha(0.3))
                                })
                                .child(self.value.clone()),
                        )
                    })
                    .when(is_focused && !self.has_selection() && !is_empty, |d| {
                        // Show cursor after text
                        let cursor_offset = self.cursor_offset();
                        let text_before = &self.value[..cursor_offset.min(self.value.len())];
                        d.child(
                            div()
                                .absolute()
                                .top_0()
                                .left(px(text_before.len() as f32 * 7.0)) // Approximate char width
                                .w(px(2.))
                                .h_full()
                                .bg(theme.outline),
                        )
                    })
                    .when(is_focused && !self.has_selection() && is_empty, |d| {
                        // Show cursor at start when empty
                        d.child(
                            div()
                                .absolute()
                                .top_0()
                                .left_0()
                                .w(px(2.))
                                .h_full()
                                .bg(theme.outline),
                        )
                    }),
            )
    }
}
