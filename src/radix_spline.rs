//! # A builder for radix spline index
//! Building the `spline points` and `radix table` in **one-pass**.

use crate::common::Line;
use crate::common::Point;

/// `RadixSpline` builds an index for sorted data (assuming `u64`).
/// Given a `key`, we compute it by `shift_radix_bits` -> the index of `table`. And the value of `table` is a pointer, indicting the position of `points`. `points` is an error-bounded spline by interpolating, and it can be used to predict the position of `key`.
pub struct RadixSpline<'a> {
    data: &'a Vec<u64>, // sorted data
    min_key: u64,
    shift_radix_bits: u32, // it is computed from `num_radix_bits`
    max_error: usize,      // max error bound
    points: Vec<Point>,    // spline points
    table: Vec<usize>,     // radix table
}

fn get_num_shift_bits(diff: u64, num_radix_bits: u32) -> u32 {
    let zeros = diff.leading_zeros();
    // note all keys here `u64`.
    if 64 - zeros < num_radix_bits {
        0
    } else {
        64 - num_radix_bits - zeros
    }
}

impl<'a> RadixSpline<'a> {
    /// `data` is sorted, whose size is at least 3.
    pub fn new(data: &'a Vec<u64>, num_radix_bits: u32, max_error: usize) -> Self {
        assert!(data.len() >= 3);
        let min_key = data[0];
        let max_key = data[data.len() - 1];

        let shift_radix_bits = get_num_shift_bits(max_key - min_key, num_radix_bits);

        let max_prefix = (max_key - min_key) >> shift_radix_bits;
        let mut table = vec![0 as usize; (max_prefix + 2) as usize];

        let mut points: Vec<Point> = vec![];

        // build `points` and `table`
        RadixSpline::build(
            &mut points,
            &mut table,
            data,
            min_key,
            shift_radix_bits,
            max_error,
        );

        RadixSpline {
            data,
            min_key,
            shift_radix_bits,
            max_error,
            points,
            table,
        }
    }

    fn build_table(data: &[u64], table: &mut Vec<usize>, min_key: u64, point_key: u64, points_size: usize, last_prefix: usize, shift_radix_bits: u32) -> usize {
        let mut last_prefix = last_prefix;
        for &value in data {
            let curr_prefix = ((value - min_key) >> shift_radix_bits) as usize;
            // get the `position` of `data[idx]`
            // only need to compare with the last element in `points` (i.e., `c_base`)
            let position = if value == point_key {
                points_size - 1
            } else {
                points_size - 2
            };
            for i in last_prefix + 1..=curr_prefix {
                table[i] = position;
            }
            last_prefix = curr_prefix;
        }
        last_prefix
    }

    fn build(
        points: &mut Vec<Point>,
        table: &mut Vec<usize>,
        data: &Vec<u64>,
        min_key: u64,
        shift_radix_bits: u32,
        max_error: usize,
    ) {
        points.push(Point::new(data[0], 0));

        // let mut p_base;
        let mut c_base = Point::new(data[0], 0);

        // error corridor bounds
        let mut upper = Point::new(data[1], 1 + max_error);
        let mut lower = Point::new(data[1], 1usize.saturating_sub(max_error));

        let mut last_index = 1; // `index` of data
        let mut last_prefix = 0usize; // after shifting

        // note `i` starts from `0`
        for (i, &key) in data[2..].iter().enumerate() {
            let i = i + 2;
            let point_c = Point::new(key, i);

            // line BC (base -> point_c)
            let bc = Line::new(c_base, point_c);
            // line BU (base -> upper)
            let bu = Line::new(c_base, upper);
            // line BL (base -> lower)
            let bl = Line::new(c_base, lower);

            // continue if `bc` or `bu` or `bl`'s `dx` is 0
            // skip the repeated values
            if bc.is_vertical() || bu.is_vertical() || bl.is_vertical() {
                upper = Point::new(point_c.key(), i + max_error);
                lower = Point::new(point_c.key(), i.saturating_sub(max_error));
                continue;
            }

            if bc.is_left(&bu) || bc.is_right(&bl) {
                c_base = Point::new(data[i - 1], i - 1);
                points.push(c_base);

                upper = Point::new(point_c.key(), i + max_error);
                lower = Point::new(point_c.key(), i.saturating_sub(max_error));

                // update table
                last_prefix = RadixSpline::build_table(&data[last_index..i], table, min_key, c_base.key(), points.len(), last_prefix, shift_radix_bits);
                last_index = i;
            } else {
                let _upper = Point::new(point_c.key(), i + max_error);
                let _lower = Point::new(point_c.key(), i.saturating_sub(max_error));

                // line BU' (base -> _upper)
                let _bu = Line::new(c_base, _upper);
                // line BL' (base -> _lower)
                let _bl = Line::new(c_base, _lower);
                if bu.is_left(&_bu) {
                    upper = _upper;
                }
                if bl.is_right(&_bl) {
                    lower = _lower;
                }
            }
        } // end of for

        // p_base = c_base;
        let n = data.len();
        points.push(Point::new(data[n - 1], n - 1));

        // update table
        RadixSpline::build_table(&data[last_index..n], table, min_key, data[n-1], points.len(), last_prefix, shift_radix_bits);
    }

    /// default `max_radix_bits` is 18, and default `max_error` is 32
    pub fn default(data: &'a Vec<u64>) -> Self {
        RadixSpline::new(data, 18, 32)
    }

    /// search a given `key`
    pub fn search(&self, key: u64) -> Option<usize> {
        let c_prefix = ((key - self.min_key) >> self.shift_radix_bits) as usize;

        let start = self.points[self.table[c_prefix]];

        if start.key() == key {
            return Some(start.position());
        }
        let end = self.points[self.table[c_prefix] + 1];

        // no need to use `f64` as `usize` is faster.
        // it is fine to always lose the precision.
        let predicted = start.position()
            + (key as usize - start.key() as usize) * (end.position() - start.position())
                / (end.key() as usize - start.key() as usize);

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

        let range = Uniform::from(0..10000000);
        let mut data: Vec<u64> = rand::thread_rng()
            .sample_iter(&range)
            .take(1000000)
            .collect();

        let value = 2000;
        data.push(value);

        data.sort_unstable();

        let radix_spline = RadixSpline::default(&data);

        match radix_spline.search(value) {
            Some(idx) => assert_eq!(data[idx], value),
            None => panic!("Error when searching!"),
        }
    }
}
