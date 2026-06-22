use gesture_rt::gestures::temporal_engine::{MotionHistory, GestureFeatures};
use gesture_rt::gestures::confidence::ConfidenceEngine;
use nalgebra::Vector3;

#[test]
fn test_motion_history() {
    let mut history = MotionHistory::new(5);
    
    history.push(Vector3::new(0.0, 0.0, 0.0), 0);
    history.push(Vector3::new(1.0, 0.0, 0.0), 1_000_000_000);
    history.push(Vector3::new(2.0, 0.0, 0.0), 2_000_000_000);
    
    assert_eq!(history.positions.len(), 3);
    assert_eq!(history.velocities.len(), 2);
}

#[test]
fn test_feature_extraction() {
    let mut history = MotionHistory::new(10);
    
    history.push(Vector3::new(0.0, 0.0, 0.0), 0);
    history.push(Vector3::new(0.1, 0.0, 0.0), 100_000_000);
    history.push(Vector3::new(0.2, 0.0, 0.0), 200_000_000);
    history.push(Vector3::new(0.3, 0.0, 0.0), 300_000_000);
    
    let features = GestureFeatures::from_history(&history).unwrap();
    
    assert!(features.mean_velocity.x > 0.0);
    assert!(features.path_length > 0.0);
}

#[test]
fn test_confidence_engine() {
    let mut engine = ConfidenceEngine::new(0.75, 0.3);
    let features = GestureFeatures {
        mean_velocity: Vector3::new(1.0, 0.0, 0.0),
        path_length: 0.5,
        displacement: Vector3::new(0.4, 0.0, 0.0),
        direction_consistency: 0.8,
        duration_secs: 0.5,
    };
    
    let confidence = engine.classify(&features);
    
    assert!(confidence.swipe_right > 0.0 || confidence.swipe_left > 0.0);
}