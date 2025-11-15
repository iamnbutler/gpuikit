//! GPUI Element implementation for rendering an Editor

use crate::buffer::TextBuffer;
use crate::editor::Editor;
use gpui::*;

/// A GPUI Element that renders an Editor
pub struct EditorElement {
    editor: Editor,
}

impl EditorElement {
    /// Create a new EditorElement from an Editor
    pub fn new(editor: Editor) -> Self {
        Self { editor }
    }

    /// Get a reference to the underlying Editor
    pub fn editor(&self) -> &Editor {
        &self.editor
    }

    /// Get a mutable reference to the underlying Editor
    pub fn editor_mut(&mut self) -> &mut Editor {
        &mut self.editor
    }

    fn line_bounds(&self, row: usize, bounds: Bounds<Pixels>) -> Bounds<Pixels> {
        let config = self.editor.config();
        Bounds {
            origin: point(
                bounds.origin.x + config.gutter_width,
                bounds.origin.y + config.line_height * row as f32,
            ),
            size: size(bounds.size.width - config.gutter_width, config.line_height),
        }
    }

    fn cursor_position_px(&self, bounds: Bounds<Pixels>, window: &mut Window) -> Point<Pixels> {
        let config = self.editor.config();
        let cursor_pos = self.editor.get_cursor_position();
        let line = self
            .editor
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
            bounds.origin.y + config.line_height * cursor_pos.row as f32,
        )
    }

    fn paint_editor_background(&self, window: &mut Window, bounds: Bounds<Pixels>) {
        let config = self.editor.config();
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
        let config = self.editor.config();
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
        let config = self.editor.config();
        let cursor_pos = self.editor.get_cursor_position();
        let bg_color: Hsla = config.active_line_bg_color.into();

        if bg_color.is_opaque() {
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
        let config = self.editor.config();

        if let Some((start, end)) = self.editor.get_selection_range() {
            let selection_color = rgba(0x264f78ff);

            for row in start.row..=end.row {
                if let Some(line) = self.editor.get_buffer().get_line(row) {
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

    fn paint_lines(&mut self, cx: &mut App, window: &mut Window, bounds: Bounds<Pixels>) {
        let _config = self.editor.config();
        let lines = self.editor.get_buffer().all_lines();

        for (i, line) in lines.iter().enumerate() {
            let line_bounds = self.line_bounds(i, bounds);
            self.paint_line_number(cx, window, i + 1, line_bounds, bounds);
            self.paint_line(cx, window, line, i, line_bounds);
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
        let config = self.editor.config();
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
        let config = self.editor.config();
        let font_family = config.font_family.clone();
        let font_size = config.font_size;
        let line_height = config.line_height;
        let font_size_f32: f32 = font_size.into();
        let text_runs = self
            .editor
            .highlight_line(&line, line_index, font_family, font_size_f32);

        let shaped_line =
            window
                .text_system()
                .shape_line(line.clone(), font_size, &text_runs, None);

        let _ = shaped_line.paint(point(text_x, line_bounds.origin.y), line_height, window, cx);
    }

    fn paint_cursor(&self, window: &mut Window, bounds: Bounds<Pixels>) {
        let config = self.editor.config();
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

impl IntoElement for EditorElement {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

impl Element for EditorElement {
    type RequestLayoutState = ();
    type PrepaintState = ();

    fn id(&self) -> Option<ElementId> {
        Some(self.editor.id().clone())
    }

    fn source_location(&self) -> Option<&'static core::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&gpui::InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        let mut style = Style::default();
        style.flex_grow = 1.0;
        style.size.width = relative(1.0).into();
        style.size.height = relative(1.0).into();
        let layout_id = window.request_layout(style, None, cx);
        (layout_id, ())
    }

    fn prepaint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&gpui::InspectorElementId>,
        _: Bounds<Pixels>,
        _: &mut Self::RequestLayoutState,
        _window: &mut Window,
        _cx: &mut App,
    ) -> Self::PrepaintState {
        ()
    }

    fn paint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&gpui::InspectorElementId>,
        bounds: Bounds<Pixels>,
        _: &mut Self::RequestLayoutState,
        _: &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        self.paint_gutter_background(window, bounds);
        self.paint_editor_background(window, bounds);
        self.paint_active_line_background(window, bounds);
        self.paint_selection(window, bounds);
        self.paint_lines(cx, window, bounds);
        self.paint_cursor(window, bounds);
    }
}
