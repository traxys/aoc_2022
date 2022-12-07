use bstr::BString;
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use problems::solutions::day5::*;

fn day_bench(c: &mut Criterion) {
    std::env::set_var("AOC_BENCH", "1");

    let input_path = std::env::var("AOC_INPUT").unwrap();
    let input: BString = std::fs::read(&input_path).unwrap().into();

    c.bench_with_input(BenchmarkId::new("parsing", &input_path), &input, |b, i| {
        b.iter(|| parsing(i))
    });

    let parsed = parsing(&input).unwrap();

    c.bench_with_input(BenchmarkId::new("part1", &input_path), &parsed, |b, i| {
        b.iter(|| part1(i.clone()))
    });
    c.bench_with_input(BenchmarkId::new("part2", &input_path), &parsed, |b, i| {
        b.iter(|| part2(i.clone()))
    });
}

criterion_group!(benches, day_bench);
criterion_main!(benches);
