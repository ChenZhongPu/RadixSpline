use radix_spline::Builder;
use rand::{distributions::Uniform, Rng};
use std::time::Instant;

fn main() {
    let range = Uniform::from(0..10000000);
    let mut data: Vec<u64> = rand::thread_rng()
        .sample_iter(&range)
        .take(1000000)
        .collect();

    let value = 2000;
    data.push(value);

    data.sort_unstable();

    let builder = Builder::default(&data);

    let start = Instant::now();
    if let Some(idx) = builder.search(value) {
        assert_eq!(data[idx], value);
    }
    let elapsed = start.elapsed();
    println!("SplineRadix using {:?} ns", elapsed.as_nanos());

    let start = Instant::now();
    if let Ok(idx) = data.binary_search(&value) {
        assert_eq!(data[idx], value);
    }
    let elapsed = start.elapsed();
    println!("Binary using {:?} ns", elapsed.as_nanos());
}
