use nalgebra::Vector3;

/// One-Euro filter for low-latency smoothing of hand landmarks
/// Superior to Kalman for gesture tracking due to:
/// - Lower latency
/// - Easier tuning
/// - Better handling of jitter
pub struct OneEuroFilter {
    min_cutoff: f32,      // Minimum cutoff frequency (Hz)
    beta: f32,            // Speed coefficient
    d_cutoff: f32,        // Derivative cutoff frequency
    
    // State
    prev_position: Vector3<f32>,
    prev_velocity: Vector3<f32>,
    prev_time: Option<f64>,
    initialized: bool,
}

impl OneEuroFilter {
    pub fn new(min_cutoff: f32, beta: f32, d_cutoff: f32) -> Self {
        Self {
            min_cutoff,
            beta,
            d_cutoff,
            prev_position: Vector3::zeros(),
            prev_velocity: Vector3::zeros(),
            prev_time: None,
            initialized: false,
        }
    }
    
    /// Default parameters for hand tracking
    pub fn default_hand() -> Self {
        Self::new(1.0, 0.5, 1.0)
    }
    
    pub fn filter(&mut self, position: Vector3<f32>, timestamp_secs: f64) -> Vector3<f32> {
        if !self.initialized {
            self.prev_position = position;
            self.prev_time = Some(timestamp_secs);
            self.initialized = true;
            return position;
        }
        
        // Calculate time delta
        let dt = if let Some(prev_time) = self.prev_time {
            (timestamp_secs - prev_time).max(0.001)
        } else {
            0.001
        };
        self.prev_time = Some(timestamp_secs);
        
        // Calculate velocity
        let velocity = (position - self.prev_position) / dt as f32;
        
        // Apply filtering
        let filtered_velocity = self.filter_velocity(velocity, dt);
        
        // Update position
        let filtered_position = self.prev_position + filtered_velocity * dt as f32;
        
        // Update state
        self.prev_position = filtered_position;
        self.prev_velocity = filtered_velocity;
        
        filtered_position
    }
    
    fn filter_velocity(&self, velocity: Vector3<f32>, dt: f64) -> Vector3<f32> {
        let d_cutoff = self.d_cutoff;
        let beta = self.beta;
        
        // Alpha for derivative filter
        let alpha = self.alpha(d_cutoff, dt);
        
        // Filter each component
        let vx = self.filter_component(velocity.x, self.prev_velocity.x, alpha);
        let vy = self.filter_component(velocity.y, self.prev_velocity.y, alpha);
        let vz = self.filter_component(velocity.z, self.prev_velocity.z, alpha);
        
        Vector3::new(vx, vy, vz)
    }
    
    fn filter_component(&self, current: f32, previous: f32, alpha: f32) -> f32 {
        alpha * current + (1.0 - alpha) * previous
    }
    
    fn alpha(&self, cutoff: f32, dt: f64) -> f32 {
        let tau = 1.0 / (2.0 * std::f32::consts::PI * cutoff);
        let dt = dt as f32;
        1.0 / (1.0 + tau / dt)
    }
}

impl Default for OneEuroFilter {
    fn default() -> Self {
        Self::default_hand()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;
    
    #[test]
    fn test_one_euro_filter() {
        let mut filter = OneEuroFilter::default_hand();
        let positions = vec![
            (0.0, 0.0, 0.0),
            (0.1, 0.0, 0.0),
            (0.2, 0.0, 0.0),
            (0.3, 0.0, 0.0),
        ];
        
        let mut time = 0.0;
        for (x, y, z) in positions {
            time += 0.033;
            let pos = Vector3::new(x, y, z);
            let filtered = filter.filter(pos, time);
            
            // Filtered position should be close to input
            assert_relative_eq!(filtered.x, x, epsilon = 0.05);
        }
    }
}