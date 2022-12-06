use std::time::Instant;

use bstr::{BStr, BString, ByteSlice};
use itertools::Itertools;
use crate::load;

type Parsed<'a> = Vec<(Vec<u8>, Vec<u8>)>;

fn comp(s: &BStr) -> Vec<u8> {
    s.iter()
        .map(|&i| match i {
            b'a'..=b'z' => i - b'a',
            b'A'..=b'Z' => i - b'A' + 26,
            _ => unreachable!(),
        })
        .collect()
}

pub fn parsing(input: &BString) -> color_eyre::Result<Parsed> {
    Ok(input
        .trim()
        .split_str("\n")
        .map(|bag| {
            let bag = bag.trim();
            (
                comp(bag[0..bag.len() / 2].as_bstr()),
                comp(bag[bag.len() / 2..].as_bstr()),
            )
        })
        .collect())
}

fn occurences(part: &[u8]) -> [u32; 26 * 2] {
    let mut used = [0; 26 * 2];
    for &i in part {
        used[i as usize] += 1;
    }

    used
}

fn duplicate(a: &[u8], b: &[u8]) -> usize {
    let used_a = occurences(a);
    let used_b = occurences(b);

    used_a
        .iter()
        .zip(&used_b)
        .enumerate()
        .find(|(_, (&a, &b))| a != 0 && b != 0)
        .unwrap()
        .0
}

pub fn part1(input: Parsed) {
    let prio_sum: u64 = input.iter().map(|(a, b)| duplicate(a, b) as u64 + 1).sum();
    println!("Sum of priority: {prio_sum}");
}

fn merge_bag(mut a: [u32; 26 * 2], b: [u32; 26 * 2]) -> [u32; 26 * 2] {
    for (a, b) in a.iter_mut().zip(b) {
        *a += b;
    }

    a
}

pub fn part2(input: Parsed) {
    let mut total = 0;

    for group in &input
        .iter()
        .map(|(a, b)| merge_bag(occurences(a), occurences(b)))
        .chunks(3)
    {
        let mut present = [0; 26 * 2];
        for part in group {
            for (p, &v) in present.iter_mut().zip(&part) {
                *p += (v != 0) as i32;
            }
        }

        total += present.iter().enumerate().find(|(_, &p)| p == 3).unwrap().0 + 1;
    }

    println!("Total priority: {total}");
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
