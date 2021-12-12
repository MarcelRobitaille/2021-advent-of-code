use itertools::Itertools;
use ndarray::prelude::*;
use ndarray::{Array, ShapeError};
use std::collections::HashSet;
use std::env;
use std::io::{stdin, BufRead};
use std::process::exit;
use thiserror::Error;

const SIZE: usize = 10;

type Flashed = HashSet<(usize, usize)>;

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

    #[error(transparent)]
    Shape(#[from] ShapeError),

    #[error("Please specify `part-one' or `part-two' as the first argument.")]
    NoPartArgument,

    #[error("Could not parse char `{c}' to numeric digit.")]
    Parse { c: char },
}

fn print(a: &Array<u32, Ix2>) {
    // Pretty print the array like in the website

    for row in a.rows() {
        println!("{}", row.iter().map(|x| x.to_string()).join(""));
    }
}

fn flash(flash_x: usize, flash_y: usize, a: &mut Array2<u32>, flashed: &Flashed) -> Flashed {
    // Flash given spot, incrementing it surroundings and recursively checking for more flashes
    let xmin = if flash_x == 0 { 0 } else { flash_x - 1 };
    let ymin = if flash_y == 0 { 0 } else { flash_y - 1 };

    if flashed.contains(&(flash_x, flash_y)) {
        return Flashed::new();
    }

    itertools::iproduct!(xmin..=flash_x + 1, ymin..=flash_y + 1).fold(
        // Get a new set with the new flashed spot
        flashed | &Flashed::from([(flash_x, flash_y)]),
        // Fold in recursive calls
        |flashed, (x, y)| {
            // Increment everything in the flash zone
            if let Some(v) = a.get_mut((x, y)) {
                *v += 1;
            }

            // Check if this flash triggered more flashes
            match a.get((x, y)) {
                Some(v) if v > &9 => &flashed | &flash(x, y, a, &flashed),
                _ => flashed,
            }
        },
    )
}

fn recurse(
    a: &mut Array2<u32>,
    question_part: QuestionPart,
    steps: usize,
    flashes: usize,
) -> Result<usize, AdventError> {
    // I tried to make this as immutable as possible, but I can't find a way to do the array
    // without a mutable reference

    // Increment entire grid
    let mut a = Array::<u32, _>::from_iter(a.iter().map(|x| x + 1)).into_shape((SIZE, SIZE))?;

    // Calculate what flashed
    let flashed = itertools::iproduct!(0..SIZE, 0..SIZE).fold(Flashed::new(), |flashed, (x, y)| {
        match a.get((x, y)) {
            Some(v) if v > &9 && !flashed.contains(&(x, y)) => {
                &flashed | &flash(x, y, &mut a, &flashed)
            }
            _ => flashed,
        }
    });

    // Set everything that flashed back to zero
    for (x, y) in &flashed {
        if let Some(v) = a.get_mut((*x, *y)) {
            *v = 0;
        }
    }

    let flashes = flashes + flashed.len();

    println!();
    print(&a);
    match question_part {
        // In part one, we're looking for the number of flashes after 100 steps
        QuestionPart::One if steps == 100 => Ok(flashes),

        // In part two, we're looking for the number of steps until the entire grid flashes
        QuestionPart::Two if flashed.len() == SIZE * SIZE => Ok(steps),

        // Otherwise, recurse
        _ => recurse(&mut a, question_part, steps + 1, flashes),
    }
}

fn day_10() -> Result<usize, AdventError> {
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
    let input = input
        .iter()
        .map(|l| {
            l.chars()
                .map(|c| c.to_digit(10).ok_or(AdventError::Parse { c }))
        })
        .flatten()
        .collect::<Result<Vec<_>, AdventError>>()?;

    let mut a = Array::<u32, _>::from_iter(input)
        // ndarray major axis is vertical so give height and width backwards and then transpose
        // Otherwise, it doesn't work with the order of the input
        .into_shape((SIZE, SIZE))?;

    print(&a);
    recurse(&mut a, question_part, 1, 0)
}

fn main() {
    match day_10() {
        Ok(answer) => println!("{}", answer),
        Err(err) => {
            eprintln!("{}", err);
            exit(1);
        }
    }
}
