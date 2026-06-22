use nalgebra::Vector3;

/// Visualization for debugging and development
pub struct Visualization {
    enabled: bool,
    show_axes: bool,
    show_trajectory: bool,
    trajectory: Vec<Vector3<f32>>,
    max_trajectory_points: usize,
}

impl Visualization {
    pub fn new() -> Self {
        Self {
            enabled: true,
            show_axes: true,
            show_trajectory: true,
            trajectory: Vec::new(),
            max_trajectory_points: 100,
        }
    }
    
    pub fn enable(&mut self) {
        self.enabled = true;
    }
    
    pub fn disable(&mut self) {
        self.enabled = false;
    }
    
    pub fn update_landmarks(&self, landmarks: &[Vector3<f32>]) {
        if self.enabled {
            // In Phase 1, just log the count
            if !landmarks.is_empty() {
                println!("[Vis] {} landmarks detected", landmarks.len());
            }
        }
    }
    
    pub fn update_position(&mut self, position: Vector3<f32>) {
        if self.enabled && self.show_trajectory {
            self.trajectory.push(position);
            if self.trajectory.len() > self.max_trajectory_points {
                self.trajectory.remove(0);
            }
        }
    }
    
    pub fn draw_coordinate_axes(&self) {
        if self.enabled && self.show_axes {
            println!("[Vis] Coordinate System: X=right, Y=up, Z=toward user");
        }
    }
    
    pub fn get_trajectory(&self) -> &[Vector3<f32>] {
        &self.trajectory
    }
    
    pub fn clear_trajectory(&mut self) {
        self.trajectory.clear();
    }
    
    pub fn set_max_trajectory_points(&mut self, max: usize) {
        self.max_trajectory_points = max;
    }
}

impl Default for Visualization {
    fn default() -> Self {
        Self::new()
    }
}