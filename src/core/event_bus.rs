use tokio::sync::mpsc::{self, Sender, Receiver};
use crate::gestures::confidence::GestureType;
use nalgebra::Vector3;

#[derive(Debug, Clone)]
pub enum RuntimeEvent {
    CameraFrame(u64, Vec<u8>), // timestamp_ns, frame_data
    LandmarksExtracted(u64, Vec<Vector3<f32>>), // timestamp, landmarks (21 points)
    SpatialState(u64, Vector3<f32>, Vector3<f32>), // timestamp, position, velocity
    GestureDetected(GestureType, f32), // gesture, confidence
    GestureDispatched(GestureType),
    CalibrationRequested,
    CalibrationReset,
    Error(String),
    SystemShutdown,
}

#[derive(Clone)]
pub struct EventBus {
    sender: Sender<RuntimeEvent>,
}

impl EventBus {
    pub fn new() -> (Self, Receiver<RuntimeEvent>) {
        let (tx, rx) = mpsc::channel(256); // bounded queue for backpressure
        (Self { sender: tx }, rx)
    }
    
    pub async fn emit(&self, event: RuntimeEvent) -> Result<(), mpsc::error::SendError<RuntimeEvent>> {
        self.sender.send(event).await
    }
    
    pub fn try_emit(&self, event: RuntimeEvent) -> Result<(), mpsc::error::TrySendError<RuntimeEvent>> {
        self.sender.try_send(event)
    }
}