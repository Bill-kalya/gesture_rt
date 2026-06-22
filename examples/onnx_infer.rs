fn main() {
    #[cfg(not(feature = "onnx"))]
    {
        println!("Compile with --features onnx to enable the ONNX inference example");
        return;
    }

    #[cfg(feature = "onnx")]
    {
        println!("ONNX example compiled. Please ensure ONNX Runtime is installed on your system.");
        println!("This example is a skeleton. Place a model at ./models/hand_landmark.onnx and run:");
        println!("  cargo run --example onnx_infer --features onnx");

        // Real inference requires linking to ONNX Runtime (system deps). Implement inference logic here.
    }
}
