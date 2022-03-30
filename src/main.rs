use radix_spline::{GreedySplineCorridor, RadixSpline};
use rand::{distributions::Uniform, Rng};
use std::time::Instant;

fn main() {
    let range = Uniform::from(0..10000000);
    let mut data: Vec<u64> = rand::thread_rng()
        .sample_iter(&range)
        .take(1000000)
        .collect();

    let value = 8888;
    data.push(value);

    data.sort_unstable();


    // method 1: binary search
    let start = Instant::now();
    if let Ok(idx) = data.binary_search(&value) {
        assert_eq!(data[idx], value);
    } else {
        panic!("Error when binary searching!");
    }
    let elapsed = start.elapsed();
    println!("Binary search using {:?} ns", elapsed.as_nanos());

    let spline = GreedySplineCorridor::default(&data);
    // method 2: spline search
    let start = Instant::now();
    if let Some(idx) = spline.search(value) {
        assert_eq!(data[idx], value);
    } else {
        panic!("Error when spline searching!");
    }
    let elapsed = start.elapsed();
    println!("Spline search using {:?} ns", elapsed.as_nanos());  
    
    // method 3: radix spline search
    let radix_spline = RadixSpline::default(&data);
    let start = Instant::now();
    if let Some(idx) = radix_spline.search(value) {
        assert_eq!(data[idx], value);
    } else {
        panic!("Error when radix spline searching!");
    }
    let elapsed = start.elapsed();
    println!("SplineRadix search using {:?} ns", elapsed.as_nanos());
}
