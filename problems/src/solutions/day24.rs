use std::{
    collections::{HashMap, HashSet, VecDeque},
    time::Instant,
};

use crate::{load, print_res};
use bstr::{BString, ByteSlice};
use itertools::Itertools;
use tinyvec::ArrayVec;

#[derive(Clone, Copy, Debug)]
pub enum Blizzard {
    Increasing(u16),
    Decreasing(u16),
}

impl Blizzard {
    fn position(&self, len: u16, t: u64) -> u16 {
        match *self {
            Blizzard::Increasing(i) => ((i as u64 + t) % (len as u64)) as u16,
            Blizzard::Decreasing(i) => len - 1 - ((i as u64 + t) % (len as u64)) as u16,
        }
    }

    fn repr(&self, vertical: bool) -> char {
        match self {
            Blizzard::Increasing(_) if vertical => 'v',
            Blizzard::Increasing(_) => '>',
            Blizzard::Decreasing(_) if vertical => '^',
            Blizzard::Decreasing(_) => '<',
        }
    }
}

type Parsed = (Box<[Box<[Blizzard]>]>, Box<[Box<[Blizzard]>]>);

// y - |y-(t % 2y)|

pub fn parsing(input: &BString) -> color_eyre::Result<Parsed> {
    let row_len = input.find("\n").unwrap() - 2;
    let row_count = input.lines().count() - 2;
    let mut rows: Vec<_> = std::iter::repeat_with(Vec::<Blizzard>::new)
        .take(row_count)
        .collect();
    let mut cols: Vec<_> = std::iter::repeat_with(Vec::<Blizzard>::new)
        .take(row_len)
        .collect();

    for (row_idx, (row, blizz_row)) in input
        .lines()
        .skip(1)
        .take(row_count)
        .zip(rows.iter_mut())
        .enumerate()
    {
        for (col_idx, (&item, blizz_col)) in row
            .iter()
            .skip(1)
            .take(row_len)
            .zip(cols.iter_mut())
            .enumerate()
        {
            match item {
                b'<' => {
                    let t0 = Blizzard::Decreasing((row_len - col_idx - 1) as _);
                    blizz_row.push(t0);
                }
                b'>' => {
                    let t0 = Blizzard::Increasing(col_idx as _);
                    blizz_row.push(t0);
                }
                b'^' => {
                    let t0 = Blizzard::Decreasing((row_count - row_idx - 1) as _);
                    blizz_col.push(t0);
                }
                b'v' => {
                    let t0 = Blizzard::Increasing(row_idx as _);
                    blizz_col.push(t0);
                }
                b'.' => (),
                _ => color_eyre::eyre::bail!("Invalid character: {}", char::from(item)),
            }
        }
    }
    Ok((
        rows.into_iter().map(Vec::into_boxed_slice).collect(),
        cols.into_iter().map(Vec::into_boxed_slice).collect(),
    ))
}

type BlizzardContainer<'a> = &'a [Box<[Blizzard]>];

#[allow(dead_code)]
fn print_board(rows: BlizzardContainer, cols: BlizzardContainer, t: u64) {
    let mut positions = HashMap::new();
    for (b, pos) in rows_positions(rows, cols.len() as _, t) {
        positions
            .entry(pos)
            .or_insert_with(Vec::new)
            .push(b.repr(false));
    }

    for (b, pos) in cols_positions(cols, rows.len() as _, t) {
        positions
            .entry(pos)
            .or_insert_with(Vec::new)
            .push(b.repr(true));
    }

    for y in 0..rows.len() {
        print!("#");
        for x in 0..cols.len() {
            match positions.get(&(x as _, y as _)).map(|v| v.as_slice()) {
                None => print!("."),
                Some([x]) => print!("{x}"),
                Some(v) if v.len() < 10 => print!("{}", v.len()),
                Some(_) => print!("*"),
            }
        }
        println!("#");
    }
}

fn row_positions(
    row: &[Blizzard],
    row_num: i16,
    row_len: u16,
    t: u64,
) -> impl Iterator<Item = (Blizzard, (i16, i16))> + '_ {
    row.iter()
        .map(move |&b| (b, (b.position(row_len, t) as i16, row_num)))
}

fn rows_positions(
    rows: BlizzardContainer,
    row_len: u16,
    t: u64,
) -> impl Iterator<Item = (Blizzard, (i16, i16))> + '_ {
    rows.iter()
        .enumerate()
        .flat_map(move |(r, row)| row_positions(row, r as _, row_len, t))
}

fn col_positions(
    col: &[Blizzard],
    col_num: i16,
    col_len: u16,
    t: u64,
) -> impl Iterator<Item = (Blizzard, (i16, i16))> + '_ {
    col.iter()
        .map(move |&b| (b, (col_num, b.position(col_len, t) as _)))
}

fn cols_positions(
    cols: BlizzardContainer,
    col_len: u16,
    t: u64,
) -> impl Iterator<Item = (Blizzard, (i16, i16))> + '_ {
    cols.iter()
        .enumerate()
        .flat_map(move |(c, col)| col_positions(col, c as _, col_len, t))
}

fn possible_positions(
    x: i16,
    y: i16,
    t: u64,
    rows: BlizzardContainer,
    cols: BlizzardContainer,
) -> ArrayVec<[(i16, i16); 5]> {
    [(-1, 0), (1, 0), (0, 0), (0, -1), (0, 1)]
        .iter()
        .map(|(ox, oy)| (x + ox, y + oy))
        .filter(|&(nx, ny)| {
            (nx == 0 && ny == -1)
                || (nx == (cols.len() - 1) as i16 && ny == rows.len() as i16)
                || (nx >= 0 && nx < cols.len() as _ && ny >= 0 && ny < rows.len() as _)
                    && (!col_positions(&cols[nx as usize], nx, rows.len() as _, t)
                        .map(|(_, pos)| pos)
                        .contains(&(nx, ny)))
                    && (!row_positions(&rows[ny as usize], ny, cols.len() as _, t)
                        .map(|(_, pos)| pos)
                        .contains(&(nx, ny)))
        })
        .collect()
}

fn time_from(
    x: i16,
    y: i16,
    t0: u64,
    dx: i16,
    dy: i16,
    rows: BlizzardContainer,
    cols: BlizzardContainer,
) -> u64 {
    let total_mod = (rows.len() * cols.len()) as u64;

    let mut paths = VecDeque::new();
    paths.push_front((x, y, t0));

    let mut visited = HashSet::new();

    while let Some((x, y, t)) = paths.pop_front() {
        if !visited.insert((x, y, t % total_mod)) {
            continue;
        }

        if x == dx && y == dy {
            return t;
        }

        for (nx, ny) in possible_positions(x, y, t + 1, rows, cols) {
            paths.push_back((nx, ny, t + 1));
        }
    }

    panic!("No path found")
}

pub fn part1((rows, cols): Parsed) {
    /* print_board(&rows, &cols, 0);
    println!("----");
    print_board(&rows, &cols, 1); */

    print_res!(
        "Number of minutes to traverse: {}",
        time_from(
            0,
            -1,
            0,
            (cols.len() - 1) as _,
            rows.len() as _,
            &rows,
            &cols
        )
    );
}

pub fn part2((rows, cols): Parsed) {
    let sx = 0;
    let sy = -1;
    let ex = (cols.len() - 1) as i16;
    let ey = rows.len() as i16;
    let go = time_from(sx, sy, 0, ex, ey, &rows, &cols);
    let back = time_from(ex, ey, go, sx, sy, &rows, &cols);
    let end = time_from(sx, sy, back, ex, ey, &rows, &cols);
    print_res!("Number of minutes to round trip: {end}");
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
