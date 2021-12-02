use std::env;
use std::io::{stdin, BufRead};
use std::process::exit;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AdventError {
    #[error("Invalid input")]
    InvalidInput,

    #[error(transparent)]
    ParseInt(#[from] std::num::ParseIntError),

    #[error("Invalid command `{command:?}'. Expected `part-one' or `part-two'.")]
    InvalidCommand { command: String },

    #[error("")]
    NoCommand,
}

fn part_one() -> Result<i32, AdventError> {
    let (x, y) =
        stdin()
            .lock()
            .lines()
            .filter_map(|l| l.ok())
            .try_fold((0, 0), |(x, y), line| {
                let i = line.find(' ').ok_or(AdventError::InvalidInput)?;
                let (direction, distance) = line.split_at(i);
                let distance: i32 = distance.trim().parse()?;

                match direction {
                    "forward" => Ok((x + distance, y)),
                    "up" => Ok((x, y - distance)),
                    "down" => Ok((x, y + distance)),
                    _ => Err(AdventError::InvalidInput),
                }
            })?;
    Ok(x * y)
}

fn part_two() -> Result<i32, AdventError> {
    let (x, y, _aim) = stdin().lock().lines().filter_map(|l| l.ok()).try_fold(
        (0, 0, 0),
        |(x, y, aim), line| {
            let i = line.find(' ').ok_or(AdventError::InvalidInput)?;
            let (direction, distance) = line.split_at(i);
            let distance: i32 = distance.trim().parse()?;

            match direction {
                "forward" => Ok((x + distance, y + aim * distance, aim)),
                "up" => Ok((x, y, aim - distance)),
                "down" => Ok((x, y, aim + distance)),
                _ => Err(AdventError::InvalidInput),
            }
        },
    )?;
    Ok(x * y)
}

fn main() {
    let args: Vec<String> = env::args().collect();

    match args.get(1) {
        Some(command) => {
            let result = match &command[..] {
                "part-one" => part_one(),
                "part-two" => part_two(),
                _ => Err(AdventError::InvalidCommand {
                    command: args[1].to_string(),
                }),
            };

            match result {
                Ok(result) => println!("{}", result),
                Err(err) => {
                    eprintln!("Error: {}", err);
                    exit(1);
                }
            }
        }
        None => {
            eprintln!("Please specify `part-one' or `part-two' as the first argument.");
            exit(1);
        }
    }
}
