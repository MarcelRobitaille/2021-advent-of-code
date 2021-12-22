use itertools::Itertools;
use std::env;
use std::fmt;
use std::io::{stdin, BufRead};
use std::ops::Add;
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

    #[error("Could not parse character `{c}' to digit.")]
    ParseInt { c: char },

    #[error("Please specify `part-one' or `part-two' as the first argument.")]
    NoPartArgument,

    #[error("Invalid input. Expected `target area: x=<x1>..<x2>, y=<y1>..<y2>'. Found `{line}'.")]
    InputError { line: String },

    #[error("Failed to parse line of input. Expected `,' in substring `{haystack}'.")]
    NoComma { haystack: String },

    #[error("Failed to parse line of input. Expected substring `{haystack}' to start with `['.")]
    NoOpenBrace { haystack: String },

    #[error("Failed to parse line of input. Expected substring `{haystack}' to end with `]'.")]
    NoCloseBrace { haystack: String },

    #[error("Empty input given")]
    EmptyInput,

    #[error("Attempted to add two parent nodes, which is prohibited.")]
    NotALeaf,
}

// Tree implementation
#[derive(Debug, Clone, PartialEq)]
pub struct ParentNode {
    left: Node,
    right: Node,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Node {
    Parent(Box<ParentNode>),
    Leaf(u8),
}

impl ParentNode {
    pub fn new(left: u8, right: u8) -> Self {
        Self {
            left: Node::Leaf(left),
            right: Node::Leaf(right),
        }
    }

    fn where_left(&self, left: Node) -> Self {
        // Get a new node where left is given by the argument
        Self {
            left,
            right: self.right.clone(),
        }
    }

    fn where_right(&self, right: Node) -> Self {
        // Get a new node where right is given by the argument
        Self {
            right,
            left: self.left.clone(),
        }
    }

    fn zero_right(&self) -> Self {
        // Get a new node with zero as a right leaf
        self.where_right(Node::Leaf(0))
    }

    fn zero_left(&self) -> Self {
        // Get a new node with zero as a left leaf
        self.where_left(Node::Leaf(0))
    }

    // When we add to the left or right,
    // we want the number to trickle down on the opposite side until a leaf is hit
    fn add_left(self, other: &Node) -> Self {
        match other {
            Node::Leaf(other) => Self {
                right: self.right,
                left: self.left.trickle_right(*other),
            },
            Node::Parent(_) => panic!("Cannot add parent node to a node."),
        }
    }

    fn add_right(self, other: &Node) -> Self {
        match other {
            Node::Leaf(other) => Self {
                left: self.left,
                right: self.right.trickle_left(*other),
            },
            Node::Parent(_) => panic!("Cannot add parent node to a node."),
        }
    }

    // Turn a parent struct into a full node on the heap
    fn node(self) -> Node {
        Node::Parent(Box::new(self))
    }
}

impl Node {
    // When we add to the left or right,
    // we want the number to trickle down on the opposite side until a leaf is hit
    fn trickle_right(self, other: u8) -> Self {
        match self {
            Node::Parent(parent) => ParentNode {
                left: parent.left,
                right: parent.right.trickle_right(other),
            }
            .node(),
            Node::Leaf(leaf) => Node::Leaf(leaf + other),
        }
    }

    fn trickle_left(self, other: u8) -> Self {
        match self {
            Node::Parent(parent) => ParentNode {
                right: parent.right,
                left: parent.left.trickle_left(other),
            }
            .node(),
            Node::Leaf(leaf) => Node::Leaf(leaf + other),
        }
    }
}

impl fmt::Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Node::Parent(parent) => {
                write!(f, "[{},{}]", parent.left, parent.right)
            }
            Node::Leaf(c) => {
                write!(f, "{}", c)
            }
        }
    }
}

// Makes it easy to add to leaf nodes
impl Add<u8> for Node {
    type Output = u8;
    fn add(self, other: Self::Output) -> Self::Output {
        match self {
            Node::Parent(_) => panic!("Attempt to add u8 to parent node"),
            Node::Leaf(x) => x + other,
        }
    }
}

fn find_comma(input: &[char]) -> Option<usize> {
    // Find the comma that separates a pair
    // Move through the string until the number of [ and ] seen are equal
    // and the current character is a comma

    fn recurse(i: usize, depth: usize, input: &[char]) -> Option<usize> {
        match input.get(i) {
            Some(c) => match c {
                ',' if depth == 0 => Some(i),
                '[' => recurse(i + 1, depth + 1, input),
                ']' => recurse(i + 1, depth - 1, input),
                _ => recurse(i + 1, depth, input),
            },
            None => None,
        }
    }

    recurse(0, 0, input)
}

fn parse(line: &[char]) -> Result<Node, AdventError> {
    // Parse one line of input (one snail number)

    // Base case
    if line.len() == 1 {
        let c = line[0];
        Ok(Node::Leaf(
            c.to_digit(10).ok_or(AdventError::ParseInt { c })? as u8,
        ))
    } else {
        // Ensure start and end are braces
        if line.first() != Some(&'[') {
            return Err(AdventError::NoOpenBrace {
                haystack: line.iter().collect(),
            });
        }
        if line.last() != Some(&']') {
            return Err(AdventError::NoCloseBrace {
                haystack: line.iter().collect(),
            });
        }

        // Remove start and end braces
        let line = &line[1..line.len() - 1];

        // Split on the comma
        let split = find_comma(line).ok_or(AdventError::NoComma {
            haystack: line.iter().collect(),
        })?;

        Ok(ParentNode {
            left: parse(&line[..split])?,
            right: parse(&line[split + 1..])?,
        }
        .node())
    }
}

fn parse_str(line: &str) -> Result<Node, AdventError> {
    parse(&line.chars().collect::<Vec<_>>())
}

fn explode(node: &Node, depth: usize) -> Result<Node, (Node, ParentNode)> {
    // Explode if a number is eligible
    // Returns Ok(head) if the number does not explode,
    // and Err(head, _) otherwise after propagating the explosion
    // The other argument is a { left, right} with the number that should be added on each side
    // While unwinding, if we find an aunt or uncle on that side, add this then zero it

    match node {
        Node::Parent(parent) => {
            // If depth is 4 and we're a parent, explode
            // When we explode, we become zero and our left and right bubble up
            if depth == 4 {
                return Err((Node::Leaf(0), *parent.clone()));
            }

            // Otherwise, recurse left and right
            // If left explodes, bubble right
            // If right explodes, bubble left
            Ok(ParentNode {
                left: explode(&parent.left, depth + 1).map_err(|(tree, other)| {
                    (
                        parent.where_left(tree).add_right(&other.right).node(),
                        other.zero_right(),
                    )
                })?,
                right: explode(&parent.right, depth + 1).map_err(|(tree, other)| {
                    (
                        parent.where_right(tree).add_left(&other.left).node(),
                        other.zero_left(),
                    )
                })?,
            }
            .node())
        }
        Node::Leaf(_) => Ok(node.to_owned()),
    }
}

fn split(node: &Node) -> Result<Node, Node> {
    // Recursively check for values that must be split
    match node {
        Node::Leaf(x) => {
            // Any value greater than 9 has to become a pair of floor(9/2) and ceil(9/2)
            if x > &9 {
                Err(ParentNode {
                    left: Node::Leaf(x / 2),
                    right: Node::Leaf(x / 2 + x % 2),
                }
                .node())
            } else {
                Ok(node.to_owned())
            }
        }
        Node::Parent(parent) => Ok(ParentNode {
            left: split(&parent.left).map_err(|tree| parent.where_left(tree).node())?,
            right: split(&parent.right).map_err(|tree| parent.where_right(tree).node())?,
        }
        .node()),
    }
}

fn reduce(node: Node) -> Node {
    // Reduce a snail number until it doesn't need to explode or be split
    let node = explode(&node, 0).unwrap_or_else(|(tree, _)| {
        // println!("Explode: {}", tree);
        reduce(tree)
    });
    split(&node).unwrap_or_else(|tree| {
        // println!("Split: {}", tree);
        reduce(tree)
    })
}

fn magnitude(node: Node) -> usize {
    // Get the magnitude of a snail number
    // It's 3 times the left and 2 times the right
    match node {
        Node::Parent(parent) => 3 * magnitude(parent.left) + 2 * magnitude(parent.right),
        Node::Leaf(x) => x as usize,
    }
}

fn day_18() -> Result<usize, AdventError> {
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
    let numbers = lines
        .iter()
        .map(|line| parse_str(line))
        .collect::<Result<Vec<_>, AdventError>>()?;

    Ok(match question_part {
        QuestionPart::One => {
            // In part one, get the magnitude of the sum of the numbers in the input
            let sum = numbers
                .into_iter()
                .reduce(|left, right| reduce(ParentNode { left, right }.node()))
                .ok_or(AdventError::EmptyInput)?;
            magnitude(sum)
        }
        QuestionPart::Two => numbers
            // In part two, get the maximum sum of any two numbers in the input
            .into_iter()
            .permutations(2)
            .map(|items| {
                let node = ParentNode {
                    left: items[0].clone(),
                    right: items[1].clone(),
                }
                .node();
                let node = reduce(node);
                magnitude(node)
            })
            .max()
            .ok_or(AdventError::EmptyInput)?,
    })
}

fn main() {
    match day_18() {
        Ok(answer) => println!("{}", answer),
        Err(err) => {
            eprintln!("{}", err);
            exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_explode() -> Result<(), AdventError> {
        for (before, after, (left, right)) in [
            ("[[[[[9,8],1],2],3],4]", "[[[[0,9],2],3],4]", (9, 0)),
            ("[7,[6,[5,[4,[3,2]]]]]", "[7,[6,[5,[7,0]]]]", (0, 2)),
            ("[[6,[5,[4,[3,2]]]],1]", "[[6,[5,[7,0]]],3]", (0, 0)),
            (
                "[[3,[2,[1,[7,3]]]],[6,[5,[4,[3,2]]]]]",
                "[[3,[2,[8,0]]],[9,[5,[4,[3,2]]]]]",
                (0, 0),
            ),
            (
                "[[3,[2,[8,0]]],[9,[5,[4,[3,2]]]]]",
                "[[3,[2,[8,0]]],[9,[5,[7,0]]]]",
                (0, 2),
            ),
        ] {
            let expected_tree = parse_str(after)?;
            let expected_extra = ParentNode::new(left, right);
            match explode(&parse_str(before)?, 0) {
                Ok(_) => panic!("{} did not explode", before),
                Err((tree, extra)) => {
                    if tree != expected_tree {
                        panic!("Expected {}, found {}", expected_tree, tree);
                    }
                    if extra != expected_extra {
                        panic!("Expected {:?}, found {:?}", expected_extra, extra);
                    }
                }
            }
        }
        Ok(())
    }

    #[test]
    fn test_add() -> Result<(), AdventError> {
        assert_eq!(
            reduce(parse_str(
                "[[[[0,[4,5]],[0,0]],[[[4,5],[2,6]],[9,5]]],[7,[[[3,7],[4,3]],[[6,3],[8,8]]]]]"
            )?),
            parse_str("[[[[4,0],[5,4]],[[7,7],[6,0]]],[[8,[7,7]],[[7,9],[5,0]]]]")?,
        );
        Ok(())
    }

    #[test]
    fn test_magnitude() -> Result<(), AdventError> {
        assert_eq!(magnitude(parse_str("[9,1]")?), 29);
        Ok(())
    }
}
