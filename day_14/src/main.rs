use itertools::Itertools;
use std::collections::HashMap;
use std::env;
use std::hash::Hash;
use std::io::{stdin, BufRead};
use std::ops::{Add, Div};
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

    #[error("Invalid input detected")]
    InvalidInput,

    #[error("Empty input. Maximum - minimum of nothing is undefined.")]
    EmptyInput,

    #[error("Invalid pair insertion in input. Expected `AB -> C', found `{line}'.")]
    PairError { line: String },

    #[error("Required pair insertion `{pair}' definition not found.")]
    MissingPair { pair: String },

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error("Please specify `part-one' or `part-two' as the first argument.")]
    NoPartArgument,
}

// Small wrapper around a hashmap to add some function programming niceties
// Allows doing things like `.fold(Counter::new(), |acc, x| acc.extend((x, 1))`
type Key = (char, char);
#[derive(Debug)]
struct Counter<T>
where
    T: Eq + Hash,
{
    counts: HashMap<T, usize>,
}

impl<T> Counter<T>
where
    T: Eq + Hash + Copy,
{
    fn new() -> Counter<T> {
        Counter {
            counts: HashMap::<T, usize>::new(),
        }
    }

    // Return the result of adding a value to the counter
    // Does not mutate, instead returns the result
    fn extend(self, tuple: (T, usize)) -> Counter<T> {
        let (key, count) = tuple;
        let prev = &self.counts.get(&key).unwrap_or(&0).to_owned();
        Counter {
            counts: self
                .counts
                .into_iter()
                .chain(HashMap::<T, usize>::from([(key, count + prev)]))
                .collect(),
        }
    }

    // Map a function over all the values
    // Does not mutate the counter, instead returns a new one with the result
    fn map_values<F>(self, mapper: F) -> Counter<T>
    where
        F: Fn(usize) -> usize,
    {
        Counter {
            counts: HashMap::<T, usize>::from_iter(
                self.counts
                    .into_iter()
                    .map(|(k, v)| (k, mapper(v)))
                    .collect::<Vec<(T, usize)>>(),
            ),
        }
    }
}

// Syntactic sugar to divide every value in a counter
impl<T> Div<usize> for Counter<T>
where
    T: Eq + Hash + Copy,
{
    type Output = Self;
    fn div(self, rhs: usize) -> Self::Output {
        self.map_values(|x| x / rhs)
    }
}

impl<T> Add<(T, usize)> for Counter<T>
where
    T: Eq + Hash + Copy,
{
    type Output = Self;
    fn add(self, rhs: (T, usize)) -> Self::Output {
        self.extend(rhs)
    }
}

type RecurseResult = Result<Counter<Key>, AdventError>;
fn recurse(
    pair_insertions: &HashMap<Key, char>,
    remaining: usize,
    counter: Counter<Key>,
) -> RecurseResult {
    // Recursively do pair insertions
    // Actually, track the count of each pair, which is much more efficient than tracking the
    // entire sequence, which grows exponentially

    if remaining == 0 {
        return Ok(counter);
    }

    recurse(
        pair_insertions,
        remaining - 1,
        counter
            .counts
            .into_iter()
            // Grow a new counter from zero
            // where giving the count of each pair
            // to the first element in the pair plus the inserter
            // and to the inserter and the second element in the pair
            .try_fold::<_, _, RecurseResult>(Counter::new(), |acc, (key, count)| {
                let inserted = *pair_insertions.get(&key).ok_or(AdventError::MissingPair {
                    pair: [key.0, key.1].into_iter().collect(),
                })?;
                Ok(acc + ((key.0, inserted), count) + ((inserted, key.1), count))
            })?,
    )
}

fn day_14() -> Result<usize, AdventError> {
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

    // Polymer template and pair insertions are separated by an empty line
    let (template, pair_insertions) = input
        .split(|l| l.is_empty())
        .collect_tuple()
        .ok_or(AdventError::InvalidInput)?;

    // Ensure only one line of polymer template is given
    if template.len() != 1 {
        return Err(AdventError::InvalidInput);
    }
    let template = template[0].to_string();

    // Parse the pair insertions into map from char char -> char
    let pair_insertions = pair_insertions
        .iter()
        .map(|l| {
            let (left, right) = l
                .split(" -> ")
                .collect_tuple()
                .ok_or(AdventError::PairError {
                    line: l.to_string(),
                })?;
            let (a, b) = left.chars().collect_tuple().ok_or(AdventError::PairError {
                line: l.to_string(),
            })?;
            Ok((
                (a, b),
                right.chars().next().ok_or(AdventError::PairError {
                    line: l.to_string(),
                })?,
            ))
        })
        .collect::<Result<Vec<(Key, char)>, AdventError>>()?;
    let pair_insertions = HashMap::<Key, char>::from_iter(pair_insertions);

    // Count pairs in template
    let counter = template
        .chars()
        .into_iter()
        .tuple_windows::<(_, _)>()
        .fold(Counter::new(), |acc, x| acc + (x, 1));

    let steps = match question_part {
        QuestionPart::One => 10,
        QuestionPart::Two => 40,
    };

    // Recursively find the pair counts after 10/40 steps
    let counter = recurse(&pair_insertions, steps, counter)?;

    // Counter of element pairs to counter of individual elements
    let element_counts = counter
        .counts
        .into_iter()
        // Assign pair count to each element in pair
        .fold(Counter::new(), |acc, (key, count)| {
            acc + (key.0, count) + (key.1, count)
        })
        // Add 1 to all odd values before dividing by 2
        // The first and last element in the template should not be divided by two,
        // since they are the only two that are not doubled.
        // If they are divided by two using integer division, they will be truncated down to the
        // wrong value
        // Add 1 to any odd numbers (first and last in template) to double them, then all are
        // double and all can be divided by 2
        .map_values(|v| match v % 2 {
            0 => v,
            1 => v + 1,
            _ => unreachable!(),
        })
        // Divide by two because adjacent pairs both count each element (except for first and last
        // in template, as explained above)
        / 2;

    // Answer is max - min
    let max = element_counts
        .counts
        .values()
        .max()
        .ok_or(AdventError::EmptyInput)?;
    let min = element_counts
        .counts
        .values()
        .min()
        .ok_or(AdventError::EmptyInput)?;
    Ok(max - min)
}

fn main() {
    match day_14() {
        Ok(answer) => println!("{}", answer),
        Err(err) => {
            eprintln!("{}", err);
            exit(1);
        }
    }
}
