use criterion::{Criterion, criterion_group, criterion_main};
use hashbrown::HashMap;
use std::{hint::black_box, time::Duration};
use txmap::prelude::*;

fn iter(c: &mut Criterion) {
    let mut group = c.benchmark_group("iter");
    group.warm_up_time(Duration::from_secs(5));
    group.measurement_time(Duration::from_secs(10));
    group.sample_size(1000);

    for size in [1_000usize, 10_000, 100_000] {
        let txmap: TxMap<u64, u64> = TxMap::new();
        let mut hashbrownmap: HashMap<u64, u64> = HashMap::new();
        for i in 0..size {
            txmap.insert(i as u64, i as u64);
            hashbrownmap.insert(i as u64, i as u64);
        }

        group.bench_function(format!("hashbrownmap_iter_{size}"), |b| {
            b.iter(|| {
                let mut sum = 0u64;
                for (_, v) in &hashbrownmap {
                    sum += *v;
                }
                black_box(sum);
            });
        });
        group.bench_function(format!("txmap_iter_{size}"), |b| {
            b.iter(|| {
                let mut sum = 0u64;
                for (_, v) in txmap.iter() {
                    sum += *v;
                }
                black_box(sum);
            });
        });
    }
}

criterion_group!(benches, iter);
criterion_main!(benches);
