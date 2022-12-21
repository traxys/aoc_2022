use std::{collections::HashMap, str::FromStr, time::Instant};

use crate::{load, print_res};
use bstr::{BStr, BString, ByteSlice};

#[derive(Debug, Clone, Copy)]
pub enum OpKind {
    Sub,
    Add,
    Div,
    Mult,
}

impl FromStr for OpKind {
    type Err = color_eyre::Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "+" => Ok(OpKind::Add),
            "-" => Ok(OpKind::Sub),
            "*" => Ok(OpKind::Mult),
            "/" => Ok(OpKind::Div),
            _ => Err(color_eyre::eyre::eyre!("Invalid operation")),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Operation {
    Literal(i64),
    BinOp { lhs: u16, rhs: u16, kind: OpKind },
}

type OperationMap<'a> = HashMap<u16, (&'a BStr, Operation)>;

type Parsed<'a> = (u16, u16, OperationMap<'a>);

pub fn parsing(input: &BString) -> color_eyre::Result<Parsed> {
    let mut monkeys = HashMap::new();
    for (i, line) in input.lines().enumerate() {
        let Some((name, sentence)) = line.split_once_str(":") else {
            color_eyre::eyre::bail!("Invalid line, no colon")
        };

        monkeys.insert(name, (i as u16, sentence));
    }
    let mut monkeys_interned = HashMap::new();

    let resolve_monkey = |m: &str| {
        monkeys
            .get(m.as_bytes())
            .map(|m| m.0)
            .ok_or(color_eyre::eyre::eyre!("Unresolved monkey {m}"))
    };

    let mut root = None;
    let mut human = None;

    for (&name, &(idx, operation)) in &monkeys {
        let mut operation = operation.trim().to_str()?.split_whitespace();
        let operation = match (operation.next(), operation.next(), operation.next()) {
            (Some(i), None, None) => Operation::Literal(i.parse()?),
            (Some(lhs), Some(op), Some(rhs)) => Operation::BinOp {
                lhs: resolve_monkey(lhs)?,
                rhs: resolve_monkey(rhs)?,
                kind: op.parse()?,
            },
            _ => todo!(),
        };

        monkeys_interned.insert(idx, (name.as_bstr(), operation));

        if name == b"root" {
            root = Some(idx);
        } else if name == b"humn" {
            human = Some(idx);
        }
    }

    Ok((
        root.ok_or(color_eyre::eyre::eyre!("No root found"))?,
        human.ok_or(color_eyre::eyre::eyre!("No human found"))?,
        monkeys_interned,
    ))
}

fn monkey_value(monkey: u16, operations: &mut OperationMap) -> i64 {
    match operations[&monkey] {
        (_, Operation::Literal(l)) => l,
        (name, Operation::BinOp { lhs, rhs, kind }) => {
            let lhs = monkey_value(lhs, operations);
            let rhs = monkey_value(rhs, operations);
            let value = match kind {
                OpKind::Sub => lhs - rhs,
                OpKind::Add => lhs + rhs,
                OpKind::Div => lhs / rhs,
                OpKind::Mult => lhs * rhs,
            };
            operations.insert(monkey, (name, Operation::Literal(value)));
            value
        }
    }
}

pub fn part1((root, _, mut monkeys): Parsed) {
    print_res!("Value of root: {}", monkey_value(root, &mut monkeys))
}

fn tree_contains(monkey: u16, from: u16, operations: &OperationMap) -> bool {
    if monkey == from {
        true
    } else {
        match operations[&from].1 {
            Operation::Literal(_) => false,
            Operation::BinOp { lhs, rhs, kind: _ } => {
                tree_contains(monkey, lhs, operations) || tree_contains(monkey, rhs, operations)
            }
        }
    }
}

fn reduce_branches(from: u16, human: u16, operations: &mut OperationMap) {
    if let (_, Operation::BinOp { lhs, rhs, kind: _ }) = operations[&from] {
        if tree_contains(human, lhs, operations) {
            reduce_branches(lhs, human, operations)
        } else {
            monkey_value(lhs, operations);
        }

        if tree_contains(human, rhs, operations) {
            reduce_branches(rhs, human, operations)
        } else {
            monkey_value(rhs, operations);
        }
    }
}

fn set_equal(monkey: u16, value: i64, human: u16, operations: &mut OperationMap) -> i64 {
    if monkey == human {
        return value;
    }

    let Operation::BinOp {lhs, rhs, kind } = operations[&monkey].1 else {
        panic!("Can't set equal literal")
    };

    let (variable, constant) = if tree_contains(human, lhs, operations) {
        assert!(!tree_contains(human, rhs, operations));
        (lhs, rhs)
    } else {
        (rhs, lhs)
    };
    let constant_value = monkey_value(constant, operations);

    match kind {
        OpKind::Add => set_equal(variable, value - constant_value, human, operations),
        OpKind::Mult => {
            assert_eq!(value % constant_value, 0);
            set_equal(variable, value / constant_value, human, operations)
        }
        OpKind::Sub => {
            if constant == lhs {
                set_equal(variable, constant_value - value, human, operations)
            } else {
                set_equal(variable, value + constant_value, human, operations)
            }
        }
        OpKind::Div => {
            if constant == lhs {
                assert_eq!(constant_value % value, 0);
                set_equal(variable, constant_value / value, human, operations)
            } else {
                set_equal(variable, value * constant_value, human, operations)
            }
        }
    }
}

pub fn part2((root, human, mut monkeys): Parsed) {
    let Operation::BinOp {lhs, rhs, kind: _} = monkeys[&root].1 else {
        panic!()
    };
    let (variable, constant) = if tree_contains(human, lhs, &monkeys) {
        assert!(!tree_contains(human, rhs, &monkeys));
        (lhs, rhs)
    } else {
        (rhs, lhs)
    };
    let constant_value = monkey_value(constant, &mut monkeys);
    reduce_branches(variable, human, &mut monkeys);

    let human_value = set_equal(variable, constant_value, human, &mut monkeys);
    print_res!("Value needed for human: {}", human_value)
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
