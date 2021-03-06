use std::fmt;
use std::mem;
use std::str::FromStr;
use std::thread;

use num_integer::Integer;

pub use config::Settings;
use grid::{Grid, Point};
use {Error, Result};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum View {
    Centered,
    Fixed,
    Follow,
}

impl FromStr for View {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "centered" => Ok(View::Centered),
            "fixed" => Ok(View::Fixed),
            "follow" => Ok(View::Follow),
            s => bail!("invalid value for view '{}'", s),
        }
    }
}

impl fmt::Display for View {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                View::Centered => "centered",
                View::Fixed => "fixed",
                View::Follow => "follow",
            }
        )
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Viewport {
    pub origin: Point,
    pub scroll: Point,
    pub width: u64,
    pub height: u64,
}

impl Viewport {
    /// Return bounds starting at the lowest x and y values of live Cells present in the Game.
    pub fn bounds(&self) -> (Point, Point) {
        let Point(x0, y0) = self.origin + self.scroll;
        (
            Point(x0, y0),
            Point(x0 + self.width as i64 - 1, y0 + self.height as i64 - 1),
        )
    }

    /// Return bounds centered around existing live Cells.
    pub fn centered(&self, Point(x, y): Point) -> (Point, Point) {
        let (dx0, dx1) = split_int(self.width as i64);
        let (dy0, dy1) = split_int(self.height as i64);
        (Point(x - dx0, y - dy0), Point(x + dx1 - 1, y + dy1 - 1))
    }

    pub fn center(&mut self, point: Point) {
        let (origin, _) = self.centered(point);
        self.scroll = origin - self.origin;
    }

    pub fn scroll(&mut self, dx: i64, dy: i64) {
        self.scroll += Point(dx, dy);
    }
}

pub struct GameIter<'a> {
    game: &'a mut Game,
    with_delay: bool,
}

impl<'a> GameIter<'a> {
    pub fn with_delay(mut self, with_delay: bool) -> Self {
        self.with_delay = with_delay;
        self
    }
}

impl<'a> Iterator for GameIter<'a> {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        if self.game.is_over() {
            None
        } else {
            if self.with_delay {
                self.game.tick_with_delay();
            } else {
                self.game.tick();
            }
            Some(self.game.draw())
        }
    }
}

/// Game holds the high-level gameplay logic.
#[derive(Debug, Clone)]
pub struct Game {
    grid: Grid,
    swap: Grid,
    pub opts: Settings,
    pub viewport: Viewport,
}

impl Game {
    pub fn new(grid: Grid, opts: Settings, bounds: (Option<u64>, Option<u64>)) -> Game {
        let swap = Grid::empty();

        let (origin, Point(x1, y1)) = grid.bounds();
        let (width, height) = bounds;

        let viewport = Viewport {
            origin,
            width: width.unwrap_or((x1 - origin.0 + 1) as u64),
            height: height.unwrap_or((y1 - origin.1 + 1) as u64),
            scroll: Point::origin(),
        };

        let mut game = Game {
            grid,
            swap,
            opts,
            viewport,
        };
        game.viewport.center(game.grid.midpoint());
        game
    }

    /// Return an iterator over the turns of the Game, returning the rendered output of each turn.
    ///
    /// Iteration finishes when the Game is over.
    pub fn iter(&mut self) -> GameIter {
        GameIter {
            game: self,
            with_delay: false,
        }
    }

    /// Execute the next turn in the Game of Life.
    ///
    /// Applies the rules of game to each individual cell, killing some and reviving others.
    pub fn tick(&mut self) {
        for cell in self.grid.active_cells() {
            if self.survives(&cell) {
                self.swap.set_alive(cell);
            }
        }
        self.grid.clear();
        mem::swap(&mut self.grid, &mut self.swap);
    }

    /// Call `tick`, then sleep for `self.opts.delay`.
    pub fn tick_with_delay(&mut self) {
        thread::sleep(self.opts.delay);
        self.tick();
    }

    /// Render the Game as a grid of cells.
    pub fn draw(&self) -> String {
        self.draw_viewport(self.viewport())
    }

    /// Render the section of the Game, within the given bounds, as a grid of cells.
    fn draw_viewport(&self, (Point(x0, y0), Point(x1, y1)): (Point, Point)) -> String {
        let mut output = String::new();
        for y in y0..=y1 {
            for x in x0..=x1 {
                output.push(if self.grid.is_alive(&Point(x, y)) {
                    self.opts.char_alive
                } else {
                    self.opts.char_dead
                });
            }
            output.push('\n');
        }
        output
    }

    /// Shift the Game's viewport by the given x and y deltas.
    pub fn scroll(&mut self, dx: i64, dy: i64) {
        self.viewport.scroll += Point(dx, dy);
    }

    /// Move the Game's viewport to the center of the grid.
    pub fn center_viewport(&mut self) {
        self.viewport.center(self.grid.midpoint());
    }

    /// Calculate and return the lower and upper bounds of the Game's viewport.
    pub fn viewport(&self) -> (Point, Point) {
        match &self.opts.view {
            View::Fixed => self.viewport.bounds(),
            View::Centered => self.viewport.centered(self.grid.midpoint()),
            _ => unimplemented!(),
        }
    }

    /// Return whether the Game is over (true if the grid is empty, else false).
    // TODO: make this `is_stablized` and increase functionality.
    pub fn is_over(&self) -> bool {
        self.grid.is_empty()
    }

    /// Return whether the cell at the given Point survives an application of The Rules.
    pub fn survives(&self, cell: &Point) -> bool {
        let live_neighbors = self.grid.live_neighbors(cell);
        if self.grid.is_alive(cell) {
            match live_neighbors {
                2 | 3 => true,
                _ => false,
            }
        } else {
            match live_neighbors {
                3 => true,
                _ => false,
            }
        }
    }
}

// Split an integer into 2 halves that always add up to the given number.
fn split_int<T: Integer + Copy>(n: T) -> (T, T) {
    let two = T::one() + T::one();
    let (quotient, remainder) = n.div_rem(&two);
    (quotient, quotient + remainder)
}

#[cfg(test)]
mod test {
    use super::*;

    fn mk_game(cells: Vec<Point>, bounds: (Option<u64>, Option<u64>)) -> Game {
        Game::new(Grid::new(cells), Settings::default(), bounds)
    }

    // Viewport width/height should be taken from Settings if given.
    #[test]
    fn test_size_provided() {
        let game = mk_game(vec![Point::origin(), Point(5, 5)], (Some(8), Some(8)));
        assert_eq!(game.viewport.width, 8);
        assert_eq!(game.viewport.height, 8);
    }

    // Viewport width/height should be derived from the Grid if not given in Settings.
    #[test]
    fn test_size_auto() {
        let game = mk_game(vec![Point::origin(), Point(5, 5)], (None, None));
        assert_eq!(game.viewport.width, 6);
        assert_eq!(game.viewport.height, 6);
    }

    // Test `Game.survive`.
    #[test]
    fn test_survives() {
        let game = mk_game(vec![Point(1, 0), Point(1, 1), Point(1, 2)], (None, None));
        assert!(
            game.survives(&Point(1, 1)),
            "a live cell with 2 live neighbors should survive"
        );
        assert!(
            game.survives(&Point(0, 1)),
            "a dead cell with 3 live neighbors should survive"
        );
        assert!(
            game.survives(&Point(2, 1)),
            "a dead cell with 3 live neighbors should survive"
        );
        assert!(
            !game.survives(&Point(1, 0)),
            "a live cell with < 2 live neighbors should die"
        );
        assert!(
            !game.survives(&Point(1, 2)),
            "a live cell with < 2 live neighbors should die"
        );
    }

    // Tests for `Game.viewport` and related functionality.
    mod viewport {
        use super::*;

        // Test `Viewport.fixed`.
        #[test]
        fn test_fixed_1() {
            assert_eq!(
                Viewport {
                    origin: Point(-3, 0),
                    width: 7,
                    height: 7,
                    scroll: Point::origin(),
                }
                .bounds(),
                (Point(-3, 0), Point(3, 6)),
                "should pad content to fit width/height"
            );
        }

        // ...
        #[test]
        fn test_fixed_2() {
            assert_eq!(
                Viewport {
                    origin: Point(-12, 1),
                    width: 88,
                    height: 12,
                    scroll: Point::origin(),
                }
                .bounds(),
                (Point(-12, 1), Point(75, 12))
            );
        }

        // ...
        #[test]
        fn test_fixed_3() {
            assert_eq!(
                Viewport {
                    origin: Point(2, 2),
                    width: 10,
                    height: 3,
                    scroll: Point::origin(),
                }
                .bounds(),
                (Point(2, 2), Point(11, 4)),
            );
        }

        // Test that `Viewport.fixed` adjusts for scroll.
        #[test]
        fn test_viewport_fixed_with_scroll() {
            assert_eq!(
                Viewport {
                    origin: Point(2, 2),
                    width: 10,
                    height: 3,
                    scroll: Point(1, -5),
                }
                .bounds(),
                (Point(3, -3), Point(12, -1))
            );
        }

        // Test `Viewport.centered`.
        #[test]
        fn test_centered_1() {
            assert_eq!(
                Viewport {
                    width: 7,
                    height: 7,
                    ..Default::default()
                }
                .centered(Point(0, 1)),
                (Point(-3, -2), Point(3, 4)),
                "should expand to fit width/height"
            );
        }

        // ...
        #[test]
        fn test_centered_2() {
            assert_eq!(
                Viewport {
                    width: 88,
                    height: 12,
                    ..Default::default()
                }
                .centered(Point(21, 17)),
                (Point(-23, 11), Point(64, 22)),
                "should narrow to fit width/height"
            );
        }

        // ...
        #[test]
        fn test_centered_3() {
            assert_eq!(
                Viewport {
                    width: 10,
                    height: 3,
                    ..Default::default()
                }
                .centered(Point(4, 3)),
                (Point(-1, 2), Point(8, 4)),
            );
        }

        // Test that `Viewport.centered` ignores current scroll.
        #[test]
        fn test_centered_with_scroll() {
            assert_eq!(
                Viewport {
                    width: 10,
                    height: 3,
                    scroll: Point(1, -5),
                    ..Default::default()
                }
                .centered(Point(4, 3)),
                (Point(-1, 2), Point(8, 4))
            );
        }

        // Test `Game.scroll`.
        #[test]
        fn test_scroll() {
            let mut game = mk_game(vec![Point(3, 0), Point(-1, 1), Point(0, -3)], (None, None));
            game.viewport.scroll = Point::origin();
            assert_eq!(game.viewport.bounds(), (Point(-1, -3), Point(3, 1)));
            game.scroll(2, -4);
            assert_eq!(game.viewport.bounds(), (Point(1, -7), Point(5, -3)));
        }

        // Test `Game.center_viewport`.
        #[test]
        fn test_center_viewport() {
            let mut game = mk_game(
                vec![Point(2, 3), Point(3, 3), Point(5, 4), Point(4, 2)],
                (Some(10), Some(3)),
            );
            let expected = game.viewport.centered(game.grid.midpoint());
            game.center_viewport();
            assert_eq!(game.viewport.bounds(), expected);
        }

        // `Game.center_viewport` should account for current scroll.
        #[test]
        fn test_center_viewport_with_scroll() {
            let mut game = mk_game(
                vec![Point(2, 3), Point(3, 3), Point(5, 4), Point(4, 2)],
                (Some(10), Some(3)),
            );
            game.scroll(-1, 2);
            let expected = game.viewport.centered(game.grid.midpoint());
            game.center_viewport();
            assert_eq!(game.viewport.bounds(), expected);
        }
    }

    // Test `split_int`.
    #[test]
    fn test_split_int() {
        assert_eq!(split_int(30), (15, 15));
        assert_eq!(split_int(31), (15, 16));
        assert_eq!(split_int(32), (16, 16));
        assert_eq!(split_int(0), (0, 0));
        assert_eq!(split_int(1), (0, 1));
        assert_eq!(split_int(2), (1, 1));
    }
}
