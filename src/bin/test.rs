use radix_spline::RadixSpline;
use rand::{distributions::Uniform, Rng};
use std::time::Instant;

fn main() {
    // let range = Uniform::from(0..50);
    // let mut data: Vec<u64> = rand::thread_rng()
    //     .sample_iter(&range)
    //     .take(100)
    //     .collect();

    // data.sort_unstable();

    let data = vec![
        0, 0, 0, 1, 1, 2, 4, 5, 5, 5, 5, 6, 6, 8, 8, 8, 8, 8, 9, 10, 11, 11, 11, 11, 12, 13, 14,
        18, 19, 19, 20, 21, 21, 22, 22, 22, 23, 23, 23, 24, 24, 26, 26, 26, 27, 27, 28, 28, 29, 29,
        29, 29, 30, 30, 30, 31, 31, 31, 31, 31, 32, 32, 32, 32, 32, 33, 33, 33, 34, 34, 35, 35, 35,
        36, 36, 36, 36, 36, 37, 37, 38, 38, 38, 39, 40, 40, 40, 41, 41, 42, 42, 43, 43, 44, 45, 46,
        47, 48, 48, 49,
    ];

    println!("{:?}", &data);

    // method 3: radix spline search
    let radix_spline = RadixSpline::new(&data, 4, 2);
    // println!("points: {:?}", radix_spline.points);
    let mut total = 0;
    for &key in &data {
        let start = Instant::now();
        if let Some(idx) = radix_spline.search(key) {
            assert_eq!(data[idx], key);
        } else {
            panic!("Error when radix spline searching {}!", key);
        }
        let elapsed = start.elapsed();
        total += elapsed.as_nanos();
    }
    println!(
        "SplineRadix search using {:?} ns",
        total / data.len() as u128
    );
}
