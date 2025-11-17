//! GPUI Element implementation for rendering an Editor

use crate::buffer::TextBuffer;
use crate::editor::{CursorPosition, Editor};
use gpui::{canvas, Stateful, *};
use std::cell::RefCell;
use std::rc::Rc;

/// A GPUI Element that renders an Editor
pub struct EditorElement {
    editor: Rc<RefCell<Editor>>,
}

impl EditorElement {
    /// Create a new EditorElement from an Editor
    pub fn new(editor: Editor) -> Self {
        Self {
            editor: Rc::new(RefCell::new(editor)),
        }
    }

    /// Get a reference to the underlying Editor
    pub fn editor(&self) -> std::cell::Ref<'_, Editor> {
        self.editor.borrow()
    }

    /// Get a mutable reference to the underlying Editor
    pub fn editor_mut(&self) -> std::cell::RefMut<'_, Editor> {
        self.editor.borrow_mut()
    }

    fn line_bounds(&self, row: usize, bounds: Bounds<Pixels>) -> Bounds<Pixels> {
        let config = self.editor.borrow().config().clone();
        let scroll_row = self.editor.borrow().scroll_row();
        let visual_row = row.saturating_sub(scroll_row);
        Bounds {
            origin: point(
                bounds.origin.x + config.gutter_width,
                bounds.origin.y + config.line_height * visual_row as f32,
            ),
            size: size(bounds.size.width - config.gutter_width, config.line_height),
        }
    }

    fn cursor_position_px(&self, bounds: Bounds<Pixels>, window: &mut Window) -> Point<Pixels> {
        let editor = self.editor.borrow();
        let config = editor.config();
        let cursor_pos = editor.get_cursor_position();
        let scroll_row = editor.scroll_row();
        let visual_row = cursor_pos.row.saturating_sub(scroll_row);
        let line = editor
            .get_buffer()
            .get_line(cursor_pos.row)
            .unwrap_or_else(|| String::new());

        let text_before_cursor = &line[..cursor_pos.col.min(line.len())];
        let text_x = bounds.origin.x + config.gutter_width + config.gutter_padding;

        let offset_x = if !text_before_cursor.is_empty() {
            let shaped = window.text_system().shape_line(
                SharedString::from(text_before_cursor.to_string()),
                config.font_size,
                &[TextRun {
                    len: text_before_cursor.len(),
                    font: Font {
                        family: config.font_family.clone(),
                        features: Default::default(),
                        weight: FontWeight::NORMAL,
                        style: FontStyle::Normal,
                        fallbacks: Default::default(),
                    },
                    color: config.text_color.into(),
                    background_color: None,
                    underline: None,
                    strikethrough: None,
                }],
                None,
            );
            shaped.width
        } else {
            px(0.0)
        };

        point(
            text_x + offset_x,
            bounds.origin.y + config.line_height * visual_row as f32,
        )
    }

    fn paint_editor_background(&self, window: &mut Window, bounds: Bounds<Pixels>) {
        let config = self.editor.borrow().config().clone();
        let bg_color: Hsla = config.editor_bg_color.into();

        if bg_color.is_opaque() {
            let editor_bounds = Bounds {
                origin: point(bounds.origin.x + config.gutter_width, bounds.origin.y),
                size: size(bounds.size.width - config.gutter_width, bounds.size.height),
            };
            window.paint_quad(PaintQuad {
                bounds: editor_bounds,
                corner_radii: (0.0).into(),
                background: config.editor_bg_color.into(),
                border_color: transparent_black(),
                border_widths: (0.0).into(),
                border_style: BorderStyle::Solid,
            });
        }
    }

    fn paint_gutter_background(&self, window: &mut Window, bounds: Bounds<Pixels>) {
        let config = self.editor.borrow().config().clone();
        let bg_color: Hsla = config.gutter_bg_color.into();

        if bg_color.is_opaque() {
            let gutter_bounds = Bounds {
                origin: bounds.origin,
                size: size(config.gutter_width, bounds.size.height),
            };
            window.paint_quad(PaintQuad {
                bounds: gutter_bounds,
                corner_radii: (0.0).into(),
                background: config.gutter_bg_color.into(),
                border_color: transparent_black(),
                border_widths: (0.0).into(),
                border_style: BorderStyle::Solid,
            });
        }
    }

    fn paint_active_line_background(&self, window: &mut Window, bounds: Bounds<Pixels>) {
        let editor = self.editor.borrow();
        let config = editor.config();
        let cursor_pos = editor.get_cursor_position();
        let bg_color: Hsla = config.active_line_bg_color.into();
        let visible_range = editor.visible_row_range(bounds.size.height.into());

        // Only paint active line if it's visible
        if bg_color.is_opaque()
            && cursor_pos.row >= visible_range.start
            && cursor_pos.row < visible_range.end
        {
            let active_line_bounds = self.line_bounds(cursor_pos.row, bounds);
            window.paint_quad(PaintQuad {
                bounds: active_line_bounds,
                corner_radii: (0.0).into(),
                background: config.active_line_bg_color.into(),
                border_color: transparent_black(),
                border_widths: (0.0).into(),
                border_style: BorderStyle::Solid,
            });
        }
    }

    fn paint_selection(&self, window: &mut Window, bounds: Bounds<Pixels>) {
        let editor = self.editor.borrow();
        let config = editor.config();
        let visible_range = editor.visible_row_range(bounds.size.height.into());

        if let Some((start, end)) = editor.get_selection_range() {
            let selection_color = rgba(0x264f78ff);

            for row in start.row..=end.row {
                // Only paint selection for visible rows
                if row >= visible_range.start && row < visible_range.end {
                    if let Some(line) = editor.get_buffer().get_line(row) {
                        let line_bounds = self.line_bounds(row, bounds);

                        let start_col = if row == start.row { start.col } else { 0 };
                        let end_col = if row == end.row { end.col } else { line.len() };

                        let text_x_start = line_bounds.origin.x + config.gutter_padding;

                        let start_x = if start_col > 0 {
                            let text_before =
                                SharedString::from(line[..start_col.min(line.len())].to_string());
                            let shaped = window.text_system().shape_line(
                                text_before.clone(),
                                config.font_size,
                                &[TextRun {
                                    len: text_before.len(),
                                    font: Font {
                                        family: config.font_family.clone(),
                                        features: Default::default(),
                                        weight: FontWeight::NORMAL,
                                        style: FontStyle::Normal,
                                        fallbacks: Default::default(),
                                    },
                                    color: config.text_color.into(),
                                    background_color: None,
                                    underline: None,
                                    strikethrough: None,
                                }],
                                None,
                            );
                            shaped.width
                        } else {
                            px(0.0)
                        };

                        let end_x = if end_col > 0 {
                            let text_to_end =
                                SharedString::from(line[..end_col.min(line.len())].to_string());
                            let shaped = window.text_system().shape_line(
                                text_to_end.clone(),
                                config.font_size,
                                &[TextRun {
                                    len: text_to_end.len(),
                                    font: Font {
                                        family: config.font_family.clone(),
                                        features: Default::default(),
                                        weight: FontWeight::NORMAL,
                                        style: FontStyle::Normal,
                                        fallbacks: Default::default(),
                                    },
                                    color: config.text_color.into(),
                                    background_color: None,
                                    underline: None,
                                    strikethrough: None,
                                }],
                                None,
                            );
                            shaped.width
                        } else {
                            px(0.0)
                        };

                        let selection_bounds = Bounds {
                            origin: point(text_x_start + start_x, line_bounds.origin.y),
                            size: size(end_x - start_x, config.line_height),
                        };

                        window.paint_quad(PaintQuad {
                            bounds: selection_bounds,
                            corner_radii: (0.0).into(),
                            background: selection_color.into(),
                            border_color: transparent_black(),
                            border_widths: (0.0).into(),
                            border_style: BorderStyle::Solid,
                        });
                    }
                }
            }
        }
    }

    fn paint_lines(&mut self, cx: &mut App, window: &mut Window, bounds: Bounds<Pixels>) {
        let visible_range = self
            .editor
            .borrow()
            .visible_row_range(bounds.size.height.into());

        for row in visible_range {
            let line = self.editor.borrow().get_buffer().get_line(row);
            if let Some(line) = line {
                let line_bounds = self.line_bounds(row, bounds);
                self.paint_line_number(cx, window, row + 1, line_bounds, bounds);
                self.paint_line(cx, window, line, row, line_bounds);
            }
        }
    }

    fn paint_line_number(
        &self,
        cx: &mut App,
        window: &mut Window,
        line_number: usize,
        line_bounds: Bounds<Pixels>,
        editor_bounds: Bounds<Pixels>,
    ) {
        let config = self.editor.borrow().config().clone();
        let line_number_str = SharedString::new(line_number.to_string());
        let line_number_len = line_number_str.len();
        let gutter_padding = px(10.0);
        let line_number_x =
            editor_bounds.origin.x + config.gutter_width - gutter_padding - px(20.0);

        let shaped_line_number = window.text_system().shape_line(
            line_number_str,
            config.font_size,
            &[TextRun {
                len: line_number_len,
                font: Font {
                    family: config.font_family.clone(),
                    features: Default::default(),
                    weight: FontWeight::NORMAL,
                    style: FontStyle::Normal,
                    fallbacks: Default::default(),
                },
                color: config.line_number_color.into(),
                background_color: None,
                underline: None,
                strikethrough: None,
            }],
            None,
        );

        let _ = shaped_line_number.paint(
            point(line_number_x, line_bounds.origin.y),
            config.line_height,
            window,
            cx,
        );
    }

    fn paint_line(
        &mut self,
        cx: &mut App,
        window: &mut Window,
        line: impl Into<SharedString>,
        line_index: usize,
        line_bounds: Bounds<Pixels>,
    ) {
        let gutter_padding = px(10.0);
        let text_x = line_bounds.origin.x + gutter_padding;
        let line = line.into();

        // Get syntax highlighted text runs
        let config = self.editor.borrow().config().clone();
        let font_family = config.font_family.clone();
        let font_size = config.font_size;
        let line_height = config.line_height;
        let font_size_f32: f32 = font_size.into();
        let text_runs =
            self.editor
                .borrow_mut()
                .highlight_line(&line, line_index, font_family, font_size_f32);

        let shaped_line =
            window
                .text_system()
                .shape_line(line.clone(), font_size, &text_runs, None);

        let _ = shaped_line.paint(point(text_x, line_bounds.origin.y), line_height, window, cx);
    }

    fn paint_cursor(&self, window: &mut Window, bounds: Bounds<Pixels>) {
        let editor = self.editor.borrow();
        let config = editor.config();
        let cursor_pos_row = editor.get_cursor_position().row;
        let visible_range = editor.visible_row_range(bounds.size.height.into());

        // Only paint cursor if it's visible
        if cursor_pos_row >= visible_range.start && cursor_pos_row < visible_range.end {
            let cursor_pos = self.cursor_position_px(bounds, window);
            let cursor_bounds = Bounds {
                origin: cursor_pos,
                size: size(px(2.0), config.line_height),
            };

            window.paint_quad(PaintQuad {
                bounds: cursor_bounds,
                corner_radii: (0.0).into(),
                background: rgb(0xffffff).into(),
                border_color: transparent_black(),
                border_widths: (0.0).into(),
                border_style: BorderStyle::Solid,
            });
        }
    }
}

impl IntoElement for EditorElement {
    type Element = Stateful<Div>;

    fn into_element(self) -> Self::Element {
        let editor_id = self.editor.borrow().id().to_string();
        let editor_clone = self.editor.clone();
        let editor_for_render = self.editor.clone();

        div()
            .id(ElementId::Name(editor_id.into()))
            .size_full()
            .on_mouse_down(MouseButton::Left, move |event, _window, _cx| {
                let mut editor = editor_clone.borrow_mut();

                // Calculate position from mouse coordinates
                // We need to use the event position and bounds to figure out where the click was
                // For now, we'll use a simple approximation based on the event position
                let config = editor.config();
                let line_height = config.line_height;
                let gutter_width = config.gutter_width + config.gutter_padding;
                let scroll_row = editor.scroll_row();

                // Calculate relative position within the editor area
                let editor_x = event.position.x.max(gutter_width) - gutter_width;
                let editor_y = event.position.y;

                // Calculate row from y position
                let row_float = editor_y / line_height;
                let clicked_row = row_float.floor() as usize + scroll_row;
                let max_row = editor.get_buffer().line_count().saturating_sub(1);
                let row = clicked_row.min(max_row);

                // Calculate column from x position (rough approximation)
                let char_width = config.font_size * 0.6;
                let col_float = editor_x / char_width;
                let clicked_col = col_float.floor() as usize;

                // Get the actual line to clamp column properly
                let max_col = editor
                    .get_buffer()
                    .get_line(row)
                    .map(|line| line.len())
                    .unwrap_or(0);
                let col = clicked_col.min(max_col);

                let position = CursorPosition::new(row, col);

                // Clear selection and set cursor position
                editor.clear_selection();
                editor.set_cursor_position(position);
                editor.ensure_cursor_visible();
            })
            .child(
                canvas(
                    move |_bounds, _window, _cx| {
                        // Prepaint - nothing needed here
                    },
                    move |bounds, _, window, cx| {
                        // Create a temporary EditorElement for rendering
                        let mut temp_element = EditorElement {
                            editor: editor_for_render.clone(),
                        };

                        temp_element.paint_gutter_background(window, bounds);
                        temp_element.paint_editor_background(window, bounds);
                        temp_element.paint_active_line_background(window, bounds);
                        temp_element.paint_selection(window, bounds);
                        temp_element.paint_lines(cx, window, bounds);
                        temp_element.paint_cursor(window, bounds);
                    },
                )
                .size_full(),
            )
    }
}
