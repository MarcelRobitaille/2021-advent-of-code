use itertools::chain;
use lazy_static::lazy_static;
use regex::Regex;
use std::cmp::{max, min};
use std::env;
use std::io::{stdin, Read};
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

    #[error("Please specify `part-one' or `part-two' as the first argument.")]
    NoPartArgument,

    #[error("Invalid format. Chunk {i} does not match regex.")]
    FormatError { i: usize },
}

#[derive(Debug)]
struct Instruction {
    divisor: i32,
    x_increment: i32,
    y_increment: i32,
}

impl Instruction {
    fn parse(input: &str) -> Option<Self> {
        lazy_static! {
            static ref RE: Regex = Regex::new(
                r"mul x 0
add x z
mod x 26
div z (26|1)
add x (-?[0-9]+)
eql x w
eql x 0
mul y 0
add y 25
mul y x
add y 1
mul z y
mul y 0
add y w
add y (-?[0-9]+)
mul y x
add z y"
            )
            .unwrap();
        }

        let caps = RE.captures(input)?;

        // If the regex matches, it should be safe to unwrap
        Some(Self {
            divisor: caps[1].parse().unwrap(),
            x_increment: caps[2].parse().unwrap(),
            y_increment: caps[3].parse().unwrap(),
        })
    }
}

fn parse(input: &str) -> Result<Vec<Instruction>, AdventError> {
    input
        .split("inp w")
        .enumerate()
        .filter(|(_, part)| part != &"")
        .map(|(i, part)| Instruction::parse(part).ok_or(AdventError::FormatError { i }))
        .collect()
}

// This problem is similar to a stack machine, as discussed on the advent of code subreddit. A
// "push" occurs anytime the divisor is 1. A "pop" occurs anytime the divisor is 26. In the latter
// case, w must be equal to z % 26 + x_increment. Therefore, we must push a value in the former
// case to make this satisfiable.
//
// My solution assumes that every push operation has a matching pop operation. This means a push
// could be followed by a pop, two pushes could be folioed by two pops, and so on. The recurse
// function will match pushes to pops, processing any nested operations, and the remaining,
// following operations as appropriate
//
// For example, here are the calls to recurse for my input. The push and pop for one recurse are
// marked, and nested instructions are indented
// Recurse
// ├ { divisor: 1, x_increment: 10, y_increment: 13 }
// │ Recurse
// │ ├ { divisor: 1, x_increment: 13, y_increment: 10 }
// │ │ Recurse
// │ │ ├ { divisor: 1, x_increment: 13, y_increment: 3 }
// │ │ └ { divisor: 26, x_increment: -11, y_increment: 1 }
// │ │ Recurse
// │ │ ├ { divisor: 1, x_increment: 11, y_increment: 9 }
// │ │ └ { divisor: 26, x_increment: -4, y_increment: 3 }
// │ │ Recurse
// │ │ ├ { divisor: 1, x_increment: 12, y_increment: 5 }
// │ │ │ Recurse
// │ │ │ ├ { divisor: 1, x_increment: 12, y_increment: 1 }
// │ │ │ │ Recurse
// │ │ │ │ ├ { divisor: 1, x_increment: 15, y_increment: 0 }
// │ │ │ │ └ { divisor: 26, x_increment: -2, y_increment: 13 }
// │ │ │ └ { divisor: 26, x_increment: -5, y_increment: 7 }
// │ │ └ { divisor: 26, x_increment: -11, y_increment: 15 }
// │ └ { divisor: 26, x_increment: -13, y_increment: 12 }
// └ { divisor: 26, x_increment: -10, y_increment: 8 }
//
// Once pushes and pops are paired, I can calculate the maximum (or minimum, for part 2) value of
// the left digit that satisfies the range of the right digit. This is similar to the stack machine
// that I have seen many discussions about, but uses recursion rather than an actual stack (well,
// it uses the call stack). I first did a solution using a stack, but pushing a value onto the
// stack requires future information, which is why I pair up a push and a pop and calculate the
// value for both at once.

fn recurse(
    instructions: &[Instruction],
    question_part: QuestionPart,
) -> (&[Instruction], Vec<i32>) {
    // If we've exhausted input or we get to somebody's right hand side,
    // stop recursing
    if instructions.is_empty() || instructions[0].divisor == 26 {
        return (instructions, vec![]);
    }

    // Get the left instruction (divisor = 1)
    let (left, instructions) = instructions.split_first().unwrap();

    // Parse all the instructions in between this pair
    let (instructions, mid) = recurse(instructions, question_part);

    // Get the right instruction (divisor = 26)
    let (right, instructions) = instructions.split_first().unwrap();

    let left_output = match question_part {
        // Calculate the maximum value the left digit can be without making the right value go over
        // 9
        QuestionPart::One => min(9, 9 - left.y_increment - right.x_increment),
        // Calculate the minimum value the left digit can be in the same way
        QuestionPart::Two => max(1, 1 - left.y_increment - right.x_increment),
    };
    // Calculate right digit based on left digit, left y-increment, and right x-increment
    let right_output = left_output + left.y_increment + right.x_increment;

    // Get digits from the remainder of the input
    let (instructions, tail) = recurse(instructions, question_part);

    // Chain left, mid, right, and remainder
    (
        instructions,
        chain!([left_output], mid, [right_output], tail).collect::<Vec<i32>>(),
    )
}

fn day_24() -> Result<usize, AdventError> {
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

    let mut input = String::new();
    stdin().lock().read_to_string(&mut input)?;

    let instructions = parse(&input[..])?;

    let (instructions, ans) = recurse(&instructions[..], question_part);
    assert!(instructions.is_empty());

    let ans = ans
        .into_iter()
        .rev()
        .enumerate()
        .fold(0, |acc, (i, x)| acc + (x as usize) * 10_usize.pow(i as u32));

    Ok(ans)
}

fn main() {
    match day_24() {
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

    const INPUT: &str = "inp w
mul x 0
add x z
mod x 26
div z 1
add x 10
eql x w
eql x 0
mul y 0
add y 25
mul y x
add y 1
mul z y
mul y 0
add y w
add y 13
mul y x
add z y
inp w
mul x 0
add x z
mod x 26
div z 1
add x 13
eql x w
eql x 0
mul y 0
add y 25
mul y x
add y 1
mul z y
mul y 0
add y w
add y 10
mul y x
add z y
inp w
mul x 0
add x z
mod x 26
div z 1
add x 13
eql x w
eql x 0
mul y 0
add y 25
mul y x
add y 1
mul z y
mul y 0
add y w
add y 3
mul y x
add z y
inp w
mul x 0
add x z
mod x 26
div z 26
add x -11
eql x w
eql x 0
mul y 0
add y 25
mul y x
add y 1
mul z y
mul y 0
add y w
add y 1
mul y x
add z y
inp w
mul x 0
add x z
mod x 26
div z 1
add x 11
eql x w
eql x 0
mul y 0
add y 25
mul y x
add y 1
mul z y
mul y 0
add y w
add y 9
mul y x
add z y
inp w
mul x 0
add x z
mod x 26
div z 26
add x -4
eql x w
eql x 0
mul y 0
add y 25
mul y x
add y 1
mul z y
mul y 0
add y w
add y 3
mul y x
add z y
inp w
mul x 0
add x z
mod x 26
div z 1
add x 12
eql x w
eql x 0
mul y 0
add y 25
mul y x
add y 1
mul z y
mul y 0
add y w
add y 5
mul y x
add z y
inp w
mul x 0
add x z
mod x 26
div z 1
add x 12
eql x w
eql x 0
mul y 0
add y 25
mul y x
add y 1
mul z y
mul y 0
add y w
add y 1
mul y x
add z y
inp w
mul x 0
add x z
mod x 26
div z 1
add x 15
eql x w
eql x 0
mul y 0
add y 25
mul y x
add y 1
mul z y
mul y 0
add y w
add y 0
mul y x
add z y
inp w
mul x 0
add x z
mod x 26
div z 26
add x -2
eql x w
eql x 0
mul y 0
add y 25
mul y x
add y 1
mul z y
mul y 0
add y w
add y 13
mul y x
add z y
inp w
mul x 0
add x z
mod x 26
div z 26
add x -5
eql x w
eql x 0
mul y 0
add y 25
mul y x
add y 1
mul z y
mul y 0
add y w
add y 7
mul y x
add z y
inp w
mul x 0
add x z
mod x 26
div z 26
add x -11
eql x w
eql x 0
mul y 0
add y 25
mul y x
add y 1
mul z y
mul y 0
add y w
add y 15
mul y x
add z y
inp w
mul x 0
add x z
mod x 26
div z 26
add x -13
eql x w
eql x 0
mul y 0
add y 25
mul y x
add y 1
mul z y
mul y 0
add y w
add y 12
mul y x
add z y
inp w
mul x 0
add x z
mod x 26
div z 26
add x -10
eql x w
eql x 0
mul y 0
add y 25
mul y x
add y 1
mul z y
mul y 0
add y w
add y 8
mul y x
add z y";

    #[test]
    fn test_part_one() -> Result<(), AdventError> {
        let instructions = parse(&INPUT)?;

        let (instructions, ans) = recurse(&instructions[..], QuestionPart::One);
        assert!(instructions.is_empty());

        let ans = ans
            .into_iter()
            .rev()
            .enumerate()
            .fold(0, |acc, (i, x)| acc + (x as usize) * 10_usize.pow(i as u32));

        assert_eq!(ans, 69914999975369);
        Ok(())
    }

    #[test]
    fn test_part_two() -> Result<(), AdventError> {
        let instructions = parse(&INPUT)?;

        let (instructions, ans) = recurse(&instructions[..], QuestionPart::Two);
        assert!(instructions.is_empty());

        let ans = ans
            .into_iter()
            .rev()
            .enumerate()
            .fold(0, |acc, (i, x)| acc + (x as usize) * 10_usize.pow(i as u32));

        assert_eq!(ans, 14911675311114);
        Ok(())
    }
}
