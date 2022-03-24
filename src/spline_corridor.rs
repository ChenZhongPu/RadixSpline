//! # Greedy Spline Corridor
//!
//! Neumann, Thomas, and Sebastian Michel. "Smooth interpolating histograms with error guarantees." British National Conference on Databases. Springer, Berlin, Heidelberg, 2008.
//!
//! For simplicity, only `u64` data type is allowed.


#[derive(Clone, Copy, Debug)]
pub struct Point {
    key: u64,        // x
    position: usize, // y
}

impl Point {
    pub fn new(key: u64, position: usize) -> Self {
        Point { key, position }
    }
}

impl PartialEq for Point {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
    }
}

impl PartialOrd for Point {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.key.partial_cmp(&other.key)
    }
}

impl Ord for Point {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.key.cmp(&other.key)
    }
}

impl Eq for Point {}

enum Direction {
    Left,
    Right,
    Coincide,
}

struct Line {
    start: Point,
    end: Point,
}

impl Line {
    fn new(start: Point, end: Point) -> Self {
        Line { start, end }
    }

    fn is_vertical(&self) -> bool {
        self.start.key == self.end.key
    }

    fn get_direction(&self, other: &Line) -> Direction {
        // dy can be less than 0
        let (dy, dx) = (
            self.end.position as f64 - self.start.position as f64,
            self.end.key - self.start.key,
        );
        assert!(dx > 0);

        let (other_dy, other_dx) = (
            other.end.position as f64 - other.start.position as f64,
            other.end.key - other.start.key,
        );
        assert!(other_dx > 0);

        let sin = dy / dx as f64;
        let other_sin = other_dy / other_dx as f64;

        match sin.partial_cmp(&other_sin) {
            Some(std::cmp::Ordering::Equal) => Direction::Coincide,
            Some(std::cmp::Ordering::Greater) => Direction::Left,
            Some(std::cmp::Ordering::Less) => Direction::Right,
            _ => panic!("key is not monotonically increasing"),
        }
    }

    fn is_left(&self, other: &Line) -> bool {
        matches!(self.get_direction(other), Direction::Left)
    }

    fn is_right(&self, other: &Line) -> bool {
        matches!(self.get_direction(other), Direction::Right)
    }
}

/// A greedy method to get spline points.
/// Note that the underlying data should be sorted.
pub struct GreedySplineCorridor<'a> {
    data: &'a Vec<u64>,
    max_error: usize,
    points: Vec<Point>,
}

// to do: how to handle repeated elements?
// There is a bug for repeated elements as sometimes `dx` can be 0
// the assert can fail in this case.
impl<'a> GreedySplineCorridor<'a> {
    pub fn new(data: &'a Vec<u64>, max_error: usize) -> Self {
        GreedySplineCorridor { data, max_error, points: GreedySplineCorridor::spline_points(data, max_error) }
    }

    fn spline_points(data: &Vec<u64>, max_error: usize) -> Vec<Point> {
        assert!(data.len() > 3);

        let mut points = vec![];
        points.push(Point::new(data[0], 0));

        let mut base = Point::new(data[0], 0);

        // error corridor bounds
        let mut upper = Point::new(data[1], 1 + max_error);
        let mut lower = Point::new(data[1], 1usize.saturating_sub(max_error));

        // note `i` starts from `0`.
        for (i, &key) in data[2..].iter().enumerate() {

            let i = i + 2;
            let point_c = Point::new(key, i);

            // line BC (base -> point_c)
            let bc = Line::new(base, point_c);
            // line BU (base -> upper)
            let bu = Line::new(base, upper);
            // line BL (base -> lower)
            let bl = Line::new(base, lower);

            // continue if `bc` or `bu` or `bl`'s `dx` is 0
            // skip the repeated values
            if bc.is_vertical() || bu.is_vertical() || bl.is_vertical() {
                upper = Point::new(point_c.key, i + max_error);
                lower = Point::new(point_c.key, i.saturating_sub(max_error));
                continue;
            }

            if bc.is_left(&bu) || bc.is_right(&bl) {
                base = Point::new(data[i - 1], i - 1);
                points.push(base);

                upper = Point::new(point_c.key, i + max_error);
                lower = Point::new(point_c.key, i.saturating_sub(max_error));
            } else {
                let _upper = Point::new(point_c.key, i + max_error);
                let _lower = Point::new(point_c.key, i.saturating_sub(max_error));

                // line BU' (base -> _upper)
                let _bu = Line::new(base, _upper);
                // line BL' (base -> _lower)
                let _bl = Line::new(base, _lower);
                if bu.is_left(&_bu) {
                    upper = _upper;
                }
                if bl.is_right(&_bl) {
                    lower = _lower;
                }
            }
        }
        let n = data.len();
        points.push(Point::new(data[n - 1], n - 1));
        points
    }

    pub fn search(&self, key: u64) -> Option<usize> {
        let key_point = Point::new(key, 0); // the search position can be arbitrary
        match self.points.binary_search(&key_point) {
            Ok(idx) => Some(self.points[idx].position),
            Err(idx) if idx > 0 => {
                let start = self.points[idx - 1];
                let end = self.points[idx];
                let predicted = start.position as f64 + (key as f64 - start.key as f64) * (end.position as f64 - start.position as f64) / (end.key as f64 - start.key as f64);
                let from = (predicted - self.max_error as f64).ceil() as usize;
                let to = (predicted + self.max_error as f64).floor() as usize;
                // binary search `from` `to` in `data`
                match self.data[from..=to].binary_search(&key) {
                    Ok(p) => Some(p+from),
                    _ => None,
                }
                // how about linear search after predicating?
                // match self.data[from..=to].iter().position(|&x| x == key) {
                //     Some(i) => Some(i + from),
                //     _ => None,
                // } 
            },
            _ => None
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn line_directions() {
        let a = Line::new(Point::new(0, 0), Point::new(1, 2));

        let b = Line::new(Point::new(0, 0), Point::new(2, 2));

        let c = Line::new(Point::new(0, 0), Point::new(3, 2));

        assert!(a.is_left(&b));
        assert!(c.is_right(&b));
    }

    #[test]
    fn spline_points() {
        let data: Vec<u64> = vec![3, 4, 8, 10, 19, 20];

        let spline = GreedySplineCorridor::new(&data, 1);

        assert_eq!(
            vec![Point::new(3, 0), Point::new(10, 3), Point::new(20, 5)],
            spline.points
        );
    }

    #[test]
    fn spline_repeated_points() {
        let data: Vec<u64> = vec![3, 4, 8, 8, 10, 10, 19, 20];

        let spline = GreedySplineCorridor::new(&data, 1);

        assert_eq!(
            vec![Point::new(3, 0), Point::new(10, 5), Point::new(20, 7)],
            spline.points
        );
    }

    #[test]
    fn search() {
        let data: Vec<u64> = vec![3, 4, 8, 8, 10, 10, 19, 20];

        let spline = GreedySplineCorridor::new(&data, 1);
        
        assert_eq!(spline.search(8), Some(3));

        assert_eq!(spline.search(10), Some(4));

        assert_eq!(spline.search(4), Some(1));

        assert_eq!(spline.search(5), None);
    }

    #[test]
    fn large_search() {
        use rand::{distributions::Uniform, Rng};
        use std::time::Instant;

        let range = Uniform::from(0..10000000);
        let mut data: Vec<u64> = rand::thread_rng().sample_iter(&range).take(1000000).collect();

        let value = 10000;
        data.push(value);
        
        data.sort_unstable();
        
        let spline = GreedySplineCorridor::new(&data, 32);

        let start = Instant::now();
        if let Some(idx) = spline.search(value) {
            assert_eq!(data[idx], value);
        }
        let elapsed = start.elapsed();
        println!("Spline using {:?} ns", elapsed.as_nanos());

        let start = Instant::now();
        if let Ok(idx) = data.binary_search(&value) {
            assert_eq!(data[idx], value);
        }
        let elapsed = start.elapsed();
        println!("Binary using {:?} ns", elapsed.as_nanos());
    }
}
