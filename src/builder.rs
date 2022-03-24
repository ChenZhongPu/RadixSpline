//! # A builder for radix table
//! Note this version is NOT a *single-pass*. And we will optimize it later.

use crate::{GreedySplineCorridor, spline_corridor::Point};
pub struct Builder<'a> {
    min_key: u64,
    max_key: u64,
    num_radix_bits: usize,
    shift_radix_bits: usize,
    max_error: usize,
    data: &'a Vec<u64>,
    points: Vec<Point>,
    table: Vec<usize>,
}

fn get_num_shift_bits(diff: u64, num_radix_bits: usize) -> usize {
    let zeros = diff.leading_zeros() as usize;
    // note all keys here `u64`.
    if 64 - zeros < num_radix_bits {
        0
    } else {
        64 - num_radix_bits - zeros
    }
}

impl<'a> Builder<'a> {
    pub fn new(data: &'a Vec<u64>, num_radix_bits: usize, max_error: usize) -> Self {
        let min_key = data[0];
        let max_key = data[data.len() - 1];
        assert!(max_key > min_key);

        let shift_radix_bits = get_num_shift_bits(max_key - min_key, num_radix_bits);

        let max_prex = (max_key - min_key) >> shift_radix_bits;
        let mut table: Vec<usize> = vec![0; (max_prex + 2) as usize];

        let spline = GreedySplineCorridor::new(data, max_error);

        Builder::build_table(spline.points(), data, num_radix_bits, min_key, &mut table);

        Builder { min_key, max_key, num_radix_bits, shift_radix_bits, max_error, data, points: spline.points().clone(), table }

    }

    /// default `max_radix_bits` is 18, and default `max_error` is 32
    pub fn default(data: &'a Vec<u64>) -> Self {
        Builder::new(data, 18, 32)
    }

    fn build_table(points: &Vec<Point>, data: &Vec<u64>, num_radix_bits: usize, min_key: u64, table: &mut Vec<usize>) {
        let last_prefix = 0usize;
        // start from index `1`
        for &key in &data[1..] {
            let curr_prefix = ((key - min_key) >> num_radix_bits) as usize;
            
            // find the position in `points`
            // <this step can be speeded up (omitted) in *single-pass* version>
            let point = Point::new(key, 0); // the position can be arbitrary
            let idx = match points.binary_search(&point) {
                Ok(idx) => idx,
                Err(idx) => idx - 1,
            };
            
            // from `last_prefix + 1` to `curr_prefix`
            // the value (pointer) is `idx`
            for i in (last_prefix + 1)..=curr_prefix {
                table[i] = idx;
            }
        }
    }

    pub fn search(&self, key: u64) -> Option<usize> {
        let curr_prefix = ((key - self.min_key) >> self.num_radix_bits) as usize;

        let start = self.points[self.table[curr_prefix]];
        let end = self.points[self.table[curr_prefix] + 1];
        
        let predicted = start.position as f64 + (key as f64 - start.key as f64) * (end.position as f64 - start.position as f64) / (end.key as f64 - start.key as f64);
        let from = (predicted - self.max_error as f64).ceil() as usize;
        let to = (predicted + self.max_error as f64).floor() as usize;

        // binary search `from` `to` in `data`
        match self.data[from..=to].binary_search(&key) {
            Ok(p) => Some(p+from),
            _ => None,
        }
    }
}