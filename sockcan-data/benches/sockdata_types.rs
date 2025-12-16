use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn validate_can_bmcdata_invariants() {
    let opcode = sockcan::prelude::CanBcmOpCode::RxSetup;
    let frame = sockdata::types::CanBmcData::new(0x123, opcode, 42, vec![1, 2, 3, 4], 4);

    // These assertions run once and don't impact the measured loop.
    assert_eq!(frame.get_id(), 0x123);
    assert_eq!(frame.get_stamp(), 42);
    assert_eq!(frame.get_len(), 4);
    assert!(matches!(frame.get_opcode(), sockcan::prelude::CanBcmOpCode::RxSetup));
}

fn bench_can_bmcdata_new_len8(c: &mut Criterion) {
    validate_can_bmcdata_invariants();

    c.bench_function("sockdata::CanBmcData::new (len=8, alloc)", |b| {
        b.iter(|| {
            let canid = black_box(0x123u32);
            let stamp = black_box(42u64);
            let len = black_box(8u8);

            let opcode = black_box(sockcan::prelude::CanBcmOpCode::RxSetup);

            // NOTE: This includes Vec allocation, because the constructor takes Vec<u8>.
            let data = vec![1u8, 2, 3, 4, 5, 6, 7, 8];

            let frame = sockdata::types::CanBmcData::new(canid, opcode, stamp, data, len);
            black_box(frame);
        })
    });
}

fn bench_can_bmcdata_serde_json_roundtrip(c: &mut Criterion) {
    validate_can_bmcdata_invariants();

    let opcode = sockcan::prelude::CanBcmOpCode::RxSetup;
    let frame =
        sockdata::types::CanBmcData::new(0x123, opcode, 42, vec![1u8, 2, 3, 4, 5, 6, 7, 8], 8);

    c.bench_function("sockdata::CanBmcData serde_json roundtrip", |b| {
        b.iter(|| {
            let json = serde_json::to_vec(black_box(&frame)).unwrap();
            let decoded: sockdata::types::CanBmcData =
                serde_json::from_slice(black_box(&json)).unwrap();
            black_box(decoded);
        })
    });
}

criterion_group!(benches, bench_can_bmcdata_new_len8, bench_can_bmcdata_serde_json_roundtrip);
criterion_main!(benches);

#[cfg(test)]
mod tests {
    #[test]
    fn can_bmcdata_getters_are_consistent() {
        let opcode = sockcan::prelude::CanBcmOpCode::RxSetup;
        let frame = sockdata::types::CanBmcData::new(0x42, opcode, 7, vec![9, 8, 7], 3);

        assert_eq!(frame.get_id(), 0x42);
        assert_eq!(frame.get_stamp(), 7);
        assert_eq!(frame.get_len(), 3);
        assert!(matches!(frame.get_opcode(), sockcan::prelude::CanBcmOpCode::RxSetup));
    }

    #[test]
    fn can_bmcdata_json_roundtrip_is_stable() {
        let opcode = sockcan::prelude::CanBcmOpCode::RxSetup;
        let frame = sockdata::types::CanBmcData::new(0x123, opcode, 42, vec![1, 2, 3, 4], 4);

        let v1 = serde_json::to_value(&frame).unwrap();
        let decoded: sockdata::types::CanBmcData = serde_json::from_value(v1.clone()).unwrap();
        let v2 = serde_json::to_value(&decoded).unwrap();

        assert_eq!(v1, v2);
    }
}
