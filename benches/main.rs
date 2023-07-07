use std::collections::HashMap;
use std::path::Path;

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};

use gen_fixture::{fixture_path, gen_fixture, units::Bytes};
use rusty_reading;

// target = 5 GiB = 5,368,709,120 bytes
// bs = 1 Mib
// count = 5 Gib / 1 Mib = 5,368,709,120 bytes / 1,048,576 bytes = 5,120
//
// dd if=/dev/random of=fixtures/5gib.bin bs=1MiB count=5120
//
//
// target = 500 MiB = 524,288,000 bytes
// bs = 1 Mib
// count = 500
//
// dd if=/dev/random of=fixtures/500mib.bin bs=1MiB count=500

struct Inputs<'a>((usize, &'a str));

impl std::fmt::Display for Inputs<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (size, file) = self.0;
        write!(f, "buffer={},input={}", size, file)
    }
}

const FIXTURES_DIR: &str = "fixtures";

fn bench_read(c: &mut Criterion) {
    const FILE_SIZE: Bytes = Bytes::from_bytes(5 * 1024 * 1024 * 1024);
    let file = fixture_path(Path::new(FIXTURES_DIR), &FILE_SIZE);

    if !file.exists() {
        gen_fixture(Path::new(FIXTURES_DIR), &FILE_SIZE).unwrap();
    } else {
        eprintln!("Using existing fixture: {}", file.to_string_lossy());
    }

    let mut group = c.benchmark_group("rusty_reading");
    group.sample_size(50);
    group.throughput(Throughput::Bytes(FILE_SIZE.to_bytes() as u64));

    // Test from min to max and some in between

    let count = 5;

    let min = 1024;
    let max = 2147479552;

    let inputs = Inputs((256000, file.to_str().unwrap()));
    group.bench_with_input(BenchmarkId::new("read()", &inputs), &inputs, |b, inputs| {
        b.iter(|| rusty_reading::read(inputs.0 .0, inputs.0 .1).unwrap())
    });

    for i in 0..count {
        let size = min + (max - min) / count * i;
        let size: usize = size
            .try_into()
            .map_err(|_| format!("can't convert {} to usize", size))
            .unwrap();

        let inputs = Inputs((size, file.to_str().unwrap()));
        group.bench_with_input(BenchmarkId::new("read()", &inputs), &inputs, |b, inputs| {
            b.iter(|| rusty_reading::read(inputs.0 .0, inputs.0 .1).unwrap())
        });
    }

    // Most efficient so far
    // let middle_size = 1024000i32;

    // Largest read on Windows WSL
    // let middle_size = 2147479552;

    // let count = 1;
    // assert!(count % 2 == 1, "count must be odd");
    // let middle_count = count / 2;
    // for i in 0..count {
    //     let offset = i as i64 - middle_count as i64;

    //     let size = match offset {
    //         0 => middle_size,
    //         x if x > 0 => middle_size * 2i64.pow(offset.unsigned_abs().try_into().unwrap()),
    //         x if x < 0 => middle_size / 2i64.pow(offset.unsigned_abs().try_into().unwrap()),
    //         _ => unreachable!(),
    //     };
    //     let size: usize = size
    //         .try_into()
    //         .map_err(|_| format!("can't convert {} to usize", size))
    //         .unwrap();

    //     // dbg!(size, &file);
    //     // let mut read_sizes = rusty_reading::read_sizes(size, file.to_str().unwrap()).unwrap();
    //     // dbg!(Stats::new(&mut read_sizes));

    //     let inputs = Inputs((size, file.to_str().unwrap()));
    //     group.bench_with_input(BenchmarkId::new("read()", &inputs), &inputs, |b, inputs| {
    //         b.iter(|| rusty_reading::read(inputs.0 .0, inputs.0 .1).unwrap())
    //     });
    // }
}

fn bench_read_bufreader(c: &mut Criterion) {
    const FILE_SIZE_BYTES: u64 = 5 * 1024 * 1024 * 1024;
    const FILE_SIZE_FORMATTED: &str = "5gib";
    let file = Path::new(FIXTURES_DIR).join(format!("{}.bin", FILE_SIZE_FORMATTED));

    let mut group = c.benchmark_group("rusty_reading");
    group.sample_size(50);
    group.throughput(Throughput::Bytes(FILE_SIZE_BYTES));

    // Test from min to max and some in between

    let count = 5;

    let min = 1024;
    let max = 2147479552;

    let inputs = Inputs((256000, file.to_str().unwrap()));
    group.bench_with_input(
        BenchmarkId::new("read_bufreader()", &inputs),
        &inputs,
        |b, inputs| b.iter(|| rusty_reading::read_bufreader(inputs.0 .0, inputs.0 .1).unwrap()),
    );

    for i in 0..count {
        let size = min + (max - min) / count * i;
        let size: usize = size
            .try_into()
            .map_err(|_| format!("can't convert {} to usize", size))
            .unwrap();

        let inputs = Inputs((size, file.to_str().unwrap()));
        group.bench_with_input(
            BenchmarkId::new("read_bufreader()", &inputs),
            &inputs,
            |b, inputs| b.iter(|| rusty_reading::read_bufreader(inputs.0 .0, inputs.0 .1).unwrap()),
        );
    }
}

#[derive(Debug)]
struct Stats {
    count: usize,
    min: usize,
    max: usize,
    mean: f64,
    median: usize,
    mode: usize,
    std_dev: f64,
}

impl Stats {
    fn new(data: &mut Vec<usize>) -> Self {
        assert!(data.len() > 0, "data must have at least one element");

        data.sort_unstable();

        let min = *data.first().unwrap_or(&0);
        let max = *data.last().unwrap_or(&0);
        let mean = data.iter().sum::<usize>() as f64 / data.len() as f64;
        let median = data[data.len() / 2];
        let mode = Stats::mode(data);
        let std_dev = Stats::standard_deviation(data, mean);

        Stats {
            count: data.len(),
            min,
            max,
            mean,
            median,
            mode,
            std_dev,
        }
    }

    fn mode(data: &Vec<usize>) -> usize {
        let mut counts = HashMap::new();
        for &value in data {
            *counts.entry(value).or_insert(0) += 1;
        }

        counts
            .iter()
            .fold((0usize, 0i32), |(mode, mode_count), (&value, &count)| {
                if count > mode_count {
                    (value, count)
                } else {
                    (mode, mode_count)
                }
            })
            .0
    }

    fn standard_deviation(data: &Vec<usize>, mean: f64) -> f64 {
        let variance = data
            .iter()
            .map(|&value| (value as f64 - mean).powf(2.0))
            .sum::<f64>()
            / data.len() as f64;

        variance.sqrt()
    }
}

criterion_group!(benches, bench_read, bench_read_bufreader);
criterion_main!(benches);
