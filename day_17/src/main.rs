use itertools::iproduct;
use itertools::Itertools;
use regex::Regex;
use std::cmp::max;
use std::env;
use std::io::{stdin, BufRead};
use std::ops::{Add, RangeInclusive};
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

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Regex(#[from] regex::Error),

    #[error(transparent)]
    ParseInt(#[from] std::num::ParseIntError),

    #[error("Please specify `part-one' or `part-two' as the first argument.")]
    NoPartArgument,

    #[error("Invalid input. Expected `target area: x=<x1>..<x2>, y=<y1>..<y2>'. Found `{line}'.")]
    InputError { line: String },

    #[error("Uh oh. The calculated optimal launch ({v:?}) misses the target!")]
    MissedTarget { v: Vec2<i32> },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec2<T> {
    x: T,
    y: T,
}

impl<T> Add<Vec2<T>> for Vec2<T>
where
    T: Add<Output = T>,
{
    type Output = Vec2<T>;
    fn add(self, other: Self::Output) -> Self::Output {
        Vec2 {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

fn launch(
    target: &Vec2<RangeInclusive<i32>>,
    initial_velocity: Vec2<i32>,
) -> Option<Vec<Vec2<i32>>> {
    // Launch!
    // Recurse until we detect that we overshot or we hit the target

    fn recurse(
        target: &Vec2<RangeInclusive<i32>>,
        pos: Vec2<i32>,
        velocity: Vec2<i32>,
    ) -> Option<Vec<Vec2<i32>>> {
        if target.x.contains(&pos.x) && target.y.contains(&pos.y) {
            return Some(vec![pos]);
        }

        if velocity.y < 0 && pos.y < *target.y.start() || pos.x > *target.x.end() {
            return None;
        }

        recurse(
            target,
            pos + velocity,
            Vec2 {
                x: velocity.x - velocity.x.signum(),
                y: velocity.y - 1,
            },
        )
        .map(|history| history.into_iter().chain(vec![pos]).collect())
    }

    recurse(target, Vec2 { x: 0, y: 0 }, initial_velocity)
}

fn calc_x_velocity(target: RangeInclusive<i32>) -> i32 {
    // Calculate the initial x velocity to get the highest y value
    // I assume that if we're reaching the max, drag causes the x velocity to be zero by the end
    // Therefore, work backwards, initializing the velocity to zero
    // and incrementing it as we move the target
    // When zero is inside the translated target, then the current speed with reach the target
    fn recurse(target: RangeInclusive<i32>, velocity: i32) -> i32 {
        if target.contains(&0) {
            return velocity;
        }

        recurse(
            (target.start() - velocity)..=(target.end() - velocity),
            velocity + 1,
        )
    }

    recurse(target, 0)
}

fn print(history: &[Vec2<i32>], target: Vec2<RangeInclusive<i32>>) {
    for y in (-20..50).rev() {
        for x in 0..40 {
            let point = Vec2 { x, y };
            if x == 0 && y == 0 {
                print!("S");
            } else if history.contains(&point) {
                print!("#");
            } else if target.x.contains(&x) && target.y.contains(&y) {
                print!("T");
            } else {
                print!(".");
            }
        }
        println!();
    }
}

fn day_17() -> Result<i32, AdventError> {
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
    let input = input.trim();

    let re = Regex::new(r"target area: x=(-?\d+)\.\.(-?\d+), y=(-?\d+)..(-?\d+)")?;
    let (x1, x2, y1, y2) = re
        .captures(input)
        .and_then(|captures| {
            captures
                .iter()
                .skip(1)
                .filter_map(|x| x.map(|x| x.as_str().parse::<i32>()))
                .collect_tuple()
        })
        .ok_or(AdventError::InputError {
            line: input.to_string(),
        })?;

    let target = Vec2::<RangeInclusive<i32>> {
        x: x1?..=x2?,
        y: y1?..=y2?,
    };

    Ok(match question_part {
        QuestionPart::One => {
            // Part one we can calculate pretty easily
            let velocity = Vec2 {
                // Get the x velocity as explained above
                x: calc_x_velocity(target.x.to_owned()),
                // Get the y velocity as follows: on the way up, we decelerate by 1 every step,
                // on the way down, we accelerate 1 every step
                // By symmetry, we have one step (besides the initial launch) at y=0
                // Therefore, the highest initial velocity is at the minimum target y below y=0
                y: -target.y.start() - 1,
            };

            let history =
                launch(&target, velocity).ok_or(AdventError::MissedTarget { v: velocity })?;
            print(&history, target);

            // In part one, we just return the highest y reached
            // Unwrap because history returned something
            history.into_iter().map(|p| p.y).max().unwrap()
        }

        // Part two is tricky
        // I just brute-forced it (takes 0.13 seconds, who cares)
        // At least we can calculate the bounds
        // If x velocity greater than the right edge of the target, we will overshoot
        // If y velocity is smaller than the bottom of the target, we will overshoot
        // As explained below, the maximum initial y velocity is the inverse of the target y bottom
        QuestionPart::Two => iproduct!(
            0..=*target.x.end(),
            *target.y.start()..=max(-*target.y.start() + 1, *target.y.end())
        )
        .map(|(x_initial_velocity, y_initial_velocity)| {
            launch(
                &target,
                Vec2 {
                    x: x_initial_velocity,
                    y: y_initial_velocity,
                },
            )
        })
        .filter(|x| x.is_some())
        .count() as i32,
    })
}

fn main() {
    match day_17() {
        Ok(answer) => println!("{}", answer),
        Err(err) => {
            eprintln!("{}", err);
            exit(1);
        }
    }
}
