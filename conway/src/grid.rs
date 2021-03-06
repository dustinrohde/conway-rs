use std::collections::HashSet;
use std::str::FromStr;

pub use point::Point;
use {Error, ErrorKind, Result};

pub const READ_CHAR_ALIVE: char = 'x';
pub const READ_CHAR_DEAD: char = '.';
pub const COMMENT_CHAR: char = '#';

static DIRECTIONS: &'static [(i64, i64)] = &[
    (-1, -1),
    (-1, 0),
    (-1, 1),
    (0, -1),
    (0, 1),
    (1, -1),
    (1, 0),
    (1, 1),
];

/// A Grid represents the physical world in which Conway's Game of Life takes place.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Grid {
    cells: HashSet<Point>,
}

impl Grid {
    /// Create a new Grid with the given cells.
    pub fn new<I: IntoIterator<Item = Point>>(cells: I) -> Self {
        Grid {
            cells: cells.into_iter().collect(),
        }
    }

    /// Create an empty Grid.
    pub fn empty() -> Self {
        Grid {
            cells: HashSet::new(),
        }
    }

    /*
     * Points
     */

    /// Return the number of living Cells that are adjacent to the given Point.
    pub fn live_neighbors(&self, point: &Point) -> usize {
        self.adjacent_cells(point)
            .iter()
            .filter(|c| self.is_alive(c))
            .count()
    }

    /// Return the set of all Cells that should be evaluated for survival.
    pub fn active_cells(&self) -> HashSet<Point> {
        self.cells
            .iter()
            .flat_map(|cell| {
                let mut cells = self.adjacent_cells(cell);
                cells.insert(*cell);
                cells
            })
            .collect()
    }

    /// Return all 8 Points that are directly adjacent to the given Point.
    pub fn adjacent_cells(&self, cell: &Point) -> HashSet<Point> {
        let Point(x, y) = cell;
        DIRECTIONS
            .iter()
            .map(|(dx, dy)| Point(x + dx, y + dy))
            .collect()
    }

    /// Return whether the Grid is empty.
    pub fn is_empty(&self) -> bool {
        self.cells.is_empty()
    }

    /// Return whether the cell at the given Point is alive.
    pub fn is_alive(&self, cell: &Point) -> bool {
        self.cells.contains(cell)
    }

    /// Bring the cell at the given Point to life.
    pub fn set_alive(&mut self, cell: Point) -> bool {
        self.cells.insert(cell)
    }

    /// Kill the cell at the given Point.
    pub fn set_dead(&mut self, cell: &Point) -> bool {
        self.cells.remove(cell)
    }

    /// Clear the Grid of all living cells.
    pub fn clear(&mut self) {
        self.cells.clear()
    }

    /*
     * Geometry
     */

    /// Return the Point closest to the center of the Grid.
    pub fn midpoint(&self) -> Point {
        let (Point(x0, y0), Point(x1, y1)) = self.bounds();
        Point((x0 + x1 + 1) / 2, (y0 + y1 + 1) / 2)
    }

    // Return the lowest and highest X and Y coordinates represented in the Grid.
    pub fn bounds(&self) -> (Point, Point) {
        let mut cells = self.cells.iter();
        if let Some(&Point(x, y)) = cells.next() {
            let ((mut x0, mut y0), (mut x1, mut y1)) = ((x, y), (x, y));
            for &Point(x, y) in cells {
                if x < x0 {
                    x0 = x;
                } else if x > x1 {
                    x1 = x;
                }
                if y < y0 {
                    y0 = y;
                } else if y > y1 {
                    y1 = y;
                }
            }
            (Point(x0, y0), Point(x1, y1))
        } else {
            (Point::origin(), Point::origin())
        }
    }
}

/// Parse a Grid from a block of structured text.
impl FromStr for Grid {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        let mut cells = Vec::new();

        for (y, line) in s
            .trim()
            .lines()
            .filter(|line| !line.starts_with(COMMENT_CHAR))
            .enumerate()
        {
            for (x, ch) in line.chars().enumerate() {
                match ch {
                    // Living Points are added to the Grid.
                    READ_CHAR_ALIVE => cells.push(Point(x as i64, y as i64)),
                    // Dead Points are ignored.
                    READ_CHAR_DEAD => (),
                    // Skip the rest of the line after a comment char.
                    COMMENT_CHAR => break,
                    // Anything else is invalid.
                    _ => bail!(ErrorKind::ParseGrid(format!("unknown character: '{}'", ch))),
                };
            }
        }

        Ok(Grid::new(cells))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::default::Default;

    mod constructors {
        use super::*;

        #[test]
        fn test_from_str() {
            // Should look like this after comments are removed:
            // 110
            // 001
            // 010
            let grid: Grid = vec![
                " \t".to_string(), // Leading whitespace should be removed.
                format!( // Everything after an inline comment should be ignored.
                    "{}{}{}{}",
                    READ_CHAR_ALIVE, READ_CHAR_ALIVE, COMMENT_CHAR, READ_CHAR_ALIVE
                ),
                format!("{}{}{}", READ_CHAR_DEAD, READ_CHAR_DEAD, READ_CHAR_ALIVE),
                format!("{}hello world", COMMENT_CHAR), // Line comments should be skipped.
                format!("{}{}{}", READ_CHAR_DEAD, READ_CHAR_ALIVE, READ_CHAR_DEAD),
                "\n\t \n".to_string(), // Trailing whitespace should be ignored.
            ]
            .join("\n")
            .parse()
            .unwrap();

            assert_eq!(
                grid.cells,
                hashset![Point(0, 0), Point(1, 0), Point(2, 1), Point(1, 2)],
            );
            assert!(Grid::from_str("abc\ndef").is_err())
        }
    }

    mod cells {
        use super::*;

        #[test]
        fn test_active_cells() {
            let grid = Grid::new(vec![Point(0, 0), Point(1, 1)]);
            assert_eq!(
                grid.active_cells(),
                hashset![
                    Point(0, 0),
                    Point(-1, -1),
                    Point(0, -1),
                    Point(1, -1),
                    Point(1, 0),
                    Point(1, 1),
                    Point(0, 1),
                    Point(-1, 1),
                    Point(-1, 0),
                    Point(2, 0),
                    Point(2, 1),
                    Point(2, 2),
                    Point(1, 2),
                    Point(0, 2),
                ]
            )
        }

        #[test]
        fn test_live_neighbors() {
            let grid = Grid::new(vec![Point(-1, -1), Point(-1, -2), Point(0, 0), Point(1, 0)]);
            assert_eq!(
                grid.live_neighbors(&Point(0, 0)),
                2,
                "it should work for a live cell"
            );
            assert_eq!(
                grid.live_neighbors(&Point(-1, -3)),
                1,
                "it should work for a dead cell"
            )
        }

        #[test]
        fn test_is_empty() {
            let grid: Grid = Default::default();
            assert!(grid.is_empty());
            let grid = Grid::new(vec![Point(0, 0)]);
            assert!(!grid.is_empty());
        }

        #[test]
        fn test_is_alive() {
            let grid = Grid::new(vec![Point(-1, 4), Point(8, 8)]);
            assert!(&grid.is_alive(&Point(-1, 4)));
            assert!(&grid.is_alive(&Point(8, 8)));
            assert!(!&grid.is_alive(&Point(8, 4)));
        }

        #[test]
        fn test_set_alive_or_dead() {
            let mut grid: Grid = Default::default();
            let cell = Point(3, -3);
            assert!(!&grid.is_alive(&cell));
            grid.set_alive(cell);
            assert!(&grid.is_alive(&cell));
            grid.set_dead(&cell);
            assert!(!&grid.is_alive(&cell));
        }
    }

    mod geometry {
        use super::*;

        #[test]
        fn test_midpoint() {
            assert_eq!(
                Grid::new(vec![Point(-2, -1), Point(2, 1)]).midpoint(),
                Point(0, 0),
            );
            assert_eq!(
                Grid::new(vec![Point(0, -6), Point(8, 2)]).midpoint(),
                Point(4, -1),
            );
            assert_eq!(
                Grid::new(vec![Point(2, 7), Point(-5, 6)]).midpoint(),
                Point(-1, 7),
            );
        }

        #[test]
        fn test_bounds() {
            assert_eq!(
                Grid::new(vec![Point(2, 1), Point(-3, 0), Point(-2, 1), Point(-2, 0)]).bounds(),
                (Point(-3, 0), Point(2, 1))
            );
            assert_eq!(
                Grid::new(vec![Point(53, 4), Point(2, 1), Point(-12, 33)]).bounds(),
                (Point(-12, 1), Point(53, 33))
            );
        }
    }
}
