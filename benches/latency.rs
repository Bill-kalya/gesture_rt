use criterion::{black_box, criterion_group, criterion_main, Criterion};
use gesture_rt::spatial::filters::kalman::KalmanFilter;
use gesture_rt::gestures::confidence::ConfidenceEngine;
use gesture_rt::gestures::temporal_engine::{MotionHistory, GestureFeatures};
use nalgebra::Vector3;

fn bench_kalman_update(c: &mut Criterion) {
    let mut kf = KalmanFilter::new(0.1, 1.0, 0.5);
    let pos = Vector3::new(10.0, 20.0, 5.0);
    
    c.bench_function("kalman_update", |b| {
        b.iter(|| {
            kf.update(black_box(pos));
        })
    });
}

fn bench_confidence_classify(c: &mut Criterion) {
    let mut engine = ConfidenceEngine::new(0.75, 0.3);
    let features = GestureFeatures {
        mean_velocity: Vector3::new(1.0, 0.0, 0.0),
        path_length: 0.5,
        displacement: Vector3::new(0.4, 0.0, 0.0),
        direction_consistency: 0.8,
        duration_secs: 0.5,
    };
    
    c.bench_function("confidence_classify", |b| {
        b.iter(|| {
            engine.classify(black_box(&features));
        })
    });
}

fn bench_motion_history_push(c: &mut Criterion) {
    let mut history = MotionHistory::new(10);
    let pos = Vector3::new(1.0, 2.0, 3.0);
    
    c.bench_function("motion_history_push", |b| {
        b.iter(|| {
            history.push(black_box(pos), black_box(1000000));
        })
    });
}

criterion_group!(benches, bench_kalman_update, bench_confidence_classify, bench_motion_history_push);
criterion_main!(benches);