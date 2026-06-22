use gesture_rt::spatial::filters::kalman::KalmanFilter;
use nalgebra::Vector3;
use approx::assert_relative_eq;

#[test]
fn test_kalman_initialization() {
    let kf = KalmanFilter::new(0.1, 1.0, 0.5);
    assert_eq!(kf.position(), Vector3::zeros());
}

#[test]
fn test_kalman_first_measurement() {
    let mut kf = KalmanFilter::new(0.1, 1.0, 0.5);
    let measurement = Vector3::new(5.0, 10.0, 2.0);
    kf.update(measurement);
    
    assert_relative_eq!(kf.position().x, 5.0, epsilon = 1e-5);
    assert_relative_eq!(kf.position().y, 10.0, epsilon = 1e-5);
    assert_relative_eq!(kf.position().z, 2.0, epsilon = 1e-5);
}

#[test]
fn test_kalman_temporal() {
    let mut kf = KalmanFilter::new(0.1, 1.0, 0.5);
    
    kf.set_time(0);
    kf.update(Vector3::new(0.0, 0.0, 0.0));
    
    kf.set_time(1_000_000_000);
    kf.update(Vector3::new(1.0, 0.0, 0.0));
    
    assert_relative_eq!(kf.velocity().x, 1.0, epsilon = 0.2);
}