use std::time::Instant;

use crate::{load, print_res};
use bstr::{BString, ByteSlice};
use color_eyre::eyre::Context;
use itertools::Itertools;
use serde::Deserialize;

#[derive(Clone, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum Value {
    Num(i64),
    List(Vec<Value>),
}

impl std::fmt::Debug for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Num(arg0) => write!(f, "{arg0}"),
            Self::List(arg0) => f.debug_list().entries(arg0).finish(),
        }
    }
}

macro_rules! v {
    ($($items:expr)?) => {
        Value::List(vec![$($items)*])
    };
}

impl Ord for Value {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match (self, other) {
            (Value::Num(a), Value::Num(b)) => a.cmp(b),
            (&Value::Num(x), b @ Value::List(_)) => Value::List(vec![Value::Num(x)]).cmp(b),
            (a @ Value::List(_), &Value::Num(x)) => a.cmp(&Value::List(vec![Value::Num(x)])),
            (Value::List(a), Value::List(b)) => a.cmp(b),
        }
    }
}

impl PartialOrd for Value {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

type Parsed = Vec<(Value, Value)>;

pub fn parsing(input: &BString) -> color_eyre::Result<Parsed> {
    input
        .split_str("\n\n")
        .map(|packets| -> color_eyre::Result<_> {
            let Some((first, second)) = packets.split_once_str("\n") else {
                color_eyre::eyre::bail!("Invalid packet pair: {}", packets.as_bstr())
            };

            Ok((
                serde_json::from_slice(first)
                    .with_context(|| format!("Invalid json list: {}", first.as_bstr()))?,
                serde_json::from_slice(second)
                    .with_context(|| format!("Invalid json list: {}", second.as_bstr()))?,
            ))
        })
        .collect()
}

pub fn part1(input: Parsed) {
    let index_sum: usize = input
        .iter()
        .enumerate()
        .filter(|(_, (a, b))| a < b)
        .map(|(i, _)| i + 1)
        .sum();
    print_res!("Sum of in order indices is: {index_sum}");
}

pub fn part2(input: Parsed) {
    let extra_values = [&v![v![Value::Num(2)]], &v![v![Value::Num(6)]]];
    let decoder_key: usize = input
        .iter()
        .flat_map(|(a, b)| [a, b])
        .chain(extra_values)
        .sorted()
        .enumerate()
        .filter(|(_, x)| extra_values.contains(x))
        .map(|(i, _)| i + 1)
        .product();
    print_res!("Decoder key: {decoder_key}");
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
