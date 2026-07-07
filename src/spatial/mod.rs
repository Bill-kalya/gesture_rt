pub mod coordinate_system;
pub mod kinematics;
pub mod orientation;
pub mod prediction;
pub mod filters;
pub mod features;
pub mod depth;
pub mod normalization;
pub mod spatial_state;

// Re-exports
pub use coordinate_system::WorldCoordinate;
pub use filters::kalman::KalmanFilter;
pub use filters::one_euro::OneEuroFilter;
pub use features::{FeatureGenerator, GestureFeatures};
pub use depth::{DepthEstimator, DepthMethod};
pub use normalization::SpatialNormalizer;
pub use orientation::OrientationEstimator;
pub use spatial_state::*;
