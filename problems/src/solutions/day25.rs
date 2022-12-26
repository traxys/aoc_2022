use std::time::Instant;

use crate::{load, print_res};
use bstr::{BString, ByteSlice};

type Parsed = Vec<i64>;

fn parse_snafu(num: &[u8]) -> color_eyre::Result<i64> {
    num.iter()
        .rev()
        .enumerate()
        .map(|(i, &v)| {
            let snafuit = match v {
                b'0' => 0,
                b'1' => 1,
                b'2' => 2,
                b'-' => -1,
                b'=' => -2,
                _ => color_eyre::eyre::bail!("Invalid digit: {}", char::from(v)),
            };
            Ok((i, snafuit))
        })
        .try_fold(0, |acc, res| -> color_eyre::Result<_> {
            let (i, snafuit) = res?;
            Ok(acc + snafuit * 5i64.pow(i as u32))
        })
}

pub fn parsing(input: &BString) -> color_eyre::Result<Parsed> {
    input.lines().map(parse_snafu).collect()
}

fn snafu_bits(n: i64) -> u32 {
    let lower_bits = (2 * n + 1).ilog(5);
    if max_snafu(lower_bits) == n {
        lower_bits
    } else {
        lower_bits + 1
    }
}

fn max_snafu(bits: u32) -> i64 {
    (5i64.pow(bits) - 1) / 2
}

fn to_snafu(mut n: i64) -> String {
    let mut num = String::new();
    let mut current_bits = snafu_bits(n);

    let symbol = |neg: bool, v: u16| {
        if neg {
            match v {
                1 => "-",
                2 => "=",
                _ => unreachable!(),
            }
        } else {
            match v {
                1 => "1",
                2 => "2",
                _ => unreachable!(),
            }
        }
    };

    while current_bits > 0 {
        let abs_n = n.abs();
        let bits = snafu_bits(abs_n);
        if current_bits == bits && bits != 0 {
            let mx = max_snafu(bits - 1);
            let mut digit = 5i64.pow(bits - 1);
            if abs_n <= mx + digit {
                num += symbol(n < 0, 1);
            } else {
                num += symbol(n < 0, 2);
                digit *= 2;
            }
            n = n.signum() * (abs_n - digit);
            current_bits -= 1;
        } else {
            for _ in bits..current_bits {
                num += "0"
            }
            current_bits = bits;
        }
    }
    num
}

// A SNAFU number N needs log_5(2N+1) snafu digit
pub fn part1(input: Parsed) {
    let sum: i64 = input.iter().sum();
    print_res!("Result is: {}", to_snafu(sum))
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
