use std::{cell::RefCell, collections::VecDeque, time::Instant};

use crate::{load, print_res};
use bstr::{BString, ByteSlice};
use itertools::Itertools;

#[derive(Debug, Clone)]
enum WorryOperand {
    Literal(u64),
    Old,
}

impl WorryOperand {
    fn from_bytes(b: &[u8]) -> color_eyre::Result<Self> {
        match b.trim() {
            b"old" => Ok(Self::Old),
            lit => Ok(Self::Literal(lit.to_str()?.parse()?)),
        }
    }
}

#[derive(Debug, Clone)]
enum WorryUpdate {
    Mult(WorryOperand, WorryOperand),
    Sum(WorryOperand, WorryOperand),
}

#[derive(Debug, Clone)]
pub struct Monkey {
    items: VecDeque<u64>,
    update: WorryUpdate,
    test_diviser: u64,
    true_target: usize,
    false_target: usize,

    inpsected: usize,
}

type Parsed = Vec<Monkey>;

pub fn parsing(input: &BString) -> color_eyre::Result<Parsed> {
    input
        .split_str("\n\n")
        .map(|monkey| -> color_eyre::Result<_> {
            let mut lines = monkey.lines();

            let mut next_line = || match lines.next() {
                Some(x) => Ok(x),
                None => color_eyre::eyre::bail!("Unexpected EOF"),
            };

            color_eyre::eyre::ensure!(
                next_line()?.starts_with(b"Monkey"),
                "Monkey does not start with magic"
            );

            let Some(items) = next_line()?.trim().strip_prefix(b"Starting items:") else {
                color_eyre::eyre::bail!("Missing starting items magic")
            };

            let items = items
                .to_str()?
                .split(',')
                .map(|i| i.trim().parse())
                .collect::<Result<_, _>>()?;

            let Some(update) = next_line()?.trim().strip_prefix(b"Operation: new = ") else {
                color_eyre::eyre::bail!("Missing operation magic")
            };
            let op = if update.contains(&b'*') { "*" } else { "+" };
            let (op_a, op_b) = update.split_once_str(op).unwrap();

            let op_a = WorryOperand::from_bytes(op_a)?;
            let op_b = WorryOperand::from_bytes(op_b)?;

            let update = match op {
                "*" => WorryUpdate::Mult(op_a, op_b),
                "+" => WorryUpdate::Sum(op_a, op_b),
                _ => unreachable!(),
            };

            let Some(test_diviser) = next_line()?.trim().strip_prefix(b"Test: divisible by ") else {
                color_eyre::eyre::bail!("Missing test magic")
            };
            let test_diviser = test_diviser.to_str()?.parse()?;

            let Some(true_target) = next_line()?.trim().strip_prefix(b"If true: throw to monkey ") else {
                color_eyre::eyre::bail!("Missing true target magic")
            };
            let true_target = true_target.to_str()?.parse()?;

            let Some(false_target) = next_line()?.trim().strip_prefix(b"If false: throw to monkey ") else {
                color_eyre::eyre::bail!("Missing true target magic")
            };
            let false_target = false_target.to_str()?.parse()?;

            Ok(Monkey {
                items,
                update,
                test_diviser,
                true_target,
                false_target,

                inpsected: 0,
            })
        })
        .collect()
}

impl WorryOperand {
    fn value(&self, old: u64) -> u64 {
        match self {
            &WorryOperand::Literal(l) => l,
            WorryOperand::Old => old,
        }
    }
}

impl WorryUpdate {
    fn monkey_inspects(&self, old: u64) -> u64 {
        match self {
            WorryUpdate::Mult(a, b) => a.value(old) * b.value(old),
            WorryUpdate::Sum(a, b) => a.value(old) + b.value(old),
        }
    }
}

impl Monkey {
    fn turn(&mut self, monkeys: &[RefCell<Monkey>]) {
        while let Some(item) = self.items.pop_front() {
            self.inpsected += 1;

            let worried_item = self.update.monkey_inspects(item);
            let worried_item = worried_item / 3;
            let target = if worried_item % self.test_diviser == 0 {
                self.true_target
            } else {
                self.false_target
            };
            monkeys[target].borrow_mut().items.push_back(worried_item);
        }
    }

    fn long_turn(&mut self, monkeys: &[RefCell<Monkey>], modulo_count: u64) {
        while let Some(item) = self.items.pop_front() {
            self.inpsected += 1;

            let worried_item = self.update.monkey_inspects(item);
            let worried_item = worried_item % modulo_count;
            let target = if worried_item % self.test_diviser == 0 {
                self.true_target
            } else {
                self.false_target
            };
            monkeys[target].borrow_mut().items.push_back(worried_item);
        }
    }
}

pub fn part1(input: Parsed) {
    let monkeys: Vec<_> = input.into_iter().map(RefCell::new).collect();

    for _ in 0..20 {
        for monkey in &monkeys {
            monkey.borrow_mut().turn(&monkeys);
        }
    }
    let monkey_levels: usize = monkeys
        .into_iter()
        .map(|m| std::cmp::Reverse(m.into_inner().inpsected))
        .k_smallest(2)
        .map(|k| k.0)
        .product();

    print_res!("Monkey levels are: {monkey_levels}")
}

pub fn part2(input: Parsed) {
    let modulo_count: u64 = input.iter().map(|m| m.test_diviser).product();

    let monkeys: Vec<_> = input.into_iter().map(RefCell::new).collect();

    for _ in 0..10000 {
        for monkey in &monkeys {
            monkey.borrow_mut().long_turn(&monkeys, modulo_count);
        }
    }
    let monkey_levels: usize = monkeys
        .into_iter()
        .map(|m| std::cmp::Reverse(m.into_inner().inpsected))
        .k_smallest(2)
        .map(|k| k.0)
        .product();

    print_res!("Monkey levels are: {monkey_levels}")
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
