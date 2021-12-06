use itertools::Itertools;
use ndarray::prelude::*;
use ndarray::Array;
use regex::Regex;
use std::cmp::max;
use std::env;
use std::io::{stdin, BufRead};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AdventError {
    #[error("Invalid input")]
    InvalidInput,

    #[error(transparent)]
    Regex(#[from] regex::Error),

    #[error("Invalid command `{command:?}'. Expected `part-one' or `part-two'.")]
    InvalidCommand { command: String },

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error("Please specify `part-one' or `part-two' as the first argument.")]
    NoPartArgument,
}

enum QuestionPart {
    One,
    Two,
}

fn sorted<A, T>(mut array: A) -> A
where
    A: AsMut<[T]>,
    T: Ord,
{
    let slice = array.as_mut();
    slice.sort();

    array
}

#[derive(Debug, Clone)]
struct Line {
    x1: usize,
    y1: usize,
    x2: usize,
    y2: usize,
}

impl Line {
    fn parse(text: &str) -> Result<Line, AdventError> {
        let re = Regex::new(r"(\d+),(\d+) -> (\d+),(\d+)")?;
        let (x1, y1, x2, y2) = re
            .captures(text)
            .ok_or(AdventError::InvalidInput)?
            .iter()
            .filter_map(|x| x?.as_str().parse().ok())
            .collect_tuple()
            .ok_or(AdventError::InvalidInput)?;
        Ok(Line { x1, y1, x2, y2 })
    }

    fn sort(&self) -> Line {
        // Sort pair of points by x value
        // This guarantee is useful later on when doing the diagonals

        if self.x1 < self.x2 {
            self.clone()
        } else {
            Line {
                x1: self.x2,
                x2: self.x1,
                y1: self.y2,
                y2: self.y1,
            }
        }
    }
}

fn print(a: &Array<i32, Ix2>) {
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

fn main() -> Result<(), AdventError> {
    let args: Vec<String> = env::args().collect();
    let command = args.get(1).ok_or(AdventError::NoPartArgument)?;
    let question_part = match &command[..] {
        "part-one" => Ok(QuestionPart::One),
        "part-two" => Ok(QuestionPart::Two),
        _ => Err(AdventError::InvalidCommand {
            command: args[1].to_string(),
        }),
    }?;

    let lines = stdin()
        .lock()
        .lines()
        .map(|line| match line {
            Ok(line) => Line::parse(&line[..]),
            Err(e) => Err(AdventError::Io(e)),
        })
        .collect::<Result<Vec<Line>, AdventError>>()?;
    let lines: Vec<Line> = lines.iter().map(|l| l.sort()).collect();

    let x_max = lines
        .iter()
        .map(|line| max(line.x1, line.x2))
        .max()
        .unwrap()
        + 1;
    let y_max = lines
        .iter()
        .map(|line| max(line.y1, line.y2))
        .max()
        .unwrap()
        + 1;
    let mut a: Array<i32, Ix2> = Array::zeros((x_max, y_max));

    print(&a);
    println!();
    let points = lines
        .iter()
        .map(|line| {
            if line.x1 == line.x2 {
                let [small, big] = sorted([line.y1, line.y2]);
                (small..=big).map(|y| [line.x1, y]).collect()
            } else if line.y1 == line.y2 {
                let [small, big] = sorted([line.x1, line.x2]);
                (small..=big).map(|x| [x, line.y1]).collect()
            } else {
                match question_part {
                    // In part one, we don't consider the diagonals
                    QuestionPart::One => Vec::new(),
                    QuestionPart::Two => (0..=(line.x2 - line.x1) as i32)
                        .map(|i| {
                            let [x, y1, y2] = [line.x1, line.y1, line.y2].map(|x| x as i32);
                            [x + i, (y1 + (if y2 > y1 { i } else { -i }))].map(|x| x as usize)
                        })
                        .collect(),
                }
            }
        })
        .flatten()
        .collect::<Vec<_>>();

    for [x, y] in points {
        // Cannot find a better way besides mutating
        // Wanted to use reduce, but I cannot find a way to return the ndarray with a single cell
        // incremented
        if let Some(cell) = a.get_mut((y, x)) {
            *cell += 1;
        }
    }
    print(&a);

    println!(
        "{:?}",
        a.iter().map(|x| if x >= &2 { 1 } else { 0 }).sum::<i32>()
    );

    Ok(())
}
