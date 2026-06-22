#[cfg(feature = "camera")]
{
    use opencv::{
        prelude::*,
        videoio,
        core,
    };
    use anyhow::{Result, anyhow};

    pub struct CameraCapture {
        capture: videoio::VideoCapture,
        width: i32,
        height: i32,
    }

    impl CameraCapture {
        pub fn new(index: i32) -> Result<Self> {
            let mut capture = videoio::VideoCapture::new(index, videoio::CAP_ANY)?;

            if !videoio::VideoCapture::is_opened(&capture)? {
                return Err(anyhow!("Failed to open camera at index {}", index));
            }

            // Set reasonable defaults
            let width = capture.get(videoio::CAP_PROP_FRAME_WIDTH)? as i32;
            let height = capture.get(videoio::CAP_PROP_FRAME_HEIGHT)? as i32;

            Ok(Self {
                capture,
                width,
                height,
            })
        }

        pub fn capture_frame(&mut self) -> Result<core::Mat> {
            let mut frame = core::Mat::default();
            self.capture.read(&mut frame)?;

            if frame.empty() {
                return Err(anyhow!("Captured empty frame"));
            }

            Ok(frame)
        }

        pub fn dimensions(&self) -> (i32, i32) {
            (self.width, self.height)
        }
    }
}

#[cfg(not(feature = "camera"))]
{
    use anyhow::{Result, anyhow};

    pub struct CameraCapture;

    impl CameraCapture {
        pub fn new(_index: i32) -> Result<Self> {
            Err(anyhow!("Camera feature not enabled. Recompile with --features camera"))
        }

        pub fn capture_frame(&mut self) -> Result<opencv::core::Mat> {
            Err(anyhow!("Camera feature not enabled"))
        }

        pub fn dimensions(&self) -> (i32, i32) {
            (0, 0)
        }
    }
}


