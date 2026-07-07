pub mod classifiers;
pub mod confidence;
pub mod temporal_engine;
pub mod gesture_graph;
pub mod fsm;
pub mod session;

// Re-exports
pub use confidence::{ConfidenceEngine, GestureConfidence, GestureType};
pub use confidence::fuser::ConfidenceFuser;
pub use temporal_engine::{MotionHistory, GestureFeatures};
pub use gesture_graph::GestureStateMachine;
pub use fsm::GestureFSMState;
pub use session::{GestureSessionManager, GestureSession, SessionConfig, SessionState};
