use std::{collections::HashMap, time::Instant};

use crate::{load, print_res};
use bstr::{BString, ByteSlice};
use fnv::FnvHashSet;

type Parsed = FnvHashSet<(i64, i64)>;

pub fn parsing(input: &BString) -> color_eyre::Result<Parsed> {
    input
        .lines()
        .enumerate()
        .flat_map(|(y, line)| {
            line.iter().enumerate().filter_map(move |(x, c)| match c {
                b'#' => Some(Ok((x as i64, y as i64))),
                b'.' => None,
                &invalid => Some(Err(color_eyre::eyre::eyre!(
                    "Invalid character: {}",
                    char::from(invalid)
                ))),
            })
        })
        .collect()
}

#[allow(dead_code)]
fn print_board(board: &Parsed, min_x: i64, max_x: i64, min_y: i64, max_y: i64) {
    for y in min_y..=max_y {
        for x in min_x..=max_x {
            if board.contains(&(x, y)) {
                print!("#")
            } else {
                print!(".")
            }
        }
        println!()
    }
}

#[derive(Clone, Copy, Debug)]
enum Direction {
    North,
    South,
    West,
    East,
}

impl Direction {
    fn offsets(&self) -> [(i64, i64); 3] {
        match self {
            Direction::North => [(-1, -1), (0, -1), (1, -1)],
            Direction::South => [(-1, 1), (0, 1), (1, 1)],
            Direction::West => [(-1, -1), (-1, 0), (-1, 1)],
            Direction::East => [(1, -1), (1, 0), (1, 1)],
        }
    }
    fn moved(&self, x: i64, y: i64) -> (i64, i64) {
        match self {
            Direction::North => (x, y - 1),
            Direction::South => (x, y + 1),
            Direction::West => (x - 1, y),
            Direction::East => (x + 1, y),
        }
    }
}

fn neighbours(x: i64, y: i64) -> [(i64, i64); 8] {
    let mut output = [
        (-1, -1),
        (-1, 0),
        (-1, 1),
        (0, -1),
        (0, 1),
        (1, -1),
        (1, 0),
        (1, 1),
    ];

    for o in &mut output {
        *o = (o.0 + x, o.1 + y);
    }

    output
}

fn round(input: &Parsed, directions: &mut [Direction; 4]) -> (Parsed, bool) {
    let mut new_board = Parsed::default();
    let mut proposed = HashMap::new();
    for &(x, y) in input {
        let mut inserted = false;
        if neighbours(x, y).iter().any(|p| input.contains(p)) {
            for direction in directions.iter() {
                if direction
                    .offsets()
                    .iter()
                    .all(|(ox, oy)| !input.contains(&(x + ox, y + oy)))
                {
                    proposed
                        .entry(direction.moved(x, y))
                        .or_insert_with(Vec::new)
                        .push((x, y));
                    inserted = true;
                    break;
                }
            }
        }
        if !inserted {
            // println!("Could not move (all taken) {x}/{y}");
            new_board.insert((x, y));
        }
    }

    let mut moved = false;

    for ((nx, ny), old) in proposed {
        if old.len() == 1 {
            // println!("Could move to {nx}/{ny} from {}/{}", old[0].0, old[0].1);
            new_board.insert((nx, ny));
            moved = true;
        } else {
            old.iter().for_each(|&(x, y)| {
                // println!("Could not move to {nx}/{ny} from {x}/{y}");
                new_board.insert((x, y));
            })
        }
    }
    directions.rotate_left(1);

    (new_board, moved)
}

pub fn part1(mut input: Parsed) {
    //print_board(&input, 0, 4, 0, 5);
    //print_board(&input, -3, 10, -2, 9);
    let mut directions = [
        Direction::North,
        Direction::South,
        Direction::West,
        Direction::East,
    ];
    for _ in 0..10 {
        (input, _) = round(&input, &mut directions);
        //println!("------");
        //print_board(&input, 0, 4, 0, 5);
        //print_board(&input, -3, 10, -2, 9);
    }

    let mut min_x = i64::MAX;
    let mut min_y = i64::MAX;
    let mut max_x = i64::MIN;
    let mut max_y = i64::MIN;
    for &(x, y) in &input {
        min_x = x.min(min_x);
        min_y = y.min(min_y);

        max_x = x.max(max_x);
        max_y = y.max(max_y);
    }
    let x_width = (max_x - min_x) + 1;
    let y_width = (max_y - min_y) + 1;

    print_res!("Floor tiles: {}", (x_width * y_width) - input.len() as i64);
}

pub fn part2(mut input: Parsed) {
    let mut directions = [
        Direction::North,
        Direction::South,
        Direction::West,
        Direction::East,
    ];
    let mut moved = true;
    let mut count = 0;
    while moved {
        (input, moved) = round(&input, &mut directions);
        count += 1;
    }
    print_res!("Number of rounds: {count}");
}

#[cfg(test)]
mod test {
    use super::{parsing, Direction};
    use indoc::indoc;

    #[test]
    fn example_small() {
        let mut state = parsing(
            &indoc! {b"
                .....
                ..##.
                ..#..
                .....
                ..##.
                ....."}
            .to_vec()
            .into(),
        )
        .unwrap();

        let mut directions = [
            Direction::North,
            Direction::South,
            Direction::West,
            Direction::East,
        ];

        let step = parsing(
            &indoc! {b"
                ..##.
                .....
                ..#..
                ...#.
                ..#..
                ....."}
            .to_vec()
            .into(),
        )
        .unwrap();
        (state, _) = super::round(&state, &mut directions);
        assert_eq!(state, step);

        let step = parsing(
            &indoc! {b"
                .....
                ..##.
                .#...
                ....#
                .....
                ..#.."}
            .to_vec()
            .into(),
        )
        .unwrap();
        (state, _) = super::round(&state, &mut directions);
        assert_eq!(state, step);

        let step = parsing(
            &indoc! {b"
                ..#..
                ....#
                #....
                ....#
                .....
                ..#.."}
            .to_vec()
            .into(),
        )
        .unwrap();
        (state, _) = super::round(&state, &mut directions);
        assert_eq!(state, step);
    }
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
