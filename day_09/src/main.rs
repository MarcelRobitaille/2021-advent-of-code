use itertools::Itertools;
use ndarray::prelude::*;
use ndarray::{Array, ShapeError};
use std::collections::{HashSet, VecDeque};
use std::env;
use std::io::{stdin, Read};
use std::process::exit;
use thiserror::Error;

type Point = (usize, usize);

#[derive(Debug)]
enum QuestionPart {
    One,
    Two,
}

#[derive(Error, Debug)]
pub enum AdventError {
    #[error("Invalid command `{command:?}'. Expected `part-one' or `part-two'.")]
    InvalidCommand { command: String },

    #[error("Invalid input")]
    InvalidInput,

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error("Please specify `part-one' or `part-two' as the first argument.")]
    NoPartArgument,

    #[error(transparent)]
    Shape(#[from] ShapeError),

    #[error("Could not parse char `{c}' to numeric digit.")]
    Parse { c: char },
}

fn print(a: &Array<usize, Ix2>) {
    // Pretty print the array like in the website

    for row in a.rows() {
        println!(
            "{}",
            row.iter()
                .map(|x| if x == &0 {
                    ".".to_string()
                } else {
                    x.to_string()
                })
                .join("")
        );
    }
}

fn discover_basin(low_point: Point, input: &Array<usize, Ix2>) -> usize {
    // Discover the size of a basin using something similar to BFS

    let mut basin_size: usize = 0;

    if let [width, height] = input.shape() {
        let (width, height) = (*width, *height);
        let mut q = VecDeque::<Point>::new();
        q.push_back(low_point);

        let mut visited = HashSet::<Point>::new();

        let mut vis = Array2::<usize>::zeros((height, width)).reversed_axes();

        while let Some((x, y)) = q.pop_front() {
            if *input.get((x, y)).unwrap() == 9 || visited.contains(&(x, y)) {
                continue;
            }

            basin_size += 1;
            visited.insert((x, y));
            *vis.get_mut((x, y)).unwrap() = *input.get((x, y)).unwrap();

            if x > 0 {
                q.push_back((x - 1, y));
            }
            if x + 1 < width {
                q.push_back((x + 1, y));
            }
            if y > 0 {
                q.push_back((x, y - 1));
            }
            if y + 1 < height {
                q.push_back((x, y + 1));
            }
        }
        print(&vis.reversed_axes());
    }

    basin_size
}

fn solve(input: &str, question_part: QuestionPart) -> Result<usize, AdventError> {
    let input = input.trim().split('\n').collect::<Vec<_>>();
    let width = input.get(0).ok_or(AdventError::InvalidInput)?.len();
    let height = input.len();

    let input = input
        .iter()
        .map(|l| {
            l.chars()
                .map(|c| c.to_digit(10).ok_or(AdventError::Parse { c }))
        })
        .flatten()
        .collect::<Result<Vec<_>, AdventError>>()?;
    let input = input.iter().map(|x| *x as usize).collect::<Vec<usize>>();

    let input = Array::<usize, _>::from_iter(input)
        // ndarray major axis is vertical so give height and width backwards and then transpose
        // Otherwise, it doesn't work with the order of the input
        .into_shape((height, width))?
        .reversed_axes();
    println!("{:?}", input.t());

    // Get all the points that are less than all their immediate neighbours (excluding diagonals)
    let low_points = itertools::iproduct!(0..width, 0..height)
        .filter_map(|(x, y)| {
            let target = input.get((x, y)).unwrap();
            let left = if x > 0 { input.get((x - 1, y)) } else { None }.unwrap_or(&usize::MAX);
            let right = input.get((x + 1, y)).unwrap_or(&usize::MAX);
            let up = input.get((x, y + 1)).unwrap_or(&usize::MAX);
            let down = if y > 0 { input.get((x, y - 1)) } else { None }.unwrap_or(&usize::MAX);
            if target < *vec![left, right, up, down].iter().min().unwrap() {
                Some((x, y))
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    Ok(match question_part {
        // In part one, we care about the sum of the risk levels of all the low points
        QuestionPart::One => low_points
            .iter()
            .map(|(x, y)| input.get((*x, *y)).unwrap() + 1)
            .sum::<usize>(),

        // In part two, we want the product of the sizes of the three largest basins
        QuestionPart::Two => {
            let mut basin_sizes = low_points
                .iter()
                .map(|low_point| discover_basin(*low_point, &input))
                .collect::<Vec<_>>();
            basin_sizes.sort_unstable();

            basin_sizes.iter().rev().take(3).product::<usize>()
        }
    })
}

fn day_9() -> Result<usize, AdventError> {
    let args: Vec<String> = env::args().collect();
    let command = args.get(1).ok_or(AdventError::NoPartArgument)?;
    let question_part = match &command[..] {
        "part-one" => Ok(QuestionPart::One),
        "part-two" => Ok(QuestionPart::Two),
        _ => Err(AdventError::InvalidCommand {
            command: args[1].to_string(),
        }),
    }?;

    let mut input = String::new();
    stdin().lock().read_to_string(&mut input)?;

    solve(&input, question_part)
}

fn main() {
    match day_9() {
        Ok(answer) => println!("{}", answer),
        Err(err) => {
            eprintln!("{}", err);
            exit(1);
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    const INPUT: &str = "2199943210
3987894921
9856789892
8767896789
9899965678
";

    #[test]
    fn test_part_one() -> Result<(), AdventError> {
        assert_eq!(solve(INPUT, QuestionPart::One)?, 15);
        Ok(())
    }

    #[test]
    fn test_part_two() -> Result<(), AdventError> {
        assert_eq!(solve(INPUT, QuestionPart::Two)?, 1134);
        Ok(())
    }
}
