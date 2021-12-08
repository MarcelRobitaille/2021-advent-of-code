use itertools::Itertools;
use std::collections::{HashMap, HashSet};
use std::env;
use std::io::{stdin, BufRead};
use std::process::exit;
use thiserror::Error;

type Digit = HashSet<char>;
enum QuestionPart {
    One,
    Two,
}

#[derive(Error, Debug)]
pub enum AdventError {
    #[error("Invalid command `{command:?}'. Expected `part-one' or `part-two'.")]
    InvalidCommand { command: String },

    #[error("Invalid input")]
    InvalidInput,

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error("Please specify `part-one' or `part-two' as the first argument.")]
    NoPartArgument,
}

fn split_into_tuple(text: &str, separator: char) -> Option<(&str, &str)> {
    // Split a string into a tuple of strings without the separator
    match text.find(separator) {
        None => None,
        Some(i) => {
            let (left, right) = text.split_at(i);
            Some((
                left,
                // Remove the separator from the right part
                &right[1..],
            ))
        }
    }
}

fn partition_one<F: Fn(&Digit) -> bool>(
    haystack: Vec<Digit>,
    predicate: F,
) -> Result<(Digit, Vec<Digit>), AdventError> {
    // Find an element in a vector by a predicate, return the item and the vector without the item
    let i = haystack
        .iter()
        .position(predicate)
        .ok_or(AdventError::InvalidInput)?;
    let mut haystack = haystack;
    let one = haystack.remove(i);
    Ok((one, haystack))
}

fn extra_segment(a: &Digit, b: &Digit) -> Result<char, AdventError> {
    // Get the segment that is in a but not b as an owned char
    Ok((a - b)
        .iter()
        .next()
        .ok_or(AdventError::InvalidInput)?
        .to_owned())
}

fn part_one((_left, right): (Vec<String>, Vec<String>)) -> Result<usize, AdventError> {
    // Mapping from number of segments in a digit to the digit's numeric value
    // where the this is unique
    let segment_number_to_unique_digit = HashMap::from([(2, 1), (4, 4), (3, 7), (7, 8)]);

    // In part one, simply count the number of unique digits in the output
    Ok(right
        .iter()
        .map(|s| s.len())
        .filter_map(|x| segment_number_to_unique_digit.get(&x))
        .count())
}

fn part_two((left, right): (Vec<String>, Vec<String>)) -> Result<usize, AdventError> {
    let left: Vec<Digit> = left
        .iter()
        .map(|digit| HashSet::from_iter(digit.chars()))
        .collect();

    // Extract all the uniquely-sized digits
    let (one, left) = partition_one(left, |x| x.len() == 2)?;
    let (four, left) = partition_one(left, |x| x.len() == 4)?;
    let (seven, left) = partition_one(left, |x| x.len() == 3)?;
    let (eight, left) = partition_one(left, |x| x.len() == 7)?;

    // Three is the only digit with 5 segments that is a superset of one
    let (three, left) = partition_one(left, |x| x.len() == 5 && x.is_superset(&one))?;

    // Nine is the only digit with 6 segments that is a superset of three
    let (nine, left) = partition_one(left, |x| x.len() == 6 && x.is_superset(&three))?;
    // There are two digits with 6 segments remaining: zero and six
    // Zero is the only one that is a superset of one
    let (zero, left) = partition_one(left, |x| x.len() == 6 && x.is_superset(&one))?;
    // Six is the final digit with six segments
    let (six, left) = partition_one(left, |x| x.len() == 6)?;

    // a (top segment) is the only segment present in seven but not one
    let a = extra_segment(&seven, &one)?;
    // e (lower left) is the only segment present in eight but not nine
    let _e = extra_segment(&eight, &nine)?;
    // c (upper right) is the only segment present in one but not six
    let c = extra_segment(&one, &six)?;
    // f (lower right) is the other segment in one
    let _f = one
        .iter()
        .find(|x| x != &&c)
        .ok_or(AdventError::InvalidInput)?;
    // d (middle) is the only segment in eight and not zero
    let d = extra_segment(&eight, &zero)?;

    // There are two digits left: 2 and 5
    // Two is the one with a c-segment
    let (two, left) = partition_one(left, |x| x.contains(&c))?;
    let (five, _) = partition_one(left, |_| true)?;

    let e = extra_segment(&six, &five)?;
    let _g = extra_segment(&two, &HashSet::from([a, c, d, e]))?;

    // Get a mapping from segments to numeric values
    // Use sorted strings as keys because sets don't seem to play nice
    let map = [zero, one, two, three, four, five, six, seven, eight, nine]
        .iter()
        .enumerate()
        .map(|(i, x)| (x.iter().sorted().collect::<String>(), i))
        .collect::<HashMap<String, usize>>();

    // Get the numeric value corresponding to each number in right
    Ok(right
        .iter()
        .map(|x| {
            map.get(&x.chars().sorted().collect::<String>())
                .ok_or(AdventError::InvalidInput)
        })
        .collect::<Result<Vec<&usize>, AdventError>>()?
        // Convert digits to base 10 number
        .iter()
        .rev()
        .enumerate()
        .map(|(i, x)| *x * 10_usize.pow(i as u32))
        .sum::<usize>())
}

fn day_8() -> Result<usize, AdventError> {
    let args: Vec<String> = env::args().collect();
    let command = args.get(1).ok_or(AdventError::NoPartArgument)?;
    let question_part = match &command[..] {
        "part-one" => Ok(QuestionPart::One),
        "part-two" => Ok(QuestionPart::Two),
        _ => Err(AdventError::InvalidCommand {
            command: args[1].to_string(),
        }),
    }?;

    // Function that maps a line to its answer
    let answer_for_line = match question_part {
        QuestionPart::One => part_one,
        QuestionPart::Two => part_two,
    };

    let answer = stdin()
        .lock()
        .lines()
        .map(|line| {
            let line = line?;

            // Split left and right part of input
            let (left, right) =
                split_into_tuple(&line[..], '|').ok_or(AdventError::InvalidInput)?;

            // Parse left and right part into vecs of string
            let [left, right] = [left, right].map(|part| {
                part.trim()
                    .split(' ')
                    .map(|x| x.to_string())
                    .collect::<Vec<_>>()
            });

            Ok((left, right))
        })
        .map(|tuple| match tuple {
            Ok(tuple) => answer_for_line(tuple),
            Err(err) => Err(err),
        })
        .collect::<Result<Vec<usize>, AdventError>>()?
        .iter()
        .sum();
    Ok(answer)
}

fn main() {
    match day_8() {
        Ok(answer) => println!("{}", answer),
        Err(err) => {
            eprintln!("{}", err);
            exit(1);
        }
    }
}
