use lazy_static::lazy_static;
use priority_queue::PriorityQueue;
use regex::Regex;
use std::cmp::Reverse;
use std::collections::{HashMap, HashSet};
use std::env;
use std::io::{stdin, Read};
use std::mem::discriminant;
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

    #[error("Could not find a solution for the given input.")]
    NoSolution,

    #[error("Input does not conform to expected format.")]
    FormatError,
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
        // Cost to move different amphipod types one spot
        match self {
            Self::Amber => 1,
            Self::Bronze => 10,
            Self::Copper => 100,
            Self::Desert => 1000,
        }
    }
}

// I represent state as a fixed-length array of the Amphipod enum
// I use the same container for each part for simplicity (in part one, I make sure that I don't use
// the last 8 spots). I do not know a good way to use two fixed-sized arrays interchangeably like
// this. I use a fixed-length array and not a vec to keep things on the stack
//
// The array is allocated from left to right, top to bottom. That is, in the order the "ABCD."
// characters appear in the problem input if you joined the lines. Here is how the indices are assigned:
// ###################################
// #00 01 02 03 04 05 06 07 08 09  10#
// ###### 11 ## 12 ## 13 ## 14 #######
//      # 15 ## 16 ## 17 ## 18 #
//      # 19 ## 20 ## 21 ## 22 #
//      # 23 ## 24 ## 25 ## 26 #
//      ########################

const STATE_LENGTH_PART_ONE: usize = 19;
const STATE_LENGTH_PART_TWO: usize = 27;
type State = [Option<Amphipod>; STATE_LENGTH_PART_TWO];

// Indices of the valid spots an amphipod can wait
// It can only wait in hallway spots that are not just outside a room
const WAIT_SPOTS: [usize; 7] = [0, 1, 3, 5, 7, 9, 10];

// Length of the hallway
const HALLWAY_LENGTH: usize = 11;

// Number of rooms
const NUM_ROOMS: usize = 4;

// Totally empty state
// Used for tests
#[cfg(test)]
const EMPTY_STATE: State = [None; STATE_LENGTH_PART_TWO];

fn in_range(question_part: QuestionPart) -> impl Fn(&usize) -> bool {
    // Check if an index is in range for a given question part
    move |i| match question_part {
        QuestionPart::One => i < &STATE_LENGTH_PART_ONE,
        QuestionPart::Two => i < &STATE_LENGTH_PART_TWO,
    }
}

impl Amphipod {
    fn room(&self, question_part: QuestionPart) -> Vec<usize> {
        // Indices of the room designated to a given amphipod type
        match self {
            Self::Amber => [11, 15, 19, 23],
            Self::Bronze => [12, 16, 20, 24],
            Self::Copper => [13, 17, 21, 25],
            Self::Desert => [14, 18, 22, 26],
        }
        .into_iter()
        // Filter out the high indices for part one
        .filter(in_range(question_part))
        .collect()
    }
}

fn room(i: usize, question_part: QuestionPart) -> Option<Vec<usize>> {
    // Similar to the above, but gets the rest of the room given any index
    match i {
        11 | 15 | 19 | 23 => Some([11, 15, 19, 23]),
        12 | 16 | 20 | 24 => Some([12, 16, 20, 24]),
        13 | 17 | 21 | 25 => Some([13, 17, 21, 25]),
        14 | 18 | 22 | 26 => Some([14, 18, 22, 26]),
        _ => None,
    }
    .map(|room| room.into_iter().filter(in_range(question_part)).collect())
}

fn root(i: usize) -> usize {
    // Get the root of a room given any of its indices. The root is the hallway cell directly
    // outside the room
    match i {
        11 | 15 | 19 | 23 => 2,
        12 | 16 | 20 | 24 => 4,
        13 | 17 | 21 | 25 => 6,
        14 | 18 | 22 | 26 => 8,
        _ => i,
    }
}

fn variant_eq<T>(a: &T, b: &T) -> bool {
    // Check if two enums have the same variant
    discriminant(a) == discriminant(b)
}

fn print(state: &State) {
    // Print a state in the format of the problem description and input
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
    let (left, right) = state.split_at(HALLWAY_LENGTH);
    println!("#{}#", left.join(""));
    let mut chunks = right.chunks(4);
    println!("###{}###", chunks.next().unwrap().join("#"));
    for chunk in chunks {
        println!("  #{}#", chunk.join("#"));
    }
    println!("  #########");
}

fn parse_part(input: &str) -> Vec<Option<Amphipod>> {
    // Extract all the "ABCD." from a string into a vec of amphipods
    lazy_static! {
        static ref RE: Regex = Regex::new(r"[A-D\.]").unwrap();
    }

    RE.find_iter(input)
        .into_iter()
        .map(|m| m.as_str())
        .map(|x| match x {
            "A" => Some(Amphipod::Amber),
            "B" => Some(Amphipod::Bronze),
            "C" => Some(Amphipod::Copper),
            "D" => Some(Amphipod::Desert),
            "." => None,
            // If the regex matches, this should be all the possible states
            _ => unreachable!(),
        })
        .collect()
}

fn parse(input: &str, question_part: QuestionPart) -> State {
    // Parse the input into a `State`. This is a "dumb" function that does no validation.
    // It unwraps options and results, relying on the regex matching in `parse_formatted` for
    // validation. This is a separate function to be used to quickly give states for tests.
    let initial_state = parse_part(input);

    let initial_state = match initial_state.len() {
        // If we got a full input with 4 spot in each room, it will have length 27. This never
        // happens for the real input, but this function allows this for tests and so on
        STATE_LENGTH_PART_TWO => initial_state,
        // If we get a partial input with 2 spots in each room, it will have length 19. This is
        // always the case for the real input; even in part two, two constant lines get added after
        // reading input
        STATE_LENGTH_PART_ONE => match question_part {
            // In part one, add two empty lines at the end
            QuestionPart::One => [initial_state, [None; 8].to_vec()].concat(),
            // In part two, add two constant lines between lines 1 and 4
            QuestionPart::Two => [
                &initial_state[..15],
                &parse_part("DCBADBAC"),
                &initial_state[15..],
            ]
            .concat(),
        },
        // Use regex for validation. I could hit this when using this function direction from
        // tests, but I will know how to fix it
        _ => unreachable!(),
    };

    // If we get through the match, the length will definitely be correct
    initial_state.try_into().unwrap()
}

fn parse_formatted(input: &str, question_part: QuestionPart) -> Result<State, AdventError> {
    // Parse a problem input, checking the format for errors. The simple `parse` function is useful
    // to concisely specify problem inputs for tests
    lazy_static! {
        static ref RE: Regex = Regex::new(
            r"\#{13}\n\#[A-D\.]{11}\#\n\#\#(?:\#[A-D\.]){4}\#\#\#\n  (?:\#[A-D\.]){4}\#\n  \#{9}"
        )
        .unwrap();
    }
    if !RE.is_match(input) {
        Err(AdventError::FormatError)
    } else {
        Ok(parse(input, question_part))
    }
}

fn room_has_bad_guys(amphipod: Amphipod, state: State, question_part: QuestionPart) -> bool {
    // Check if a room has "bad guys" (any other type of amphipod)
    amphipod
        .room(question_part)
        .into_iter()
        .filter_map(|i| state[i])
        .any(|other| !variant_eq(&other, &amphipod))
}

fn cost_to_hallway(state: &State, from: usize, question_part: QuestionPart) -> Option<usize> {
    // Calculate the cost to get from `from` index to hallway (root), or None if unreachable
    match room(from, question_part) {
        // If not in room, we're already there and cost is zero
        None => Some(0),
        // If we're in a room, and we're being blocked, return None
        Some(room)
            if room
                .iter()
                .filter(|i| i < &&from)
                .any(|x| state[*x].is_some()) =>
        {
            None
        }
        // Otherwise, it's the index inside the rooms divided (and floored) by the number of rooms
        _ => Some((from - HALLWAY_LENGTH) / NUM_ROOMS + 1),
    }
}

#[test]
fn test_cost_to_hallway() {
    assert!(cost_to_hallway(&EMPTY_STATE, 15, QuestionPart::One).is_some());
}

fn reachable(state: &State, from: usize, to: usize, question_part: QuestionPart) -> Option<usize> {
    // If `to` is reachable from `from`, return the cost, or None otherwise
    fn low_high(from: usize, to: usize) -> (usize, usize) {
        if to > from {
            (from, to)
        } else {
            (to, from)
        }
    }

    let room_from = room(from, question_part);

    // If `from` and `to` are in the same room, it's easy to calculate
    if let Some(room_from) = room_from {
        if room_from.contains(&to) {
            // Check for blocking amphipods between low and high
            let (low, high) = low_high(from, to);
            println!("{} {}", low, high);
            return match room_from
                .iter()
                .filter(|i| (low..=high).contains(i))
                .find(|x| state[**x].is_some())
            {
                // If we find some blocking amphipod, return None
                Some(_) => None,
                // Otherwise, it's all in one room, so it's just high - low
                None => Some((high - low) / NUM_ROOMS),
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
        let from_cost_to_hallway = cost_to_hallway(state, from, question_part)?;
        let to_cost_to_hallway = cost_to_hallway(state, to, question_part)?;
        Some(high - low + from_cost_to_hallway + to_cost_to_hallway)
    }
}

#[test]
fn test_reachable() {
    let qp = QuestionPart::One;
    assert_eq!(reachable(&EMPTY_STATE, 15, 16, qp), Some(6));
    assert_eq!(reachable(&EMPTY_STATE, 0, 18, qp), Some(10));

    let input = "
        #...A.......#
        ###.#.#.#.###
          #.#.#.#.#";
    assert_eq!(reachable(&parse(input, qp), 15, 16, qp), None);

    let input = "
        #A.........B#
        ###.#.#.#.###
          #.#.#C#D#";
    assert_eq!(reachable(&parse(input, qp), 0, 15, qp), Some(4));

    assert_eq!(reachable(&EMPTY_STATE, 23, 11, qp), Some(3));
    assert_eq!(reachable(&EMPTY_STATE, 24, 12, qp), Some(3));
}

// Represents a move or a step
#[derive(Debug, PartialEq, Eq)]
struct Move {
    // Index of position to move from
    from: usize,
    // Index of position to move to
    to: usize,
    // State after move
    state: State,
    // Cost of move
    cost: usize,
}

fn state_swap(state: State, from: usize, to: usize) -> State {
    // Swap the amphipod in position `from` to position `to` and set `from` to None. It is not the
    // responsibility of this function to check that `to` is empty.
    let mut state = state;
    state[to] = state[from];
    state[from] = None;
    state
}

impl Move {
    fn new(from: usize, to: usize, state: State, cost: usize) -> Self {
        Self {
            from,
            to,
            state: state_swap(state, from, to),
            cost,
        }
    }

    fn amphipod(&self) -> Option<Amphipod> {
        self.state[self.to]
    }
}

fn go_home(state: State, question_part: QuestionPart) -> Option<Move> {
    // Return a move for the first amphipod found that is able to go directly home
    // or None if all are still blocked
    state
        .iter()
        .enumerate()
        // Select only spaces that have some amphipod
        .filter_map(|(i, x)| x.map(|x| (i, x)))
        // Filter out amphipods already in their own room
        .filter(|(i, amphipod)| !amphipod.room(question_part).contains(i))
        // Filter out amphipods whose rooms have bad guys (in this case, the amphipod should not go
        // home even if it can)
        .filter(|(_, amphipod)| !room_has_bad_guys(*amphipod, state, question_part))
        // If the amphipod can get home (not blocked), return the corresponding move
        .find_map(|(from, amphipod)| {
            let room = amphipod.room(question_part);
            // Get the bottom-most empty spot in the amphipod's room
            let to = *room.iter().rev().find(|i| state[**i].is_none()).unwrap();
            let cost = reachable(&state, from, to, question_part)?;
            Some(Move::new(from, to, state, cost))
        })
}

#[test]
fn test_go_home() {
    let qp = QuestionPart::One;

    let input = "
        #A.........B#
        ###.#.#.#.###
          #.#.#C#D#";
    let m = go_home(parse(input, qp), qp).unwrap();
    assert_eq!(m.from, 0);
    assert_eq!(m.to, 15);
    assert_eq!(m.cost, 4);

    let input = "
        #..........B#
        ###.#.#.#.###
          #A#C#C#D#";
    let m = go_home(parse(input, qp), qp).unwrap();
    assert_eq!(m.from, 16);
    assert_eq!(m.to, 13);
    assert_eq!(m.cost, 5);

    let input = "
        #..........B#
        ###.#.#C#.###
          #A#.#C#D#";
    let m = go_home(parse(input, qp), qp).unwrap();
    assert_eq!(m.from, 10);
    assert_eq!(m.to, 16);
    assert_eq!(m.cost, 8);
}

fn get_all_unblock_moves(state: State, question_part: QuestionPart) -> Vec<Move> {
    // Get all the currently possible unblock moves
    state
        .iter()
        .enumerate()
        // Filter out all the amphipods in the hallways
        // Unblock moves are not moves to their home, so are not allowed from in the hallway
        .filter(|(from, _amphipod)| from > &HALLWAY_LENGTH)
        // Filter out empty spaces
        .filter_map(|(from, amphipod)| amphipod.map(|x| (from, x)))
        .filter(|(from, amphipod)| {
            // Filter out amphipods that are already home
            !amphipod.room(question_part).contains(from)
                // But keep them if they are blocking others
                || room_has_bad_guys(*amphipod, state, question_part)
        })
        // Add a possible move for each of the wait spots
        // It would be more efficient to figure out what the amphipod is blocking to reduce the
        // wait spots that make sense, but this is plenty fast
        .map(|(from, _amphipod)| {
            WAIT_SPOTS.into_iter().filter_map(move |to| {
                let cost = reachable(&state, from, to, question_part)?;
                Some(Move::new(from, to, state, cost))
            })
        })
        .flatten()
        .collect::<Vec<_>>()
}

fn search(initial_state: State, question_part: QuestionPart) -> Result<usize, AdventError> {
    // Find the minimal cost to get from the initial state to the desired state
    // This solution is basically Dijkstra's algorithm, moving from one possible state to another
    // By adding possible moves to a priority queue and always selecting the least costly, we are
    // guaranteed that if a solution is found, it will be the least costly overall (like Dijkstra).

    let target_state = parse(
        match question_part {
            QuestionPart::One => {
                "#...........#
                 ###A#B#C#D###
                   #A#B#C#D#"
            }
            QuestionPart::Two => {
                "#...........#
                 ###A#B#C#D###
                   #A#B#C#D#
                   #A#B#C#D#
                   #A#B#C#D#"
            }
        },
        question_part,
    );

    // Set up priority queue and seed it with source cell
    let mut q = PriorityQueue::<State, Reverse<usize>>::new();
    q.push(initial_state, Reverse(0));

    // Distance of each cell from the source
    // Gets updated as we find better ways to get to each cell
    let mut dists = HashMap::from([(initial_state, 0)]);

    // List of visited nodes
    let mut seen = HashSet::<State>::new();

    while !q.is_empty() {
        // Get closest (highest priority) node in queue
        let (current, _priority) = q.pop().unwrap();

        // This should never happen given the check in the for loop, but I want to know if there is
        // a regression
        assert!(!seen.contains(&current));

        // Add the current state to seen
        seen.insert(current);

        // If the current state is the target state, return the distance to get to this state
        if current == target_state {
            return dists
                .get(&target_state)
                .copied()
                .ok_or(AdventError::NoSolution);
        }

        // Save to current node
        let current_dist = *dists.get(&current).unwrap_or(&usize::MAX);

        // If there is a possibility for an amphipod to go home, always put that move next
        let neighbours = if let Some(home) = go_home(current, question_part) {
            vec![home]
        // Otherwise, consider all possible unblock moves (we will consider the cheapest first to
        // ensure that we find the cheapest overall solution)
        } else {
            get_all_unblock_moves(current, question_part)
        };

        // Add all neighbour moves to queue
        // Reject already-seen states
        for neighbour in neighbours.into_iter().filter(|n| !seen.contains(&n.state)) {
            let neighbour_dist = dists.get(&neighbour.state).unwrap_or(&usize::MAX);

            // If it would be quicker to get to neighbour from current node,
            // then update the distance
            let dist_to_neighbour_through_current =
                current_dist + neighbour.amphipod().unwrap().weight() * neighbour.cost;
            if dist_to_neighbour_through_current < *neighbour_dist {
                dists.insert(neighbour.state, dist_to_neighbour_through_current);
            }

            // Enqueue neighbour
            q.push(
                neighbour.state,
                Reverse(*dists.get(&neighbour.state).unwrap()),
            );
        }
    }

    Err(AdventError::NoSolution)
}

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

    let mut input = String::new();
    stdin().lock().read_to_string(&mut input).unwrap();

    let initial_state = parse_formatted(&input, question_part)?;
    print(&initial_state);

    search(initial_state, question_part)
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

#[cfg(test)]
mod test {
    use super::*;

    const INPUT: &str = "#############
#...........#
###D#A#A#D###
  #C#C#B#B#
  #########";

    #[test]
    fn test_part_one() -> Result<(), AdventError> {
        let input = parse_formatted(INPUT, QuestionPart::One)?;
        assert_eq!(search(input, QuestionPart::One)?, 14467);
        Ok(())
    }

    #[test]
    fn test_part_two() -> Result<(), AdventError> {
        let input = parse_formatted(INPUT, QuestionPart::Two)?;
        assert_eq!(search(input, QuestionPart::Two)?, 48759);
        Ok(())
    }
}
