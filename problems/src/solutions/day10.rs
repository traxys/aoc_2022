use std::time::Instant;

use crate::{load, print_res, print_res_part};
use bstr::{BString, ByteSlice};
use either::Either;

#[derive(Debug, Clone, Copy)]
pub enum Instr {
    Noop,
    AddxStart,
    AddxDo(i64),
}

type Parsed = Vec<Instr>;

pub fn parsing(input: &BString) -> color_eyre::Result<Parsed> {
    input
        .lines()
        .flat_map(|v| {
            if v == b"noop" {
                Either::Left(std::iter::once(Ok(Instr::Noop)))
            } else if let Some(num) = v.strip_prefix(b"addx") {
                let num = match num.trim().to_str() {
                    Ok(n) => n.parse().map_err(Into::into),
                    Err(e) => Err(e.into()),
                };
                let instr = num.map(Instr::AddxDo);
                Either::Right([Ok(Instr::AddxStart), instr].into_iter())
            } else {
                Either::Left(std::iter::once(Err(color_eyre::eyre::eyre!(
                    "Invalid instruction: {}",
                    v.as_bstr()
                ))))
            }
        })
        .collect()
}

pub fn part1(input: Parsed) {
    let mut acc = 1;
    let mut signals = 0;
    for (cycle, instr) in input.iter().enumerate().map(|(i, v)| (i + 1, v)).take(220) {
        if cycle >= 20 && (cycle - 20) % 40 == 0 {
            signals += acc * (cycle as i64);
        }

        match instr {
            Instr::Noop | Instr::AddxStart => {}
            Instr::AddxDo(num) => {
                acc += num;
            }
        }
    }

    print_res!("Signal sum is: {signals}")
}

pub fn part2(input: Parsed) {
    let mut acc = 1;
    for (cycle, instr) in input.iter().enumerate().take(40 * 6) {
        let position = (cycle % 40) as i64;

        if position == 0 {
            print_res!()
        }

        if position.abs_diff(acc) <= 1 {
            print_res_part!("#");
        } else {
            print_res_part!(".");
        }

        match instr {
            Instr::Noop | Instr::AddxStart => {}
            Instr::AddxDo(num) => {
                acc += num;
            }
        }
    }
    print_res!()
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
