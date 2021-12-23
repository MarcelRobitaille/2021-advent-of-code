use lazy_static::lazy_static;
use regex::Regex;
use std::cmp::{max, min};
use std::env;
use std::io::{stdin, BufRead};
use std::ops::Sub;
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

    #[error("Please specify `part-one' or `part-two' as the first argument.")]
    NoPartArgument,

    #[error("Format error. Expected `(on|off) ...cuboid...' but no space found in line `{l}'.")]
    NoSpace { l: String },

    #[error("Format error. Expected `on' or `off' but found `{state}'.")]
    InvalidState { state: String },

    #[error("Does not match cuboid specification: `{l}'.")]
    FormatError { l: String },
}

// Axis which can be used to construct rectangle or cuboid
#[derive(Debug, Clone, Copy, PartialEq)]
struct Axis {
    start: i64,
    end: i64,
}

impl Axis {
    fn new(start: i64, end: i64) -> Self {
        Self { start, end }
    }
    fn is_empty(&self) -> bool {
        self.end < self.start
    }
    fn extent(&self) -> i64 {
        // Inclusive so plus 1
        self.end - self.start + 1
    }

    fn limit(&self, start: i64, end: i64) -> Option<Self> {
        // Restrict an axis down to start..end or None if it's outside of that range
        if self.start > end || self.end < start {
            None
        } else {
            Some(Self {
                start: max(self.start, start),
                end: min(self.end, end),
            })
        }
    }
}

// Rectangle, used for projections of cuboids
#[derive(Debug, Clone, PartialEq)]
struct Rect {
    x: Axis,
    y: Axis,
}

impl Rect {
    // Check if two rectangles intersect
    // If bottom left above or right of other's top right
    // or top right below or left of other's bottom left
    // then they don't intersect
    fn intersects(&self, other: &Rect) -> bool {
        (self.x.start <= other.x.end)
            && (self.x.end >= other.x.start)
            && (self.y.start <= other.y.end)
            && (self.y.end >= other.y.start)
    }
}

#[derive(Debug, Clone, PartialEq, Copy)]
struct Cuboid {
    x: Axis,
    y: Axis,
    z: Axis,
}

impl Cuboid {
    fn new(x: Axis, y: Axis, z: Axis) -> Self {
        Self { x, y, z }
    }

    // Projections for each axis into rectangle
    // The map from y, z to x, y is sometimes arbitrary, but as long as it's consistent, it should
    // be logically correct
    fn project_x(&self) -> Rect {
        Rect {
            x: self.y,
            y: self.z,
        }
    }

    fn project_y(&self) -> Rect {
        Rect {
            x: self.x,
            y: self.z,
        }
    }

    fn project_z(&self) -> Rect {
        Rect {
            x: self.x,
            y: self.y,
        }
    }

    fn intersects(&self, other: &Self) -> bool {
        // Two cuboids intersect if and only if all three of their projections intersect
        self.project_x().intersects(&other.project_x())
            && self.project_y().intersects(&other.project_y())
            && self.project_z().intersects(&other.project_z())
    }

    // Get a clone of the cuboid where the given axis is changed
    fn where_x(&self, x: Axis) -> Self {
        Self {
            x,
            y: self.y,
            z: self.z,
        }
    }

    fn where_y(&self, y: Axis) -> Self {
        Self {
            x: self.x,
            y,
            z: self.z,
        }
    }
    fn where_z(&self, z: Axis) -> Self {
        Self {
            x: self.x,
            y: self.y,
            z,
        }
    }

    fn limit(&self, start: i64, end: i64) -> Option<Self> {
        // Restrict a cuboid down to start..end in all dimensions
        // or None if any axis is outside of that range
        Some(Self {
            x: self.x.limit(start, end)?,
            y: self.y.limit(start, end)?,
            z: self.z.limit(start, end)?,
        })
    }

    fn has_volume(&self) -> bool {
        // Whether a cuboid has (positive) volume
        !self.x.is_empty() && !self.y.is_empty() && !self.z.is_empty()
    }

    fn volume(&self) -> i64 {
        // Get a cuboid's volume (width * length * height)
        self.x.extent() * self.y.extent() * self.z.extent()
    }

    pub fn parse(line: &str) -> Result<Self, AdventError> {
        // Parse a cuboid from a string in the same format as the challenge
        // Also useful to concisely instantiate in tests
        lazy_static! {
            static ref RE: Regex = Regex::new(r"x=(?P<x1>-?\d+)\.\.(?P<x2>-?\d+),y=(?P<y1>-?\d+)\.\.(?P<y2>-?\d+),z=(?P<z1>-?\d+)\.\.(?P<z2>-?\d+)")
            .unwrap();
        }
        let caps = RE.captures(line).ok_or(AdventError::FormatError {
            l: line.to_string(),
        })?;

        // We've already checked that we match, so given this regex, it's safe to also unwrap
        let [x1, x2, y1, y2, z1, z2] = ["x1", "x2", "y1", "y2", "z1", "z2"]
            .map(|which| caps.name(which).unwrap().as_str().parse::<i64>().unwrap());

        Ok(Cuboid::new(
            Axis::new(x1, x2),
            Axis::new(y1, y2),
            Axis::new(z1, z2),
        ))
    }
}

impl<'a> Sub<&'a Cuboid> for &'a Cuboid {
    type Output = Vec<Cuboid>;
    fn sub(self, other: Self) -> Self::Output {
        // This is kind of the secret of this implementation
        // Return a set of cuboids whose union is the difference between `self` minus `other`
        // These returned cuboids must not intersect

        // Trivial case. If other doesn't intersect us, just return us
        // This is actually required and not an optimization. The rest of the algorithm does not
        // work if not intersecting
        if !self.intersects(other) {
            return vec![*self];
        }

        // First, do the x axis and find `left` and `right`, which are the part of ourself to the
        // left of other's left face and the part of ourself to the right of other's right face,
        // respectively
        // `shrink_x` becomes the new search space. The returned cuboids cannot intersect,
        // so we now have to consider ourselves - left - right
        let shrink_x = self.where_x(Axis::new(
            max(self.x.start, other.x.start),
            min(other.x.end, self.x.end),
        ));
        let left = self.where_x(Axis::new(self.x.start, shrink_x.x.start - 1));
        let right = self.where_x(Axis::new(shrink_x.x.end + 1, self.x.end));

        // Same logic as x except now we are searching in the reduced space (-left-right)
        let shrink_y = shrink_x.where_y(Axis::new(
            max(self.y.start, other.y.start),
            min(other.y.end, self.y.end),
        ));
        let back = shrink_x.where_y(Axis::new(shrink_x.y.start, shrink_y.y.start - 1));
        let front = shrink_x.where_y(Axis::new(shrink_y.y.end + 1, shrink_x.y.end));

        // Same logic again but for z, searching in the space reduced in x and y
        let shrink_z = shrink_y.where_z(Axis::new(
            max(self.z.start, other.z.start),
            min(other.z.end, self.z.end),
        ));
        let bottom = shrink_y.where_z(Axis::new(shrink_y.z.start, shrink_z.z.start - 1));
        let top = shrink_y.where_z(Axis::new(shrink_z.z.end + 1, shrink_y.z.end));

        // Filter out all the cuboids with negative volume (that means that `other` was sticking
        // out of ourselves on that side)
        [left, right, back, front, bottom, top]
            .into_iter()
            .filter(|c| c.has_volume())
            .collect()
    }
}

fn parse(line: &str) -> Result<(bool, Cuboid), AdventError> {
    // Parse a line into an on/off state and cuboid
    let (left, right) = line.split_at(line.find(' ').ok_or(AdventError::NoSpace {
        l: line.to_string(),
    })?);

    if !["on", "off"].contains(&left) {
        return Err(AdventError::InvalidState {
            state: left.to_string(),
        });
    }

    Ok((left == "on", Cuboid::parse(right)?))
}

fn num_on_cubes(lines: Vec<&str>, question_part: QuestionPart) -> Result<i64, AdventError> {
    // Calculate the number of "on" cubes after performing all steps in the input

    // Every time we see an "on" cuboid in the input, subtract it from all the others, then add the
    // new one
    // Every time we see an "off" cuboid in the input, subtract it from all the saved "on" cuboids
    // This is equivalent, but allows us to calculate the volume of all these non-intersecting
    // cuboids to get the number of 1x1x1 cubes that are "on" without a n^3 loop, which is much
    // much more efficient

    let cuboids = lines
        .iter()
        .map(|l| parse(l))
        .collect::<Result<Vec<_>, AdventError>>()?;

    let cuboids = match question_part {
        // In part one, only consider the cuboids 50 spaces from the origin
        QuestionPart::One => cuboids
            .into_iter()
            .filter_map(|(s, c)| Some((s, c.limit(-50, 50)?)))
            .collect(),
        // In part two, consider all cuboids
        QuestionPart::Two => cuboids,
    };

    fn recurse(state: Vec<Cuboid>, cuboids: Vec<(bool, Cuboid)>) -> Vec<Cuboid> {
        match cuboids.split_first() {
            None => state,
            Some(((toggle_to, cuboid), cuboids)) => {
                let state = state
                    .iter()
                    .map(|c| c - cuboid)
                    .flatten()
                    .collect::<Vec<Cuboid>>();

                // If the current cuboid is "on", add it after transforming all the others so that they
                // don't intersect
                // If the current cuboid is "off", just subtract it from all the others, don't add it
                let state = if *toggle_to {
                    state
                        .into_iter()
                        .chain([*cuboid].into_iter())
                        .collect::<Vec<Cuboid>>()
                } else {
                    state
                };
                recurse(state, cuboids.to_vec())
            }
        }
    }
    let final_state = recurse(vec![], cuboids);

    // Get the volume of all the non-intersecting, "on" cuboids
    // Equivalent to the number of "on" cubes
    Ok(final_state.iter().map(|c| c.volume()).sum::<i64>())
}

fn day_22() -> Result<i64, AdventError> {
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

    let lines = stdin()
        .lock()
        .lines()
        .collect::<Result<Vec<String>, std::io::Error>>()?;
    let lines = lines.iter().map(|line| &line[..]).collect();

    num_on_cubes(lines, question_part)
}

fn main() {
    match day_22() {
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

    #[test]
    fn test_sub() -> Result<(), AdventError> {
        assert_eq!(
            &Cuboid::parse("x=0..10,y=0..10,z=0..10")? - &Cuboid::parse("x=5..15,y=5..15,z=5..15")?,
            vec![
                Cuboid::parse("x=0..4,y=0..10,z=0..10")?,
                Cuboid::parse("x=5..10,y=0..4,z=0..10")?,
                Cuboid::parse("x=5..10,y=5..10,z=0..4")?
            ]
        );

        assert_eq!(
            &Cuboid::parse("x=0..10,y=0..10,z=0..10")? - &Cuboid::parse("x=1..9,y=1..9,z=1..9")?,
            vec![
                Cuboid::parse("x=0..0,y=0..10,z=0..10")?,
                Cuboid::parse("x=10..10,y=0..10,z=0..10")?,
                Cuboid::parse("x=1..9,y=0..0,z=0..10")?,
                Cuboid::parse("x=1..9,y=10..10,z=0..10")?,
                Cuboid::parse("x=1..9,y=1..9,z=0..0")?,
                Cuboid::parse("x=1..9,y=1..9,z=10..10")?,
            ]
        );

        let a = Cuboid::parse("x=8..26,y=-36..-30,z=-47..7")?;
        let b = Cuboid::parse("x=8..26,y=-21..17,z=-47..-39")?;

        assert_eq!(&a - &b, vec![a]);

        Ok(())
    }

    #[test]
    fn volume() -> Result<(), AdventError> {
        let c = Cuboid::parse("x=0..0,y=0..0,z=0..0")?;
        assert_eq!(c.volume(), 1);

        let c = Cuboid::parse("x=0..9,y=0..9,z=0..9")?;
        assert_eq!(c.volume(), 1000);

        let c = Cuboid::parse("x=0..9,y=0..9,z=0..0")?;
        assert_eq!(c.volume(), 100);

        Ok(())
    }

    #[test]
    fn has_volume() -> Result<(), AdventError> {
        let c = Cuboid::parse("x=0..0,y=0..0,z=0..0")?;
        assert!(c.has_volume());

        let c = Cuboid::parse("x=0..-1,y=0..0,z=0..0")?;
        assert!(!c.has_volume());

        let c = Cuboid::parse("x=0..1,y=0..0,z=0..-1")?;
        assert!(!c.has_volume());

        Ok(())
    }

    #[test]
    fn intersects() -> Result<(), AdventError> {
        let a = Cuboid::parse("x=-10..10,y=-2..2,z=-1..1")?;
        let b = Cuboid::parse("x=-1..1,y=-1..1,z=-10..10")?;

        assert!(a.intersects(&b));

        Ok(())
    }

    #[test]
    fn test_not_intersecting() -> Result<(), AdventError> {
        let a = Cuboid::parse("x=0..1,y=0..1,z=0..1")?;
        let b = Cuboid::parse("x=2..3,y=2..3,z=2..3")?;

        assert!(!a.intersects(&b));
        assert_eq!(&a - &b, vec![a]);

        let a = Cuboid::parse("x=0..1,y=0..1,z=0..1")?;
        let b = Cuboid::parse("x=0..1,y=0..1,z=2..3")?;

        assert!(!a.intersects(&b));
        assert_eq!(&a - &b, vec![a]);

        let pos = Cuboid::parse("x=-20..26,y=-21..17,z=-47..-27")?;
        let neg = Cuboid::parse("x=-22..28,y=-29..23,z=-38..16")?;

        for sub in &pos - &neg {
            assert!(!sub.intersects(&neg));
            assert!(!neg.intersects(&sub));
        }

        Ok(())
    }

    #[test]
    fn test_example_part_one() -> Result<(), AdventError> {
        let input = vec![
            "on x=-20..26,y=-36..17,z=-47..7",
            "on x=-20..33,y=-21..23,z=-26..28",
            "on x=-22..28,y=-29..23,z=-38..16",
            "on x=-46..7,y=-6..46,z=-50..-1",
            "on x=-49..1,y=-3..46,z=-24..28",
            "on x=2..47,y=-22..22,z=-23..27",
            "on x=-27..23,y=-28..26,z=-21..29",
            "on x=-39..5,y=-6..47,z=-3..44",
            "on x=-30..21,y=-8..43,z=-13..34",
            "on x=-22..26,y=-27..20,z=-29..19",
            "off x=-48..-32,y=26..41,z=-47..-37",
            "on x=-12..35,y=6..50,z=-50..-2",
            "off x=-48..-32,y=-32..-16,z=-15..-5",
            "on x=-18..26,y=-33..15,z=-7..46",
            "off x=-40..-22,y=-38..-28,z=23..41",
            "on x=-16..35,y=-41..10,z=-47..6",
            "off x=-32..-23,y=11..30,z=-14..3",
            "on x=-49..-5,y=-3..45,z=-29..18",
            "off x=18..30,y=-20..-8,z=-3..13",
            "on x=-41..9,y=-7..43,z=-33..15",
            "on x=-54112..-39298,y=-85059..-49293,z=-27449..7877",
            "on x=967..23432,y=45373..81175,z=27513..53682",
        ];
        assert_eq!(num_on_cubes(input, QuestionPart::One)?, 590784);
        Ok(())
    }

    #[test]
    fn test_example_part_two() -> Result<(), AdventError> {
        let input = vec![
            "on x=-5..47,y=-31..22,z=-19..33",
            "on x=-44..5,y=-27..21,z=-14..35",
            "on x=-49..-1,y=-11..42,z=-10..38",
            "on x=-20..34,y=-40..6,z=-44..1",
            "off x=26..39,y=40..50,z=-2..11",
            "on x=-41..5,y=-41..6,z=-36..8",
            "off x=-43..-33,y=-45..-28,z=7..25",
            "on x=-33..15,y=-32..19,z=-34..11",
            "off x=35..47,y=-46..-34,z=-11..5",
            "on x=-14..36,y=-6..44,z=-16..29",
            "on x=-57795..-6158,y=29564..72030,z=20435..90618",
            "on x=36731..105352,y=-21140..28532,z=16094..90401",
            "on x=30999..107136,y=-53464..15513,z=8553..71215",
            "on x=13528..83982,y=-99403..-27377,z=-24141..23996",
            "on x=-72682..-12347,y=18159..111354,z=7391..80950",
            "on x=-1060..80757,y=-65301..-20884,z=-103788..-16709",
            "on x=-83015..-9461,y=-72160..-8347,z=-81239..-26856",
            "on x=-52752..22273,y=-49450..9096,z=54442..119054",
            "on x=-29982..40483,y=-108474..-28371,z=-24328..38471",
            "on x=-4958..62750,y=40422..118853,z=-7672..65583",
            "on x=55694..108686,y=-43367..46958,z=-26781..48729",
            "on x=-98497..-18186,y=-63569..3412,z=1232..88485",
            "on x=-726..56291,y=-62629..13224,z=18033..85226",
            "on x=-110886..-34664,y=-81338..-8658,z=8914..63723",
            "on x=-55829..24974,y=-16897..54165,z=-121762..-28058",
            "on x=-65152..-11147,y=22489..91432,z=-58782..1780",
            "on x=-120100..-32970,y=-46592..27473,z=-11695..61039",
            "on x=-18631..37533,y=-124565..-50804,z=-35667..28308",
            "on x=-57817..18248,y=49321..117703,z=5745..55881",
            "on x=14781..98692,y=-1341..70827,z=15753..70151",
            "on x=-34419..55919,y=-19626..40991,z=39015..114138",
            "on x=-60785..11593,y=-56135..2999,z=-95368..-26915",
            "on x=-32178..58085,y=17647..101866,z=-91405..-8878",
            "on x=-53655..12091,y=50097..105568,z=-75335..-4862",
            "on x=-111166..-40997,y=-71714..2688,z=5609..50954",
            "on x=-16602..70118,y=-98693..-44401,z=5197..76897",
            "on x=16383..101554,y=4615..83635,z=-44907..18747",
            "off x=-95822..-15171,y=-19987..48940,z=10804..104439",
            "on x=-89813..-14614,y=16069..88491,z=-3297..45228",
            "on x=41075..99376,y=-20427..49978,z=-52012..13762",
            "on x=-21330..50085,y=-17944..62733,z=-112280..-30197",
            "on x=-16478..35915,y=36008..118594,z=-7885..47086",
            "off x=-98156..-27851,y=-49952..43171,z=-99005..-8456",
            "off x=2032..69770,y=-71013..4824,z=7471..94418",
            "on x=43670..120875,y=-42068..12382,z=-24787..38892",
            "off x=37514..111226,y=-45862..25743,z=-16714..54663",
            "off x=25699..97951,y=-30668..59918,z=-15349..69697",
            "off x=-44271..17935,y=-9516..60759,z=49131..112598",
            "on x=-61695..-5813,y=40978..94975,z=8655..80240",
            "off x=-101086..-9439,y=-7088..67543,z=33935..83858",
            "off x=18020..114017,y=-48931..32606,z=21474..89843",
            "off x=-77139..10506,y=-89994..-18797,z=-80..59318",
            "off x=8476..79288,y=-75520..11602,z=-96624..-24783",
            "on x=-47488..-1262,y=24338..100707,z=16292..72967",
            "off x=-84341..13987,y=2429..92914,z=-90671..-1318",
            "off x=-37810..49457,y=-71013..-7894,z=-105357..-13188",
            "off x=-27365..46395,y=31009..98017,z=15428..76570",
            "off x=-70369..-16548,y=22648..78696,z=-1892..86821",
            "on x=-53470..21291,y=-120233..-33476,z=-44150..38147",
            "off x=-93533..-4276,y=-16170..68771,z=-104985..-24507",
        ];
        assert_eq!(num_on_cubes(input.clone(), QuestionPart::One)?, 474140);
        assert_eq!(num_on_cubes(input, QuestionPart::Two)?, 2758514936282235);
        Ok(())
    }
}
