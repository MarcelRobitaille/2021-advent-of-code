use itertools::iproduct;
use std::collections::{HashMap, HashSet};
use std::env;
use std::hash::Hash;
use std::io::{stdin, BufRead};
use std::ops::BitOr;
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

#[derive(Debug)]
struct ExtensibleHashMap<K, V> {
    base: HashMap<K, V>,
}

impl<K, V> BitOr<HashMap<K, V>> for ExtensibleHashMap<K, V>
where
    K: Hash + Eq,
{
    type Output = ExtensibleHashMap<K, V>;
    fn bitor(self, rhs: HashMap<K, V>) -> Self::Output {
        ExtensibleHashMap {
            base: self.base.into_iter().chain(rhs).collect(),
        }
    }
}

fn unwind(source: Point, target: Point, parent: HashMap<Point, Point>) -> Vec<Point> {
    fn recurse(current: Point, source: Point, parent: HashMap<Point, Point>) -> Vec<Point> {
        if current == source {
            return vec![source];
        }
        recurse(*parent.get(&current).unwrap(), source, parent)
            .into_iter()
            .chain([current].into_iter())
            .collect()
    }

    recurse(target, source, parent)
}

fn dijkstra(
    source: Point,
    weights: &HashMap<Point, u32>,
    extent: Point,
    target: Point,
) -> Vec<Point> {
    fn recurse(
        q: HashSet<Point>,
        dists: ExtensibleHashMap<Point, u32>,
        parent: ExtensibleHashMap<Point, Point>,
        seen: HashSet<Point>,
        weights: &HashMap<Point, u32>,
        extent: Point,
        target: Point,
    ) -> ExtensibleHashMap<Point, Point> {
        let u = *q.iter().min_by_key(|u| dists.base.get(u)).unwrap();
        if u == target {
            return parent;
        }

        let dist_u = dists.base.get(&u).unwrap_or(&u32::MAX);

        let updated = [u.top(), u.left(), u.right(), u.bottom()]
            .iter()
            .filter_map(|v| *v)
            .filter(|v| v.x < extent.x && v.y < extent.y)
            .filter(|v| !seen.contains(v))
            .filter_map(|v| {
                let dist_v = dists.base.get(&v).unwrap_or(&u32::MAX);
                let alt = dist_u + weights.get(&u).unwrap();
                if alt < *dist_v {
                    Some((v, alt, u))
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        let q = &q | &HashSet::from_iter(updated.iter().map(|x| x.0));
        let q = &q - &HashSet::from([u]);

        let seen = &seen | &HashSet::from([u]);
        let dists = dists | HashMap::from_iter(updated.iter().map(|x| (x.0, x.1)));
        let parent = parent | HashMap::from_iter(updated.iter().map(|x| (x.0, x.2)));
        recurse(q, dists, parent, seen, weights, extent, target)
    }

    let q = HashSet::from([source]);
    let dists = ExtensibleHashMap::<Point, u32> {
        base: HashMap::from([(source, 0)]),
    };
    let seen = HashSet::new();
    let parent = ExtensibleHashMap {
        base: HashMap::new(),
    };
    let parent = recurse(q, dists, parent, seen, weights, extent, target);
    unwind(source, target, parent.base)
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

fn day_13() -> Result<u32, AdventError> {
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
    match day_13() {
        Ok(answer) => println!("{}", answer),
        Err(err) => {
            eprintln!("{}", err);
            exit(1);
        }
    }
}
