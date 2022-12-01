use std::time::Instant;

use bstr::{BString, ByteSlice};
use itertools::Itertools;
use problems::load;

type Parsed = Vec<Vec<u64>>;

fn parsing(input: &BString) -> color_eyre::Result<Parsed> {
    Ok(input
        .trim()
        .split_str("\n\n")
        .map(|elf| {
            elf.split_str("\n")
                .map(|item| item.trim().to_str().unwrap().parse().unwrap())
                .collect()
        })
        .collect())
}

fn part1(input: Parsed) {
    let max = input.iter().map(|b| b.iter().sum::<u64>()).max().unwrap();
    println!("Max calories are: {max}");
}

fn part2(input: Parsed) {
    let first_three: u64 = input
        .iter()
        .map(|b| b.iter().sum::<u64>())
        .sorted_by(|a, b| b.cmp(a))
        .take(3)
        .sum();
    println!("Sum of three first elf's calories: {first_three}")
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
