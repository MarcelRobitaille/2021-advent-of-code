use itertools::Itertools;
use ndarray::prelude::*;
use ndarray::{Array, ShapeError, Slice};
use regex::Regex;
use std::env;
use std::fmt;
use std::io::{stdin, BufRead};
use std::process::exit;
use thiserror::Error;

#[derive(Debug)]
enum QuestionPart {
    One,
    Two,
}

#[derive(Debug)]
enum Answer {
    PartOne(usize),
    PartTwo(String),
}

impl fmt::Display for Answer {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Answer::PartOne(v) => write!(f, "{}", v),
            Answer::PartTwo(s) => write!(f, "{}", s),
        }
    }
}

#[derive(Debug)]
enum Fold {
    X(usize),
    Y(usize),
}

#[derive(Error, Debug)]
pub enum AdventError {
    #[error("Invalid command `{command:?}'. Expected `part-one' or `part-two'.")]
    InvalidCommand { command: String },

    #[error("Invalid input detected.")]
    InvalidInput,

    #[error("Your transparent paper has no dots!")]
    NoDots,

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Regex(#[from] regex::Error),

    #[error(transparent)]
    Shape(#[from] ShapeError),

    #[error("Please specify `part-one' or `part-two' as the first argument.")]
    NoPartArgument,

    #[error(transparent)]
    Parse(#[from] std::num::ParseIntError),

    #[error("Invalid coordinate `{line}'. Expected `<num>,<num>'.")]
    InvalidCoordinate { line: String },

    #[error("Invalid fold `{line}'. Expected `fold along <x|y>=<num>`.")]
    FoldFormat { line: String },
}

fn print(a: &Array<bool, Ix2>) {
    // Pretty print the array like in the website

    for row in a.rows() {
        println!(
            "{}",
            row.iter().map(|x| if *x { '#' } else { '.' }).join("")
        );
    }
}

fn abs_diff(a: usize, b: usize) -> usize {
    if a > b {
        a - b
    } else {
        b - a
    }
}

fn day_13() -> Result<Answer, AdventError> {
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
    let (dots, folds) = input
        .split(|l| l.is_empty())
        .collect_tuple()
        .ok_or(AdventError::InvalidInput)?;
    let dots = dots
        .iter()
        .map(|l| {
            let (x, y) = l
                .split(',')
                .collect_tuple()
                .ok_or(AdventError::InvalidCoordinate {
                    line: l.to_string(),
                })?;
            let x = x.parse::<usize>().map_err(AdventError::Parse)?;
            let y = y.parse::<usize>().map_err(AdventError::Parse)?;
            Ok((x, y))
        })
        .collect::<Result<Vec<_>, AdventError>>()?;

    let re = Regex::new(r"fold along (x|y)=(\d+)")?;
    let folds = folds
        .iter()
        .map(|l| {
            let cap = re.captures(l).ok_or(AdventError::FoldFormat {
                line: l.to_string(),
            })?;
            let v = &cap[2].parse()?;
            match &cap[1] {
                "x" => Ok(Fold::X(*v)),
                "y" => Ok(Fold::Y(*v)),
                // The regex only matches for these two options, so this is unreachable
                _ => unreachable!(),
            }
        })
        .collect::<Result<Vec<Fold>, AdventError>>()?;

    let width = *dots
        .iter()
        .map(|(x, _y)| x)
        .max()
        .ok_or(AdventError::NoDots)?
        + 1;
    let height = *dots
        .iter()
        .map(|(_x, y)| y)
        .max()
        .ok_or(AdventError::NoDots)?
        + 1;
    let mut a = Array::from_elem((width, height), false);

    for (x, y) in dots {
        if let Some(v) = a.get_mut((x, y)) {
            *v = true;
        }
    }

    for fold in folds {
        // Get stuff needed to make the slice
        let (axis, position) = match fold {
            Fold::X(v) => (Axis(0), v),
            Fold::Y(v) => (Axis(1), v),
        };
        let (v1, v2) = a.view().split_at(axis, position);
        // The line of the fold is not kept
        let mut v2 = v2.slice_axis(axis, Slice::from(1..));

        // Flip the other part
        // We always fold up or left
        // The part that gets folded gets mirrored
        v2.invert_axis(axis);

        // Get the missing width / height of the smaller part
        // We must make them the same shape before we broadcast
        let (missing_width, missing_height) = match fold {
            Fold::Y(_) => (v2.nrows(), abs_diff(v2.ncols(), v1.ncols())),
            Fold::X(_) => (abs_diff(v2.nrows(), v1.nrows()), v2.ncols()),
        };

        // Make an empty array of the missing width and height
        // This is like the virtual paper over the end of the real paper
        let mut zeros = Array::from_elem((missing_width, missing_height), false);

        // Grow the smaller part so that both parts are the same shape
        let (v1, v2) = if v1.shape() < v2.shape() {
            // Append the top/left view to the end of the zeros
            // If we're folding up or left, the blank space needs to go at the start
            zeros.append(axis, v1)?;
            (zeros, v2.to_owned())
        } else {
            // Append the bottom/right view to the end of the zeros
            // If this part is smaller, then the blank space should go at the END before folding
            // We already did `invert_axis` (folded), so it should go at the START just like the
            // other case
            zeros.append(axis, v2)?;
            (v1.to_owned(), zeros)
        };

        // Laminate transparent paper into single sheet
        a = &v1 | &v2;

        // In part one, we only do the first fold
        if matches!(question_part, QuestionPart::One) {
            break;
        }
    }

    Ok(match question_part {
        QuestionPart::One => Answer::PartOne(a.iter().filter(|x| **x).count()),
        QuestionPart::Two => {
            // Print result
            Answer::PartTwo(
                a.reversed_axes()
                    .rows()
                    .into_iter()
                    .map(|row| row.iter().map(|x| if *x { '#' } else { '.' }).join(""))
                    .join("\n"),
            )
        }
    })
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
