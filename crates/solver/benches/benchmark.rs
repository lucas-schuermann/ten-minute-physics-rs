use criterion::{black_box, criterion_group, criterion_main, Criterion};
use solver;

fn drop_scene(_n: usize, i: usize) {
    let mut state = solver::cloth::State::new();
    //state.init();
    for _ in 0..i {
        state.update();
    }
}

fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("sample-size-10");
    group.sample_size(10);
    group.warm_up_time(std::time::Duration::from_secs(20));
    group.bench_function("drop scene: n=1, i=100", |b| {
        b.iter(|| drop_scene(black_box(1), black_box(100)))
    });
    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
