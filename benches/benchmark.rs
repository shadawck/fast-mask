use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use fast_mask::PatchMaskGenerator;

fn criterion_benchmark(c: &mut Criterion) {
    let image = image::open("cat.jpg").expect("Failed to load image");

    c.bench_with_input(
        BenchmarkId::new("transform", "cat_image"),
        &image,
        |b, s| {
            b.iter(|| PatchMaskGenerator::new(black_box(0.2), black_box(16)).transform(s.clone()));
        },
    );
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
