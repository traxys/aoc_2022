use std::time::Instant;

use crate::{load, print_res};
use bstr::{BString, ByteSlice};
use ndarray::Array2;

type Parsed = Array2<u8>;

pub fn parsing(input: &BString) -> color_eyre::Result<Parsed> {
    let mut line_count = 0;
    let mut line_len = 0;
    let iter = input
        .lines()
        .flat_map(|line| {
            line_len = line.len();
            line_count += 1;
            line.iter().map(|i| i - b'0')
        })
        .collect();
    Array2::from_shape_vec((line_count, line_len), iter).map_err(Into::into)
}

pub fn part1(input: Parsed) {
    let mut visible: Array2<u8> =
        Array2::from_shape_fn((input.nrows(), input.ncols()), |(i, j)| {
            (i == 0 || j == 0 || (i == input.nrows() - 1) || (j == input.ncols() - 1)) as _
        });

    for (row, mut visible) in input.rows().into_iter().zip(visible.rows_mut()) {
        let mut max_height = row[0];
        for (&tree, vis) in row.iter().zip(visible.iter_mut()) {
            if tree > max_height {
                *vis += 1;
                max_height = tree;
            }
        }

        let mut max_height = *row.last().unwrap();
        for (&tree, vis) in row.iter().zip(visible.iter_mut()).rev() {
            if tree > max_height {
                *vis += 1;
                max_height = tree;
            }
        }
    }

    for (col, mut visible) in input.columns().into_iter().zip(visible.columns_mut()) {
        let mut max_height = col[0];
        for (&tree, vis) in col.iter().zip(visible.iter_mut()) {
            if tree > max_height {
                *vis += 1;
                max_height = tree;
            }
        }

        let mut max_height = *col.last().unwrap();
        for (&tree, vis) in col.iter().zip(visible.iter_mut()).rev() {
            if tree > max_height {
                *vis += 1;
                max_height = tree;
            }
        }
    }

    let visible_count = visible.iter().filter(|&&x| x > 0).count();

    print_res!("Visible tree count: {visible_count}");
}

pub fn part2(input: Parsed) {
    let mut scenic_score: Array2<u64> = Array2::ones((input.ncols(), input.nrows()));

    fn scenic_score_dir(
        current_tree: u8,
        trees: impl Iterator<Item = u8>,
        slice_len: usize,
    ) -> u64 {
        let smaller_count = trees
            .take_while(|&other_tree| current_tree > other_tree)
            .count();

        if smaller_count == slice_len {
            smaller_count as _
        } else {
            (smaller_count + 1) as _
        }
    }

    for i in 0..input.nrows() {
        for j in 0..input.ncols() {
            let current_tree = input[(i, j)];

            {
                let left = input.slice(ndarray::s![i, 0..j]);
                scenic_score[(i, j)] *=
                    scenic_score_dir(current_tree, left.iter().rev().copied(), left.len());
            }
            {
                let right = input.slice(ndarray::s![i, (j + 1)..]);
                scenic_score[(i, j)] *=
                    scenic_score_dir(current_tree, right.iter().copied(), right.len());
            }
            {
                let top = input.slice(ndarray::s![0..i, j]);
                scenic_score[(i, j)] *=
                    scenic_score_dir(current_tree, top.iter().rev().copied(), top.len());
            }
            {
                let bot = input.slice(ndarray::s![(i + 1).., j]);
                scenic_score[(i, j)] *=
                    scenic_score_dir(current_tree, bot.iter().copied(), bot.len());
            }
        }
    }

    let max_score = scenic_score.iter().max().unwrap();
    print_res!("Max score is: {max_score}");
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
