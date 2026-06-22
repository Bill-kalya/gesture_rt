fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    
    #[cfg(feature = "camera")]
    {
        // OpenCV linking hints for Windows
        #[cfg(target_os = "windows")]
        {
            println!("cargo:rustc-link-search=native=C:/opencv/build/x64/vc15/lib");
            println!("cargo:rustc-link-lib=opencv_world470");
        }
        
        // For Linux
        #[cfg(target_os = "linux")]
        {
            println!("cargo:rustc-link-lib=opencv_core");
            println!("cargo:rustc-link-lib=opencv_videoio");
            println!("cargo:rustc-link-lib=opencv_imgproc");
        }
        
        // For macOS
        #[cfg(target_os = "macos")]
        {
            println!("cargo:rustc-link-lib=opencv_core");
            println!("cargo:rustc-link-lib=opencv_videoio");
        }
    }
}