use std::{collections::VecDeque, time::Instant};

use bstr::{BString, ByteSlice};
use crate::load;

type Parsed = (Vec<VecDeque<u8>>, Vec<(u8, u8, u8)>);

pub fn parsing(input: &BString) -> color_eyre::Result<Parsed> {
    let Some((crates, instructions)) = input.split_once_str("\n\n") else {
        color_eyre::eyre::bail!("Malformed input, has no empty line");
    };

    let Some(last_line) = crates.lines().last() else {
        color_eyre::eyre::bail!("Crate input is empty");
    };
    let stack_count = last_line.to_str()?.trim().split_ascii_whitespace().count();
    let mut stacks: Vec<_> = std::iter::repeat_with(VecDeque::new)
        .take(stack_count)
        .collect();
    for row in crates.lines().rev().skip(1) {
        for (part, stack) in row.chunks(4).zip(&mut stacks) {
            if part.starts_with(b"[") {
                stack.push_front(part[1] - b'A');
            }
        }
    }

    // Instructions
    let instructions: Result<_, _> = instructions
        .lines()
        .map(|part| -> color_eyre::Result<_> {
            let part = part.to_str()?;
            let mut instrs = part.split_ascii_whitespace();
            instrs.next();
            let count = instrs
                .next()
                .ok_or_else(|| color_eyre::eyre::eyre!("invalid instruction {part}"))?;

            instrs.next();
            let from = instrs
                .next()
                .ok_or_else(|| color_eyre::eyre::eyre!("invalid instruction {part}"))?;

            instrs.next();
            let to = instrs
                .next()
                .ok_or_else(|| color_eyre::eyre::eyre!("invalid instruction {part}"))?;

            Ok((count.parse()?, from.parse()?, to.parse()?))
        })
        .collect();

    Ok((stacks, instructions?))
}

pub fn part1(input: Parsed) {
    let (mut state, instr) = input;

    for (count, from, to) in instr {
        for _ in 0..count {
            let var = state[from as usize - 1]
                .pop_front()
                .unwrap_or_else(|| panic!("Stack {from} was empty"));
            state[to as usize - 1].push_front(var);
        }
    }

    print!("Crates are: ");
    for mut stack in state {
        let c: char = (stack.pop_front().expect("Stack is empty") + b'A').into();
        print!("{c}");
    }
    println!()
}

pub fn part2(input: Parsed) {
    let (mut state, instr) = input;

    for (count, from, to) in instr {
        let items: Vec<_> = state[from as usize - 1]
            .drain(0..(count as usize))
            .rev()
            .collect();
        items
            .iter()
            .for_each(|&i| state[to as usize - 1].push_front(i))
    }

    print!("Crates are: ");
    for mut stack in state {
        let c: char = (stack.pop_front().expect("Stack is empty") + b'A').into();
        print!("{c}");
    }
    println!()
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
