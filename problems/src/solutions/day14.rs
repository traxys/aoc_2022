use std::{cmp::Ordering, collections::HashMap, time::Instant};

use crate::{load, print_res};
use bstr::{BString, ByteSlice};
use either::Either;
use itertools::Itertools;

#[derive(Clone, Copy, Debug)]
pub enum Blocker {
    Sand,
    Rock,
}

#[derive(Clone, thiserror::Error, Debug)]
#[error("Could not parse pair")]
enum PairErr {
    NoComma,
    Utf8Error(#[from] bstr::Utf8Error),
    ParseInt(#[from] std::num::ParseIntError),
}

type Parsed = HashMap<(u64, u64), Blocker>;

fn try_flat_map<U, T, E, F, I>(
    f: F,
    item: Result<U, E>,
) -> impl Iterator<Item = color_eyre::Result<T>>
where
    F: Fn(U) -> I,
    E: Into<color_eyre::Report>,
    I: Iterator<Item = T>,
{
    match item {
        Ok(i) => Either::Left(f(i).map(Ok)),
        Err(e) => Either::Right(std::iter::once(Err(e.into()))),
    }
}

pub fn parsing(input: &BString) -> color_eyre::Result<Parsed> {
    input
        .lines()
        .flat_map(|path| {
            path.split_str("->")
                .map(|pair| -> Result<(u64, u64), PairErr> {
                    let Some((start,end)) = pair.split_once_str(",") else {
                    return Err(PairErr::NoComma);
                };

                    Ok((
                        start.to_str()?.trim().parse()?,
                        end.to_str()?.trim().parse()?,
                    ))
                })
                .tuple_windows()
                .map(|(start, end)| match (start, end) {
                    (Ok(a), Ok(b)) => Ok((a, b)),
                    (Ok(_), Err(a)) => Err(a),
                    (Err(a), Ok(_)) => Err(a),
                    (Err(a), Err(_)) => Err(a),
                })
                .flat_map(|res| {
                    try_flat_map(
                        |(start, end)| {
                            #[derive(Debug, Clone, Copy)]
                            enum PathDir {
                                Incr,
                                Decr,
                            }

                            #[derive(Debug, Clone, Copy)]
                            enum GlobalDir {
                                Vertical,
                                Horizontal,
                            }

                            let calc_mult = |a: u64, b: u64| match a.cmp(&b) {
                                Ordering::Less => PathDir::Decr,
                                Ordering::Equal => panic!("both can't be equal"),
                                Ordering::Greater => PathDir::Incr,
                            };

                            let (direction, path_dir, len) = if start.0 == end.0 {
                                (
                                    GlobalDir::Horizontal,
                                    calc_mult(end.1, start.1),
                                    end.1.abs_diff(start.1),
                                )
                            } else {
                                (
                                    GlobalDir::Vertical,
                                    calc_mult(end.0, start.0),
                                    end.0.abs_diff(start.0),
                                )
                            };

                            (0..=len).map(move |x| {
                                let start_single = match direction {
                                    GlobalDir::Vertical => start.0,
                                    GlobalDir::Horizontal => start.1,
                                };
                                let segment = match path_dir {
                                    PathDir::Incr => start_single + x,
                                    PathDir::Decr => start_single - x,
                                };

                                let coord = match direction {
                                    GlobalDir::Vertical => (segment, start.1),
                                    GlobalDir::Horizontal => (start.0, segment),
                                };

                                (coord, Blocker::Rock)
                            })
                        },
                        res,
                    )
                })
        })
        .collect()
}

/* fn draw_grid(grid: &Parsed) {
    let mut min_x = u64::MAX;
    let mut max_x = 0;
    let mut max_y = 0;

    for &(x, y) in grid.keys() {
        if x < min_x {
            min_x = x;
        }
        if x > max_x {
            max_x = x;
        }

        if y > max_y {
            max_y = y;
        }
    }

    for y in 0..=(max_y + 1) {
        for x in min_x..=(max_x + 1) {
            if (x, y) != (500, 0) {
                match grid.get(&(x, y)) {
                    Some(Blocker::Rock) => print!("#"),
                    Some(Blocker::Sand) => print!("o"),
                    None => print!("."),
                }
            } else {
                print!("+")
            }
        }
        println!()
    }
} */

fn find_shelves(grid: &Parsed) -> ((u64, u64), Vec<u64>) {
    let mut min_x = u64::MAX;
    let mut max_x = 0;
    for &(x, _) in grid.keys() {
        if x < min_x {
            min_x = x
        }

        if x > max_x {
            max_x = x
        }
    }

    let mut shelves = vec![0; (max_x - min_x + 1) as usize];

    for &(x, y) in grid.keys() {
        let index = (x - min_x) as usize;
        if shelves[index] < y {
            shelves[index] = y;
        }
    }

    ((min_x, max_x), shelves)
}

fn lay_sand_abyss(grid: &mut Parsed, shelves: &[u64], min_x: u64, max_x: u64) -> bool {
    let (mut x, mut y) = (500, 0);

    loop {
        y += 1;
        if !grid.contains_key(&(x, y)) {
            if x < min_x || x > max_x {
                return false;
            }

            let shelf_index = (x - min_x) as usize;
            if y >= shelves[shelf_index] {
                return false;
            } else {
                continue;
            }
        } else if !grid.contains_key(&(x - 1, y)) {
            x -= 1;
            continue;
        } else if !grid.contains_key(&(x + 1, y)) {
            x += 1;
            continue;
        } else {
            grid.insert((x, y - 1), Blocker::Sand);
            return true;
        }
    }
}

pub fn part1(mut input: Parsed) {
    let ((min_x, max_x), shelves) = find_shelves(&input);

    let mut sand_count = 0;

    while lay_sand_abyss(&mut input, &shelves, min_x, max_x) {
        sand_count += 1;
    }

    print_res!("Sand count: {sand_count}");
}

fn lay_sand_floor(grid: &mut Parsed, floor_y: u64) {
    let (mut x, mut y) = (500, 0);

    let is_blocked = |px, py| grid.contains_key(&(px, py)) || py == floor_y;

    loop {
        y += 1;
        if !is_blocked(x, y) {
        } else if !is_blocked(x - 1, y) {
            x -= 1;
            continue;
        } else if !is_blocked(x + 1, y) {
            x += 1;
            continue;
        } else {
            grid.insert((x, y - 1), Blocker::Sand);
            return;
        }
    }
}

pub fn part2(mut input: Parsed) {
    let max_y = input.keys().map(|&(_, y)| y).max().unwrap();

    let mut sand_count = 0;
    while !input.contains_key(&(500, 0)) {
        lay_sand_floor(&mut input, max_y + 2);
        sand_count += 1;
    }

    print_res!("Sand count: {sand_count}");
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
