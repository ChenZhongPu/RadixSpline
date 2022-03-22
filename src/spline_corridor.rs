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

    fn get_direction(&self, other: &Line) -> Direction {
        let (dy, dx) = (
            self.end.position - self.start.position,
            self.end.key - self.start.key,
        );
        assert!(dx > 0);

        let (other_dy, other_dx) = (
            other.end.position - other.start.position,
            other.end.key - other.start.key,
        );
        assert!(other_dx > 0);

        let sin = dy as f64 / dx as f64;
        let other_sin = other_dy as f64 / other_dx as f64;

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
}

// to do: how to handle repeated elements?
impl<'a> GreedySplineCorridor<'a> {
    pub fn new(data: &'a Vec<u64>, max_error: usize) -> Self {
        GreedySplineCorridor { data, max_error }
    }

    fn spline_points(&self) -> Vec<Point> {
        assert!(self.data.len() > 3);

        let mut points = vec![];
        points.push(Point::new(self.data[0], 0));

        let mut base = Point::new(self.data[0], 0);

        // skip the repeated data
        // `idx` is the first index differs from `self.data[0]`
        let mut idx: usize = 1;
        while idx < self.data.len() && self.data[idx] == self.data[0] {
            idx += 1;
        }
        // error corridor bounds
        let mut upper = Point::new(self.data[idx], idx + self.max_error);
        let mut lower = Point::new(self.data[idx], idx.saturating_sub(self.max_error));

        // note `i` starts from `0`.
        for (i, &key) in self.data[idx+1..].iter().enumerate() {
            // skip the repeated data
            if key == upper.key || key == lower.key {
                continue;
            }

            let i = i + idx + 1;
            let point_c = Point::new(key, i);

            // line BC (base -> point_c)
            let bc = Line::new(base, point_c);
            // line BU (base -> upper)
            let bu = Line::new(base, upper);
            // line BL (base -> lower)
            let bl = Line::new(base, lower);

            if bc.is_left(&bu) || bc.is_right(&bl) {
                base = Point::new(self.data[i - 1], i - 1);
                points.push(base);

                upper = Point::new(point_c.key, i + self.max_error);
                lower = Point::new(point_c.key, i.saturating_sub(self.max_error));
            } else {
                let _upper = Point::new(point_c.key, i + self.max_error);
                let _lower = Point::new(point_c.key, i.saturating_sub(self.max_error));

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
        let n = self.data.len();
        points.push(Point::new(self.data[n - 1], n - 1));
        points
    }

    pub fn search(&self, key: u64) -> Option<usize> {
        let key_point = Point::new(key, 0); // the search position can be arbitrary
        let points = self.spline_points(); // to do: `points` should be stored
        match points.binary_search(&key_point) {
            Ok(idx) => Some(points[idx].position),
            Err(idx) if idx > 0 => {
                let start = points[idx - 1];
                let end = points[idx];
                let predicted = start.position as f64 + (key as f64 - start.key as f64) * (end.position as f64 - start.position as f64) / (end.key as f64 - start.key as f64);
                let from = (predicted - self.max_error as f64).ceil() as usize;
                let to = (predicted + self.max_error as f64).floor() as usize;
                // binary search `from` `to` for `data`
                match self.data[from..=to].binary_search(&key) {
                    Ok(p) => Some(p+from),
                    _ => None,
                }
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
            spline.spline_points()
        );
    }

    #[test]
    fn spline_repeated_points() {
        let data: Vec<u64> = vec![3, 4, 8, 8, 10, 10, 19, 20];

        let spline = GreedySplineCorridor::new(&data, 1);

        assert_eq!(
            vec![Point::new(3, 0), Point::new(10, 5), Point::new(20, 7)],
            spline.spline_points()
        );
    }

    #[test]
    fn search() {
        let data: Vec<u64> = vec![3, 4, 8, 8, 10, 10, 19, 20];

        let spline = GreedySplineCorridor::new(&data, 1);
        
        println!("{:?}", spline.search(8));

        println!("{:?}", spline.search(10));

        println!("{:?}", spline.search(4));

        println!("{:?}", spline.search(5));
        
    }
}
