//! # A builder for radix table
//! Note this version is NOT a *single-pass*. And we will optimize it later.

use crate::{spline_corridor::Point, GreedySplineCorridor};
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

/// Build a spline radix table from sorted data.
impl<'a> Builder<'a> {
    pub fn new(data: &'a Vec<u64>, num_radix_bits: usize, max_error: usize) -> Self {
        let min_key = data[0];
        let max_key = data[data.len() - 1];
        assert!(max_key > min_key);

        let shift_radix_bits = get_num_shift_bits(max_key - min_key, num_radix_bits);

        let max_prex = (max_key - min_key) >> shift_radix_bits;
        let mut table: Vec<usize> = vec![0; (max_prex + 2) as usize];

        let spline = GreedySplineCorridor::new(data, max_error);

        Builder::build_table(spline.points(), data, shift_radix_bits, min_key, &mut table);

        Builder {
            min_key,
            max_key,
            num_radix_bits,
            shift_radix_bits,
            max_error,
            data,
            points: spline.points().clone(),
            table,
        }
    }

    /// default `max_radix_bits` is 18, and default `max_error` is 32
    pub fn default(data: &'a Vec<u64>) -> Self {
        Builder::new(data, 18, 32)
    }

    fn build_table(
        points: &Vec<Point>,
        data: &Vec<u64>,
        shift_radix_bits: usize,
        min_key: u64,
        table: &mut Vec<usize>,
    ) {
        let mut last_prefix = 0usize;
        // start from index `1`: `table[0] == 0`
        for &key in &data[1..] {
            let curr_prefix = ((key - min_key) >> shift_radix_bits) as usize;

            // find the position in `points`
            // <this step can be speeded up (omitted) in *single-pass* version>
            let point = Point::new(key, 0); // the position can be arbitrary
            let idx = match points.binary_search(&point) {
                Ok(idx) => idx,
                Err(idx) => idx - 1,
            };

            // from `last_prefix + 1` (inclusive) to `curr_prefix` (inclusive)
            // the value (pointer) is `idx`
            for i in (last_prefix + 1)..=curr_prefix {
                table[i] = idx;
            }

            last_prefix = curr_prefix;
        }
    }

    pub fn search(&self, key: u64) -> Option<usize> {
        let curr_prefix = ((key - self.min_key) >> self.shift_radix_bits) as usize;

        let start = self.points[self.table[curr_prefix]];
        if start.key == key {
            return Some(start.position);
        }
        let end = self.points[self.table[curr_prefix] + 1];

        let predicted = start.position + (key as usize - start.key as usize) * (end.position - start.position) / (end.key as usize - start.key as usize);

        let from = predicted.saturating_sub(self.max_error);
        let to = if predicted + self.max_error > self.data.len() - 1 {
            self.data.len() - 1
        } else {
            predicted + self.max_error
        };

        // binary search `from` `to` in `data`
        match self.data[from..=to].binary_search(&key) {
            Ok(p) => Some(p + from),
            _ => None,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn search() {
        use rand::{distributions::Uniform, Rng};
        use std::time::Instant;

        let range = Uniform::from(0..10000000);
        let mut data: Vec<u64> = rand::thread_rng()
            .sample_iter(&range)
            .take(1000000)
            .collect();

        let value = 20;
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
}
