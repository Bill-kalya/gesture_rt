use nalgebra::Vector3;
use opencv::core::Rect;

/// Simple ROI tracker for a single hand. Keeps a running bbox and velocity.
#[derive(Debug, Clone)]
pub struct RoiTracker {
    pub bbox: Option<Rect>,
    // velocity in pixels/frame for bbox top-left
    pub vx: f32,
    pub vy: f32,
    // last update timestamp (frame index)
    pub last_frame_idx: Option<u64>,
    // how many frames to keep without detection before resetting
    pub patience: u64,
    pub missed: u64,
}

impl Default for RoiTracker {
    fn default() -> Self {
        Self { bbox: None, vx: 0.0, vy: 0.0, last_frame_idx: None, patience: 30, missed: 0 }
    }
}

impl RoiTracker {
    /// Predict the next bbox using simple constant velocity model.
    pub fn predict(&self, frame_idx: u64) -> Option<Rect> {
        let bbox = self.bbox?;
        let last = self.last_frame_idx.unwrap_or(frame_idx);
        let dt = (frame_idx.saturating_sub(last)) as f32;
        if dt <= 0.0 {
            return Some(bbox);
        }

        let nx = (bbox.x as f32 + self.vx * dt).round() as i32;
        let ny = (bbox.y as f32 + self.vy * dt).round() as i32;
        Some(Rect::new(nx, ny, bbox.width, bbox.height))
    }

    /// Update tracker with a newly detected bbox (in pixel coords). Also updates velocity.
    pub fn update(&mut self, new_bbox: Rect, frame_idx: u64) {
        if let Some(prev) = self.bbox {
            let last = self.last_frame_idx.unwrap_or(frame_idx);
            let dt = (frame_idx.saturating_sub(last)).max(1) as f32;
            self.vx = (new_bbox.x as f32 - prev.x as f32) / dt;
            self.vy = (new_bbox.y as f32 - prev.y as f32) / dt;
        }
        self.bbox = Some(new_bbox);
        self.last_frame_idx = Some(frame_idx);
        self.missed = 0;
    }

    /// Mark missed detection for this frame. When missed exceeds patience, reset tracker.
    pub fn missed(&mut self) {
        self.missed += 1;
        if self.missed > self.patience {
            self.reset();
        }
    }

    pub fn reset(&mut self) {
        self.bbox = None;
        self.vx = 0.0;
        self.vy = 0.0;
        self.last_frame_idx = None;
        self.missed = 0;
    }

    /// Expand the bbox slightly to account for movement and to provide context for landmark model.
    pub fn expanded_bbox(&self, frame_w: i32, frame_h: i32, pad_factor: f32) -> Option<Rect> {
        let b = self.bbox?;
        let cx = b.x + (b.width/2);
        let cy = b.y + (b.height/2);
        let half_w = ((b.width as f32) * 0.5 * pad_factor).round() as i32;
        let half_h = ((b.height as f32) * 0.5 * pad_factor).round() as i32;
        let nx = (cx - half_w).max(0);
        let ny = (cy - half_h).max(0);
        let nw = ((half_w * 2).min(frame_w - nx)).max(1);
        let nh = ((half_h * 2).min(frame_h - ny)).max(1);
        Some(Rect::new(nx, ny, nw as i32, nh as i32))
    }
}
