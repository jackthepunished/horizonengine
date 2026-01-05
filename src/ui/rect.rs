//! Rectangle and layout primitives
//!
//! Provides positioning and sizing for UI elements.

use glam::Vec2;

/// Anchor point for positioning
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Anchor {
    /// Top-left corner
    #[default]
    TopLeft,
    /// Top-center
    TopCenter,
    /// Top-right corner
    TopRight,
    /// Middle-left
    MiddleLeft,
    /// Center
    Center,
    /// Middle-right
    MiddleRight,
    /// Bottom-left corner
    BottomLeft,
    /// Bottom-center
    BottomCenter,
    /// Bottom-right corner
    BottomRight,
}

impl Anchor {
    /// Get the anchor offset as a normalized vector (0.0 to 1.0)
    #[must_use]
    pub const fn offset(&self) -> (f32, f32) {
        match self {
            Self::TopLeft => (0.0, 0.0),
            Self::TopCenter => (0.5, 0.0),
            Self::TopRight => (1.0, 0.0),
            Self::MiddleLeft => (0.0, 0.5),
            Self::Center => (0.5, 0.5),
            Self::MiddleRight => (1.0, 0.5),
            Self::BottomLeft => (0.0, 1.0),
            Self::BottomCenter => (0.5, 1.0),
            Self::BottomRight => (1.0, 1.0),
        }
    }
}

/// Rectangle style
#[derive(Debug, Clone)]
pub struct RectStyle {
    /// Background color (RGBA)
    pub background_color: [f32; 4],
    /// Border color (RGBA)
    pub border_color: [f32; 4],
    /// Border width
    pub border_width: f32,
    /// Corner radius
    pub corner_radius: f32,
}

impl Default for RectStyle {
    fn default() -> Self {
        Self {
            background_color: [0.2, 0.2, 0.2, 1.0],
            border_color: [0.4, 0.4, 0.4, 1.0],
            border_width: 1.0,
            corner_radius: 4.0,
        }
    }
}

impl RectStyle {
    /// Set background color
    #[must_use]
    pub const fn with_background(mut self, color: [f32; 4]) -> Self {
        self.background_color = color;
        self
    }

    /// Set border color
    #[must_use]
    pub const fn with_border_color(mut self, color: [f32; 4]) -> Self {
        self.border_color = color;
        self
    }

    /// Set border width
    #[must_use]
    pub const fn with_border_width(mut self, width: f32) -> Self {
        self.border_width = width;
        self
    }

    /// Set corner radius
    #[must_use]
    pub const fn with_corner_radius(mut self, radius: f32) -> Self {
        self.corner_radius = radius;
        self
    }
}

/// A 2D rectangle for UI layout
#[derive(Debug, Clone)]
pub struct Rect {
    /// Position (in pixels from anchor)
    pub position: Vec2,
    /// Size (width, height)
    pub size: Vec2,
    /// Anchor point
    pub anchor: Anchor,
    /// Style
    pub style: RectStyle,
}

impl Rect {
    /// Create a new rectangle
    #[must_use]
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            position: Vec2::new(x, y),
            size: Vec2::new(width, height),
            anchor: Anchor::TopLeft,
            style: RectStyle::default(),
        }
    }

    /// Set anchor
    #[must_use]
    pub fn with_anchor(mut self, anchor: Anchor) -> Self {
        self.anchor = anchor;
        self
    }

    /// Set style
    #[must_use]
    pub fn with_style(mut self, style: RectStyle) -> Self {
        self.style = style;
        self
    }

    /// Calculate absolute position in screen space
    #[must_use]
    pub fn absolute_position(&self, parent_size: Vec2) -> Vec2 {
        let (ox, oy) = self.anchor.offset();
        Vec2::new(
            parent_size.x * ox + self.position.x - self.size.x * ox,
            parent_size.y * oy + self.position.y - self.size.y * oy,
        )
    }

    /// Check if a point is inside the rectangle
    #[must_use]
    pub fn contains(&self, point: Vec2, parent_size: Vec2) -> bool {
        let pos = self.absolute_position(parent_size);
        point.x >= pos.x
            && point.x <= pos.x + self.size.x
            && point.y >= pos.y
            && point.y <= pos.y + self.size.y
    }

    /// Get the bounds as (min, max)
    #[must_use]
    pub fn bounds(&self, parent_size: Vec2) -> (Vec2, Vec2) {
        let pos = self.absolute_position(parent_size);
        (pos, pos + self.size)
    }
}

impl Default for Rect {
    fn default() -> Self {
        Self::new(0.0, 0.0, 100.0, 100.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rect_contains() {
        let rect = Rect::new(10.0, 10.0, 100.0, 50.0);
        let parent = Vec2::new(800.0, 600.0);

        assert!(rect.contains(Vec2::new(50.0, 30.0), parent));
        assert!(!rect.contains(Vec2::new(5.0, 5.0), parent));
    }

    #[test]
    fn test_rect_anchor_center() {
        let rect = Rect::new(0.0, 0.0, 100.0, 50.0).with_anchor(Anchor::Center);
        let parent = Vec2::new(800.0, 600.0);

        let pos = rect.absolute_position(parent);
        assert!((pos.x - 350.0).abs() < 0.01); // (800 - 100) / 2
        assert!((pos.y - 275.0).abs() < 0.01); // (600 - 50) / 2
    }
}
