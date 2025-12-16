use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn validate_parse_action_cases() {
    // These checks run once per benchmark function (not in the hot loop).
    assert!(matches!(
        dbcapi::logic::parse_action("SUBSCRIBE"),
        Some(dbcapi::logic::Action::Subscribe)
    ));
    assert!(matches!(
        dbcapi::logic::parse_action("unsubscribe"),
        Some(dbcapi::logic::Action::Unsubscribe)
    ));
    assert!(matches!(dbcapi::logic::parse_action("Read"), Some(dbcapi::logic::Action::Read)));
    assert!(matches!(dbcapi::logic::parse_action("RESET"), Some(dbcapi::logic::Action::Reset)));
    assert!(dbcapi::logic::parse_action("nope").is_none());
}

fn validate_should_emit_cases() {
    // Updated: rate-gated
    assert!(dbcapi::logic::should_emit(
        sockcan::prelude::CanDataStatus::Updated,
        10_000_000,
        9_000_000,
        500,
        10_000,
        sockdata::types::SubscribeFlag::NEW,
    ));

    // Not updated: watchdog-gated, only if flag == ALL
    // NOTE: The rule is strict `>` (not `>=`). Ensure we exceed the threshold.
    assert!(dbcapi::logic::should_emit(
        sockcan::prelude::CanDataStatus::Timeout,
        10_000_001,
        0,
        500,
        10_000,
        sockdata::types::SubscribeFlag::ALL,
    ));

    assert!(!dbcapi::logic::should_emit(
        sockcan::prelude::CanDataStatus::Timeout,
        10_000_000,
        0,
        500,
        10_000,
        sockdata::types::SubscribeFlag::NEW,
    ));
}

fn bench_parse_action(c: &mut Criterion) {
    validate_parse_action_cases();

    let inputs = ["SUBSCRIBE", "unsubscribe", "Read", "RESET", "nope"];
    c.bench_function("dbcapi::logic::parse_action", |b| {
        b.iter(|| {
            for s in &inputs {
                let r = dbcapi::logic::parse_action(black_box(s));
                black_box(r);
            }
        })
    });
}

fn bench_should_emit_updated(c: &mut Criterion) {
    validate_should_emit_cases();

    c.bench_function("dbcapi::logic::should_emit (Updated)", |b| {
        b.iter(|| {
            let r = dbcapi::logic::should_emit(
                sockcan::prelude::CanDataStatus::Updated,
                black_box(10_000_000),
                black_box(9_000_000),
                black_box(500),
                black_box(10_000),
                black_box(sockdata::types::SubscribeFlag::NEW),
            );
            black_box(r);
        })
    });
}

fn bench_should_emit_watchdog_all(c: &mut Criterion) {
    validate_should_emit_cases();

    c.bench_function("dbcapi::logic::should_emit (watchdog, ALL)", |b| {
        b.iter(|| {
            let r = dbcapi::logic::should_emit(
                sockcan::prelude::CanDataStatus::Timeout,
                black_box(10_000_000),
                black_box(0),
                black_box(500),
                black_box(10_000),
                black_box(sockdata::types::SubscribeFlag::ALL),
            );
            black_box(r);
        })
    });
}

criterion_group!(
    benches,
    bench_parse_action,
    bench_should_emit_updated,
    bench_should_emit_watchdog_all
);
criterion_main!(benches);

#[cfg(test)]
mod tests {
    #[test]
    fn parse_action_is_case_insensitive_and_rejects_unknown() {
        assert!(matches!(
            dbcapi::logic::parse_action("subscribe"),
            Some(dbcapi::logic::Action::Subscribe)
        ));
        assert!(matches!(
            dbcapi::logic::parse_action("UNSUBSCRIBE"),
            Some(dbcapi::logic::Action::Unsubscribe)
        ));
        assert!(matches!(dbcapi::logic::parse_action("read"), Some(dbcapi::logic::Action::Read)));
        assert!(matches!(dbcapi::logic::parse_action("reset"), Some(dbcapi::logic::Action::Reset)));
        assert!(dbcapi::logic::parse_action("???").is_none());
        assert!(dbcapi::logic::parse_action("").is_none());
    }

    #[test]
    fn should_emit_matches_expected_gating() {
        // Updated path (rate)
        assert!(dbcapi::logic::should_emit(
            sockcan::prelude::CanDataStatus::Updated,
            2000,
            0,
            1,
            10,
            sockdata::types::SubscribeFlag::NEW,
        ));

        // Timeout path (watchdog) only when ALL
        assert!(!dbcapi::logic::should_emit(
            sockcan::prelude::CanDataStatus::Timeout,
            20_000,
            0,
            1,
            100,
            sockdata::types::SubscribeFlag::NEW,
        ));
        assert!(dbcapi::logic::should_emit(
            sockcan::prelude::CanDataStatus::Timeout,
            200_000,
            0,
            1,
            100,
            sockdata::types::SubscribeFlag::ALL,
        ));
    }
}
