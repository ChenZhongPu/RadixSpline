use radix_spline::{GreedySplineCorridor, RadixSpline};
use rand::{distributions::Uniform, Rng};
use std::time::Instant;

fn main() {
    let range = Uniform::from(0..100000000);
    let mut data: Vec<u64> = rand::thread_rng()
        .sample_iter(&range)
        .take(10000000)
        .collect();

    data.sort_unstable();


    let mut search_keys = vec![];
    let len = data.len();
    search_keys.extend_from_slice(&data[100..110]);
    search_keys.extend_from_slice(&data[1000..1010]);
    search_keys.extend_from_slice(&data[10000..10010]);
    search_keys.extend_from_slice(&data[100000..100010]);
    search_keys.extend_from_slice(&data[500000..500010]);
    search_keys.extend_from_slice(&data[len - 100000..len - 99990]);
    search_keys.extend_from_slice(&data[len - 10000..len - 9990]);
    search_keys.extend_from_slice(&data[len - 1000..len - 990]);
    search_keys.extend_from_slice(&data[len - 100..len - 90]);
    // method 1: binary search
    let mut total = 0;
    for &key in &search_keys {
        let start = Instant::now();
        if let Ok(idx) = data.binary_search(&key) {
            assert_eq!(data[idx], key);
        } else {
            panic!("Error when binary searching!");
        }
        let elapsed = start.elapsed();
        total += elapsed.as_nanos();
    }

    println!("Binary search using {:?} ns", total / search_keys.len() as u128);

    let spline = GreedySplineCorridor::default(&data);
    let mut total = 0;
    // method 2: spline search
    for &key in &search_keys {
        let start = Instant::now();
        if let Some(idx) = spline.search(key) {
            assert_eq!(data[idx], key);
        } else {
            panic!("Error when spline searching!");
        }
        let elapsed = start.elapsed();
        total += elapsed.as_nanos();
    }

    println!("Spline search using {:?} ns", total / search_keys.len() as u128);
    
    // method 3: radix spline search
    let radix_spline = RadixSpline::default(&data);
    let mut total = 0;
    for &key in &search_keys {
        let start = Instant::now();
        if let Some(idx) = radix_spline.search(key) {
            assert_eq!(data[idx], key);
        } else {
            panic!("Error when radix spline searching!");
        }
        let elapsed = start.elapsed();
        total += elapsed.as_nanos();
    }
    println!("SplineRadix search using {:?} ns", total / search_keys.len() as u128);
}
