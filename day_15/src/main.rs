use itertools::iproduct;
use priority_queue::PriorityQueue;
use std::cmp::Reverse;
use std::collections::{HashMap, HashSet};
use std::env;
use std::hash::Hash;
use std::io::{stdin, BufRead};
use std::process::exit;
use termion::{color, style};
use thiserror::Error;

#[derive(Debug)]
enum QuestionPart {
    One,
    Two,
}

#[derive(Error, Debug)]
pub enum AdventError {
    #[error("Invalid command `{command:?}'. Expected `part-one' or `part-two'.")]
    InvalidCommand { command: String },

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error("Please specify `part-one' or `part-two' as the first argument.")]
    NoPartArgument,

    #[error("Could not parse char `{c}' to numeric digit.")]
    Parse { c: char },
}

#[derive(Clone, PartialEq, Eq, Hash, Copy, Debug)]
struct Point {
    x: usize,
    y: usize,
}

impl Point {
    fn new(x: usize, y: usize) -> Point {
        Point { x, y }
    }
    fn left(self) -> Option<Point> {
        if self.x > 0 {
            Some(Point::new(self.x - 1, self.y))
        } else {
            None
        }
    }
    fn top(self) -> Option<Point> {
        if self.y > 0 {
            Some(Point::new(self.x, self.y - 1))
        } else {
            None
        }
    }
    fn right(self) -> Option<Point> {
        Some(Point::new(self.x + 1, self.y))
    }
    fn bottom(self) -> Option<Point> {
        Some(Point::new(self.x, self.y + 1))
    }
}

fn unwind(source: Point, target: Point, prev: HashMap<Point, Point>) -> Vec<Point> {
    // Get the path from the prev map
    fn recurse(current: Point, source: Point, prev: HashMap<Point, Point>) -> Vec<Point> {
        if current == source {
            return vec![source];
        }
        recurse(*prev.get(&current).unwrap(), source, prev)
            .into_iter()
            .chain([current].into_iter())
            .collect()
    }

    recurse(target, source, prev)
}

fn dijkstra(
    source: Point,
    weights: &HashMap<Point, u32>,
    extent: Point,
    target: Point,
) -> Vec<Point> {
    // Dijkstra's algorithm where we're allowed to visit directly adjacent cells in the grid
    // More mutable than I'd like, but my previous recursive function caused a stack overflow
    // even though it looked like it could have used tail recursion

    // Set up priority queue and seed it with source cell
    let mut q = PriorityQueue::<Point, Reverse<u32>>::new();
    q.push(source, Reverse(0));

    // Distance of each cell from the source
    // Gets updated as we find better ways to get to each cell
    let mut dists = HashMap::from([(source, 0)]);

    // List of visisted nodes
    let mut seen = HashSet::new();

    // prev of each node
    // How we can discover the path once we find a solution
    let mut prev = HashMap::<Point, Point>::new();

    while !q.is_empty() {
        // Get closest (highest priority) node in queue
        let (current, _priority) = q.pop().unwrap();
        seen.insert(current);

        // We're done!
        if current == target {
            break;
        }

        // Save to current node
        let current_dist = *dists.get(&current).unwrap_or(&u32::MAX);

        // Check all neighbours
        for neighbour in [
            current.top(),
            current.left(),
            current.right(),
            current.bottom(),
        ]
        .iter()
        .filter_map(|v| *v)
        // Ensure neighbour on grid
        .filter(|v| v.x < extent.x && v.y < extent.y)
        // Ensure we haven't visited neighbour already
        .filter(|v| !seen.contains(v))
        {
            let neighbour_dist = dists.get(&neighbour).unwrap_or(&u32::MAX);

            // If it would be quicker to get to neighbour from current node,
            // then update the distance and prev of v
            let dist_to_neighbour_through_current = current_dist + weights.get(&current).unwrap();
            if dist_to_neighbour_through_current < *neighbour_dist {
                dists.insert(neighbour, dist_to_neighbour_through_current);
                prev.insert(neighbour, current);
            }

            // Enqueue neighbour
            q.push(neighbour, Reverse(*dists.get(&neighbour).unwrap()));
        }
    }

    // Get the path from the prev map
    unwind(source, target, prev)
}

fn print(extent: Point, path: &[Point], weights: &HashMap<Point, u32>) {
    for y in 0..extent.x {
        for x in 0..extent.y {
            let point = Point { x, y };
            if path.contains(&point) {
                print!(
                    "{}{}{}",
                    color::Fg(color::Red),
                    &weights.get(&point).unwrap(),
                    style::Reset
                );
            } else {
                print!("{}", &weights.get(&point).unwrap());
            }
        }
        println!();
    }
}

fn day_15() -> Result<u32, AdventError> {
    let args: Vec<String> = env::args().collect();
    let command = args.get(1).ok_or(AdventError::NoPartArgument)?;
    let question_part = match &command[..] {
        "part-one" => Ok(QuestionPart::One),
        "part-two" => Ok(QuestionPart::Two),
        _ => Err(AdventError::InvalidCommand {
            command: args[1].to_string(),
        }),
    }?;

    let input = stdin()
        .lock()
        .lines()
        .map(|l| l.map_err(AdventError::Io))
        .collect::<Result<Vec<_>, AdventError>>()?;

    let nrows = input.len();
    let ncols = input[0].len();
    let weights = input
        .iter()
        .enumerate()
        .map(|(y, l)| {
            l.chars().enumerate().map(move |(x, c)| {
                Ok((
                    Point::new(x, y),
                    c.to_digit(10).ok_or(AdventError::Parse { c })?,
                ))
            })
        })
        .flatten()
        .collect::<Result<Vec<_>, AdventError>>()?;

    let weights = HashMap::<Point, u32>::from_iter(weights);

    let (ncols, nrows, weights) = match question_part {
        QuestionPart::One => (ncols, nrows, weights),
        QuestionPart::Two => (
            ncols * 5,
            nrows * 5,
            HashMap::<Point, u32>::from_iter(iproduct!(0..5 * ncols, 0..5 * nrows).map(
                |(x, y)| {
                    let point = Point { x, y };
                    let region = Point {
                        x: x / ncols,
                        y: y / nrows,
                    };
                    (
                        point,
                        (weights.get(&Point::new(x % ncols, y % nrows)).unwrap()
                            + (region.x + region.y) as u32
                            - 1)
                            % 9
                            + 1,
                    )
                },
            )),
        ),
    };
    let source = Point::new(0, 0);
    let target = Point::new(ncols - 1, nrows - 1);
    let extent = Point { x: ncols, y: nrows };

    let path = dijkstra(source, &weights, extent, target);

    print(extent, &path, &weights);

    // The source point is never counted
    let path_weight = path.iter().fold(0, |acc, point| {
        if point == &source {
            acc
        } else {
            acc + weights.get(point).unwrap()
        }
    });
    Ok(path_weight)
}

fn main() {
    match day_15() {
        Ok(answer) => println!("{}", answer),
        Err(err) => {
            eprintln!("{}", err);
            exit(1);
        }
    }
}
