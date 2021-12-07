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
    #[error("Invalid input")]
    InvalidInput,

    #[error("Invalid command `{command:?}'. Expected `part-one' or `part-two'.")]
    InvalidCommand { command: String },

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error("Please specify `part-one' or `part-two' as the first argument.")]
    NoPartArgument,
}

fn part_one(days_remaining: i32, state: &[i8]) -> Result<usize, AdventError> {
    // For part one, I modeled it exactly as described
    // I keep the number of each fish in a big vec and updated it each day,
    // then returned the length

    if days_remaining == 0 {
        return Ok(state.len());
    }
    let state = state
        .iter()
        .map(|x| match x {
            0 => vec![6, 8],
            _ => vec![x - 1],
        })
        .flatten()
        .collect::<Vec<_>>();
    // println!("Days remaining {} days: {:?}", days_remaining, state);

    part_one(days_remaining - 1, &state[..])
}

#[memoize]
fn part_two(x: i8, i: i32) -> usize {
    // For the second part, the previous method did not work
    // The size of the state vector grows exponentially with days,
    // as it doubles in size on average every 7 days

    // I tried to do something clever similar to 2^(days/7),
    // but the pesky "new fish take slightly longer" complicates this

    // I worked out this recursion on some paper, and after memoizing,
    // it can be run very quickly

    if i == 0 {
        return 1;
    }
    match x {
        0 => part_two(6, i - 1) + part_two(8, i - 1),
        _ => part_two(x - 1, i - 1),
    }
}

fn day_6() -> Result<(), AdventError> {
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

    let initial_state = input
        .trim()
        .split(',')
        .map(|x| x.parse().map_err(|_| AdventError::InvalidInput))
        .collect::<Result<Vec<i8>, AdventError>>()?;

    let days = match question_part {
        QuestionPart::One => 80,
        QuestionPart::Two => 256,
    };
    let result = match question_part {
        QuestionPart::One => part_one(days, &initial_state[..])?,
        QuestionPart::Two => initial_state.iter().map(|x| part_two(*x, days)).sum(),
    };
    println!("{}", result);
    Ok(())
}

fn main() {
    day_6().unwrap_or_else(|err| {
        eprintln!("{}", err);
        exit(1);
    });
}
