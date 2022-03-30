//! # Common data type: `Point` and `Line`

/// `x` is the *key* (assuming it is always `u64); `y` is the *position*.
/// Note data\[y\] == x holds.
/// When it is compared, only *key* is involved.
#[derive(Clone, Copy, Debug, Default)]
pub struct Point {
    key: u64, // x
    position: usize, // y
}

impl Point {
    pub fn new(key: u64, position: usize) -> Self {
        Point {key, position }
    }

    pub fn key(&self) -> u64 {
        self.key
    }

    pub fn position(&self) -> usize {
        self.position
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

/// How are two lines (with the same starting point) related?
pub enum Direction {
    Left,
    Right,
    Coincide,
}

/// A line connecting `start` and `end` point
pub struct Line {
    start: Point,
    end: Point,
}

impl Line {
    pub fn new(start: Point, end: Point) -> Self {
        Line { start, end }
    }

    pub fn is_vertical(&self) -> bool {
        self.start.key == self.end.key
    }

    /// Note that it is applied when two lines have the same starting point.
    fn get_direction(&self, other: &Line) -> Direction {
        // dy can be less than 0
        let (dy, dx) = (
            self.end.position as i64 - self.start.position as i64,
            self.end.key - self.start.key,
        );
        assert!(dx > 0);

        let (other_dy, other_dx) = (
            other.end.position as i64 - other.start.position as i64,
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

    pub fn is_left(&self, other: &Line) -> bool {
        matches!(self.get_direction(other), Direction::Left)
    }

    pub fn is_right(&self, other: &Line) -> bool {
        matches!(self.get_direction(other), Direction::Right)
    }
}