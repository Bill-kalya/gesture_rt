use opencv::{prelude::*, videoio, core};

fn main() -> opencv::Result<()> {
    // Open default camera (index 0)
    let mut cam = videoio::VideoCapture::new(0, videoio::CAP_ANY)?;
    if !videoio::VideoCapture::is_opened(&cam)? {
        eprintln!("Failed to open default camera");
        return Ok(());
    }

    let width = cam.get(videoio::CAP_PROP_FRAME_WIDTH)? as i32;
    let height = cam.get(videoio::CAP_PROP_FRAME_HEIGHT)? as i32;
    println!("Opened camera: {}x{}", width, height);

    let mut frame = core::Mat::default();
    cam.read(&mut frame)?;
    println!("Captured frame empty?: {}", frame.empty());

    Ok(())
}
