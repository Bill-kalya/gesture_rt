use gesture_rt::vision::projection::PinholeCamera;
use nalgebra::Vector3;
use approx::assert_relative_eq;

#[test]
fn test_pinhole_project() {
    let cam = PinholeCamera::from_fov(640.0, 480.0, 60.0);
    let world = Vector3::new(0.1, 0.05, 1.0);
    
    let projected = cam.project(world).unwrap();
    
    assert!(projected.x > 0.0);
    assert!(projected.y > 0.0);
}

#[test]
fn test_pinhole_unproject() {
    let cam = PinholeCamera::from_fov(640.0, 480.0, 60.0);
    let world = Vector3::new(0.2, 0.1, 0.8);
    
    let proj = cam.project(world).unwrap();
    let back = cam.unproject(proj, world.z);
    
    assert_relative_eq!(back.x, world.x, epsilon = 1e-5);
    assert_relative_eq!(back.y, world.y, epsilon = 1e-5);
}