use criterion::{black_box, criterion_group, criterion_main, Criterion};
use intent_core::{build_plan, normalize_signature, ParsedIntent};
use serde_json::json;

fn sample_intents(n: usize) -> Vec<ParsedIntent> {
    (0..n)
        .map(|i| ParsedIntent {
            id: format!("i{i}"),
            action_name: if i % 2 == 0 {
                "switch".into()
            } else {
                "open".into()
            },
            target_app_id: Some(if i % 3 == 0 {
                "hosts".into()
            } else {
                "clipboard".into()
            }),
            params: json!({"idx": i}),
            confidence: 1.0,
            source_start: 0,
            source_end: 0,
            explicit: true,
        })
        .collect()
}

fn bench_plan(c: &mut Criterion) {
    let intents = sample_intents(10);
    c.bench_function("normalize_signature_10", |b| {
        b.iter(|| {
            let sig = normalize_signature(black_box(&intents));
            black_box(sig);
        })
    });

    c.bench_function("build_plan_10", |b| {
        b.iter(|| {
            let plan = build_plan(black_box(&intents), 4, "input");
            black_box(plan);
        })
    });
}

criterion_group!(benches, bench_plan);
criterion_main!(benches);
