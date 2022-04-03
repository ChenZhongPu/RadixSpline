use radix_spline::GreedySplineCorridor;
use radix_spline::RadixSpline;
use rand::{distributions::Uniform, Rng};
use std::process::exit;
use std::time::Instant;

use std::fs::File;
use std::io::{BufReader, Read};

fn load_data(path: &str) -> Vec<u64> {
    let mut buffer = [0u8; std::mem::size_of::<u64>()];
    let mut file = BufReader::new(File::open(path).expect("Unable to open file"));
    file.read_exact(&mut buffer).expect("Unable to read file");
    let size = u64::from_le_bytes(buffer) as usize;
    let mut data: Vec<u64> = Vec::with_capacity(size);
    for _ in 0..size {
        let mut buffer = [0u8; std::mem::size_of::<u64>()];
        file.read_exact(&mut buffer).expect("Unable to read file");
        let d = u64::from_le_bytes(buffer);
        data.push(d)
    }
    data
}

fn random_data_keys() -> (Vec<u64>, Vec<u64>) {
    let range = Uniform::from(0..100000000);
    let mut data: Vec<u64> = rand::thread_rng()
        .sample_iter(&range)
        .take(10000000)
        .collect();
    let mut keys = vec![];
    keys.extend_from_slice(&data[0..100000]);
    data.sort_unstable();
    (data, keys)
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let data;
    let keys;
    dbg!(&args);
    if args.len() == 2 {
        println!("Generating random data...");
        (data, keys) = random_data_keys();
    } else if args.len() == 4 {
        println!("Loading data...");
        data = load_data(&args[1]);
        keys = load_data(&args[2]);
    } else {
        println!("Usage: {} <path> <path>", args[0]);
        exit(0);
    }
    bench(&data, &keys);
}

fn bench(data: &Vec<u64>, keys: &Vec<u64>) {
    let spline = GreedySplineCorridor::default(data);
    let radix_spline = RadixSpline::default(data);
    let mut binary_total = 0;
    let mut spline_total = 0;
    let mut radix_spline_total = 0;
    for key in keys {
        let start = Instant::now();
        if let Ok(idx) = data.binary_search(key) {
            assert_eq!(&data[idx], key);
        } else {
            panic!("Error when binary search");
        }
        let elapsed = start.elapsed();
        binary_total += elapsed.as_nanos();

        let start = Instant::now();
        if let Some(idx) = spline.search(*key) {
            assert_eq!(&data[idx], key);
        } else {
            panic!("Error when spiline search");
        }
        let elapsed = start.elapsed();
        spline_total += elapsed.as_nanos();

        let start = Instant::now();
        if let Some(idx) = radix_spline.search(*key) {
            assert_eq!(&data[idx], key);
        } else {
            panic!("Error when radix spline searching!");
        }
        let elapsed = start.elapsed();
        radix_spline_total += elapsed.as_nanos();
    }
    println!("Binary Search: {} ns", binary_total / keys.len() as u128);
    println!("Spline Search: {} ns", spline_total / keys.len() as u128);
    println!(
        "Radix Spline Search: {} ns",
        radix_spline_total / keys.len() as u128
    );
}
