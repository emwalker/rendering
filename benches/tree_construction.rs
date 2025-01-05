use criterion::{criterion_group, criterion_main, Criterion};
use rendering::html5::html5ever;
use rendering::testing::tree_construction::fixture_from_filename;
use std::hint::black_box;
use std::time::Duration;

fn bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("html5ever::Dom");
    group.warm_up_time(Duration::new(10, 0));

    group.bench_function("results", |b| {
        b.iter(|| {
            let tests =
                fixture_from_filename(black_box("tests22.dat")).expect("error loading fixture");

            for test in tests.iter() {
                let results = test.results::<html5ever::Dom>().unwrap();
                for mut result in results {
                    let _ = result.run();
                }
            }
        })
    });
}

criterion_group!(benches, bench);
criterion_main!(benches);
