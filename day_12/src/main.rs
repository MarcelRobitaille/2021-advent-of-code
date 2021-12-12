use itertools::{chain, Itertools};
use std::collections::{HashMap, HashSet};
use std::env;
use std::io::{stdin, BufRead};
use std::process::exit;
use thiserror::Error;

#[derive(Debug)]
enum QuestionPart {
    One,
    Two,
}

#[derive(Eq, PartialEq, Hash, Clone, Debug)]
enum Vertex {
    Start,
    End,
    Small(String),
    Big(String),
}
type AdjacencyList = HashMap<Vertex, HashSet<Vertex>>;

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

    #[error("Invalid vertex `{x}'. Expected `start', `end', or a sequence of all-uppercase or all-lowercase letters.")]
    InvalidVertex { x: String },

    #[error("No start vertex in input")]
    NoStart,
}

fn recurse(
    v: Vertex,
    adjacency_list: &AdjacencyList,
    small_caves: &HashSet<Vertex>,
    path: &[Vertex],
    visited_small_cave_twice: bool,
) -> Vec<Vec<Vertex>> {
    let path: Vec<_> = chain![path, [&v]].map(|v| v.to_owned()).collect();
    match v {
        Vertex::Start => vec![],
        Vertex::End => vec![path],
        Vertex::Small(_) => {
            // In part two, we're allowed to visit a single small cave twice
            // If we've visited v and we haven't visited any other small cave twice,
            // visit v anyway
            if !small_caves.contains(&v) || !visited_small_cave_twice {
                // Safe to unwrap. Adjacency list is symmetrical
                chain!(adjacency_list.get(&v).unwrap().iter().map(|w| {
                    recurse(
                        w.to_owned(),
                        adjacency_list,
                        &(small_caves | &HashSet::from([v.to_owned()])),
                        &path,
                        small_caves.contains(&v) || visited_small_cave_twice,
                    )
                }))
                .flatten()
                .collect()
            } else {
                vec![]
            }
        }
        // Safe to unwrap. Adjacency list is symmetrical
        Vertex::Big(_) => chain!(adjacency_list.get(&v).unwrap().iter().map(|w| recurse(
            w.to_owned(),
            adjacency_list,
            small_caves,
            &path,
            visited_small_cave_twice,
        )))
        .flatten()
        .collect(),
    }
}

fn print_path(path: &[Vertex]) -> String {
    // Print a path like in the website, separated by comma
    path.iter()
        .map(|v| match v {
            Vertex::Start => "start",
            Vertex::End => "end",
            Vertex::Small(c) => c,
            Vertex::Big(c) => c,
        })
        .join(",")
}

fn day_12() -> Result<usize, AdventError> {
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
    let input = input
        .iter()
        .map(|l| {
            l.split('-')
                .collect_tuple::<(&str, &str)>()
                .ok_or(AdventError::InvalidInput)
        })
        .collect::<Result<Vec<_>, AdventError>>()?;

    // Parse vertices of strings into enum variants
    let edges = input
        .iter()
        .map(|(a, b)| {
            [a, b]
                .iter()
                .map(|x| match **x {
                    "start" => Ok(Vertex::Start),
                    "end" => Ok(Vertex::End),
                    _ => {
                        if x.chars().all(|c| ('a'..='z').contains(&c)) {
                            Ok(Vertex::Small(x.to_string()))
                        } else if x.chars().all(|c| ('A'..='Z').contains(&c)) {
                            Ok(Vertex::Big(x.to_string()))
                        } else {
                            Err(AdventError::InvalidVertex { x: x.to_string() })
                        }
                    }
                })
                .collect_tuple()
                .ok_or(AdventError::InvalidInput)
        })
        .collect::<Result<Vec<_>, AdventError>>()?;

    // Build adjacency list from list of edges
    let mut adjacency_list = AdjacencyList::new();
    for (a, b) in edges {
        // Could not find a way to collect_tuple into Result<tuple, Error>, so do it here
        let a = a?;
        let b = b?;
        adjacency_list
            .entry(a.to_owned())
            .or_insert_with(HashSet::new)
            .insert(b.to_owned());
        adjacency_list
            .entry(b.to_owned())
            .or_insert_with(HashSet::new)
            .insert(a.to_owned());
    }

    // All paths start at start
    let path = vec![Vertex::Start];

    // Track small caves seen so we don't visit them twice
    let small_caves = HashSet::<Vertex>::new();

    // Discover all paths recursively
    let adjacent_to_start = adjacency_list
        .get(&Vertex::Start)
        .ok_or(AdventError::NoStart)?;
    let paths: Vec<Vec<Vertex>> = chain!(adjacent_to_start.iter().map(|w| recurse(
        w.to_owned(),
        &adjacency_list,
        &small_caves,
        &path,
        // Hacky way to implement the question part
        // If we're part one, just tell the recursive function that we've already visited
        // our one allowed small cave twice
        matches!(question_part, QuestionPart::One)
    )))
    .flatten()
    .collect();

    for path in &paths {
        println!("{}", print_path(path));
    }
    Ok(paths.len())
}

fn main() {
    match day_12() {
        Ok(answer) => println!("{}", answer),
        Err(err) => {
            eprintln!("{}", err);
            exit(1);
        }
    }
}
