use gesture_rt::spatial::filters::kalman::KalmanFilter;
use gesture_rt::spatial::coordinate_system::WorldCoordinate;
use nalgebra::Vector3;

#[test]
fn test_full_pipeline_integration() {
    // Create Kalman filter
    let mut kf = KalmanFilter::new(0.1, 1.0, 0.5);
    
    // Simulate hand movement
    let positions = vec![
        Vector3::new(0.0, 0.0, 1.0),
        Vector3::new(0.1, 0.0, 1.0),
        Vector3::new(0.2, 0.0, 1.0),
        Vector3::new(0.3, 0.0, 1.0),
    ];
    
    for pos in positions {
        kf.update(pos);
        kf.set_time(0);
    }
    
    assert!(kf.velocity().x > 0.0);
}

#[test]
fn test_coordinate_conversion() {
    let mediapipe = (0.5, 0.3, 0.2);
    let world = WorldCoordinate::from_mediapipe(mediapipe.0, mediapipe.1, mediapipe.2);
    
    assert_eq!(world.x, 0.5);
    assert_eq!(world.y, -0.3);
    assert_eq!(world.z, 0.2);
}