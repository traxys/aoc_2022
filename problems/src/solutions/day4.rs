use std::{str::FromStr, time::Instant};

use bstr::{BString, ByteSlice};
use crate::load;

#[derive(Debug, Clone, Copy)]
pub struct Range {
    pub start: u64,
    pub end: u64,
}

impl Range {
    fn contains_range(&self, other: &Self) -> bool {
        self.start <= other.start && self.end >= other.end
    }

    fn overlap(&self, other: &Self) -> bool {
        let (smallest, largest) = if self.start <= other.start {
            (self, other)
        } else {
            (other, self)
        };

        largest.start <= smallest.end
    }
}

impl FromStr for Range {
    type Err = color_eyre::Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let Some((start,end)) = s.split_once('-') else {
            color_eyre::eyre::bail!("Malformed range {s}")
        };

        Ok(Range {
            start: start.parse()?,
            end: end.parse()?,
        })
    }
}

type Parsed = Vec<(Range, Range)>;

pub fn parsing(input: &BString) -> color_eyre::Result<Parsed> {
    input
        .lines()
        .map(|line| -> color_eyre::Result<_> {
            let Some((first,second)) = line.trim().split_once_str(",") else {
                color_eyre::eyre::bail!("Malformed range {}", line.as_bstr())
            };
            Ok((first.to_str()?.parse()?, second.to_str()?.parse()?))
        })
        .collect()
}

pub fn part1(input: Parsed) {
    let containg_count = input
        .iter()
        .filter(|(a, b)| a.contains_range(b) || b.contains_range(a))
        .count();
    println!("There are {containg_count} ranges containg others");
}

pub fn part2(input: Parsed) {
    let overlap_count = input.iter().filter(|(a, b)| a.overlap(b)).count();
    println!("There are {overlap_count} ranges overlapping");
}

pub fn main() -> color_eyre::Result<()> {
    let context = load()?;

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

#[cfg(test)]
mod test {
    use super::Parsed;

    const INPUT: &[u8] = br#"2-4,6-8
2-3,4-5
5-7,7-9
2-8,3-7
6-6,4-6
2-6,4-8"#;

    fn ranges() -> Parsed {
        super::parsing(&INPUT.to_vec().into()).unwrap()
    }

    #[test]
    fn part1() {
        let ranges = ranges();
        let containg_count = ranges
            .iter()
            .filter(|(a, b)| a.contains_range(b) || b.contains_range(a))
            .count();
        assert_eq!(containg_count, 2);
    }

    #[test]
    fn part2() {
        let ranges = ranges();
        let overlap_count = ranges.iter().filter(|(a, b)| a.overlap(b)).count();
        assert_eq!(overlap_count, 4);
    }
}
