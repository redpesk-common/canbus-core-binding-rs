use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};

use sockcan::prelude::CanBcmOpCode;
use sockdata::types::{DataBcmMsg, SubscribeFlag, SubscribeParam};

fn validate_subscribe_param_json_shape() {
    let canids: Vec<u32> = (0..4u32).map(|i| 0x200u32.wrapping_add(i)).collect();
    let param = SubscribeParam::new(canids, 10_000, 250, SubscribeFlag::ALL);

    // Validate serialization is possible and stable (without relying on field visibility).
    let v = serde_json::to_value(&param).unwrap();
    assert!(v.is_object());
}

fn validate_data_bcm_msg_roundtrip() {
    let msg = DataBcmMsg { canid: 0x123, stamp: 42, status: CanBcmOpCode::RxSetup };

    let v1 = serde_json::to_value(&msg).unwrap();
    let decoded: DataBcmMsg = serde_json::from_value(v1.clone()).unwrap();
    let v2 = serde_json::to_value(&decoded).unwrap();

    assert_eq!(v1, v2);
}

fn bench_subscribe_param_new(c: &mut Criterion) {
    validate_subscribe_param_json_shape();

    let mut group = c.benchmark_group("sockcan-binding/SubscribeParam::new");

    for &count in &[1usize, 8, 64, 256] {
        group.bench_with_input(BenchmarkId::from_parameter(count), &count, |b, &count| {
            b.iter(|| {
                let canids: Vec<u32> =
                    (0..count as u32).map(|i| 0x100u32.wrapping_add(i)).collect();

                let watchdog = black_box(10_000u64);
                let rate = black_box(250u64);
                let flag = black_box(SubscribeFlag::NEW);

                let param = SubscribeParam::new(canids, watchdog, rate, flag);
                black_box(param);
            })
        });
    }

    group.finish();
}

fn bench_subscribe_param_serde_json_roundtrip(c: &mut Criterion) {
    validate_subscribe_param_json_shape();

    let canids: Vec<u32> = (0..64u32).map(|i| 0x200u32.wrapping_add(i)).collect();
    let param = SubscribeParam::new(canids, 10_000, 250, SubscribeFlag::ALL);

    c.bench_function("sockcan-binding/SubscribeParam serde_json roundtrip", |b| {
        b.iter(|| {
            let json = serde_json::to_vec(black_box(&param)).unwrap();
            let decoded: SubscribeParam = serde_json::from_slice(black_box(&json)).unwrap();
            black_box(decoded);
        })
    });
}

fn bench_data_bcm_msg_serde_json_roundtrip(c: &mut Criterion) {
    validate_data_bcm_msg_roundtrip();

    let msg = DataBcmMsg { canid: 0x123, stamp: 42, status: CanBcmOpCode::RxSetup };

    c.bench_function("sockcan-binding/DataBcmMsg serde_json roundtrip", |b| {
        b.iter(|| {
            let json = serde_json::to_vec(black_box(&msg)).unwrap();
            let decoded: DataBcmMsg = serde_json::from_slice(black_box(&json)).unwrap();
            black_box(decoded);
        })
    });
}

criterion_group!(
    benches,
    bench_subscribe_param_new,
    bench_subscribe_param_serde_json_roundtrip,
    bench_data_bcm_msg_serde_json_roundtrip
);
criterion_main!(benches);

#[cfg(test)]
mod tests {

    #[test]
    fn subscribe_param_json_roundtrip_is_stable() {
        let canids: Vec<u32> = (0..16u32).map(|i| 0x300u32.wrapping_add(i)).collect();
        let param = SubscribeParam::new(canids, 10_000, 250, SubscribeFlag::NEW);

        let v1 = serde_json::to_value(&param).unwrap();
        let decoded: SubscribeParam = serde_json::from_value(v1.clone()).unwrap();
        let v2 = serde_json::to_value(&decoded).unwrap();

        assert_eq!(v1, v2);
    }

    #[test]
    fn data_bcm_msg_json_roundtrip_is_stable() {
        let msg = DataBcmMsg { canid: 0x7FF, stamp: 1, status: CanBcmOpCode::RxTimeout };

        let v1 = serde_json::to_value(&msg).unwrap();
        let decoded: DataBcmMsg = serde_json::from_value(v1.clone()).unwrap();
        let v2 = serde_json::to_value(&decoded).unwrap();

        assert_eq!(v1, v2);
    }
}
