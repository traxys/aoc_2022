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

    fn segment_pair(&mut self, index: usize) -> (&mut (isize, isize), &mut (isize, isize)) {
        let (start, end) = self.segments.split_at_mut(index + 1);
        (start.last_mut().unwrap(), end.first_mut().unwrap())
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

        for i in 0..self.segments.len() - 1 {
            let (point, next) = self.segment_pair(i);
            let x_dir = point.0 - next.0;
            let y_dir = point.1 - next.1;

            let unit = |v: isize| if v != 0 { v / v.abs() } else { 0 };

            if x_dir.abs() >= 2 || y_dir.abs() >= 2 {
                let x_dir = unit(x_dir);
                let y_dir = unit(y_dir);

                next.0 += x_dir;
                next.1 += y_dir;
            }
        }
    }

    #[allow(dead_code)]
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
        for _ in 0..amount {
            rope.move_dir(dir);
            visited_tails.insert(*rope.segments.last().unwrap());
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
