use std::{ops::Index, time::Instant};

use crate::{load, print_res};
use bstr::{BString, ByteSlice};

type Parsed = (Vec2D<Tile>, Instructions);

#[derive(Clone, Debug)]
pub struct Instructions {
    body: Vec<(u16, Turn)>,
    last: u16,
}

#[derive(Clone, Copy, Debug)]
pub enum Turn {
    Clockwise,
    CounterClockwise,
}

#[allow(dead_code)]
fn print_board(board: &Vec2D<Tile>) {
    for row in board.rows() {
        for &item in row.iter() {
            match item {
                Tile::Void => print!("         "),
                Tile::Space => print!(",.......,"),
                Tile::Wall => print!("#########"),
                Tile::Wraparound(Wraparound {
                    turn: _,
                    vertical,
                    horizontal,
                }) => {
                    let opt_repr = |o: Option<i16>| match o {
                        None => "---".into(),
                        Some(v) => v.to_string(),
                    };
                    print!(
                        "|{:>2},{:>2}|{:>2},{:>2}|",
                        opt_repr(vertical.map(|v| v.0)),
                        opt_repr(vertical.map(|v| v.1)),
                        opt_repr(horizontal.map(|v| v.0)),
                        opt_repr(horizontal.map(|v| v.1)),
                    )
                }
            }
        }
        println!()
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Wraparound {
    turn: [Option<Turn>; 2],
    vertical: Option<(i16, i16)>,
    horizontal: Option<(i16, i16)>,
}

#[derive(Clone, Copy, Debug)]
pub enum Tile {
    Void,
    Space,
    Wall,
    Wraparound(Wraparound),
}

#[derive(Clone, Debug)]
pub struct Vec2D<T> {
    array: Vec<T>,
    nrows: usize,
    ncols: usize,
}

impl<T> Index<(usize, usize)> for Vec2D<T> {
    type Output = T;

    fn index(&self, (col, row): (usize, usize)) -> &Self::Output {
        &self.array[self.ncols * row + col]
    }
}

impl<T> Vec2D<T> {
    pub fn rows(&self) -> impl Iterator<Item = &[T]> {
        self.array.chunks_exact(self.ncols)
    }

    pub fn rows_mut(&mut self) -> impl Iterator<Item = &mut [T]> {
        self.array.chunks_exact_mut(self.ncols)
    }

    pub fn row_mut(&mut self, row: usize) -> &mut [T] {
        &mut self.array[self.ncols * row..self.ncols * (row + 1)]
    }

    pub fn row(&self, row: usize) -> &[T] {
        &self.array[self.ncols * row..self.ncols * (row + 1)]
    }
}
impl<T: Clone> Vec2D<T> {
    fn from_elem(nrows: usize, ncols: usize, elem: T) -> Self {
        let array = vec![elem; nrows * ncols];
        Vec2D {
            array,
            nrows,
            ncols,
        }
    }
}

pub fn parsing(input: &BString) -> color_eyre::Result<Parsed> {
    let Some((board, directions)) = input.split_once_str("\n\n") else {
        color_eyre::eyre::bail!("Did not find empty line")
    };

    let (line_len, line_count) = board.lines().fold((0, 0), |(len, count), line| {
        (len.max(line.len()), count + 1)
    });

    let mut board_array = Vec2D::from_elem(line_count + 2, line_len + 2, Tile::Void);
    let mut cols: Vec<Option<usize>> = vec![None; line_len];

    fn apply_horizontal_wraparound(location: i16, vertical: i16, tile: &mut Tile) {
        match tile {
            Tile::Space | Tile::Wall => unreachable!("Can't override tile with wraparound"),
            t @ Tile::Void => {
                *t = Tile::Wraparound(Wraparound {
                    turn: [None, None],
                    vertical: None,
                    horizontal: Some((location, vertical)),
                })
            }
            Tile::Wraparound(Wraparound { horizontal, .. }) => {
                assert!(
                    horizontal.is_none(),
                    "Tried to overwrite horizontal wraparound"
                );
                *horizontal = Some((location, vertical))
            }
        }
    }

    fn apply_vertical_wraparound(location: i16, horizontal: i16, tile: &mut Tile) {
        match tile {
            Tile::Space | Tile::Wall => unreachable!("Can't override tile with wraparound"),
            t @ Tile::Void => {
                *t = Tile::Wraparound(Wraparound {
                    turn: [None, None],
                    vertical: Some((horizontal, location)),
                    horizontal: None,
                })
            }
            Tile::Wraparound(Wraparound { vertical, .. }) => {
                assert!(
                    vertical.is_none(),
                    "Tried to overwrite horizontal wraparound"
                );
                *vertical = Some((horizontal, location))
            }
        }
    }

    for (y, row) in board.lines().enumerate() {
        let mut row_start = None;
        for (x, r) in row
            .iter()
            .copied()
            .chain(std::iter::repeat(b' '))
            .take(line_len)
            .enumerate()
        {
            match r {
                b' ' => {
                    if let Some(start) = cols[x] {
                        apply_vertical_wraparound(
                            y as _,
                            (x + 1) as _,
                            &mut board_array.row_mut(start - 1)[x + 1],
                        );
                        apply_vertical_wraparound(
                            start as _,
                            (x + 1) as _,
                            &mut board_array.row_mut(y + 1)[x + 1],
                        );
                        cols[x] = None;
                    }

                    if let Some(start) = row_start {
                        row_start = None;
                        apply_horizontal_wraparound(
                            start as _,
                            (y + 1) as _,
                            &mut board_array.row_mut(y + 1)[x + 1],
                        );
                        apply_horizontal_wraparound(
                            x as _,
                            (y + 1) as _,
                            &mut board_array.row_mut(y + 1)[start - 1],
                        );
                    }
                }
                b'#' | b'.' => {
                    if row_start.is_none() {
                        row_start = Some(x + 1);
                    }

                    if cols[x].is_none() {
                        cols[x] = Some(y + 1);
                    }

                    if r == b'#' {
                        board_array.row_mut(y + 1)[x + 1] = Tile::Wall
                    } else {
                        board_array.row_mut(y + 1)[x + 1] = Tile::Space;
                    }
                }
                _ => color_eyre::eyre::bail!("Invalid character in row"),
            }
        }
        if let Some(start) = row_start {
            let board_row = board_array.row_mut(y + 1);
            apply_horizontal_wraparound(
                start as _,
                (y + 1) as _,
                &mut board_row[board_row.len() - 1],
            );
            apply_horizontal_wraparound(
                (board_row.len() - 2) as _,
                (y + 1) as _,
                &mut board_row[start - 1],
            );
        }
    }
    for (x, col) in cols.iter().enumerate() {
        if let &Some(start) = col {
            let nrows = board_array.nrows;
            apply_vertical_wraparound(
                nrows as i16 - 2,
                (x + 1) as _,
                &mut board_array.row_mut(start - 1)[x + 1],
            );
            apply_vertical_wraparound(
                start as _,
                (x + 1) as _,
                &mut board_array.row_mut(board_array.nrows - 1)[x + 1],
            );
        }
    }

    let directions = directions.to_str()?;
    let mut previous_index = 0;
    let body = directions
        .match_indices(['R', 'L'])
        .map(|(idx, letter)| -> color_eyre::eyre::Result<_> {
            let turn = match letter {
                "R" => Turn::Clockwise,
                "L" => Turn::CounterClockwise,
                _ => unreachable!(),
            };

            let num = directions[previous_index..idx].parse()?;
            previous_index = idx + 1;

            Ok((num, turn))
        })
        .collect::<Result<_, _>>()?;
    Ok((
        board_array,
        Instructions {
            body,
            last: directions[previous_index..].trim().parse()?,
        },
    ))
}

fn apply_turn((x, y): (i32, i32), turn: Turn) -> (i32, i32) {
    match turn {
        Turn::Clockwise => (-y, x),
        Turn::CounterClockwise => (y, -x),
    }
}

fn move_in_direction(
    mut direction: (i32, i32),
    mut x: i32,
    mut y: i32,
    amount: u16,
    board: &Vec2D<Tile>,
) -> (i32, i32) {
    for _ in 0..amount {
        let nx = x + direction.0;
        let ny = y + direction.1;
        match board[(nx as usize, ny as usize)] {
            Tile::Space => {
                x = nx as i32;
                y = ny as i32;
            }
            Tile::Wall => break,
            Tile::Wraparound(Wraparound {
                vertical,
                horizontal,
                turn,
            }) => {
                for t in &turn {
                    if let &Some(t) = t {
                        direction = apply_turn(direction, t);
                    }
                }
                let nx: i16;
                let ny: i16;
                if direction.0 != 0 {
                    //println!("Horizontal wrap from {nx}/{ny} ({horizontal:?})");
                    let (wx, wy) = horizontal.unwrap();
                    (nx, ny) = (wx as _, wy as _);
                } else {
                    //println!("Vertical wrap from {nx}/{ny} ({vertical:?})");
                    let (wx, wy) = vertical.unwrap();
                    (nx, ny) = (wx as _, wy as _);
                }
                match board[(nx as usize, ny as usize)] {
                    Tile::Space => {
                        x = nx as _;
                        y = ny as _;
                    }
                    Tile::Wall => break,
                    _ => unreachable!("Tried to move to {nx}/{ny}"),
                }
            }
            Tile::Void => unreachable!(),
        }
    }

    (x, y)
}

pub fn part1((board, movements): Parsed) {
    //print_board(&board);
    let start = board
        .row(1)
        .iter()
        .enumerate()
        .find(|(_, t)| matches!(t, Tile::Space))
        .unwrap();
    let mut x = start.0 as i32;
    let mut y = 1;
    let mut direction = (1, 0);
    for (amount, turn) in movements.body {
        (x, y) = move_in_direction(direction, x, y, amount, &board);
        direction = apply_turn(direction, turn);
    }
    (x, y) = move_in_direction(direction, x, y, movements.last, &board);

    let facing_value = match direction {
        // Right
        (1, 0) => 0,
        // Left
        (-1, 0) => 2,
        // Up
        (0, -1) => 3,
        // Down
        (0, 1) => 1,
        _ => unreachable!(),
    };

    let final_password = 1000 * y + 4 * x + facing_value;
    print_res!("Final password is: {final_password}");
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
