use std::time::Instant;

use crate::{load, print_res};
use bstr::BString;

type Parsed = ();

pub fn parsing(input: &BString) -> color_eyre::Result<Parsed> {
    todo!("Parsing")
}

pub fn part1(input: Parsed) {
    todo!("todo part1")
}

pub fn part2(input: Parsed) {
    todo!("todo part2")
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
