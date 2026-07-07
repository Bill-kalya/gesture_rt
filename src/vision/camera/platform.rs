use anyhow::Result;
#[cfg(feature = "camera_opencv")]
use opencv::core::Mat;

/// Platform-agnostic camera interface
pub trait CameraBackend: Send + Sync {
    fn open(&mut self, device_id: u32) -> Result<()>;
    fn capture_frame(&mut self) -> Result<Vec<u8>>;
    fn width(&self) -> u32;
    fn height(&self) -> u32;
    fn fps(&self) -> u32;
    fn close(&mut self) -> Result<()>;
}

/// Factory for platform-specific camera backends
pub struct CameraFactory;

impl CameraFactory {
    pub fn create(device_id: u32) -> Box<dyn CameraBackend> {
        #[cfg(all(target_os = "windows", feature = "camera_native"))]
        {
            return Box::new(WindowsCameraBackend::new(device_id));
        }
        #[cfg(all(target_os = "linux", feature = "camera_native"))]
        {
            return Box::new(LinuxCameraBackend::new(device_id));
        }
        #[cfg(all(target_os = "macos", feature = "camera_native"))]
        {
            return Box::new(MacOSCameraBackend::new(device_id));
        }
        #[cfg(all(target_os = "android", feature = "camera_native"))]
        {
            return Box::new(AndroidCameraBackend::new(device_id));
        }
        #[cfg(feature = "camera_opencv")]
        {
            return Box::new(OpenCVCameraBackend::new(device_id));
        }
        #[cfg(not(any(feature = "camera_native", feature = "camera_opencv")))]
        {
            unimplemented!("No camera backend enabled. Enable 'camera_opencv' or 'camera_native' feature")
        }
    }
}

#[cfg(feature = "camera_opencv")]
pub struct OpenCVCameraBackend {
    device_id: u32,
    capture: Option<opencv::videoio::VideoCapture>,
}

#[cfg(feature = "camera_opencv")]
impl OpenCVCameraBackend {
    pub fn new(device_id: u32) -> Self {
        Self {
            device_id,
            capture: None,
        }
    }
}

#[cfg(feature = "camera_opencv")]
impl CameraBackend for OpenCVCameraBackend {
    fn open(&mut self, device_id: u32) -> Result<()> {
        use opencv::videoio::VideoCapture;
        let capture = VideoCapture::new(device_id as i32, opencv::videoio::CAP_ANY)?;
        self.capture = Some(capture);
        self.device_id = device_id;
        Ok(())
    }
    
    fn capture_frame(&mut self) -> Result<Vec<u8>> {
        use opencv::prelude::*;
        let mut frame = Mat::default();
        if let Some(capture) = &mut self.capture {
            capture.read(&mut frame)?;
            if frame.empty() {
                return Err(anyhow::anyhow!("Empty frame"));
            }
            // Convert to RGB and return raw bytes
            let mut rgb = Mat::default();
            opencv::imgproc::cvt_color(&frame, &mut rgb, opencv::imgproc::COLOR_BGR2RGB)?;
            let data = rgb.data_bytes()?;
            Ok(data.to_vec())
        } else {
            Err(anyhow::anyhow!("Camera not open"))
        }
    }
    
    fn width(&self) -> u32 {
        self.capture.as_ref().map(|c| c.get(opencv::videoio::CAP_PROP_FRAME_WIDTH).unwrap_or(0.0) as u32).unwrap_or(0)
    }
    
    fn height(&self) -> u32 {
        self.capture.as_ref().map(|c| c.get(opencv::videoio::CAP_PROP_FRAME_HEIGHT).unwrap_or(0.0) as u32).unwrap_or(0)
    }
    
    fn fps(&self) -> u32 {
        self.capture.as_ref().map(|c| c.get(opencv::videoio::CAP_PROP_FPS).unwrap_or(30.0) as u32).unwrap_or(30)
    }
    
    fn close(&mut self) -> Result<()> {
        self.capture = None;
        Ok(())
    }
}

// Stubs for native backends - implement based on platform
#[cfg(target_os = "windows")]
pub struct WindowsCameraBackend {
    device_id: u32,
    // Media Foundation implementation
}

#[cfg(target_os = "windows")]
impl WindowsCameraBackend {
    pub fn new(device_id: u32) -> Self {
        Self { device_id }
    }
}

#[cfg(target_os = "windows")]
impl CameraBackend for WindowsCameraBackend {
    fn open(&mut self, _device_id: u32) -> Result<()> {
        unimplemented!("Windows Media Foundation camera backend - implement with windows crate");
    }
    fn capture_frame(&mut self) -> Result<Vec<u8>> {
        unimplemented!()
    }
    fn width(&self) -> u32 { 640 }
    fn height(&self) -> u32 { 480 }
    fn fps(&self) -> u32 { 30 }
    fn close(&mut self) -> Result<()> { Ok(()) }
}

// Similar for Linux, macOS, Android...
#[cfg(target_os = "linux")]
pub struct LinuxCameraBackend {
    device_id: u32,
}

#[cfg(target_os = "linux")]
impl LinuxCameraBackend {
    pub fn new(device_id: u32) -> Self {
        Self { device_id }
    }
}

#[cfg(target_os = "linux")]
impl CameraBackend for LinuxCameraBackend {
    fn open(&mut self, _device_id: u32) -> Result<()> {
        unimplemented!("Linux V4L2 camera backend - implement with v4l crate");
    }
    fn capture_frame(&mut self) -> Result<Vec<u8>> {
        unimplemented!()
    }
    fn width(&self) -> u32 { 640 }
    fn height(&self) -> u32 { 480 }
    fn fps(&self) -> u32 { 30 }
    fn close(&mut self) -> Result<()> { Ok(()) }
}

#[cfg(target_os = "macos")]
pub struct MacOSCameraBackend {
    device_id: u32,
}

#[cfg(target_os = "macos")]
impl MacOSCameraBackend {
    pub fn new(device_id: u32) -> Self {
        Self { device_id }
    }
}

#[cfg(target_os = "macos")]
impl CameraBackend for MacOSCameraBackend {
    fn open(&mut self, _device_id: u32) -> Result<()> {
        unimplemented!("macOS AVFoundation camera backend");
    }
    fn capture_frame(&mut self) -> Result<Vec<u8>> {
        unimplemented!()
    }
    fn width(&self) -> u32 { 640 }
    fn height(&self) -> u32 { 480 }
    fn fps(&self) -> u32 { 30 }
    fn close(&mut self) -> Result<()> { Ok(()) }
}

#[cfg(target_os = "android")]
pub struct AndroidCameraBackend {
    device_id: u32,
}

#[cfg(target_os = "android")]
impl AndroidCameraBackend {
    pub fn new(device_id: u32) -> Self {
        Self { device_id }
    }
}

#[cfg(target_os = "android")]
impl CameraBackend for AndroidCameraBackend {
    fn open(&mut self, _device_id: u32) -> Result<()> {
        unimplemented!("Android CameraX camera backend");
    }
    fn capture_frame(&mut self) -> Result<Vec<u8>> {
        unimplemented!()
    }
    fn width(&self) -> u32 { 640 }
    fn height(&self) -> u32 { 480 }
    fn fps(&self) -> u32 { 30 }
    fn close(&mut self) -> Result<()> { Ok(()) }
}