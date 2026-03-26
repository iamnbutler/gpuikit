//! Grain/noise texture overlay element for gpuikit
//!
//! Adds a subtle film-grain or noise pattern on top of UI content,
//! useful for adding texture and depth to surfaces.

use gpui::{
    canvas, div, point, prelude::FluentBuilder, px, size, App, Bounds, Div, IntoElement,
    ParentElement, Pixels, RenderOnce, Styled, Window,
};

use crate::theme::{ActiveTheme, Themeable};

/// Create a grain overlay with default settings.
pub fn grain() -> Grain {
    Grain::new()
}

/// A procedural noise/grain texture overlay.
///
/// Place this as a sibling of content inside a relative-positioned container,
/// or use it as a standalone textured surface.
///
/// # Example
///
/// ```ignore
/// div()
///     .relative()
///     .size_full()
///     .child(your_content)
///     .child(grain().size_full())
/// ```
#[derive(IntoElement)]
pub struct Grain {
    width: Option<Pixels>,
    height: Option<Pixels>,
    full_width: bool,
    full_height: bool,
    absolute: bool,
    /// Opacity multiplier for the grain dots (0.0 to 1.0).
    intensity: f32,
    /// Spacing between grain dots in pixels. Lower = denser.
    spacing: f32,
    /// Size of each grain dot in pixels.
    dot_size: f32,
    /// Use foreground color (light grain on dark). If false, uses dark grain.
    light: bool,
}

impl Grain {
    pub fn new() -> Self {
        Self {
            width: None,
            height: None,
            full_width: false,
            full_height: false,
            absolute: false,
            intensity: 0.06,
            spacing: 4.0,
            dot_size: 1.0,
            light: false,
        }
    }

    pub fn w(mut self, width: Pixels) -> Self {
        self.width = Some(width);
        self
    }

    pub fn h(mut self, height: Pixels) -> Self {
        self.height = Some(height);
        self
    }

    pub fn size(mut self, size: Pixels) -> Self {
        self.width = Some(size);
        self.height = Some(size);
        self
    }

    pub fn w_full(mut self) -> Self {
        self.full_width = true;
        self
    }

    pub fn h_full(mut self) -> Self {
        self.full_height = true;
        self
    }

    /// Make this a full-size absolute overlay (common usage).
    pub fn size_full(mut self) -> Self {
        self.full_width = true;
        self.full_height = true;
        self
    }

    /// Position absolutely within a relative container.
    pub fn absolute(mut self) -> Self {
        self.absolute = true;
        self
    }

    /// Set the grain intensity (0.0 to 1.0). Default: 0.06.
    ///
    /// Lower values produce a subtle texture, higher values are more pronounced.
    pub fn intensity(mut self, intensity: f32) -> Self {
        self.intensity = intensity.clamp(0.0, 1.0);
        self
    }

    /// Set the spacing between grain dots in pixels. Default: 4.0.
    ///
    /// Lower values produce denser grain. Minimum: 2.0.
    pub fn spacing(mut self, spacing: f32) -> Self {
        self.spacing = spacing.max(2.0);
        self
    }

    /// Set the size of each grain dot in pixels. Default: 1.0.
    pub fn dot_size(mut self, dot_size: f32) -> Self {
        self.dot_size = dot_size.max(0.5);
        self
    }

    /// Use light-colored grain (for dark backgrounds). Default: false.
    pub fn light(mut self) -> Self {
        self.light = true;
        self
    }
}

impl Default for Grain {
    fn default() -> Self {
        Self::new()
    }
}

/// Simple hash-based pseudo-random for deterministic grain pattern.
/// Uses a variant of the xxhash finalizer for good distribution.
fn hash_position(x: u32, y: u32) -> f32 {
    let mut h = x.wrapping_mul(374761393).wrapping_add(y.wrapping_mul(668265263));
    h = (h ^ (h >> 13)).wrapping_mul(1274126177);
    h = h ^ (h >> 16);
    (h & 0xFFFF) as f32 / 65535.0
}

impl RenderOnce for Grain {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let base_color = if self.light {
            theme.fg()
        } else {
            gpui::hsla(0.0, 0.0, 0.0, 1.0)
        };
        let intensity = self.intensity;
        let spacing = self.spacing;
        let dot_size = self.dot_size;

        let paint_canvas = canvas(
            move |bounds, _, _cx| bounds,
            move |bounds: Bounds<Pixels>, _, window: &mut Window, _cx| {
                paint_grain(window, bounds, base_color, intensity, spacing, dot_size);
            },
        )
        .size_full();

        div()
            .overflow_hidden()
            .when(self.absolute, |this: Div| {
                this.absolute().top_0().left_0()
            })
            .when(self.full_width, |this: Div| this.w_full())
            .when(self.full_height, |this: Div| this.h_full())
            .when_some(self.width, |this: Div, w| this.w(w))
            .when_some(self.height, |this: Div, h| this.h(h))
            .child(paint_canvas)
    }
}

fn paint_grain(
    window: &mut Window,
    bounds: Bounds<Pixels>,
    base_color: gpui::Hsla,
    intensity: f32,
    spacing: f32,
    dot_size: f32,
) {
    let origin_x: f32 = bounds.origin.x.into();
    let origin_y: f32 = bounds.origin.y.into();
    let width: f32 = bounds.size.width.into();
    let height: f32 = bounds.size.height.into();

    let dot_px = px(dot_size);

    let mut y_offset = 0.0_f32;
    let mut row: u32 = 0;
    while y_offset < height {
        let mut x_offset = 0.0_f32;
        let mut col: u32 = 0;
        while x_offset < width {
            let rand_val = hash_position(col, row);
            // Vary opacity per dot using the hash
            let alpha = rand_val * intensity;
            if alpha > 0.005 {
                let color = gpui::hsla(base_color.h, base_color.s, base_color.l, alpha);
                window.paint_quad(gpui::fill(
                    Bounds {
                        origin: point(px(origin_x + x_offset), px(origin_y + y_offset)),
                        size: size(dot_px, dot_px),
                    },
                    color,
                ));
            }
            x_offset += spacing;
            col += 1;
        }
        y_offset += spacing;
        row += 1;
    }
}
