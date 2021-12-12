use memoize::memoize;
use std::env;
use std::io::{stdin, BufRead};
use std::process::exit;
use thiserror::Error;

enum QuestionPart {
    One,
    Two,
}

#[derive(Error, Debug)]
pub enum AdventError {
    #[error("Invalid command `{command:?}'. Expected `part-one' or `part-two'.")]
    InvalidCommand { command: String },

    #[error("Could not parse `{x}' in input string into int.")]
    Parse { x: String },

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error("Please specify `part-one' or `part-two' as the first argument.")]
    NoPartArgument,

    // One line of input was given but it's empty
    #[error("There are no crabs to help you escape! (empty input)")]
    NoCrabs,
}

#[memoize]
fn fuel_for_distance_part_two(n: usize) -> usize {
    // In part two, the fuel required while taking the nth step is n
    // Therefore, the fuel required to go n steps is the fuel required to go n-1 steps + n
    match n {
        0 => 0,
        1 => 1,
        _ => fuel_for_distance_part_two(n - 1) + n,
    }
}

fn abs_diff<T: std::cmp::PartialOrd + std::ops::Sub<Output = T>>(a: T, b: T) -> T {
    // Calculate the absolute difference between unsigned integers
    // They are unsigned, so we can't do (a - b).abs()
    if a > b {
        a - b
    } else {
        b - a
    }
}

fn day_7() -> Result<(usize, usize), AdventError> {
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
    stdin().lock().read_line(&mut input)?;

    if input == "\n" {
        return Err(AdventError::NoCrabs);
    }

    // Input is one line of int separated by comma
    let initial_state = input
        .trim()
        .split(',')
        .map(|x| {
            x.parse()
                .map_err(|_| AdventError::Parse { x: x.to_string() })
        })
        .collect::<Result<Vec<usize>, AdventError>>()?;

    let fuel_for_distance = match question_part {
        // Fuel for n steps in part one is just n
        QuestionPart::One => std::convert::identity,
        // Fuel for n steps in part 2 is non-linear
        QuestionPart::Two => fuel_for_distance_part_two,
    };

    // Get range solution could have
    // NoCrabs error should be handled above, and if not we'll get a parse error,
    // but just to avoid unwrap
    let min = *initial_state.iter().min().ok_or(AdventError::NoCrabs)?;
    let max = *initial_state.iter().max().ok_or(AdventError::NoCrabs)?;
    let (align_position, fuel) = (min..=max)
        // For each guess, calculate fuel to move every crab to the guess
        .map(|align_position| {
            (
                align_position,
                initial_state
                    .iter()
                    .map(|x| fuel_for_distance(abs_diff(*x, align_position)))
                    .sum::<usize>(),
            )
        })
        // Get the minimum by fuel (not by align position)
        .min_by_key(|(_align_position, fuel)| *fuel)
        .ok_or(AdventError::NoCrabs)?;

    Ok((align_position, fuel))
}

fn main() {
    match day_7() {
        Ok((align_position, fuel)) => {
            println!("Aligning at {} would take {} fuel.", align_position, fuel)
        }
        Err(err) => {
            eprintln!("{}", err);
            exit(1);
        }
    }
}
