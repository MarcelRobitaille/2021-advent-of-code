use itertools::iproduct;
use std::collections::HashMap;
use std::env;
use std::io::{stdin, BufRead};
use std::mem::discriminant;
use std::process::exit;
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
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Cuke {
    South,
    East,
}

impl Cuke {
    fn parse(from: char) -> Option<Self> {
        match from {
            '>' => Some(Cuke::East),
            'v' => Some(Cuke::South),
            '.' => None,
            _ => unreachable!(),
        }
    }
}

// Store the grid state in a hashmap of coord to cuke
// I find ndarray kind of annoying
type State = HashMap<(usize, usize), Cuke>;

fn variant_eq<T>(a: &T, b: &T) -> bool {
    discriminant(a) == discriminant(b)
}

fn print(state: &State, extent_x: usize, extent_y: usize) {
    // Print the grid
    // Can be useful for debugging
    for y in 0..extent_y {
        for x in 0..extent_x {
            print!(
                "{}",
                match state.get(&(x, y)) {
                    Some(Cuke::East) => '>',
                    Some(Cuke::South) => 'v',
                    None => '.',
                }
            )
        }
        println!();
    }
}

fn step(state: &State, extent_x: usize, extent_y: usize, target_cuke: Cuke) -> State {
    // Perform step for either direction

    // Diff added to current position to get target
    // For checking for conflicts as well as moving
    let diff = match target_cuke {
        Cuke::East => (1, 0),
        Cuke::South => (0, 1),
    };

    // Brute force check entire grid
    iproduct!(0..extent_x, 0..extent_y)
        // Select only filled cells
        .filter_map(|(x, y)| state.get(&(x, y)).map(|cuke| ((x, y), cuke)))
        .map(|((x, y), cuke)| {
            // If it's the target cuke, check if we can move to it (check it's empty)
            let target_cell = ((x + diff.0) % extent_x, (y + diff.1) % extent_y);
            if variant_eq(&target_cuke, cuke) && state.get(&target_cell).is_none() {
                (target_cell, target_cuke)
            } else {
                // Otherwise, just keep it where it is
                ((x, y), *cuke)
            }
        })
        .collect()
}

fn recurse(
    state: HashMap<(usize, usize), Cuke>,
    depth: usize,
    extent_x: usize,
    extent_y: usize,
) -> (State, usize) {
    // Recursively step in each direction until no cukes move
    let new_state = step(&state, extent_x, extent_y, Cuke::East);
    let new_state = step(&new_state, extent_x, extent_y, Cuke::South);

    println!("Step {}", depth);

    if state == new_state {
        (state, depth)
    } else {
        recurse(new_state, depth + 1, extent_x, extent_y)
    }
}

fn day_25() -> Result<usize, AdventError> {
    // Parse input into lines and handle args, then call main function num_on_cubes
    let args: Vec<String> = env::args().collect();
    let command = args.get(1).ok_or(AdventError::NoPartArgument)?;
    let question_part = match &command[..] {
        "part-one" => Ok(QuestionPart::One),
        "part-two" => Ok(QuestionPart::Two),
        _ => Err(AdventError::InvalidCommand {
            command: args[1].to_string(),
        }),
    }?;
    println!("{:?}", question_part);

    let lines = stdin()
        .lock()
        .lines()
        .collect::<Result<Vec<String>, std::io::Error>>()?;

    let cukes = lines
        .into_iter()
        .map(|line| line.chars().map(Cuke::parse).collect())
        .collect::<Vec<Vec<Option<Cuke>>>>();

    let extent_y = cukes.len();
    let extent_x = cukes[0].len();

    let mut state = HashMap::<(usize, usize), Cuke>::new();

    // Vec of vec into hashmap
    for (y, row) in cukes.into_iter().enumerate() {
        for (x, cuke) in row.into_iter().enumerate() {
            if let Some(cuke) = cuke {
                state.insert((x, y), cuke);
            }
        }
    }

    let (_state, steps) = recurse(state, 1, extent_x, extent_y);

    Ok(steps)
}

fn main() {
    match day_25() {
        Ok(answer) => println!("{}", answer),
        Err(err) => {
            eprintln!("{}", err);
            exit(1);
        }
    }
}
