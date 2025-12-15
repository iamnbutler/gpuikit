//! Portal trait for positioning elements relative to other elements.
//!
//! This module provides utilities for creating "portal" elements - UI elements
//! that are rendered outside their normal DOM position, typically for tooltips,
//! dropdowns, popovers, and similar overlay components.
//!
//! # Example
//!
//! ```ignore
//! use gpuikit::traits::portal::{AnchorCorner, AnchorEdge, PortalPosition};
//!
//! // Position a tooltip above the trigger element
//! let position = PortalPosition::new()
//!     .anchor(AnchorCorner::TopLeft)
//!     .preferred_edge(AnchorEdge::Top)
//!     .offset(point(px(0.), px(-4.)));
//! ```

use gpui::{point, px, Pixels, Point, Size};

/// Which corner of the portal element should be anchored to the anchor position.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum AnchorCorner {
    #[default]
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

impl AnchorCorner {
    /// Get the opposite corner along the horizontal axis.
    pub fn flip_horizontal(self) -> Self {
        match self {
            Self::TopLeft => Self::TopRight,
            Self::TopRight => Self::TopLeft,
            Self::BottomLeft => Self::BottomRight,
            Self::BottomRight => Self::BottomLeft,
        }
    }

    /// Get the opposite corner along the vertical axis.
    pub fn flip_vertical(self) -> Self {
        match self {
            Self::TopLeft => Self::BottomLeft,
            Self::TopRight => Self::BottomRight,
            Self::BottomLeft => Self::TopLeft,
            Self::BottomRight => Self::TopRight,
        }
    }

    /// Get the completely opposite corner.
    pub fn opposite(self) -> Self {
        match self {
            Self::TopLeft => Self::BottomRight,
            Self::TopRight => Self::BottomLeft,
            Self::BottomLeft => Self::TopRight,
            Self::BottomRight => Self::TopLeft,
        }
    }

    /// Returns true if this corner is on the top edge.
    pub fn is_top(self) -> bool {
        matches!(self, Self::TopLeft | Self::TopRight)
    }

    /// Returns true if this corner is on the bottom edge.
    pub fn is_bottom(self) -> bool {
        matches!(self, Self::BottomLeft | Self::BottomRight)
    }

    /// Returns true if this corner is on the left edge.
    pub fn is_left(self) -> bool {
        matches!(self, Self::TopLeft | Self::BottomLeft)
    }

    /// Returns true if this corner is on the right edge.
    pub fn is_right(self) -> bool {
        matches!(self, Self::TopRight | Self::BottomRight)
    }
}

/// The preferred edge to display the portal element relative to its trigger.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum AnchorEdge {
    Top,
    #[default]
    Bottom,
    Left,
    Right,
}

impl AnchorEdge {
    /// Get the opposite edge.
    pub fn opposite(self) -> Self {
        match self {
            Self::Top => Self::Bottom,
            Self::Bottom => Self::Top,
            Self::Left => Self::Right,
            Self::Right => Self::Left,
        }
    }

    /// Returns true if this is a horizontal edge (left or right).
    pub fn is_horizontal(self) -> bool {
        matches!(self, Self::Left | Self::Right)
    }

    /// Returns true if this is a vertical edge (top or bottom).
    pub fn is_vertical(self) -> bool {
        matches!(self, Self::Top | Self::Bottom)
    }

    /// Get the appropriate anchor corner for this edge with the given alignment.
    pub fn to_anchor_corner(self, align_start: bool) -> AnchorCorner {
        match (self, align_start) {
            (Self::Top, true) => AnchorCorner::BottomLeft,
            (Self::Top, false) => AnchorCorner::BottomRight,
            (Self::Bottom, true) => AnchorCorner::TopLeft,
            (Self::Bottom, false) => AnchorCorner::TopRight,
            (Self::Left, true) => AnchorCorner::TopRight,
            (Self::Left, false) => AnchorCorner::BottomRight,
            (Self::Right, true) => AnchorCorner::TopLeft,
            (Self::Right, false) => AnchorCorner::BottomLeft,
        }
    }
}

/// How the portal element should behave when it would overflow the viewport.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum FitMode {
    /// Switch to the opposite anchor corner when an overflow would occur.
    #[default]
    SwitchAnchor,
    /// Snap the element to the viewport edge.
    SnapToViewport,
    /// Do not adjust the position (allow overflow).
    None,
}

/// Configuration for positioning a portal element.
#[derive(Debug, Clone)]
pub struct PortalPosition {
    /// Which corner of the portal anchors to the anchor position.
    pub anchor_corner: AnchorCorner,
    /// The preferred edge to display relative to the trigger.
    pub preferred_edge: AnchorEdge,
    /// Offset from the anchor position.
    pub offset: Point<Pixels>,
    /// How to handle viewport overflow.
    pub fit_mode: FitMode,
    /// Margin from viewport edges when using snap-to-viewport.
    pub viewport_margin: Pixels,
}

impl Default for PortalPosition {
    fn default() -> Self {
        Self {
            anchor_corner: AnchorCorner::TopLeft,
            preferred_edge: AnchorEdge::Bottom,
            offset: point(px(0.), px(0.)),
            fit_mode: FitMode::SwitchAnchor,
            viewport_margin: px(8.),
        }
    }
}

impl PortalPosition {
    /// Create a new portal position configuration.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set which corner of the portal element anchors to the anchor position.
    pub fn anchor(mut self, corner: AnchorCorner) -> Self {
        self.anchor_corner = corner;
        self
    }

    /// Set the preferred edge to display the portal relative to its trigger.
    pub fn preferred_edge(mut self, edge: AnchorEdge) -> Self {
        self.preferred_edge = edge;
        // Update anchor corner to match the preferred edge
        self.anchor_corner = edge.to_anchor_corner(true);
        self
    }

    /// Set an offset from the anchor position.
    pub fn offset(mut self, offset: Point<Pixels>) -> Self {
        self.offset = offset;
        self
    }

    /// Set how to handle viewport overflow.
    pub fn fit_mode(mut self, mode: FitMode) -> Self {
        self.fit_mode = mode;
        self
    }

    /// Set the margin from viewport edges when snapping.
    pub fn viewport_margin(mut self, margin: Pixels) -> Self {
        self.viewport_margin = margin;
        self
    }

    /// Create a position configuration for a tooltip (above the trigger).
    pub fn tooltip() -> Self {
        Self::new()
            .preferred_edge(AnchorEdge::Top)
            .offset(point(px(0.), px(-4.)))
    }

    /// Create a position configuration for a dropdown (below the trigger).
    pub fn dropdown() -> Self {
        Self::new()
            .preferred_edge(AnchorEdge::Bottom)
            .offset(point(px(0.), px(4.)))
    }

    /// Create a position configuration for a popover on the right.
    pub fn popover_right() -> Self {
        Self::new()
            .preferred_edge(AnchorEdge::Right)
            .offset(point(px(4.), px(0.)))
    }

    /// Create a position configuration for a popover on the left.
    pub fn popover_left() -> Self {
        Self::new()
            .preferred_edge(AnchorEdge::Left)
            .offset(point(px(-4.), px(0.)))
    }

    /// Calculate the position for a portal element.
    ///
    /// # Arguments
    /// * `trigger_origin` - The top-left corner of the trigger element
    /// * `trigger_size` - The size of the trigger element
    /// * `portal_size` - The size of the portal element
    /// * `viewport_size` - The size of the viewport
    ///
    /// # Returns
    /// The calculated origin point for the portal element
    pub fn calculate_position(
        &self,
        trigger_origin: Point<Pixels>,
        trigger_size: Size<Pixels>,
        portal_size: Size<Pixels>,
        viewport_size: Size<Pixels>,
    ) -> Point<Pixels> {
        // Calculate the anchor point on the trigger based on preferred edge
        let anchor_point = self.get_anchor_point_on_trigger(trigger_origin, trigger_size);

        // Calculate initial position based on anchor corner
        let mut position = self.position_from_anchor(anchor_point, portal_size);

        // Apply offset
        position.x = position.x + self.offset.x;
        position.y = position.y + self.offset.y;

        // Apply fit mode
        match self.fit_mode {
            FitMode::SwitchAnchor => {
                position = self.apply_switch_anchor(
                    position,
                    portal_size,
                    viewport_size,
                    trigger_origin,
                    trigger_size,
                );
            }
            FitMode::SnapToViewport => {
                position = self.apply_snap_to_viewport(position, portal_size, viewport_size);
            }
            FitMode::None => {}
        }

        position
    }

    /// Get the point on the trigger element where the portal should anchor.
    fn get_anchor_point_on_trigger(
        &self,
        trigger_origin: Point<Pixels>,
        trigger_size: Size<Pixels>,
    ) -> Point<Pixels> {
        match self.preferred_edge {
            AnchorEdge::Top => point(trigger_origin.x + trigger_size.width / 2., trigger_origin.y),
            AnchorEdge::Bottom => point(
                trigger_origin.x + trigger_size.width / 2.,
                trigger_origin.y + trigger_size.height,
            ),
            AnchorEdge::Left => point(
                trigger_origin.x,
                trigger_origin.y + trigger_size.height / 2.,
            ),
            AnchorEdge::Right => point(
                trigger_origin.x + trigger_size.width,
                trigger_origin.y + trigger_size.height / 2.,
            ),
        }
    }

    /// Calculate position based on anchor corner.
    fn position_from_anchor(
        &self,
        anchor_point: Point<Pixels>,
        portal_size: Size<Pixels>,
    ) -> Point<Pixels> {
        match self.anchor_corner {
            AnchorCorner::TopLeft => anchor_point,
            AnchorCorner::TopRight => point(anchor_point.x - portal_size.width, anchor_point.y),
            AnchorCorner::BottomLeft => point(anchor_point.x, anchor_point.y - portal_size.height),
            AnchorCorner::BottomRight => point(
                anchor_point.x - portal_size.width,
                anchor_point.y - portal_size.height,
            ),
        }
    }

    /// Apply switch-anchor fit mode.
    fn apply_switch_anchor(
        &self,
        position: Point<Pixels>,
        portal_size: Size<Pixels>,
        viewport_size: Size<Pixels>,
        trigger_origin: Point<Pixels>,
        trigger_size: Size<Pixels>,
    ) -> Point<Pixels> {
        let mut result = position;
        let mut current_corner = self.anchor_corner;

        // Check horizontal overflow
        let right_edge = result.x + portal_size.width;
        let overflows_right = right_edge > viewport_size.width;
        let overflows_left = result.x < px(0.);

        if overflows_right && !overflows_left {
            current_corner = current_corner.flip_horizontal();
            let anchor_point =
                self.get_anchor_point_for_corner(current_corner, trigger_origin, trigger_size);
            result = self.position_from_anchor_corner(anchor_point, portal_size, current_corner);
            result.x = result.x + self.offset.x;
        } else if overflows_left && !overflows_right {
            current_corner = current_corner.flip_horizontal();
            let anchor_point =
                self.get_anchor_point_for_corner(current_corner, trigger_origin, trigger_size);
            result = self.position_from_anchor_corner(anchor_point, portal_size, current_corner);
            result.x = result.x + self.offset.x;
        }

        // Check vertical overflow
        let bottom_edge = result.y + portal_size.height;
        let overflows_bottom = bottom_edge > viewport_size.height;
        let overflows_top = result.y < px(0.);

        if overflows_bottom && !overflows_top {
            current_corner = current_corner.flip_vertical();
            let anchor_point =
                self.get_anchor_point_for_corner(current_corner, trigger_origin, trigger_size);
            result = self.position_from_anchor_corner(anchor_point, portal_size, current_corner);
            result.y = result.y - self.offset.y; // Invert offset when flipping
        } else if overflows_top && !overflows_bottom {
            current_corner = current_corner.flip_vertical();
            let anchor_point =
                self.get_anchor_point_for_corner(current_corner, trigger_origin, trigger_size);
            result = self.position_from_anchor_corner(anchor_point, portal_size, current_corner);
            result.y = result.y - self.offset.y;
        }

        result
    }

    /// Get anchor point on trigger for a specific corner.
    fn get_anchor_point_for_corner(
        &self,
        corner: AnchorCorner,
        trigger_origin: Point<Pixels>,
        trigger_size: Size<Pixels>,
    ) -> Point<Pixels> {
        match corner {
            AnchorCorner::TopLeft => {
                point(trigger_origin.x, trigger_origin.y + trigger_size.height)
            }
            AnchorCorner::TopRight => point(
                trigger_origin.x + trigger_size.width,
                trigger_origin.y + trigger_size.height,
            ),
            AnchorCorner::BottomLeft => trigger_origin,
            AnchorCorner::BottomRight => {
                point(trigger_origin.x + trigger_size.width, trigger_origin.y)
            }
        }
    }

    /// Position from anchor with a specific corner.
    fn position_from_anchor_corner(
        &self,
        anchor_point: Point<Pixels>,
        portal_size: Size<Pixels>,
        corner: AnchorCorner,
    ) -> Point<Pixels> {
        match corner {
            AnchorCorner::TopLeft => anchor_point,
            AnchorCorner::TopRight => point(anchor_point.x - portal_size.width, anchor_point.y),
            AnchorCorner::BottomLeft => point(anchor_point.x, anchor_point.y - portal_size.height),
            AnchorCorner::BottomRight => point(
                anchor_point.x - portal_size.width,
                anchor_point.y - portal_size.height,
            ),
        }
    }

    /// Apply snap-to-viewport fit mode.
    fn apply_snap_to_viewport(
        &self,
        position: Point<Pixels>,
        portal_size: Size<Pixels>,
        viewport_size: Size<Pixels>,
    ) -> Point<Pixels> {
        let mut result = position;
        let margin = self.viewport_margin;

        // Snap horizontal
        let right_edge = result.x + portal_size.width;
        if right_edge > viewport_size.width - margin {
            result.x = viewport_size.width - portal_size.width - margin;
        }
        if result.x < margin {
            result.x = margin;
        }

        // Snap vertical
        let bottom_edge = result.y + portal_size.height;
        if bottom_edge > viewport_size.height - margin {
            result.y = viewport_size.height - portal_size.height - margin;
        }
        if result.y < margin {
            result.y = margin;
        }

        result
    }
}

/// Trait for elements that can be rendered as portals (tooltips, dropdowns, popovers).
///
/// Elements implementing this trait can be positioned relative to a trigger element
/// with automatic edge detection and viewport bounds handling.
pub trait Portal {
    /// Get the current position configuration for this portal.
    fn position(&self) -> &PortalPosition;

    /// Set the position configuration.
    fn with_position(self, position: PortalPosition) -> Self;

    /// Set the preferred edge for this portal.
    fn preferred_edge(self, edge: AnchorEdge) -> Self
    where
        Self: Sized,
    {
        let mut position = self.position().clone();
        position = position.preferred_edge(edge);
        self.with_position(position)
    }

    /// Set an offset from the anchor position.
    fn offset(self, offset: Point<Pixels>) -> Self
    where
        Self: Sized,
    {
        let mut position = self.position().clone();
        position = position.offset(offset);
        self.with_position(position)
    }

    /// Set the fit mode for handling viewport overflow.
    fn fit_mode(self, mode: FitMode) -> Self
    where
        Self: Sized,
    {
        let mut position = self.position().clone();
        position = position.fit_mode(mode);
        self.with_position(position)
    }
}
