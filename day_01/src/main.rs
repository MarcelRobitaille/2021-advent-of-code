use itertools::Itertools;
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
}

fn solve(input: &str, question_part: QuestionPart) -> Result<usize, AdventError> {
    let input = input
        .trim()
        .split('\n')
        .map(|line| line.parse())
        .collect::<Result<Vec<u32>, ParseIntError>>()?;

    let input = match question_part {
        // In part one, there is no rolling transformation to the data
        QuestionPart::One => input,
        // In part two, apply a rolling sum with width 3
        QuestionPart::Two => input
            .windows(3)
            .map(|x| x.iter().sum::<u32>())
            .collect::<Vec<_>>(),
    };

    // Count the number of increases
    let count = input.iter().tuple_windows().filter(|(a, b)| a < b).count();
    Ok(count)
}

fn day_01() -> Result<usize, AdventError> {
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
    match day_01() {
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

    const INPUT: &str = "199
200
208
210
200
207
240
269
260
263";

    #[test]
    fn test_part_one() -> Result<(), AdventError> {
        assert_eq!(solve(INPUT, QuestionPart::One)?, 7);
        Ok(())
    }

    #[test]
    fn test_part_two() -> Result<(), AdventError> {
        assert_eq!(solve(INPUT, QuestionPart::Two)?, 5);
        Ok(())
    }
}
