use itertools::iproduct;
use itertools::Itertools;
use lazy_static::lazy_static;
use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::env;
use std::io::{stdin, Read};
use std::ops::{Add, Sub};
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

    #[error("Could not parse `{input}' to int.")]
    ParseInt { input: String },

    #[error("Invalid format. Expected `--- scanner x ---', found `{found}'.")]
    Header { found: String },

    #[error("Failed to parse line into beacon coordinates. Expected three integers separted by commas, but found `{line}'.")]
    ParseBeacon { line: String },

    #[error("Empty scanner region detected in input.")]
    EmptyScanner,

    #[error("No solution. Expected `{parent}' and `{child}' to be connected, but could not find transformation.")]
    NoSolution { parent: usize, child: usize },

    #[error("Empty input. No scanners given.")]
    EmptyInput,
}

// Iterate through all the 2-combinations of an iterator as tuples
macro_rules! two_combinations {
    ($it:expr) => {
        $it.combinations(2).into_iter().map(|x| (x[0], x[1]))
    };
}

// Make a set from the keys of a hashmap
macro_rules! key_set {
    ($map:ident) => {
        HashSet::<&i32>::from_iter($map.keys())
    };
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
struct Point {
    x: i32,
    y: i32,
    z: i32,
}

impl Point {
    fn zero() -> Self {
        Self { x: 0, y: 0, z: 0 }
    }

    fn from_tuple(tuple: (i32, i32, i32)) -> Self {
        let (x, y, z) = tuple;
        Self { x, y, z }
    }

    fn parse(line: &str) -> Result<Point, AdventError> {
        // Parse a line from the input into a 3D coordinate
        line.split(',')
            .map(|x| {
                x.parse::<i32>().map_err(|_| AdventError::ParseInt {
                    input: x.to_string(),
                })
            })
            .collect::<Result<Vec<_>, AdventError>>()?
            .into_iter()
            .collect_tuple()
            .ok_or(AdventError::ParseBeacon {
                line: line.to_string(),
            })
            .map(Self::from_tuple)
    }

    fn manhattan_distance(&self, other: &Self) -> i32 {
        // Get the manhattan distance between two points
        let diff = self - other;
        diff.x.abs() + diff.y.abs() + diff.z.abs()
    }
}

// Axis of rotation for transformations
enum Axis {
    X,
    Y,
    Z,
}

impl Point {
    fn rotate(self, times: u8, axis: Axis) -> Self {
        // Rotate a point CCW around an axis
        if times == 0 {
            return self;
        }

        Self::from_tuple(match axis {
            Axis::X => (self.x, -self.z, self.y),
            Axis::Y => (-self.z, self.y, self.x),
            Axis::Z => (-self.y, self.x, self.z),
        })
        .rotate(times - 1, axis)
    }

    fn rotate_3d(self, x: u8, y: u8, z: u8) -> Self {
        // Rotate a point in 3D
        self.rotate(x, Axis::X)
            .rotate(y, Axis::Y)
            .rotate(z, Axis::Z)
    }
}

impl Add<&Point> for &Point {
    type Output = Point;
    fn add(self, other: &Point) -> Point {
        Point {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
        }
    }
}

impl Sub<&Point> for &Point {
    type Output = Point;
    fn sub(self, other: &Point) -> Point {
        Point {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
        }
    }
}

fn parse_scanner(input: &str) -> Result<HashSet<Point>, AdventError> {
    // Parse a chunk of input into a scanner (represented as the set of its beacons)

    lazy_static! {
        static ref RE: Regex = Regex::new(r"--- scanner \d+ ---").unwrap();
    }
    let input = input.split('\n').collect::<Vec<_>>();

    let (header, rest) = input.split_first().ok_or(AdventError::EmptyScanner)?;

    // Check that the header is a match
    // We don't actually use the number given in the input and index the scanners by the order they
    // are parsed
    if !RE.is_match(header) {
        return Err(AdventError::Header {
            found: header.to_string(),
        });
    }

    rest.iter()
        .filter(|line| !line.is_empty())
        .map(|line| Point::parse(line))
        .collect()
}

fn calc_pairwise_dists(beacons: &HashSet<Point>) -> HashMap<i32, [Point; 2]> {
    // Calculate the distances between every pair of beacons for every scanner
    HashMap::from_iter(
        two_combinations!(beacons.iter()).map(|(a, b)| (a.manhattan_distance(b), [*a, *b])),
    )
}

fn find_intersecting_scanners(
    pairwise_dists: &[HashMap<i32, [Point; 2]>],
) -> HashMap<usize, Vec<usize>> {
    // Build a graph of intersecting scanners by the intersection of the pairwise distances of
    // their beacons
    let mut adj_list = HashMap::<usize, Vec<usize>>::new();
    for ((i, a), (j, b)) in two_combinations!(pairwise_dists.iter().enumerate()) {
        let intersection = &key_set!(a) & &key_set!(b);

        // If two scanners have 66 (12 choose 2 (12 from problem statement and 2 because PAIRwise
        // distance)) distances in common, then we can assume that they are connected
        // The pairwise distances are a kind of key for a scanner that is rotation-agnostic,
        // allowing us to match scanners without brute-forcing the rotations
        if (intersection).len() >= 66 {
            adj_list.entry(i).or_insert_with(Vec::new).push(j);
            adj_list.entry(j).or_insert_with(Vec::new).push(i);
        }
    }
    adj_list
}

fn build_tree(source: usize, adj_list: &HashMap<usize, Vec<usize>>) -> HashMap<usize, Vec<usize>> {
    // Build a tree of the order in which we should visit each scanner
    // The parent of a scanner should intersect the scanner, and each scanner should only be
    // visited once
    // This is basically breadth-first search

    let mut seen = HashSet::from([source]);
    let mut q = vec![source];
    let mut children = HashMap::<usize, Vec<usize>>::new();

    while let Some(v) = q.pop() {
        let entry = children.entry(v).or_insert_with(Vec::new);

        for w in &adj_list[&v] {
            if seen.contains(w) {
                continue;
            }
            seen.insert(*w);
            q.push(*w);
            entry.push(*w);
        }
    }

    // Ensure graph is connected
    assert_eq!(seen.len(), adj_list.len());
    children
}

fn transform(
    beacons: &HashSet<Point>,
    x_rotation: u8,
    y_rotation: u8,
    z_rotation: u8,
    translate: Point,
) -> HashSet<Point> {
    HashSet::<Point>::from_iter(
        beacons
            .iter()
            .map(|beacon| &beacon.rotate_3d(x_rotation, y_rotation, z_rotation) + &translate),
    )
}

fn find_transformation(
    parent: usize,
    child: usize,
    scanners: &[HashSet<Point>],
    pairwise_dists: &[HashMap<i32, [Point; 2]>],
) -> Option<(u8, u8, u8, Point)> {
    // Find a matching point and transformation between parent scanner and child scanner

    let parent_dists = &pairwise_dists[parent];
    let child_dists = &pairwise_dists[child];

    // Loop through the common distances
    // If two pairs of points in each scanner have the same distance, then they should be a match
    // We should find a solution on the first iteration, but if not check the rest just in case
    for dist in &key_set!(parent_dists) & &key_set!(child_dists) {
        // We only know which pair of points matched (by their distance), but not which individual
        // points match. We only really need to fix one and try both options for the other,
        // but as above, check all just in case
        // Safe to unwrap; we know this value is in the hashmap because we're looping through it
        for (parent_beacon, child_beacon) in iproduct!(
            parent_dists.get(dist).unwrap(),
            child_dists.get(dist).unwrap()
        ) {
            // Check all 4*4*4 possibilities for rotations
            // Better than brute-forcing every possible pair of points between two scanners as
            // well, but still many possibilities
            // The whole algorithm is still pretty quick though
            for (x_rot, y_rot, z_rot) in iproduct!(0..4, 0..4, 0..4) {
                // Get the translation resulting in this match and rotation
                let translation = parent_beacon - &child_beacon.rotate_3d(x_rot, y_rot, z_rot);

                // Transform all the child's beacons by this rotation and translation
                let transformed = transform(&scanners[child], x_rot, y_rot, z_rot, translation);

                // If it's a match, return the transformation
                if (&scanners[parent] & &transformed).len() >= 12 {
                    return Some((x_rot, y_rot, z_rot, translation));
                }
            }
        }
    }

    // If no transformation was found, return None
    // This should not happen because this is called when the child and parent are thought to be
    // matching based on their intersection of pairwise distances
    None
}

fn build_collective(
    parent: usize,
    scanners: &[HashSet<Point>],
    pairwise_dists: &[HashMap<i32, [Point; 2]>],
    children: &HashMap<usize, Vec<usize>>,
) -> Result<[HashSet<Point>; 2], AdventError> {
    // Go through the tree and get all the beacons in the same reference. Start at the leaves and
    // transform them as seen by their parent until we reach the root
    // Also find the origin of all the scanners as required by part two.
    // This is quite similar, we just apply the same transformations as we work up the tree

    Ok(children[&parent]
        .iter()
        .map(|child| {
            let child = *child;

            // Get the child's result as seen by them
            let child_res = build_collective(child, scanners, pairwise_dists, children)?;

            // Find some transformation to match the child to us
            find_transformation(parent, child, scanners, pairwise_dists)
                .ok_or(AdventError::NoSolution { parent, child })
                // If we find a transformation, apply it to the child result to transform it to our
                // reference
                .map(|(x_rot, y_rot, z_rot, translation)| {
                    // Safe to unwrap; we're looping through a fixed-sized array
                    child_res
                        .into_iter()
                        .map(|x| transform(&x, x_rot, y_rot, z_rot, translation))
                        .collect_tuple()
                        .unwrap()
                })
        })
        .collect::<Result<Vec<(HashSet<Point>, HashSet<Point>)>, AdventError>>()?
        .into_iter()
        // Merge everything together
        // Merge the transformed collective of all of our children with our own scanners
        // Merge the transformed scanner origins discovered by all of our children with our own
        // origin (our origin is zero, but all of these will be transformed by our parent)
        .fold(
            [scanners[parent].clone(), HashSet::from([Point::zero()])],
            |acc, x| [&acc[0] | &x.0, &acc[1] | &x.1],
        ))
}

fn solve(input: String, question_part: QuestionPart) -> Result<usize, AdventError> {
    // Solve everything from parsing down to the different desired results for the different parts

    // Parse input
    let scanners = input
        .split("\n\n")
        .map(parse_scanner)
        .collect::<Result<Vec<_>, AdventError>>()?;

    // Calculate the distances between every pair of beacons for every scanner
    let pairwise_dists = scanners.iter().map(calc_pairwise_dists).collect::<Vec<_>>();

    // Build a graph of intersecting scanners
    let adj_list = find_intersecting_scanners(&pairwise_dists);

    // Find an order in which to merge the scanners (make sure that we are not trying to merge
    // non-intersecting scanners)
    let children = build_tree(0, &adj_list);

    // Merge everything
    let [collective, scanners] = build_collective(0, &scanners, &pairwise_dists, &children)?;

    Ok(match question_part {
        // In part one, we only want the number of unique beacons
        // Once we transform all the beacons relative to scanner zero, we just put them in a set to
        // get the unique count
        QuestionPart::One => collective.len(),
        // In part two, we want the maximum manhattan distance between all the scanner origins
        QuestionPart::Two => two_combinations!(scanners.iter())
            .map(|(a, b)| a.manhattan_distance(b) as usize)
            .max()
            .ok_or(AdventError::EmptyInput)?,
    })
}

fn day_19() -> Result<usize, AdventError> {
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

    solve(input, question_part)
}

fn main() {
    match day_19() {
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

    const INPUT: &str = "--- scanner 0 ---
404,-588,-901
528,-643,409
-838,591,734
390,-675,-793
-537,-823,-458
-485,-357,347
-345,-311,381
-661,-816,-575
-876,649,763
-618,-824,-621
553,345,-567
474,580,667
-447,-329,318
-584,868,-557
544,-627,-890
564,392,-477
455,729,728
-892,524,684
-689,845,-530
423,-701,434
7,-33,-71
630,319,-379
443,580,662
-789,900,-551
459,-707,401

--- scanner 1 ---
686,422,578
605,423,415
515,917,-361
-336,658,858
95,138,22
-476,619,847
-340,-569,-846
567,-361,727
-460,603,-452
669,-402,600
729,430,532
-500,-761,534
-322,571,750
-466,-666,-811
-429,-592,574
-355,545,-477
703,-491,-529
-328,-685,520
413,935,-424
-391,539,-444
586,-435,557
-364,-763,-893
807,-499,-711
755,-354,-619
553,889,-390

--- scanner 2 ---
649,640,665
682,-795,504
-784,533,-524
-644,584,-595
-588,-843,648
-30,6,44
-674,560,763
500,723,-460
609,671,-379
-555,-800,653
-675,-892,-343
697,-426,-610
578,704,681
493,664,-388
-671,-858,530
-667,343,800
571,-461,-707
-138,-166,112
-889,563,-600
646,-828,498
640,759,510
-630,509,768
-681,-892,-333
673,-379,-804
-742,-814,-386
577,-820,562

--- scanner 3 ---
-589,542,597
605,-692,669
-500,565,-823
-660,373,557
-458,-679,-417
-488,449,543
-626,468,-788
338,-750,-386
528,-832,-391
562,-778,733
-938,-730,414
543,643,-506
-524,371,-870
407,773,750
-104,29,83
378,-903,-323
-778,-728,485
426,699,580
-438,-605,-362
-469,-447,-387
509,732,623
647,635,-688
-868,-804,481
614,-800,639
595,780,-596

--- scanner 4 ---
727,592,562
-293,-554,779
441,611,-461
-714,465,-776
-743,427,-804
-660,-479,-426
832,-632,460
927,-485,-438
408,393,-506
466,436,-512
110,16,151
-258,-428,682
-393,719,612
-211,-452,876
808,-476,-593
-575,615,604
-485,667,467
-680,325,-822
-627,-443,-432
872,-547,-609
833,512,582
807,604,487
839,-516,451
891,-625,532
-652,-548,-490
30,-46,-14";

    #[test]
    fn test_part_one() -> Result<(), AdventError> {
        assert_eq!(solve(INPUT.to_string(), QuestionPart::One)?, 79);

        Ok(())
    }

    #[test]
    fn test_part_two() -> Result<(), AdventError> {
        assert_eq!(solve(INPUT.to_string(), QuestionPart::Two)?, 3621);

        Ok(())
    }
}
