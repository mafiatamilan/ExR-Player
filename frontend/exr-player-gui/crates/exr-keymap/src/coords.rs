use crate::schema::NormVec2;

/// Maps between host pixel viewport coordinates and normalized `[0.0, 1.0]` guest coordinates.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ViewportMapper {
    pub left: f32,
    pub top: f32,
    pub width: f32,
    pub height: f32,
}

impl ViewportMapper {
    pub fn new(left: f32, top: f32, width: f32, height: f32) -> Self {
        Self {
            left,
            top,
            width: width.max(1.0),
            height: height.max(1.0),
        }
    }

    pub fn host_to_norm(&self, px: f32, py: f32) -> NormVec2 {
        NormVec2 {
            x: ((px - self.left) / self.width).clamp(0.0, 1.0),
            y: ((py - self.top) / self.height).clamp(0.0, 1.0),
        }
    }

    pub fn norm_to_host(&self, norm: NormVec2) -> (f32, f32) {
        let c = norm.clamped();
        (
            self.left + c.x * self.width,
            self.top + c.y * self.height,
        )
    }

    pub fn snap_to_grid(norm: NormVec2, grid_step: f32) -> NormVec2 {
        if grid_step <= 0.0 {
            return norm.clamped();
        }
        NormVec2 {
            x: ((norm.x / grid_step).round() * grid_step).clamp(0.0, 1.0),
            y: ((norm.y / grid_step).round() * grid_step).clamp(0.0, 1.0),
        }
    }
}
