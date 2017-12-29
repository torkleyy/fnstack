#![feature(test)]

#[macro_use]
extern crate criterion;
extern crate fnstack;
extern crate test;

use criterion::{Bencher, Criterion, Fun};
use fnstack::{FnStackOnce, StaticFn};
use test::black_box;

fn sample_fn(n: usize) -> usize {
    if n == 0 {
        1
    } else {
        n * sample_fn(n - 1)
    }
}

struct Sample;

impl StaticFn<usize, usize> for Sample {
    fn call(args: usize) -> usize {
        sample_fn(args)
    }
}

fn bench_call_iter_heap(b: &mut Bencher) {
    fn setup() -> Vec<Box<Fn(usize) -> usize>> {
        let v = vec![
            Box::new(|x| sample_fn(x)) as Box<Fn(usize) -> usize>,
            Box::new(|x| sample_fn(x)) as Box<Fn(usize) -> usize>,
            Box::new(|x| sample_fn(x)) as Box<Fn(usize) -> usize>,
        ];

        black_box(v)
    }

    b.iter_with_setup(setup, |fns| {
        fns.into_iter()
            .enumerate()
            .map(|(i, f)| f(i))
            .sum::<usize>()
    })
}

fn bench_call_iter_inline(b: &mut Bencher) {
    b.iter_with_setup(|| {
        let v: Vec<&_> = vec![
            &sample_fn,
            &sample_fn,
            &sample_fn,
        ];

        black_box(v)
    }, |fns| {
        fns.into_iter()
            .enumerate()
            .map(|(i, f)| f(i))
            .sum::<usize>()
    })
}

fn bench_call_iter_stack(b: &mut Bencher) {
    fn setup() -> Vec<FnStackOnce<'static, usize, usize, [u8; 8]>> {
        let v = vec![
            FnStackOnce::new(sample_fn),
            FnStackOnce::new(sample_fn),
            FnStackOnce::new(sample_fn),
        ];

        black_box(v)
    }

    b.iter_with_setup(setup, |fns| {
        fns.into_iter()
            .enumerate()
            .map(|(i, f)| f.call(i))
            .sum::<usize>()
    })
}

fn bench_call_iter_static(b: &mut Bencher) {
    fn setup() -> Vec<FnStackOnce<'static, usize, usize, [u8; 0]>> {
        let v = vec![
            FnStackOnce::from_static::<Sample>(),
            FnStackOnce::from_static::<Sample>(),
            FnStackOnce::from_static::<Sample>(),
        ];

        black_box(v)
    }

    b.iter_with_setup(setup, |fns| {
        fns.into_iter()
            .enumerate()
            .map(|(i, f)| f.call(i))
            .sum::<usize>()
    })
}

fn benches(c: &mut Criterion) {
    use std::time::Duration;

    c.sample_size(1_000);
    c.measurement_time(Duration::from_secs(10));
    c.bench_functions(
        "iter call",
        vec![
            Fun::new("heap", |b, _| bench_call_iter_heap(b)),
            Fun::new("inline", |b, _| bench_call_iter_inline(b)),
            Fun::new("stack", |b, _| bench_call_iter_stack(b)),
            Fun::new("static", |b, _| bench_call_iter_static(b)),
        ],
        &3,
    );
}

criterion_group!(benches_group, benches);
criterion_main!(benches_group);
