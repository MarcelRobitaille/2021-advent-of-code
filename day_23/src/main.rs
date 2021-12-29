use priority_queue::PriorityQueue;
use regex::Regex;
use std::cmp::Reverse;
use std::collections::{HashMap, HashSet};
use std::env;
use std::io::{stdin, Read};
use std::mem::discriminant;
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
}

fn variant_eq<T>(a: &T, b: &T) -> bool {
    discriminant(a) == discriminant(b)
}

#[derive(Debug, Clone, Copy, PartialEq, Hash, Eq)]
enum Amphipod {
    Amber,
    Bronze,
    Copper,
    Desert,
}

impl Amphipod {
    fn weight(&self) -> usize {
        match self {
            Self::Amber => 1,
            Self::Bronze => 10,
            Self::Copper => 100,
            Self::Desert => 1000,
        }
    }

    fn room(&self) -> Vec<usize> {
        match STATE_LEN {
            27 => match self {
                Self::Amber => vec![11, 15, 19, 23],
                Self::Bronze => vec![12, 16, 20, 24],
                Self::Copper => vec![13, 17, 21, 25],
                Self::Desert => vec![14, 18, 22, 26],
            },
            19 => match self {
                Self::Amber => vec![11, 15],
                Self::Bronze => vec![12, 16],
                Self::Copper => vec![13, 17],
                Self::Desert => vec![14, 18],
            },
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Direction {
    In,
    Out,
}

// 0  1  2  3  4  5  6  7  8  9  10
//      11    12    13    14
//      15    16    17    18

//      19    20    21    22
//      23    24    25    26

const WAIT_SPOTS: [usize; 7] = [0, 1, 3, 5, 7, 9, 10];

fn print(state: &State) {
    let state = state
        .iter()
        .map(|x| match x {
            Some(Amphipod::Amber) => "A",
            Some(Amphipod::Bronze) => "B",
            Some(Amphipod::Copper) => "C",
            Some(Amphipod::Desert) => "D",
            None => ".",
        })
        .collect::<Vec<_>>();
    println!("#############");
    let (left, right) = state.split_at(11);
    println!("#{}#", left.join(""));
    let mut chunks = right.chunks(4);
    println!("###{}###", chunks.next().unwrap().join("#"));
    for chunk in chunks {
        println!("  #{}#", chunk.join("#"));
    }
    println!("  #########");
}

fn parse(input: &str) -> State {
    // let re = Regex::new(r"\#{13}\n\#[A-D\.]{11}\#\n\#\#(?:\#[A-D\.]){4}\#\#\#\n(?:  (?:\#[A-D\.]){4}\#\n){3}  \#{9}").unwrap();
    // assert!(re.is_match(input));

    let re = Regex::new(r"[A-D\.]").unwrap();
    let initial_state: State = re
        .find_iter(input)
        .into_iter()
        .map(|m| m.as_str())
        .map(|x| match x {
            "A" => Some(Amphipod::Amber),
            "B" => Some(Amphipod::Bronze),
            "C" => Some(Amphipod::Copper),
            "D" => Some(Amphipod::Desert),
            "." => None,
            _ => unreachable!(),
        })
        .collect::<Vec<_>>()
        .try_into()
        .unwrap();
    initial_state
}

fn state_move(state: State, from: usize, to: usize) -> State {
    let mut state = state;
    state[to] = state[from];
    state[from] = None;
    state
}

fn entered_room(from: usize, to: usize) -> bool {
    from <= 10 && [11, 12, 13, 14].contains(&to)
}
fn left_room(from: usize, to: usize) -> bool {
    to <= 10 && [11, 12, 13, 14].contains(&from)
}
fn entered_wrong_room(to: usize, amphipod: Amphipod) -> bool {
    // println!("entered_wrong_room {:?} {}", amphipod, to);
    match amphipod {
        Amphipod::Amber if to != 11 => true,
        Amphipod::Bronze if to != 12 => true,
        Amphipod::Copper if to != 13 => true,
        Amphipod::Desert if to != 14 => true,
        _ => false,
    }
}
fn room_has_bad_guys(amphipod: Amphipod, state: State) -> bool {
    amphipod
        .room()
        .into_iter()
        .filter_map(|i| state[i])
        .any(|other| !variant_eq(&other, &amphipod))
}

fn moved_up_in_right_room(from: usize, to: usize, amphipod: Amphipod) -> bool {
    from > 10 && from > to && amphipod.room().contains(&from)
}

fn move_is_valid(m: &Move) -> bool {
    // println!("State is valid");
    // print(&state);
    match m.amphipod() {
        Some(amphipod) => {
            if entered_room(m.from, m.to) {
                // println!("A");
                !entered_wrong_room(m.to, amphipod) && !room_has_bad_guys(amphipod, m.state)
            // } else if left_room(from, to) || moved_up_in_right_room(from, to, amphipod) {
            } else if moved_up_in_right_room(m.from, m.to, amphipod) {
                // println!("B");
                // println!(
                //     "left room {} moved up in right room {}",
                //     left_room(from, to),
                //     moved_up_in_right_room(from, to, amphipod)
                // );
                true
                // room_has_bad_guys(amphipod, state)
            } else {
                // println!("C");
                true
            }
        }
        None => true,
    }
}

fn room(i: usize) -> Option<[usize; 4]> {
    [
        [11, 15, 19, 23],
        [12, 16, 20, 24],
        [13, 17, 21, 25],
        [14, 18, 22, 26],
    ]
    .into_iter()
    .find(|room| room.contains(&i))
}

fn cost_to_hallway(state: &State, from: usize) -> Option<usize> {
    match room(from) {
        None => Some(0),
        Some(room)
            if room
                .iter()
                .filter(|i| i < &&from)
                .any(|x| state[*x].is_some()) =>
        {
            None
        }
        _ => Some((from - 11) / 4 + 1),
    }
}

// #[test]
// fn cost_to_hallway() {
//     assert!(cost_to_hallway(
//         &parse(
//             "
// #...........#
// ###.#.#.#.###
//   #.#.#.#.#"
//         ),
//         15
//     )
//     .is_some());
// }

fn reachable(state: &State, from: usize, to: usize) -> Option<usize> {
    fn low_high(from: usize, to: usize) -> (usize, usize) {
        if to > from {
            (from, to)
        } else {
            (to, from)
        }
    }

    let room_from = room(from);

    if let Some(room_from) = room_from {
        if room_from.contains(&to) {
            let (low, high) = low_high(from, to);
            return if room_from
                .iter()
                .filter(|i| i >= &&low && i <= &&high)
                .any(|x| state[*x].is_some())
            {
                None
            } else {
                Some(high - low)
            };
        }
    }

    let (low, high) = low_high(root(from), root(to));

    if (low..=high)
        .filter(|x| x != &from)
        .any(|x| state[x].is_some())
        || state[to].is_some()
    {
        None
    } else {
        let from_cost_to_hallway = cost_to_hallway(state, from)?;
        let to_cost_to_hallway = cost_to_hallway(state, to)?;
        Some(high - low + from_cost_to_hallway + to_cost_to_hallway)
    }
}

#[test]
fn test_reachable() {
    assert_eq!(
        reachable(
            &parse(
                "
#...........#
###.#.#.#.###
  #.#.#.#.#
  #########"
            ),
            15,
            16,
        ),
        Some(6)
    );

    assert_eq!(
        reachable(
            &parse(
                "
#...........#
###.#.#.#.###
  #.#.#.#.#
  #########"
            ),
            0,
            18,
        ),
        Some(10)
    );

    assert_eq!(
        reachable(
            &parse(
                "
#...A.......#
###.#.#.#.###
  #.#.#.#.#
  #########"
            ),
            15,
            16
        ),
        None
    );

    assert_eq!(
        reachable(
            &parse(
                "
#A.........B#
###.#.#.#.###
  #.#.#C#D#
  #########"
            ),
            0,
            15
        ),
        Some(4)
    );
}

fn root(i: usize) -> usize {
    match i {
        11 | 15 | 19 | 23 => 2,
        12 | 16 | 20 | 24 => 4,
        13 | 17 | 21 | 25 => 6,
        14 | 18 | 22 | 26 => 8,
        _ => i,
    }
}

#[derive(Debug, PartialEq, Eq)]
struct Move {
    from: usize,
    to: usize,
    state: State,
    cost: usize,
}

impl Move {
    fn new(from: usize, to: usize, state: State, cost: usize) -> Self {
        Self {
            from,
            to,
            state: state_move(state, from, to),
            cost,
        }
    }

    fn amphipod(&self) -> Option<Amphipod> {
        self.state[self.to]
    }
}

fn go_home(state: State) -> Option<Move> {
    state
        .iter()
        .enumerate()
        .filter_map(|(i, x)| x.map(|x| (i, x)))
        .filter(|(i, amphipod)| !amphipod.room().contains(i))
        .filter(|(_, amphipod)| !room_has_bad_guys(*amphipod, state))
        .find_map(|(from, amphipod)| {
            let room = amphipod.room();
            let to = *room.iter().rev().find(|i| state[**i].is_none()).unwrap();
            let cost = reachable(&state, from, to)?;
            Some(Move::new(from, to, state, cost))
        })
}

#[test]
fn test_go_home() {
    let m = go_home(parse(
        "
#A.........B#
###.#.#.#.###
  #.#.#C#D#
  #########",
    ))
    .unwrap();
    print(&m.state);
    assert_eq!(m.from, 0);
    assert_eq!(m.to, 15);
    assert_eq!(m.cost, 4);

    let m = go_home(parse(
        "
#..........B#
###.#.#.#.###
  #A#C#C#D#
  #########",
    ))
    .unwrap();
    assert_eq!(m.from, 16);
    assert_eq!(m.to, 13);
    assert_eq!(m.cost, 5);
    let m = go_home(parse(
        "
    #..........B#
    ###.#.#C#.###
    #A#.#C#D#
    #########",
    ))
    .unwrap();
    assert_eq!(m.from, 10);
    assert_eq!(m.to, 16);
    assert_eq!(m.cost, 8);
}

fn unwrap(prev: HashMap<State, State>, state: State) -> Vec<State> {
    match prev.clone().get(&state) {
        Some(state2) => unwrap(prev, *state2)
            .into_iter()
            .chain(vec![*state2].into_iter())
            .collect(),
        None => vec![state],
    }
}

fn search(initial_state: State) -> usize {
    let edges = [
        HashSet::from([1]),
        HashSet::from([0, 2]),
        HashSet::from([1, 3, 11]),
        HashSet::from([2, 4]),
        HashSet::from([3, 5, 12]),
        HashSet::from([4, 6]),
        HashSet::from([5, 7, 13]),
        HashSet::from([6, 8]),
        HashSet::from([7, 9, 14]),
        HashSet::from([8, 10]),
        HashSet::from([9]),
        HashSet::from([2, 15]),
        HashSet::from([4, 16]),
        HashSet::from([6, 17]),
        HashSet::from([8, 18]),
        HashSet::from([11]),
        HashSet::from([12]),
        HashSet::from([13]),
        HashSet::from([14]),
        // HashSet::from([11, 19]),
        // HashSet::from([12, 20]),
        // HashSet::from([13, 21]),
        // HashSet::from([14, 22]),
        // HashSet::from([15, 23]),
        // HashSet::from([16, 24]),
        // HashSet::from([17, 25]),
        // HashSet::from([18, 26]),
        // HashSet::from([19]),
        // HashSet::from([20]),
        // HashSet::from([21]),
        // HashSet::from([22]),
    ];

    let target_state = parse(
        "#############
#...........#
###A#B#C#D###
  #A#B#C#D#
  #A#B#C#D#
  #A#B#C#D#
  #########",
    );

    // Dijkstra's algorithm where we're allowed to visit directly adjacent cells in the grid
    // More mutable than I'd like, but my previous recursive function caused a stack overflow
    // even though it looked like it could have used tail recursion

    // Set up priority queue and seed it with source cell
    let mut q = PriorityQueue::<(State, Option<usize>), Reverse<usize>>::new();
    q.push((initial_state, None), Reverse(0));

    // Distance of each cell from the source
    // Gets updated as we find better ways to get to each cell
    let mut dists = HashMap::from([(initial_state, 0)]);

    // List of visisted nodes
    let mut seen = HashSet::<(State, Option<usize>)>::new();

    // prev of each node
    // // How we can discover the path once we find a solution
    let mut prev = HashMap::<State, State>::new();

    while !q.is_empty() {
        // Get closest (highest priority) node in queue
        let (current, _priority) = q.pop().unwrap();
        let (current_state, current_changed) = current;
        // if seen.contains(&current) {
        //     continue;
        // }
        assert!(!seen.contains(&current));
        seen.insert(current);

        // We're done!
        // print(&current_state);
        if current_state == target_state {
            println!("Found it bro!!!");
            // for state in unwrap(prev, current_state) {
            //     print(&state);
            // }
            print(&current_state);
            break;
        }

        // let get_neighbours = |candidates: Vec<(usize, Amphipod)>| {
        //     // println!("Candidates: {:?}", candidates);
        //     candidates
        //         .into_iter()
        //         .map(|(from, _amphipod)| {
        //             edges[from]
        //                 .iter()
        //                 .filter(|to| current_state[**to].is_none())
        //                 .map(move |to| Move::new(from, *to, current_state, 1))
        //         })
        //         .flatten()
        //         .filter(move_is_valid)
        //         .collect()
        // };

        // if [2, 4, 6, 8]
        //     .into_iter()
        //     .filter(|i| current_state[*i].is_some())
        //     .any(|i| match prev.get(&current_state) {
        //         Some(prev) => prev[i] == current_state[i],
        //         None => false,
        //     })
        // {
        //     continue;
        // }

        // if seen.len() % 1000 == 0 {
        // if current_state[..11]
        //     .iter()
        //     .filter(|x| matches!(x, Some(Amphipod::Amber)))
        //     .count()
        //     >= 2
        // {
        // println!("{}", seen.len());
        // print(&current_state);
        // }
        // }

        // print(&current_state);

        // Save to current node
        let current_dist = *dists.get(&current_state).unwrap_or(&usize::MAX);

        let neighbours = if let Some(home) = go_home(current_state) {
            vec![home]
        } else {
            current_state
                .iter()
                .enumerate()
                .filter(|(from, _amphipod)| from > &11)
                .filter_map(|(from, amphipod)| amphipod.map(|x| (from, x)))
                .filter(|(from, amphipod)| {
                    !amphipod.room().contains(from) || room_has_bad_guys(*amphipod, current_state)
                })
                .map(|(from, _amphipod)| {
                    WAIT_SPOTS.into_iter().filter_map(move |to| {
                        let cost = reachable(&current_state, from, to)?;
                        Some(Move::new(from, to, current_state, cost))
                    })
                })
                .flatten()
                .collect::<Vec<_>>()
        };

        // let neighbours: Vec<_> = match (current_changed, direction) {
        //     (Some(change), Some(Direction::In)) => {
        //         get_neighbours(vec![(change, current_state[change].unwrap())])
        //     }
        //     (Some(change), _)
        //         if change > 10
        //             && !room_has_bad_guys(current_state[change].unwrap(), current_state)
        //             && change + 4 < 19
        //             && current_state[change + 4].is_none() =>
        //     {
        //         let from = change;
        //         let to = change + 4;
        //         vec![(
        //             from,
        //             to,
        //             current_state[change].unwrap(),
        //             state_move(current_state, from, to),
        //         )]
        //     }
        //     _ => get_neighbours(
        //         current_state
        //             .into_iter()
        //             .enumerate()
        //             .filter_map(|(i, x)| x.map(|x| (i, x)))
        //             .collect(),
        //     ),
        // };
        // println!("Neighbours {:?}", neighbours);

        for m in neighbours.into_iter() {
            let neighbour_dist = dists.get(&m.state).unwrap_or(&usize::MAX);

            // if matches!(neighbour_amphipod, Amphipod::Desert) && *to == 12 {
            //     print(&prev[&neighbour_state]);
            //     print(&neighbour_state);
            //     // println!("{:}", edges//
            //     panic!("Wtf {} {} ", from, to);
            // }

            //             let neighbour_direction = match (current_changed, direction) {
            //                 (_, Some(direction))
            //                     if matches!(direction, Direction::In) && !matches!(m.to, 0..=10) =>
            //                 {
            //                     // println!("Moving in {:?} {} {}", neighbour_amphipod, from, to);
            //                     Direction::In
            //                 }
            //                 (Some(changed), _) if matches!(m.from, 0..=10) && changed != m.from => {
            //                     Direction::In
            //                 }
            //                 _ => Direction::Out,
            //             };

            let neighbour = (m.state, Some(m.to));
            if seen.contains(&neighbour) {
                continue;
            }

            // If it would be quicker to get to neighbour from current node,
            // then update the distance and prev of v
            let dist_to_neighbour_through_current =
                current_dist + m.amphipod().unwrap().weight() * m.cost;
            if dist_to_neighbour_through_current < *neighbour_dist {
                dists.insert(m.state, dist_to_neighbour_through_current);
                prev.insert(m.state, current_state);
            }

            // Enqueue neighbour
            q.push(neighbour, Reverse(*dists.get(&m.state).unwrap()));
        }
    }

    *dists.get(&target_state).unwrap()
}

// type State = [Option<Amphipod>; 27];
// const STATE_LEN: usize = 19;
const STATE_LEN: usize = 27;
type State = [Option<Amphipod>; STATE_LEN];

fn day_23() -> Result<usize, AdventError> {
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
    println!("{:?}", question_part);

    let mut input = String::new();
    stdin().lock().read_to_string(&mut input).unwrap();

    let initial_state = parse(&input);
    print(&initial_state);

    Ok(search(initial_state))
}

fn main() {
    match day_23() {
        Ok(answer) => println!("{}", answer),
        Err(err) => {
            eprintln!("{}", err);
            exit(1);
        }
    }
}

// #[cfg(test)]
// mod test {
//     use super::*;

//     #[test]
//     fn test_move_to_wrong_room() {
//         assert!(!state_is_valid(
//             parse(
//                 "#############
//         #...........#
//         ###.#A#.#.###
//         #.#.#.#.#
//         #########"
//             ),
//             4,
//             12,
//         ));

//         assert!(!state_is_valid(
//             parse(
//                 "#############
//         #...........#
//         ###.#D#.#.###
//         #.#.#.#.#
//         #########"
//             ),
//             4,
//             12,
//         ));

//         assert!(!state_is_valid(
//             parse(
//                 "#############
// #.....A.....#
// ###.#D#A#D###
//   #C#C#B#B#
//   #########"
//             ),
//             4,
//             12,
//         ));
//     }

//     #[test]
//     fn test_move_to_occupied_room() {
//         assert!(!state_is_valid(
//             parse(
//                 "#############
// #...........#
// ###A#.#.#.###
//   #B#.#.#.#
//   #########"
//             ),
//             2,
//             11,
//         ))
//     }

//     #[test]
//     fn test_left_good_room() {
//         assert!(!state_is_valid(
//             parse(
//                 "#############
// #..A........#
// ###.#.#.#.###
//   #.#.#.#.#
//   #########"
//             ),
//             11,
//             2,
//         ));

//         assert!(!state_is_valid(
//             parse(
//                 "#############
// #..A........#
// ###.#.#.#.###
//   #A#.#.#.#
//   #########"
//             ),
//             11,
//             2,
//         ));

//         assert!(state_is_valid(
//             parse(
//                 "#############
// #..A........#
// ###.#.#.#.###
//   #B#.#.#.#
//   #########"
//             ),
//             11,
//             2,
//         ));

//         assert!(!state_is_valid(
//             parse(
//                 "#############
// #...........#
// ###A#.#.#.###
//   #.#.#.#.#
//   #########"
//             ),
//             15,
//             11,
//         ));

//         assert!(state_is_valid(
//             parse(
//                 "#############
// #...........#
// ###B#.#.#.###
//   #.#.#.#.#
//   #########"
//             ),
//             15,
//             11,
//         ));
//         assert!(state_is_valid(
//             parse(
//                 "#############
// #..B........#
// ###.#A#C#D###
//   #A#B#C#D#
//   #########"
//             ),
//             11,
//             2
//         ));

//         assert!(state_is_valid(
//             parse(
//                 "#############
// #A.B......AB#
// ###.#.#C#D###
//   #.#.#C#D#
//   #########"
//             ),
//             11,
//             2
//         ));
//     }
// }
