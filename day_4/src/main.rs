use itertools::chain;
use ndarray::prelude::*;
use ndarray::{Array, ShapeError};
use regex::Regex;
use std::env;
use std::io::{stdin, BufRead};
use std::process::exit;
use thiserror::Error;

const BOARD_SIZE: usize = 5;
const STAMPED: i32 = -1;

#[derive(Error, Debug)]
pub enum AdventError {
    #[error(transparent)]
    Regex(#[from] regex::Error),

    #[error("Invalid command `{command:?}'. Expected `part-one' or `part-two'.")]
    InvalidCommand { command: String },

    #[error(transparent)]
    Shape(#[from] ShapeError),

    #[error("List of draws is improperly formatted or missing")]
    DrawsFormat,

    #[error("The provided input never has a solution (not all numbers are drawn)")]
    NoSolution,

    #[error("Please specify `part-one' or `part-two' as the first argument.")]
    NoPartArgument,
}

enum QuestionPart {
    One,
    Two,
}

fn find_winner(
    boards: Vec<Array<i32, Ix2>>,
    draws: &[i32],
    question_part: QuestionPart,
) -> Result<i32, AdventError> {
    // Pop off the first draw
    let (draw, draws) = draws.split_first().ok_or(AdventError::NoSolution)?;

    // Stamp all where equal to current draw
    let boards: Vec<Array<i32, Ix2>> = boards
        .into_iter()
        .map(|b| b.mapv(|x| if x == *draw { STAMPED } else { x }))
        .collect();

    // Get the indices of all winning boards so we can filter them out
    let winners: Vec<usize> = boards
        .iter()
        .enumerate()
        .filter_map(|(i, b)| {
            if chain!(b.rows(), b.columns()).any(|slice| slice == aview1(&[STAMPED; BOARD_SIZE])) {
                Some(i)
            } else {
                None
            }
        })
        .collect();

    // If there are no winners yet, recurse on remaining draws
    if winners.is_empty() {
        return find_winner(boards, draws, question_part);
    }

    // If it's part one, return the first winner
    // If it's part two and we're only considering one board at this point,
    // it's the winner
    if matches!(question_part, QuestionPart::One) || boards.len() == 1 {
        let not_null: i32 = boards[winners[0]].iter().filter(|x| x != &&STAMPED).sum();
        Ok(draw * not_null)

    // Otherwise recurse without winning boards
    } else {
        find_winner(
            boards
                .into_iter()
                .enumerate()
                .filter_map(|(i, b)| if winners.contains(&i) { None } else { Some(b) })
                .collect(),
            draws,
            question_part,
        )
    }
}

fn day_4() -> Result<(), AdventError> {
    let args: Vec<String> = env::args().collect();

    let command = args.get(1).ok_or(AdventError::NoPartArgument)?;
    let question_part = match &command[..] {
        "part-one" => Ok(QuestionPart::One),
        "part-two" => Ok(QuestionPart::Two),
        _ => Err(AdventError::InvalidCommand {
            command: args[1].to_string(),
        }),
    }?;

    let lines: Vec<String> = stdin().lock().lines().filter_map(|x| x.ok()).collect();
    let mut lines = lines.iter();

    // First line of input holds all the draws
    let draws: Vec<i32> = lines
        .next()
        .ok_or(AdventError::DrawsFormat)?
        .split(',')
        .filter_map(|x| x.parse().ok())
        .collect();

    // Parse boards
    let re = Regex::new(r"\s+")?;
    let boards = lines
        .collect::<Vec<_>>()
        // Boards are separated by a blank line (hence the +1)
        .chunks(BOARD_SIZE + 1)
        .map(|chunk| {
            Array::<i32, _>::from_iter(
                chunk
                    .iter()
                    .map(|line| re.split(line))
                    .flatten()
                    .filter(|x| x != &"")
                    .filter_map(|x| x.parse().ok()),
            )
            .into_shape((BOARD_SIZE, BOARD_SIZE))
            .map_err(AdventError::Shape)
        })
        .collect::<Result<Vec<Array<i32, Ix2>>, AdventError>>()?;

    println!("{}", find_winner(boards, &draws, question_part)?);
    Ok(())
}

fn main() {
    day_4().unwrap_or_else(|err| {
        eprintln!("{}", err);
        exit(1);
    });
}
