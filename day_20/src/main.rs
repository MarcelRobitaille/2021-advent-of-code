use itertools::Itertools;
use ndarray::prelude::*;
use ndarray::{Array, Data, ShapeError, Slice};
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
    Shape(#[from] ShapeError),

    #[error(transparent)]
    ParseInt(#[from] ParseIntError),

    #[error("Please specify `part-one' or `part-two' as the first argument.")]
    NoPartArgument,

    #[error("Invalid input format detected.")]
    FormatError,

    #[error("Failed to parse character `{c}'. Expected `.' or `#'.")]
    Bool { c: char },

    #[error("Invalid format. Expected algorithm then image separated by an empty line. Found `{input}'.")]
    Format { input: String },
}

macro_rules! print_arr {
    ($arr:expr) => {
        // Pretty print the array like in the website

        for row in $arr.rows() {
            println!(
                "{}",
                row.iter().map(|x| if *x { '#' } else { '.' }).join("")
            );
        }
    };
}

fn to_bool(c: char) -> Result<bool, AdventError> {
    // Convert a character to a bool in the format of the problem statement
    match c {
        '.' => Ok(false),
        '#' => Ok(true),
        _ => Err(AdventError::Bool { c }),
    }
}

pub fn pad<A, S, D>(
    arr: &ArrayBase<S, D>,
    pad_width: Vec<[usize; 2]>,
    const_value: A,
) -> Array<A, D>
where
    A: Clone,
    S: Data<Elem = A>,
    D: Dimension,
{
    // Pad an ndarray with given value
    // Adapted from: https://github.com/rust-ndarray/ndarray/issues/823
    assert_eq!(
        arr.ndim(),
        pad_width.len(),
        "Array ndim must match length of `pad_width`."
    );

    // Compute shape of final padded array.
    let mut padded_shape = arr.raw_dim();
    for (ax, (&ax_len, &[pad_lo, pad_hi])) in arr.shape().iter().zip(&pad_width).enumerate() {
        padded_shape[ax] = ax_len + pad_lo + pad_hi;
    }

    let mut padded = Array::from_elem(padded_shape, const_value);
    let padded_dim = padded.raw_dim();
    {
        // Select portion of padded array that needs to be copied from the
        // original array.
        let mut orig_portion = padded.view_mut();
        for (ax, &[pad_lo, pad_hi]) in pad_width.iter().enumerate() {
            orig_portion.slice_axis_inplace(
                Axis(ax),
                Slice::from(pad_lo as isize..padded_dim[ax] as isize - (pad_hi as isize)),
            );
        }
        // Copy the data from the original array.
        orig_portion.assign(arr);
    }
    padded
}

fn convolve(image: Array<bool, Ix2>, algo: &[bool]) -> Result<Array<bool, Ix2>, AdventError> {
    // Convolve the algorithm over the image
    // Make 3x3 windows, convert to a number, use this key to index the algorithm, outputting a new
    // image smaller by 2 in each dimension
    Ok(Array::from_iter(
        image
            .windows((3, 3))
            .into_iter()
            .map(|window| {
                window
                    .into_iter()
                    .fold(0, |acc, x| (acc << 1) | *x as usize)
            })
            .map(|key| algo[key]),
    )
    .into_shape((image.nrows() - 2, image.ncols() - 2))?)
}

fn step(image: Array<bool, Ix2>, algo: &[bool], i: u8) -> Result<Array<bool, Ix2>, AdventError> {
    // One enhancement step
    // First, pad the image because it's "infinitely" big
    // We need to pad with 2 because the center of the 3x3 window should sometimes fall in the
    // padded area
    // Pad with algo[0] if it's an odd step, else false
    // If algo[0] == true, then the invite grid will be flipped on each enhancement, so we need to
    // pad with true every other time
    let padding = vec![[2, 2], [2, 2]];
    let image = pad(&image, padding, i % 2 == 1 && algo[0]);
    convolve(image, algo)
}

fn enhance(
    image: Array<bool, Ix2>,
    algo: &[bool],
    times: u8,
) -> Result<Array<bool, Ix2>, AdventError> {
    // Enhance the image n times
    if times == 0 {
        Ok(image)
    } else {
        enhance(step(image, algo, times)?, algo, times - 1)
    }
}

fn solve(input: &str, question_part: QuestionPart) -> Result<usize, AdventError> {
    // Input is one line of algorithm then image separated by an empty line
    let (algo, image) = input
        .splitn(2, "\n\n")
        .collect_tuple()
        .ok_or(AdventError::Format {
            input: input.to_string(),
        })?;

    // Algorithm into bit vec
    let algo = algo
        .chars()
        .map(to_bool)
        .collect::<Result<Vec<bool>, AdventError>>()?;

    // Turn image into vec of lines to get dimensions
    let image = image.trim().split('\n').collect::<Vec<_>>();
    let width = image[0].len();
    let height = image.len();

    // Convert the image to one long bit vec
    let image = image
        .join("")
        .chars()
        .map(to_bool)
        .collect::<Result<Vec<bool>, AdventError>>()?;

    // Convert image to 2D array
    let image = Array::from_iter(image.into_iter()).into_shape((height, width))?;

    print_arr!(image);

    // Get the number of times to enhance from the question part
    let times = match question_part {
        QuestionPart::One => 2,
        QuestionPart::Two => 50,
    };

    let image = enhance(image, &algo, times)?;

    // Filter only lit pixels and count
    let num_lit = image.into_iter().filter(|x| *x).count();

    Ok(num_lit)
}

fn day_20() -> Result<usize, AdventError> {
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
    match day_20() {
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

    const EXAMPLE_INPUT: &str = "..#.#..#####.#.#.#.###.##.....###.##.#..###.####..#####..#....#..#..##..###..######.###...####..#..#####..##..#.#####...##.#.#..#.##..#.#......#.###.######.###.####...#.##.##..#..#..#####.....#.#....###..#.##......#.....#..#..#..##..#...##.######.####.####.#.#...#.......#..#.#.#...####.##.#......#..#...##.#.##..#...##.#.##..###.#......#.#.......#.#.#.####.###.##...#.....####.#..#..#.##.#....##..#.####....##...##..#...#......#.#.......#.......##..####..#...#.#.#...##..#.#..###..#####........#..####......#..#

#..#.
#....
##..#
..#..
..###";

    const REAL_INPUT: &str = "#.#..#.#...##.###.##.#.###....#..##..#.#####.#.##.#.##..##..#.#.######..##..####.#....#.#....##....#####..#######..###..#.##..#....#....#....#..#..#..##...####..###.##..##..#.#.#.#.#.#.#..##.###.##.#.##.#.##.#####..###.#.#...##..##...###...###..##...##.######....####.###...####.##.....###.##.#.##.#.....##.##..###.....#..##....#.##.#...##.###.###.#..####....#.###...#....#..###...##..#####..#.######..#.#....####.####.#.#....#.###..##...#.###.####....#.##.#....##...##.#..#....#.##...#....#.####..#.#..####.#...

####.#.#####..#.#####..###.###...##...##.#.##.###..#...#......##.##..##.####.........#..##...#.####.
#.#.###...###.####....##..##......##.#.####...#########.#.###.#.#.#..###..##...###.###.#.######.#.#.
##...#.#..##.#.##...#..##.####.###.###..###...#####.##.#..##.#.#####.###...#.###.....#######..##...#
####..#.###..####..##...#..######.##.#...#.#..#...#.##..####...##...#.#.#.##.##......##.##.#.##.....
.#########.###.###..#......##.###.####.#..###.#.##.####.#..###....##.##.##.#..##..#..##..#.##.#.#...
..#.#.#...#.###...##..#.#....#.###..#.##...#.#....#.##.#.##.##..###.##.####..#.###.#.####..##....###
.##....#.#.##.....####..#..##...#..............#...###.####....#..#....##.####..###....##...#.###.##
.#.########..###.#..###.#..###.#.##.#.#.#.###...##..##.#.#..#..##.#......###.###..#.##.####.....#...
...###.#...#.##..#...##..###...###...#....#..#.#######...#..##..###.#.#......#.#..###..#.#.###.#..##
..#####...##.##.....#.#####....#..#.#......#.#.#....####.##..#.##.##..##.#########.##.###...###...##
#...#...#####....##.#.#.###......##.#####.#.###.#.#.#.####...##.#..#.#.#..#.#..####..##...#.#....#..
.#..#.##.#.#.##.#..#.######.#.#....#.##.#..#.#.##...####..##.##..####.####.#...#...#####.#....####..
.....#.##..#.#.###.##....##....##.##.#...###.#.#..#.##.###.#...#...#.#####..#.##.....#...#.#.#.#....
....##.##...##..##.########...###...###..###..#..#...####..#..#..####.#..#.#.....###.#..#.#...#..#..
...#.#.#.###...#...#####.#....##..####..##.####.#..###..#.##.###...#...#..#.##...##.#...#.....###...
##....#..#.#...#.......#.##.######....#..##.##..#...###.#..###..#.#..##.#.........##..###.#......#.#
###...#########.#....###...#.#...#.#..#..##.##.#..#.#......##.#.##.#....#########...#.#.##..########
.###...#######.#......####.#.##.###.##.#..##.###.##..########.#..##.##..#..#..#...#.#..###..##...###
..#...##.....#.#..###.#..#.##.#..#.#.#.##.#####....####.#.#..#...##.##.##.#..####.#..#.#####......#.
#...##..##########.......#.....##.####..#####..##...#...##..###.#.#.#.#.#####.####.##.##.#.#.....###
.#.###.#..#..###.#..#.#.#....#.#####.##....#.#.#..##.#.#.#.###.#..#..##..#..###....#...##.#..#.#.#.#
...###.#..#....##.#...#...#..#....###...#.....#..####.##..#.###...#....###..##.#.##...#..##..#.##.#.
#.#.###.#..#....##.#..#####..###...##.##...#...###.#.##.#.#.#...#.#.#....####..#.##..##...#.#.##...#
.#..#..#..##########.#.###.#..##..##.#.##.#..#.##..#..#.#.##....##..##....##.######..#....#.#..#####
...#.##.#.###.#..#####.#........#.#.####.##..##..#.#..##...#..#..#..#.#.#.#..#..#.###..#..#.####.###
.###...###.#..#..#....##.#.##.#.....#..#.#.#...##.##.#...####..##..#.#..#...#....###.#..#####.#####.
...#.#.##.#......#.#....#####..#.#..##..#.#.#..###.##...#..#..##...#...#######.###.#####.###...#....
.##....#...###..####....#.#..##.#.#.##..###########...#.###..#.##.##.##.#....##.##..####......###...
#.##......#.##.###.##...##..#.##.#.##.##..#..#.#####....##.##.###.#########.#..###.....#....##..##.#
#.####.#.#...#.#.#.##..##.###..###.#..#..#####...####.####....#.##...####..#######.#....#######.#.#.
##...#..#...##...#.#...#.##.#......#.#.#.....#.#.#....#.#.#...#.#.#######.##.#.#..##..#....#.#..#..#
#..##..#.#..#.#..#.#.#.#.#....#.#.##.##.#.##.....##...##...##.#####.#.###.#..#..####.#.#..#######.#.
.#.##..#.##..#....##.#.#..#..##..##...#...######.###....#..#...##..#.#..........#.######.........#..
###.#.#..#.#..#.#.#.##..######.#.......###.###.#..#....#.##.#####.#.###.##.####.#.##..##.##.####.#..
##.#...#.....#...##.##.##....########......#.#.#...#.#..#.#....###...##.##...........####.##.##..#.#
####.###.#.#...#.##....#.##..###..#.#.#####..####.#.#..#..#....#...#..#...##...#..#.###..###..#.#.##
#.#..#..#.###....##.#..##.#..#...##..##..#.#.#.#..#...##.######..#.#..####.#.###..####..##...#.#...#
.###...#...#####...#.#...#.##..#...###.##.###...#..#.#.#.#...#.##.##.##.##.##...#...#.###.#.#......#
#.......#.#....####.....#..#.#.#..##.#####..##..#..###.#..#..##...####....##.#.#..###...#.##.#.###..
#.#..#.#.##.#.#####.#######.#####.#..###..#.#.......#.##...#..##..##....#.#..######.....##.##..##.##
#####.#....######.....##.#...#..##....#...###.##..#..##..###.###..###.####..#...#######....######...
#####.........#....#....#..#####.###.#..#.#....##.##..###..#..#.#...#.##...#.#..##..#.#.###.####.#.#
...###.#####.#.##..###.....#.#.......##...#....#.#.##.#......#.#...#.#..#...#.####.######.##...##.##
#...#..#..#..#...##.#.....#.#.##..####..##.#..#.#.#####.#.##.###.#....###...######........#..#..##..
..##.......##.##.#....###.#...#...#.##.###..###.#..####.#.######.#..#######.......#.##..#.####...#.#
......##.##..#####.##....#..#..##.#...#...####..##...##.##.#....#....#.#.###.....##..##.#...##....#.
####...#......#.#........##.#..#...#.#.###..#....##.###.#..#.#.##.##......#.#...#.##...##.###..#.###
.##....##..##.########.##..##..#.#.....#..#...#.#.#.#..#.##..#.##.#..#.#....####......#..#...#.#.#..
.##.#...#.##.##.#.#..###..#.#...##.#.#.#.##..#..######..##.#.##....###.#....##..#.###.##.##..#...#.#
..#.#.#...#...###..##....#.##..#..#...#....#.#...#####...##.####.#.#...#..#.##.###...#...#..#.##....
...#.####.###.#...#####..####.####..#.##...########...#.#...#.##.#.#..##.#..#..##.###.#####..##.##.#
##.#.#..##.###.##.#......###.#..#.#..#.#...#.#.#...#.##.###.#####.##..##...##.#..#..#.....#.###....#
......##.#..###....#.#####.###.###.###...#.....####.###.........#....###.####.#.......#..#........#.
...####.##....#.#.#.#.##..#..#.#..##....#...##..#...#.#..#.####..#.##.##.##......###..#...####..#.##
##.#.#####.....#...####.##.#.#.####.#......#.######...######.##.######....##..######..##.#...##.#..#
..#.##.......#.#####.#......###.###.##..####.#.###....#.###.#..#..#########..#..#..##.##.#....#....#
....####...#.#.#...####.#.#..###...#...#..##.##..####.#.#..##.#.#.#.###...........##..##..####.####.
#.#..###..######..##.##..#...#.#####......#.....######.##..#.#####...##..####.###.#..######...#.###.
..#####.#..#...#.####.###.#..##....####.#.#.#.#.##.####.#.##.###.###...#..#...#.##..##...#.....#....
..#.##.#..##....####.#.....##.##...##.#.#.####.##.#..####...###..#.#.######.#.###..#..#...#....##.##
#...##.#...#.###...#.##...#.#..###.#####.#....#########....##.#..###..##......#.#.........#.#...##.#
##..##.#.#...#..#..#####..#.#....#...###.##..#.####..#......##..##..##..#....#..#####.#.#.####.##..#
#.###...###..#.......#.#.##.##..###.####...##.##.#..#.#..#..#.###...#..######......##...##.##.#..#.#
.####.#...##.#..#...#.###..##.#..##.....###..###..##.####...#....#######.#.############.##.....##.##
#..####.#..#........##.#.#.#####.#.#.#.#.##.#..#...#..##...#...#.#..##.###.#.#######.#.#####..#.###.
#######...#####.#.##..####.#......##.#..#.###.####.#......####.#.##.#..##...#.....#.##.####.##.....#
#.....#......##...#.#.##.####.######...#.....#..##..##.####....##..##..#.#.##..###.##...#.##..#..###
....##.#..#.#..##.####.#.##..##.#.##.##.#..##########.#..##.....##....#.##.#...#.#..#..#..#.#.#.###.
#......###......###.#.#.#..##....##..##..##.#..#.#.####.####...##.#...##.####..#.....#...#.#.#####..
###..###.#...#..###.#......####.##.#.#.###...#..........##......####.###.###.#..#.....#####..####.##
..#.####..#...##.###...####.##.......#.##.##.#.#.#.##....#..###.....#....###.#..#...######.#...#..#.
#..#..#.#..#...#.#.###.####.##........#.##.####..#..############.#.###...###.#######.#.#....#...###.
#.######.###.##.#..#####.#.#....##.#.##..#######..##...##...#...##..##...#.##.##.#####...##.#...##..
##.###.###..##..##.#.#.##..#...#.#...##.#.........#.#.#.......###...#.##..#.##.#.##.####...##...###.
..#.##..####.##...##....###.#...#..#..##.##..###..###.###...#.#.#.####..#.#.##...#.#..####...#..##.#
#.#.#..#....#.#.#.#......#..#..#...#.#...#...#..#..#.##..#..#....#.##..#..#...##.######..#..##.#####
.#..##..#.#...#..#..##....##.....#....##.#..######...##.##.......#.#.....#####.####.#.##.#.#.##.###.
##.#..##.#...#..#.####...###..##.#####....#..#...#.##...#.#..###...####.#.##....#..#.#.##.#..####...
#.#######...##.####..#...#...##.#.###..#..###.#.#..#.###.#.##.#.###....#.##..###..#.#.########......
###.##.###.#...##....#.#..##.###.#..#.#.##.#.##....#.#####...###..###..#..##..#..#.#..#####.###..#..
.###.###.##..##..#######.##########.#.###.#.#.#....#..#.###...##.##.....#..#....###.....#....##...##
##...###.##.##....#.#....#####..#.######.##.#########..#.#...#.##....#.####.#.###.##.####.#.#.#####.
###.###.....##.###.#..##.#...#......#.###.#..##.#...###..##.#.#.#...###.#.#...##..##....#####....##.
#.#..#..###.##.##.##....#..###.#.###..#######.#...####..#####..#.......#..#.#.##.#..##..#.####..##.#
..##....##...###.###...#.###.....####.#..###.#.........#....####.#..#.#.##...#...###...#.##.....###.
.#..#.##.#.#.#.#.#..###.#..#.#.#######.####.###..#.#.##.......##.#...#...##..##.##.#.#..##.###.###..
#.#..#...#...###.#.#....#..#..###.#####.#.#.#..##.#.#........###.#.#...#.....#..###..##.##..###..#..
....#.#..#..##..#....###....#....###..##...#.#....#.###.####..#.#.##.####.#..#..##.....#.##.#...#.#.
...#..#.#.#..#.###...###...####..#....###.##.#..###.##.#.#.#.##...#...#.#...##....#.#.##.#.##..#.#..
##.#..###...#...##..#....###.#.##....##.#.##.#.##....####..#...#.#####.....#.##.#..#.#.###.#....#..#
#.#.##..#..##.##..##.##.#####.####..#..#..#....###.##.#..#####.##.#####..######.#...#...#..#.#..###.
..#.#......#..##.#...#.#..###..####.##.##..#..#####.##....#.##.....###.###........##.......##.#.####
##..##.#.#.#..##..#.#.#..##.#.######...##..##...##..#####.#...#.#.....###.#...##....#...#.#.#...###.
#......#.###.##.#####..##.#..#.#...######....#..#....#...#####.#.#.#.#.#...####..#...#.....#..#.#.##
..##.##..####...##.#...##...#####...#...#.##.###..#....####.#..#..#.####.##..##.#..####..#.#..####..
#####.#...#.##..##.#..####.#.#..#.#....#.####.#.####..#..####...#.#..##.#.#....#####..#..####..#...#
..#.......##.#..##.#..#..#.######.#...#.....###.###.####..#.#....#..#...#.#..#..##...#.##.#######.##
#.##..##..#.#.#####.........###..##.#.#...###.#.###...###.#...###..##..#.###..#.#.#....####...##....
##....#.##.##.#..###...#.#.##..#.#......#...#.#.#.#.#..###.#.....#.##.##..#.###..##..###.##....#####
...#.#..#...#..##...##....##....#.#..###..#....#..#.#.###.....##....#..#....##.###....#.#..#.#.#.#..";

    #[test]
    fn test_part_one() -> Result<(), AdventError> {
        assert_eq!(solve(EXAMPLE_INPUT, QuestionPart::One)?, 35);
        assert_eq!(solve(REAL_INPUT, QuestionPart::One)?, 5503);
        Ok(())
    }

    #[test]
    fn test_part_two() -> Result<(), AdventError> {
        assert_eq!(solve(EXAMPLE_INPUT, QuestionPart::Two)?, 3351);
        assert_eq!(solve(REAL_INPUT, QuestionPart::Two)?, 19156);
        Ok(())
    }
}