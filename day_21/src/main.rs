use lazy_static::lazy_static;
use regex::Regex;
use std::cmp::max;
use std::env;
use std::io::{stdin, Read};
use std::num::ParseIntError;
use std::process::exit;
use thiserror::Error;

#[derive(Debug, Clone, Copy)]
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
    ParseInt(#[from] ParseIntError),

    #[error("Please specify `part-one' or `part-two' as the first argument.")]
    NoPartArgument,

    #[error("Invalid input format detected.")]
    FormatError,
}

fn parse() -> Result<(u8, u8), AdventError> {
    lazy_static! {
        static ref RE: Regex =
            Regex::new(r"Player 1 starting position: (\d+)\nPlayer 2 starting position: (\d+)")
                .unwrap();
    }

    let mut input = String::new();
    stdin().lock().read_to_string(&mut input)?;

    let caps = RE.captures(&input).ok_or(AdventError::FormatError)?;
    Ok((caps[1].parse()?, caps[2].parse()?))
}

fn part_one(i: u32, pos: (u8, u8), score: (u32, u32), target_score: u32) -> u32 {
    // In part one, get the score of the loosing player times the number of rolls
    if score.1 >= target_score {
        return score.0 as u32 * i;
    }

    // Spelling it out for clarity
    let roll1 = i + 1;
    let roll2 = i + 2;
    let roll3 = i + 3;

    // -1 +1 required for modulo math as problem statement is 1-based
    let new_pos = ((pos.0 as u32 - 1 + roll1 + roll2 + roll3) % 10 + 1) as u8;

    part_one(
        i + 3,
        (pos.1, new_pos),
        (score.1, score.0 + new_pos as u32),
        target_score,
    )
}

// Possible quantum rolls and their counts
const POSIBILITIES: [(u8, u8); 7] = [(3, 1), (4, 3), (5, 6), (6, 7), (7, 6), (8, 3), (9, 1)];
fn part_two(pos: (u8, u8), score: (u8, u8), mul: u64, target_score: u8) -> (u64, u64) {
    // Naive solution, but only takes a few seconds

    // The current player is always in index zero. Flip it around each time so we don't have to
    // deal with tracking whose turn it is

    // If somebody won, return
    if score.1 >= target_score {
        return (0, mul);
    }
    POSIBILITIES
        .iter()
        .map(|(posibility, count)| {
            let new_pos = (pos.0 + posibility - 1) % 10 + 1;
            part_two(
                (pos.1, new_pos),
                (score.1, score.0 + new_pos),
                mul * *count as u64,
                target_score,
            )
        })
        .fold((0, 0), |a, b| (a.0 + b.1, a.1 + b.0))
}

fn day_21() -> Result<u64, AdventError> {
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

    let starting_position = parse()?;

    Ok(match question_part {
        QuestionPart::One => {
            let target_score = 1000;
            let starting_score = (0, 0);
            part_one(0, starting_position, starting_score, target_score).into()
        }
        QuestionPart::Two => {
            let target_score = 21;
            let starting_score = (0, 0);
            let wins = part_two(starting_position, starting_score, 1, target_score);

            // In part two, get the maximum number of wins between the two players
            max(wins.0, wins.1)
        }
    })
}

fn main() {
    match day_21() {
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

    #[test]
    fn test_part_one() {
        assert_eq!(part_one(0, (4, 8), (0, 0), 1000), 739785);
    }

    #[test]
    fn test_part_two() {
        assert_eq!(
            part_two((4, 8), (0, 0), 1, 21),
            (444356092776315, 341960390180808)
        );
    }
}
