use nalgebra as na;
use na::{Vector6, Matrix6, Vector3, Matrix3};

/// State vector: [x, y, z, vx, vy, vz]^T
pub struct KalmanFilter {
    /// State estimate (6D)
    pub x_hat: Vector6<f32>,
    /// Error covariance (6x6)
    pub p: Matrix6<f32>,
    /// Process noise covariance (6x6)
    pub q: Matrix6<f32>,
    /// Measurement noise covariance (3x3)
    pub r: Matrix3<f32>,
    /// Measurement matrix (3x6) - maps state to measurement (position only)
    pub h: na::Matrix3x6<f32>,
    /// Time of last update (nanoseconds)
    last_time: u64,
    /// Whether filter has been initialized
    initialized: bool,
}

impl KalmanFilter {
    /// Create new Kalman filter with tunable parameters
    pub fn new(
        process_noise_position: f32,
        process_noise_velocity: f32,
        measurement_noise: f32,
    ) -> Self {
        // Process noise covariance Q (diagonal)
        let mut q = Matrix6::zeros();
        q[(0,0)] = process_noise_position;
        q[(1,1)] = process_noise_position;
        q[(2,2)] = process_noise_position;
        q[(3,3)] = process_noise_velocity;
        q[(4,4)] = process_noise_velocity;
        q[(5,5)] = process_noise_velocity;

        // Measurement noise covariance R (3x3, diagonal)
        let mut r = Matrix3::zeros();
        r[(0,0)] = measurement_noise;
        r[(1,1)] = measurement_noise;
        r[(2,2)] = measurement_noise;

        // Measurement matrix H: [I_3x3 | 0_3x3]
        let mut h = na::Matrix3x6::zeros();
        h[(0,0)] = 1.0;
        h[(1,1)] = 1.0;
        h[(2,2)] = 1.0;

        Self {
            x_hat: Vector6::zeros(),
            p: Matrix6::identity(),
            q,
            r,
            h,
            last_time: 0,
            initialized: false,
        }
    }

    /// Predict step using time delta
    pub fn predict(&mut self, dt: f32) {
        if !self.initialized || dt <= 0.0 {
            return;
        }

        // State transition matrix F (constant velocity model)
        // [I, dt*I; 0, I]
        let mut f = Matrix6::identity();
        f[(0,3)] = dt;
        f[(1,4)] = dt;
        f[(2,5)] = dt;

        // x = F * x
        self.x_hat = f * self.x_hat;

        // P = F * P * F^T + Q
        self.p = f * self.p * f.transpose() + self.q;
    }

    /// Update step with position measurement (x, y, z)
    pub fn update(&mut self, measurement: Vector3<f32>) {
        if !self.initialized {
            // First measurement: initialize state
            self.x_hat[0] = measurement[0];
            self.x_hat[1] = measurement[1];
            self.x_hat[2] = measurement[2];
            // Velocity starts at 0
            self.initialized = true;
            return;
        }

        // Innovation: y = z - H*x
        let y = measurement - self.h * self.x_hat;

        // Innovation covariance: S = H*P*H^T + R
        let s = self.h * self.p * self.h.transpose() + self.r;

        // Kalman gain: K = P*H^T * S^-1
        let k = self.p * self.h.transpose() * s.try_inverse().unwrap();

        // Update state: x = x + K*y
        self.x_hat += k * y;

        // Update covariance: P = (I - K*H) * P
        let kh = k * self.h;
        self.p = (Matrix6::identity() - kh) * self.p;
    }

    /// Get current position estimate
    pub fn position(&self) -> Vector3<f32> {
        Vector3::new(self.x_hat[0], self.x_hat[1], self.x_hat[2])
    }

    /// Get current velocity estimate
    pub fn velocity(&self) -> Vector3<f32> {
        Vector3::new(self.x_hat[3], self.x_hat[4], self.x_hat[5])
    }

    /// Set timestamp for delta calculation
    pub fn set_time(&mut self, time_ns: u64) {
        if self.last_time > 0 {
            let dt_ns = time_ns.saturating_sub(self.last_time);
            let dt_secs = dt_ns as f32 / 1_000_000_000.0;
            self.predict(dt_secs);
        }
        self.last_time = time_ns;
    }

    /// Reset filter
    pub fn reset(&mut self) {
        self.x_hat = Vector6::zeros();
        self.p = Matrix6::identity();
        self.initialized = false;
        self.last_time = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn test_kalman_constant_position() {
        let mut kf = KalmanFilter::new(0.1, 1.0, 0.5);
        let pos = Vector3::new(10.0, 20.0, 5.0);
        
        kf.update(pos);
        assert_relative_eq!(kf.position().x, 10.0, epsilon = 1e-3);
        
        // Second measurement same position
        kf.update(pos);
        assert_relative_eq!(kf.position().x, 10.0, epsilon = 1e-3);
        assert_relative_eq!(kf.velocity().x, 0.0, epsilon = 1e-2);
    }

    #[test]
    fn test_kalman_moving() {
        let mut kf = KalmanFilter::new(0.1, 1.0, 0.5);
        
        // Simulate movement with known velocity
        kf.update(Vector3::new(0.0, 0.0, 0.0));
        kf.set_time(0);
        
        kf.set_time(1_000_000_000); // 1 second later
        kf.update(Vector3::new(1.0, 0.0, 0.0));
        
        assert_relative_eq!(kf.velocity().x, 1.0, epsilon = 0.1);
    }
}