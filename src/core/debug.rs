//! Debug and statistics module

use std::collections::VecDeque;
use std::time::Duration;

/// Frame statistics tracker
#[derive(Debug)]
pub struct FrameStats {
    /// Frame time history for averaging
    frame_times: VecDeque<Duration>,
    /// Maximum samples to keep
    max_samples: usize,
    /// Current FPS
    fps: f32,
    /// Average frame time in milliseconds
    avg_frame_time_ms: f32,
    /// Minimum frame time in milliseconds
    min_frame_time_ms: f32,
    /// Maximum frame time in milliseconds
    max_frame_time_ms: f32,
    /// Total frames rendered
    total_frames: u64,
}

impl FrameStats {
    /// Create a new frame stats tracker
    pub fn new() -> Self {
        Self {
            frame_times: VecDeque::with_capacity(120),
            max_samples: 120,
            fps: 0.0,
            avg_frame_time_ms: 0.0,
            min_frame_time_ms: 0.0,
            max_frame_time_ms: 0.0,
            total_frames: 0,
        }
    }

    /// Record a frame with the given delta time
    pub fn record_frame(&mut self, delta: Duration) {
        self.total_frames += 1;

        // Add to history
        if self.frame_times.len() >= self.max_samples {
            self.frame_times.pop_front();
        }
        self.frame_times.push_back(delta);

        // Calculate statistics
        self.update_stats();
    }

    fn update_stats(&mut self) {
        if self.frame_times.is_empty() {
            return;
        }

        let mut total = Duration::ZERO;
        let mut min = Duration::MAX;
        let mut max = Duration::ZERO;

        for &dt in &self.frame_times {
            total += dt;
            min = min.min(dt);
            max = max.max(dt);
        }

        let count = self.frame_times.len() as f32;
        let total_secs = total.as_secs_f32();

        // Guard against division by zero
        if total_secs > 0.0 {
            self.avg_frame_time_ms = (total_secs / count) * 1000.0;
            self.fps = count / total_secs;
        } else {
            self.avg_frame_time_ms = 0.0;
            self.fps = 0.0;
        }

        self.min_frame_time_ms = min.as_secs_f32() * 1000.0;
        self.max_frame_time_ms = max.as_secs_f32() * 1000.0;
    }

    /// Get current FPS
    pub fn fps(&self) -> f32 {
        self.fps
    }

    /// Get average frame time in milliseconds
    pub fn avg_frame_time_ms(&self) -> f32 {
        self.avg_frame_time_ms
    }

    /// Get minimum frame time in milliseconds
    pub fn min_frame_time_ms(&self) -> f32 {
        self.min_frame_time_ms
    }

    /// Get maximum frame time in milliseconds
    pub fn max_frame_time_ms(&self) -> f32 {
        self.max_frame_time_ms
    }

    /// Get total frames rendered
    pub fn total_frames(&self) -> u64 {
        self.total_frames
    }

    /// Get a formatted stats string
    pub fn format_stats(&self) -> String {
        format!(
            "FPS: {:.1} | Frame: {:.2}ms (min: {:.2}, max: {:.2})",
            self.fps, self.avg_frame_time_ms, self.min_frame_time_ms, self.max_frame_time_ms
        )
    }
}

impl Default for FrameStats {
    fn default() -> Self {
        Self::new()
    }
}

/// Debug overlay information
#[derive(Debug, Default)]
pub struct DebugInfo {
    /// Whether debug overlay is enabled
    pub enabled: bool,
    /// Frame statistics
    pub frame_stats: FrameStats,
    /// Custom debug lines
    custom_lines: Vec<String>,
}

impl DebugInfo {
    /// Create new debug info
    pub fn new() -> Self {
        Self {
            enabled: false,
            frame_stats: FrameStats::new(),
            custom_lines: Vec::new(),
        }
    }

    /// Toggle debug overlay
    pub fn toggle(&mut self) {
        self.enabled = !self.enabled;
    }

    /// Add a custom debug line
    pub fn add_line(&mut self, line: impl Into<String>) {
        self.custom_lines.push(line.into());
    }

    /// Clear custom lines
    pub fn clear_lines(&mut self) {
        self.custom_lines.clear();
    }

    /// Get all debug lines
    pub fn get_all_lines(&self) -> Vec<String> {
        let mut lines = vec![self.frame_stats.format_stats()];
        lines.extend(self.custom_lines.iter().cloned());
        lines
    }

    /// Record a frame
    pub fn record_frame(&mut self, delta: Duration) {
        self.frame_stats.record_frame(delta);
    }
}
