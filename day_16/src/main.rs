use bitvec::prelude::*;
use hex::{FromHex, FromHexError};
use std::env;
use std::io::{stdin, BufRead};
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

    #[error(transparent)]
    FromHex(#[from] FromHexError),

    #[error("Invalid type id. Expected 0..7. Found {type_id}.")]
    InvalidTypeId { type_id: u8 },

    #[error("Input ended prematurely. Expected longer input.")]
    InputEndedPrematurely,

    #[error("Too few sub packets. Expected more sub packets during collapse.")]
    TooFewSubPackets,
}

#[derive(Debug, Clone)]
enum OperatorType {
    Sum,
    Product,
    Minimum,
    Maximum,
    GreaterThan,
    LessThan,
    EqualTo,
}

#[derive(Debug, Clone)]
enum PacketType {
    Literal(usize),
    Operator(OperatorType, Vec<Packet>),
}

impl PacketType {
    fn parse_literal(
        bitvec: &BitSlice<Msb0, u8>,
    ) -> Result<(&BitSlice<Msb0, u8>, PacketType), AdventError> {
        // Parse literal value
        // Keep on evaluating nibbles until the first bit is zero
        fn recurse(
            bitvec: &BitSlice<Msb0, u8>,
        ) -> Result<(&BitSlice<Msb0, u8>, usize, usize), AdventError> {
            let (done, bitvec) = bitvec.split_at(1);
            let done = !*done
                .first()
                .as_deref()
                .ok_or(AdventError::InputEndedPrematurely)?;

            let (current, bitvec) = bitvec.split_at(4);
            let current = current.load_be();
            if done {
                return Ok((bitvec, 0, current));
            }
            let (bitvec, i, acc) = recurse(bitvec)?;
            let i = i + 4;
            Ok((bitvec, i, (current << i) + acc))
        }

        let (bitvec, _, literal) = recurse(bitvec)?;
        Ok((bitvec, PacketType::Literal(literal)))
    }

    fn parse_operator(
        bitvec: &BitSlice<Msb0, u8>,
        type_id: u8,
    ) -> Result<(&BitSlice<Msb0, u8>, PacketType), AdventError> {
        // Parse an operator packet
        // These have many sub packets and an operator for how to collapse them

        let (type_length_id, bitvec) = bitvec.split_at(1);
        let type_length_id = *type_length_id
            .first()
            .as_deref()
            .ok_or(AdventError::InputEndedPrematurely)?;

        let (length_type_id, bitvec) = if type_length_id {
            // If first bit is a one, then next 11 bits are the number of sub packets
            let (num_packets, bitvec) = bitvec.split_at(11);
            let num_packets = num_packets.load_be::<usize>();
            (LengthType::NumSubPackets(num_packets), bitvec)
        } else {
            // If first bit is a zero, then next 15 bits are the number of bits making up the
            // sub packets
            let (num_bits, bitvec) = bitvec.split_at(15);
            let num_bits = num_bits.load_be::<usize>();
            (LengthType::NumBits(bitvec.len() - num_bits), bitvec)
        };

        // Parse the sub packets
        let (bitvec, sub_packets) = Packet::parse_subpackets(bitvec, length_type_id)?;

        // Get the appropriate operator
        let operator = PacketType::Operator(
            match type_id {
                0 => Ok(OperatorType::Sum),
                1 => Ok(OperatorType::Product),
                2 => Ok(OperatorType::Minimum),
                3 => Ok(OperatorType::Maximum),
                5 => Ok(OperatorType::GreaterThan),
                6 => Ok(OperatorType::LessThan),
                7 => Ok(OperatorType::EqualTo),
                _ => Err(AdventError::InvalidTypeId { type_id }),
            }?,
            sub_packets,
        );

        Ok((bitvec, operator))
    }
}

enum LengthType {
    NumBits(usize),
    NumSubPackets(usize),
}

#[derive(Debug, Clone)]
struct Packet {
    version: u8,
    type_id: PacketType,
}

impl Packet {
    fn from_str(input: &str) -> Result<Packet, AdventError> {
        // Parse hex string into packet
        let bitvec = BitVec::<Msb0, _>::from_vec(Vec::<u8>::from_hex(input)?);
        let (_bitvec, packet) = Packet::from_bitvec(&bitvec)?;
        Ok(packet)
    }

    fn from_bitvec(
        bitvec: &BitSlice<Msb0, u8>,
    ) -> Result<(&BitSlice<Msb0, u8>, Packet), AdventError> {
        // Parse bitvec into packet

        // First three bits are version number
        let (version, bitvec) = bitvec.split_at(3);
        let version = version.load_be::<u8>();

        // Next three bits are type id
        let (type_id, bitvec) = bitvec.split_at(3);
        let type_id = type_id.load_be::<u8>();
        let (bitvec, type_id) = match type_id {
            4 => PacketType::parse_literal(bitvec)?,
            _ => PacketType::parse_operator(bitvec, type_id)?,
        };

        Ok((bitvec, Packet { type_id, version }))
    }

    fn parse_subpackets(
        bitvec: &BitSlice<Msb0, u8>,
        length_type_id: LengthType,
    ) -> Result<(&BitSlice<Msb0, u8>, Vec<Packet>), AdventError> {
        // Recursively get sub packets until we've reached the
        // 1. number of bits (length type id = 0)
        // 2. number of sub packets (length type id = 1)

        fn recurse(
            bitvec: &BitSlice<Msb0, u8>,
            length_type_id: LengthType,
            depth: usize,
        ) -> Result<(&BitSlice<Msb0, u8>, Vec<Packet>), AdventError> {
            match length_type_id {
                // Stop if we've reached the number of bits
                LengthType::NumBits(target_length) if bitvec.len() <= target_length => {
                    Ok((bitvec, Vec::new()))
                }
                // Stop if we've reached the number of sub packets
                LengthType::NumSubPackets(num_packets) if num_packets == depth => {
                    Ok((bitvec, Vec::new()))
                }
                // Otherwise, recurse
                _ => {
                    let (bitvec, packet) = Packet::from_bitvec(bitvec)?;
                    let (bitvec, packets) = recurse(bitvec, length_type_id, depth + 1)?;
                    Ok((bitvec, vec![packet].into_iter().chain(packets).collect()))
                }
            }
        }
        recurse(bitvec, length_type_id, 0)
    }

    fn collapse(self) -> Result<usize, AdventError> {
        // Collapse a packet down to a single value
        match self.type_id {
            PacketType::Literal(x) => Ok(x),
            PacketType::Operator(operator_type, sub_packets) => {
                let operator = match operator_type {
                    OperatorType::Sum => |x, y| x + y,
                    OperatorType::Product => |x, y| x * y,
                    OperatorType::Minimum => std::cmp::min,
                    OperatorType::Maximum => std::cmp::max,
                    OperatorType::GreaterThan => |x, y| if x > y { 1 } else { 0 },
                    OperatorType::LessThan => |x, y| if x < y { 1 } else { 0 },
                    OperatorType::EqualTo => |x, y| if x == y { 1 } else { 0 },
                };
                sub_packets
                    .into_iter()
                    .map(|p| p.collapse())
                    .collect::<Result<Vec<_>, AdventError>>()?
                    .into_iter()
                    .reduce(operator)
                    .ok_or(AdventError::TooFewSubPackets)
            }
        }
    }

    fn add_versions(self) -> usize {
        // In part one, we just have to add up the version numbers
        match self.type_id {
            PacketType::Literal(_) => self.version as usize,
            PacketType::Operator(_, sub_packets) => {
                self.version as usize
                    + sub_packets
                        .into_iter()
                        .map(|p| p.add_versions())
                        .sum::<usize>()
            }
        }
    }
}

fn day_16() -> Result<usize, AdventError> {
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
    let packet = Packet::from_str(input)?;

    Ok(match question_part {
        QuestionPart::One => packet.add_versions(),
        QuestionPart::Two => packet.collapse()?,
    })
}

fn main() {
    match day_16() {
        Ok(answer) => println!("{}", answer),
        Err(err) => {
            eprintln!("{}", err);
            exit(1);
        }
    }
}

#[cfg(test)]
mod tests;
