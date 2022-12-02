use std::time::Instant;

use bstr::{BString, ByteSlice};
use problems::load;

#[derive(PartialEq, Eq, Clone, Copy)]
enum Move {
    Rock,
    Paper,
    Scissors,
}

enum SecondPart {
    X,
    Y,
    Z,
}

impl SecondPart {
    fn parse_bytes(bytes: &[u8]) -> color_eyre::Result<Self> {
        match bytes {
            b"X" => Ok(SecondPart::X),
            b"Y" => Ok(SecondPart::Y),
            b"Z" => Ok(SecondPart::Z),
            _ => color_eyre::eyre::bail!("Malformed move: {}", bytes.as_bstr()),
        }
    }
}

impl Move {
    fn parse_bytes(bytes: &[u8]) -> color_eyre::Result<Self> {
        match bytes {
            b"A" => Ok(Move::Rock),
            b"B" => Ok(Move::Paper),
            b"C" => Ok(Move::Scissors),
            _ => color_eyre::eyre::bail!("Malformed move: {}", bytes.as_bstr()),
        }
    }

    fn value(&self) -> u64 {
        match self {
            Move::Rock => 1,
            Move::Paper => 2,
            Move::Scissors => 3,
        }
    }

    fn loosing(&self) -> Move {
        match self {
            Move::Rock => Move::Scissors,
            Move::Paper => Move::Rock,
            Move::Scissors => Move::Paper,
        }
    }

    fn winning(&self) -> Move {
        match self {
            Move::Rock => Move::Paper,
            Move::Paper => Move::Scissors,
            Move::Scissors => Move::Rock,
        }
    }

    fn game(&self, other: &Self) -> u64 {
        if *other == self.winning() {
            0
        } else if *other == self.loosing() {
            6
        } else {
            3
        }
    }
}

type Parsed = Vec<(Move, SecondPart)>;

fn parsing(input: &BString) -> color_eyre::Result<Parsed> {
    input
        .lines()
        .map(|line| -> color_eyre::Result<_> {
            let Some((a, b)) = line.split_once_str(" ") else {color_eyre::eyre::bail!("Malformed input {}", line.as_bstr())};
            Ok((Move::parse_bytes(a)?, SecondPart::parse_bytes(b)?))
        })
        .collect()
}

fn part1(input: Parsed) {
    let sum: u64 = input
        .iter()
        .map(|(o, s)| {
            (
                o,
                match s {
                    SecondPart::X => Move::Rock,
                    SecondPart::Y => Move::Paper,
                    SecondPart::Z => Move::Scissors,
                },
            )
        })
        .map(|(o, s)| s.value() + s.game(o))
        .sum();
    println!("Total score is {sum}");
}

fn part2(input: Parsed) {
    let sum: u64 = input
        .iter()
        .map(|(o, s)| {
            (
                o,
                match s {
                    SecondPart::X => o.loosing(),
                    SecondPart::Y => *o,
                    SecondPart::Z => o.winning(),
                },
            )
        })
        .map(|(o, s)| s.value() + s.game(o))
        .sum();
    println!("Total score is {sum}");
}

fn main() -> color_eyre::Result<()> {
    let context = load(2)?;

    let start = Instant::now();
    let parsed = parsing(&context.input)?;
    let elapsed = humantime::format_duration(start.elapsed());

    let start = Instant::now();
    if context.part == 1 {
        part1(parsed);
    } else {
        part2(parsed);
    }
    let elapsed_part = humantime::format_duration(start.elapsed());

    println!("  Parsing: {elapsed}");
    println!("  Solving: {elapsed_part}");

    Ok(())
}
