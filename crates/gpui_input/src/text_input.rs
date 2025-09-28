//! Text input field component for GPUI applications with proper text shaping.

use gpui::{
    div, point, px, size, App, Bounds, Context, Element, ElementId, ElementInputHandler,
    EntityInputHandler, FocusHandle, Focusable, GlobalElementId, InteractiveElement, IntoElement,
    KeyDownEvent, LayoutId, MouseButton, PaintQuad, ParentElement, Pixels, Point, Render,
    ShapedLine, SharedString, Style, Styled, TextRun, UTF16Selection, UnderlineStyle, Window,
};
use std::ops::Range;
use theme::ActiveTheme;

/// Text input field component that manages state and user interaction.
pub struct TextInput {
    id: ElementId,
    value: String,
    placeholder: Option<SharedString>,
    pub(crate) focus_handle: FocusHandle,
    pub(crate) selected_range: Range<usize>,
    selection_reversed: bool,
    marked_range: Option<Range<usize>>,
    last_layout: Option<ShapedLine>,
    last_bounds: Option<Bounds<Pixels>>,
    width: Option<f32>,
    disabled: bool,
    is_selecting: bool,
}

impl TextInput {
    pub fn new(id: impl Into<ElementId>, value: impl Into<String>, cx: &mut Context<Self>) -> Self {
        Self {
            id: id.into(),
            value: value.into(),
            placeholder: None,
            focus_handle: cx.focus_handle(),
            selected_range: 0..0,
            selection_reversed: false,
            marked_range: None,
            last_layout: None,
            last_bounds: None,
            width: None,
            disabled: false,
            is_selecting: false,
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

    // Movement and selection methods
    fn move_to(&mut self, offset: usize, cx: &mut Context<Self>) {
        self.selected_range = offset..offset;
        self.selection_reversed = false;
        cx.notify();
    }

    fn cursor_offset(&self) -> usize {
        if self.selection_reversed {
            self.selected_range.start
        } else {
            self.selected_range.end
        }
    }

    fn select_to(&mut self, offset: usize, cx: &mut Context<Self>) {
        if self.selection_reversed {
            self.selected_range.start = offset
        } else {
            self.selected_range.end = offset
        };
        if self.selected_range.end < self.selected_range.start {
            self.selection_reversed = !self.selection_reversed;
            self.selected_range = self.selected_range.end..self.selected_range.start;
        }
        cx.notify();
    }

    fn select_all(&mut self, cx: &mut Context<Self>) {
        self.move_to(0, cx);
        self.select_to(self.value.len(), cx);
    }

    fn index_for_mouse_position(&self, position: Point<Pixels>) -> usize {
        if self.value.is_empty() {
            return 0;
        }

        let (Some(bounds), Some(line)) = (self.last_bounds.as_ref(), self.last_layout.as_ref())
        else {
            return 0;
        };

        if position.y < bounds.top() {
            return 0;
        }
        if position.y > bounds.bottom() {
            return self.value.len();
        }
        line.closest_index_for_x(position.x - bounds.left())
    }

    // Text manipulation
    fn insert_text(&mut self, text: &str, cx: &mut Context<Self>) {
        self.value.replace_range(self.selected_range.clone(), text);
        let new_cursor = self.selected_range.start + text.len();
        self.selected_range = new_cursor..new_cursor;
        self.selection_reversed = false;
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
        self.selection_reversed = false;
        cx.notify();
    }

    fn delete(&mut self, cx: &mut Context<Self>) {
        if self.selected_range.is_empty() && self.selected_range.end < self.value.len() {
            self.value.remove(self.selected_range.end);
        } else if !self.selected_range.is_empty() {
            self.value.replace_range(self.selected_range.clone(), "");
            let cursor = self.selected_range.start;
            self.selected_range = cursor..cursor;
        }
        self.selection_reversed = false;
        cx.notify();
    }

    fn on_mouse_up(&mut self, _window: &mut Window, _cx: &mut Context<Self>) {
        self.is_selecting = false;
    }

    fn on_mouse_move(
        &mut self,
        event: &gpui::MouseMoveEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.is_selecting && !self.disabled {
            let index = self.index_for_mouse_position(event.position);
            self.select_to(index, cx);
        }
    }
}

// Implement EntityInputHandler for IME and text input
impl EntityInputHandler for TextInput {
    fn text_for_range(
        &mut self,
        range_utf16: Range<usize>,
        actual_range: &mut Option<Range<usize>>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<String> {
        // For simplicity, we're not doing UTF16 conversion here
        let range = range_utf16;
        if range.end <= self.value.len() {
            *actual_range = Some(range.clone());
            Some(self.value[range].to_string())
        } else {
            None
        }
    }

    fn selected_text_range(
        &mut self,
        _ignore_disabled_input: bool,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<UTF16Selection> {
        // For simplicity, treating UTF8 indices as UTF16
        Some(UTF16Selection {
            range: self.selected_range.clone(),
            reversed: self.selection_reversed,
        })
    }

    fn marked_text_range(
        &self,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<Range<usize>> {
        self.marked_range.clone()
    }

    fn unmark_text(&mut self, _window: &mut Window, _cx: &mut Context<Self>) {
        self.marked_range = None;
    }

    fn replace_text_in_range(
        &mut self,
        range_utf16: Option<Range<usize>>,
        new_text: &str,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let range = range_utf16
            .or(self.marked_range.clone())
            .unwrap_or(self.selected_range.clone());

        self.value =
            (self.value[0..range.start].to_owned() + new_text + &self.value[range.end..]).into();
        self.selected_range = range.start + new_text.len()..range.start + new_text.len();
        self.marked_range.take();
        cx.notify();
    }

    fn replace_and_mark_text_in_range(
        &mut self,
        range_utf16: Option<Range<usize>>,
        new_text: &str,
        new_selected_range_utf16: Option<Range<usize>>,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let range = range_utf16
            .or(self.marked_range.clone())
            .unwrap_or(self.selected_range.clone());

        self.value =
            (self.value[0..range.start].to_owned() + new_text + &self.value[range.end..]).into();

        if !new_text.is_empty() {
            self.marked_range = Some(range.start..range.start + new_text.len());
        } else {
            self.marked_range = None;
        }

        self.selected_range = new_selected_range_utf16
            .map(|new_range| new_range.start + range.start..new_range.end + range.end)
            .unwrap_or_else(|| range.start + new_text.len()..range.start + new_text.len());

        cx.notify();
    }

    fn bounds_for_range(
        &mut self,
        range_utf16: Range<usize>,
        bounds: Bounds<Pixels>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<Bounds<Pixels>> {
        let last_layout = self.last_layout.as_ref()?;
        let range = range_utf16; // For simplicity, treating as UTF8
        Some(Bounds::from_corners(
            point(
                bounds.left() + last_layout.x_for_index(range.start),
                bounds.top(),
            ),
            point(
                bounds.left() + last_layout.x_for_index(range.end),
                bounds.bottom(),
            ),
        ))
    }

    fn character_index_for_point(
        &mut self,
        point: gpui::Point<Pixels>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<usize> {
        let last_bounds = self.last_bounds?;
        let last_layout = self.last_layout.as_ref()?;
        let local_x = point.x - last_bounds.left();
        Some(last_layout.closest_index_for_x(local_x))
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
        let width = self.width.unwrap_or(200.0);

        div()
            .id(self.id.clone())
            .key_context("TextInput")
            .track_focus(&self.focus_handle)
            .on_key_down(cx.listener(|this, event: &KeyDownEvent, window, cx| {
                if this.disabled {
                    return;
                }

                match event.keystroke.key.as_str() {
                    "enter" | "escape" => window.blur(),
                    "backspace" => this.backspace(cx),
                    "delete" => this.delete(cx),
                    "left" => {
                        if this.selected_range.is_empty() {
                            this.move_to(this.selected_range.start.saturating_sub(1), cx);
                        } else {
                            this.move_to(this.selected_range.start, cx);
                        }
                    }
                    "right" => {
                        if this.selected_range.is_empty() {
                            this.move_to((this.selected_range.end + 1).min(this.value.len()), cx);
                        } else {
                            this.move_to(this.selected_range.end, cx);
                        }
                    }
                    "home" => this.move_to(0, cx),
                    "end" => this.move_to(this.value.len(), cx),
                    "a" if event.keystroke.modifiers.platform => this.select_all(cx),
                    _ => {}
                }
            }))
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, event: &gpui::MouseDownEvent, _window, cx| {
                    if !this.disabled {
                        this.is_selecting = true;
                        let index = this.index_for_mouse_position(event.position);
                        if event.modifiers.shift {
                            this.select_to(index, cx);
                        } else {
                            this.move_to(index, cx);
                        }
                    }
                }),
            )
            .on_mouse_up(
                MouseButton::Left,
                cx.listener(|this, _event: &gpui::MouseUpEvent, _window, _cx| {
                    this.on_mouse_up(_window, _cx);
                }),
            )
            .on_mouse_up_out(
                MouseButton::Left,
                cx.listener(|this, _event: &gpui::MouseUpEvent, _window, _cx| {
                    this.on_mouse_up(_window, _cx);
                }),
            )
            .on_mouse_move(cx.listener(Self::on_mouse_move))
            .w(px(width))
            .h(px(28.))
            .px(px(8.))
            .py(px(4.))
            .rounded(px(4.))
            .bg(theme.surface)
            .border_1()
            .border_color(if self.is_focused(window) && !self.disabled {
                theme.outline
            } else {
                theme.border
            })
            .cursor_pointer()
            .child(TextElement {
                input: cx.entity().clone(),
            })
    }
}

/// Custom element for rendering shaped text with selection and cursor.
struct TextElement {
    input: gpui::Entity<TextInput>,
}

struct PrepaintState {
    line: Option<ShapedLine>,
    cursor: Option<PaintQuad>,
    selection: Option<PaintQuad>,
}

impl IntoElement for TextElement {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

impl Element for TextElement {
    type RequestLayoutState = ();
    type PrepaintState = PrepaintState;

    fn id(&self) -> Option<ElementId> {
        None
    }

    fn source_location(&self) -> Option<&'static core::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&gpui::InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        let mut style = Style::default();
        style.size.width = gpui::relative(1.).into();
        style.size.height = window.line_height().into();
        (window.request_layout(style, [], cx), ())
    }

    fn prepaint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&gpui::InspectorElementId>,
        bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        window: &mut Window,
        cx: &mut App,
    ) -> Self::PrepaintState {
        let input = self.input.read(cx);
        let theme = cx.global::<theme::GlobalTheme>().0.clone();
        let content = input.value.clone();
        let selected_range = input.selected_range.clone();
        let cursor = input.cursor_offset();
        let style = window.text_style();

        let (display_text, text_color) = if content.is_empty() {
            (
                input
                    .placeholder
                    .clone()
                    .unwrap_or_else(|| SharedString::from("")),
                theme.fg.alpha(0.5),
            )
        } else {
            (SharedString::from(content), theme.fg)
        };

        let run = TextRun {
            len: display_text.len(),
            font: style.font(),
            color: text_color,
            background_color: None,
            underline: if let Some(marked_range) = input.marked_range.as_ref() {
                if marked_range.contains(&cursor) {
                    Some(UnderlineStyle {
                        color: Some(text_color),
                        thickness: px(1.0),
                        wavy: false,
                    })
                } else {
                    None
                }
            } else {
                None
            },
            strikethrough: None,
        };

        let font_size = style.font_size.to_pixels(window.rem_size());
        let line = window
            .text_system()
            .shape_line(display_text.clone(), font_size, &[run], None);

        let cursor_pos = line.x_for_index(cursor);
        let (selection, cursor) = if selected_range.is_empty() {
            (
                None,
                if input.focus_handle.is_focused(window) && !input.disabled {
                    Some(gpui::fill(
                        Bounds::new(
                            point(bounds.left() + cursor_pos, bounds.top()),
                            size(px(2.), bounds.bottom() - bounds.top()),
                        ),
                        theme.outline,
                    ))
                } else {
                    None
                },
            )
        } else {
            (
                Some(gpui::fill(
                    Bounds::from_corners(
                        point(
                            bounds.left() + line.x_for_index(selected_range.start),
                            bounds.top(),
                        ),
                        point(
                            bounds.left() + line.x_for_index(selected_range.end),
                            bounds.bottom(),
                        ),
                    ),
                    theme.outline.alpha(0.3),
                )),
                None,
            )
        };

        PrepaintState {
            line: Some(line),
            cursor,
            selection,
        }
    }

    fn paint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&gpui::InspectorElementId>,
        bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        prepaint: &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        let focus_handle = self.input.read(cx).focus_handle.clone();

        window.handle_input(
            &focus_handle,
            ElementInputHandler::new(bounds, self.input.clone()),
            cx,
        );

        if let Some(selection) = prepaint.selection.take() {
            window.paint_quad(selection);
        }

        if let Some(line) = prepaint.line.take() {
            line.paint(bounds.origin, window.line_height(), window, cx)
                .unwrap();
        }

        if let Some(cursor) = prepaint.cursor.take() {
            window.paint_quad(cursor);
        }

        self.input.update(cx, |input, _cx| {
            input.last_layout = prepaint.line.clone();
            input.last_bounds = Some(bounds);
        });
    }
}
