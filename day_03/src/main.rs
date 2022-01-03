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

    #[error("Could not parse character `{c}'. Expected `0' or `1'.")]
    Bool { c: char },
}

macro_rules! dec {
    ($it:expr) => {
        $it.fold(0, |acc, x| (acc << 1) | x as usize)
    };
}

fn transpose<T>(v: Vec<Vec<T>>) -> Vec<Vec<T>> {
    assert!(!v.is_empty());
    let len = v[0].len();
    let mut iters: Vec<_> = v.into_iter().map(|n| n.into_iter()).collect();
    (0..len)
        .map(|_| {
            iters
                .iter_mut()
                .map(|n| n.next().unwrap())
                .collect::<Vec<T>>()
        })
        .collect()
}

fn part_one(input: Vec<Vec<bool>>) -> usize {
    // Transpose the input so we can loop through columns and select the bit value with the
    // highest frequency
    let input = transpose(input);

    let base_rate = input
        .iter()
        // Get a vec of zeros and a vec of ones and select the longest
        .map(|col| col.iter().partition(|x| **x))
        .map(|(a, b): (Vec<bool>, Vec<bool>)| a.len() > b.len())
        .collect::<Vec<_>>();

    // Flip every bit from base rate before converting to dec for epsilon rate
    // Do them backwards so we can into_iter without copying (dec does not work with
    // references)
    let epsilon_rate = dec!(base_rate.iter().map(|x| !x));
    // Don't flip for gamma rate
    let gamma_rate = dec!(base_rate.into_iter());

    // In part one, the answer is the product of gamma rate and epsilon rate
    gamma_rate * epsilon_rate
}

fn part_two(input: Vec<Vec<bool>>) -> usize {
    // Find the o2 and co2 ratings by filtering down the numbers select the most common bit or the
    // least common bit at each position until there is one number left

    fn find_rating(
        input: Vec<Vec<bool>>,
        i: usize,
        selector: impl Fn(Vec<Vec<bool>>, Vec<Vec<bool>>) -> Vec<Vec<bool>>,
    ) -> Vec<bool> {
        if input.len() == 1 {
            return input.into_iter().next().unwrap();
        }

        // Partition the numbers into two lists based on the bit value at position i
        let (ones, zeros) = input.into_iter().partition(|col| col[i]);

        // Call the selector function to get the most common or least common
        let input = selector(zeros, ones);

        // Recurse with the new input for the next position
        find_rating(input, i + 1, selector)
    }

    let o2_rating = dec!(find_rating(input.clone(), 0, |zeros, ones| {
        // Notice the strict equals for tie-breaking
        if zeros.len() > ones.len() {
            zeros
        } else {
            ones
        }
    })
    .into_iter());

    let co2_rating = dec!(find_rating(input, 0, |zeros, ones| {
        // Notice the strict equals for tie-breaking
        if ones.len() < zeros.len() {
            ones
        } else {
            zeros
        }
    })
    .into_iter());

    o2_rating * co2_rating
}

fn solve(input: &str, question_part: QuestionPart) -> Result<usize, AdventError> {
    let input = input
        .trim()
        .split('\n')
        .map(|line| {
            line.chars()
                .map(|c| match c {
                    '0' => Ok(false),
                    '1' => Ok(true),
                    _ => Err(AdventError::Bool { c }),
                })
                .collect::<Result<Vec<_>, AdventError>>()
        })
        .collect::<Result<Vec<_>, AdventError>>()?;

    Ok(match question_part {
        QuestionPart::One => part_one,
        QuestionPart::Two => part_two,
    }(input))
}

fn day_03() -> Result<usize, AdventError> {
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
    match day_03() {
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

    const INPUT: &str = "00100
11110
10110
10111
10101
01111
00111
11100
10000
11001
00010
01010
";

    #[test]
    fn test_part_one() -> Result<(), AdventError> {
        assert_eq!(solve(INPUT, QuestionPart::One)?, 198);
        Ok(())
    }

    #[test]
    fn test_part_two() -> Result<(), AdventError> {
        assert_eq!(solve(INPUT, QuestionPart::Two)?, 230);
        Ok(())
    }
}
