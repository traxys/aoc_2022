use std::{collections::HashSet, time::Instant};

use crate::{load, print_res};
use bstr::{BString, ByteSlice};

#[derive(Debug, Clone, Copy)]
pub enum Direction {
    Left,
    Right,
    Up,
    Down,
}

type Parsed = Vec<(Direction, usize)>;

pub fn parsing(input: &BString) -> color_eyre::Result<Parsed> {
    input
        .lines()
        .map(|line| -> color_eyre::Result<_> {
            let Some((dir, amount)) = line.split_once_str(" ") else {
                color_eyre::eyre::bail!("Malformed line: {}", line.as_bstr());
            };
            let dir = match dir {
                b"R" => Direction::Right,
                b"L" => Direction::Left,
                b"U" => Direction::Up,
                b"D" => Direction::Down,
                _ => color_eyre::eyre::bail!("Invalid direction: {:?}", dir.as_bstr()),
            };
            Ok((dir, amount.to_str()?.parse()?))
        })
        .collect()
}

#[derive(Debug)]
struct Rope {
    segments: Vec<(isize, isize)>,
}

impl Rope {
    fn new(len: usize) -> Self {
        Rope {
            segments: vec![(0, 0); len],
        }
    }

    fn head(&mut self) -> &mut (isize, isize) {
        self.segments.first_mut().unwrap()
    }

    fn move_dir(&mut self, direction: Direction) {
        match direction {
            Direction::Left => {
                self.head().0 -= 1;
            }
            Direction::Right => {
                self.head().0 += 1;
            }
            Direction::Up => {
                self.head().1 += 1;
            }
            Direction::Down => {
                self.head().1 -= 1;
            }
        }

        for i in 1..self.segments.len() {
            if self.segments[i - 1].0.abs_diff(self.segments[i].0) > 1 {
                self.segments[i].0 += (self.segments[i - 1].0 - self.segments[i].0).signum();
                if self.segments[i - 1].1 != self.segments[i].1 {
                    self.segments[i].1 += self.segments[i - 1].1 - self.segments[i].1;
                }
            } else if self.segments[i - 1].1.abs_diff(self.segments[i].1) > 1 {
                self.segments[i].1 += (self.segments[i - 1].1 - self.segments[i].1).signum();
                if self.segments[i - 1].0 != self.segments[i].0 {
                    self.segments[i].0 += self.segments[i - 1].0 - self.segments[i].0;
                }
            }
        }
    }

    fn display_in(&self, (x_min, x_max): (isize, isize), (y_min, y_max): (isize, isize)) {
        for y in (y_min..=y_max).rev() {
            for x in x_min..=x_max {
                let segment = self
                    .segments
                    .iter()
                    .enumerate()
                    .find(|(_, &i)| i == (x, y))
                    .map(|(i, _)| i);
                match segment {
                    Some(0) => print!("H"),
                    Some(i) if i == self.segments.len() - 1 => print!("T"),
                    Some(x) => print!("{x}"),
                    None => print!("."),
                }
            }
            println!()
        }
    }
}

pub fn part1(input: Parsed) {
    let mut rope = Rope::new(2);
    let mut visited_tails = HashSet::new();
    visited_tails.insert(*rope.segments.last().unwrap());
    for (dir, amount) in input {
        for _ in 0..amount {
            rope.move_dir(dir);
            visited_tails.insert(*rope.segments.last().unwrap());
        }
    }

    print_res!("Total positions visited: {}", visited_tails.len());
}

pub fn part2(input: Parsed) {
    let mut rope = Rope::new(10);
    let mut visited_tails = HashSet::new();
    visited_tails.insert(*rope.segments.last().unwrap());
    for (dir, amount) in input {
        dbg!(dir, amount);
        for _ in 0..amount {
            println!();
            rope.move_dir(dir);
            visited_tails.insert(*rope.segments.last().unwrap());
            rope.display_in((0, 5), (0, 4));
        }
    }

    print_res!("Total positions visited: {}", visited_tails.len());
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
