use itertools::Itertools;
use std::collections::{HashMap, VecDeque};
use std::env;
use std::io::{stdin, BufRead};
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

    #[error("Invalid character `{c}' found in input.")]
    InvalidChar { c: char },

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error("Please specify `part-one' or `part-two' as the first argument.")]
    NoPartArgument,

    #[error("Could not parse char `{c}' to numeric digit.")]
    Parse { c: char },

    #[error("Closing brace `{c}' found before any opening brace.")]
    ClosingBeforeOpening { c: char },
}

enum LineResult {
    Corrupt(char),
    Incomplete(VecDeque<char>),
    Ok,
}

fn parse_line(line: &str) -> Result<LineResult, AdventError> {
    let matches = HashMap::from([('[', ']'), ('{', '}'), ('<', '>'), ('(', ')')]);
    let mut expect = VecDeque::<char>::new();
    for c in line.chars() {
        if let Some(closing) = matches.get(&c) {
            expect.push_front(*closing);
        } else if matches.values().contains(&c) {
            let closing = expect
                .pop_front()
                .ok_or(AdventError::ClosingBeforeOpening { c })?;
            if closing != c {
                // println!("Expected {} but found {} instead.", closing, c);
                return Ok(LineResult::Corrupt(c));
            }
        } else {
            return Err(AdventError::InvalidChar { c });
        }
    }
    if expect.is_empty() {
        Ok(LineResult::Ok)
    } else {
        Ok(LineResult::Incomplete(expect))
    }
}

fn part_one_score_calculator(line_result: &LineResult) -> Option<usize> {
    // In question one, simply sum up the corrupt chars, each of which having a different
    // associated score
    match line_result {
        LineResult::Corrupt(c) => Some(match c {
            ')' => 3,
            ']' => 57,
            '}' => 1197,
            '>' => 25137,
            _ => unreachable!(),
        }),
        _ => None,
    }
}

fn part_two_score_calculator(line_result: &LineResult) -> Option<usize> {
    // In part two, multiply the previous score by 5, then add a different amount for each
    // missing closing brace
    match line_result {
        LineResult::Incomplete(rest) => Some(rest.iter().fold(0, |acc, c| {
            let score = match c {
                ')' => 1,
                ']' => 2,
                '}' => 3,
                '>' => 4,
                _ => unreachable!(),
            };
            acc * 5 + score
        })),
        _ => None,
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

    let score_calculator = match question_part {
        QuestionPart::One => part_one_score_calculator,
        QuestionPart::Two => part_two_score_calculator,
    };

    let mut score = input
        .iter()
        .map(|line| parse_line(&line[..]))
        .collect::<Result<Vec<LineResult>, _>>()?
        .iter()
        .filter_map(score_calculator)
        .collect::<Vec<_>>();

    match question_part {
        // In part only, we only care about the total score
        QuestionPart::One => Ok(score.iter().sum()),
        // In part two, score each line and take the median
        QuestionPart::Two => {
            score.sort_unstable();
            Ok(score[score.len() / 2])
        }
    }
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
