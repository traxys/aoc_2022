use std::{
    collections::{hash_map::Entry, HashMap},
    time::Instant,
};

use crate::{load, print_res};
use bstr::{BString, ByteSlice};

#[derive(Clone, Copy, Debug)]
pub enum Push {
    Left,
    Right,
}

impl Push {
    fn from_byte(b: u8) -> color_eyre::Result<Self> {
        match b {
            b'<' => Ok(Self::Left),
            b'>' => Ok(Self::Right),
            _ => color_eyre::eyre::bail!("Invalid character '{}'", char::from(b)),
        }
    }
}

type Parsed = Vec<Push>;

pub fn parsing(input: &BString) -> color_eyre::Result<Parsed> {
    input.trim().iter().map(|&b| Push::from_byte(b)).collect()
}

const HEIGHT_OFFSET: usize = 3;

#[allow(clippy::unusual_byte_groupings, clippy::identity_op)]
#[rustfmt::skip]
const PIECE_0: u32 = u32::from_be_bytes([
    0b0_0000000,
    0b0_0000000,
    0b0_0000000,
    0b0_0011110,
]);
#[allow(clippy::unusual_byte_groupings)]
#[rustfmt::skip]
const PIECE_1: u32 = u32::from_be_bytes([
    0b0_0000000,
    0b0_0001000,
    0b0_0011100,
    0b0_0001000,
]);
#[allow(clippy::unusual_byte_groupings)]
#[rustfmt::skip]
const PIECE_2: u32 = u32::from_be_bytes([
    0b0_0000000,
    0b0_0000100,
    0b0_0000100,
    0b0_0011100,
]);
#[allow(clippy::unusual_byte_groupings)]
#[rustfmt::skip]
const PIECE_3: u32 = u32::from_be_bytes([
    0b0_0010000,
    0b0_0010000,
    0b0_0010000,
    0b0_0010000,
]);
#[allow(clippy::unusual_byte_groupings)]
#[rustfmt::skip]
const PIECE_4: u32 = u32::from_be_bytes([
    0b0_0000000,
    0b0_0000000,
    0b0_0011000,
    0b0_0011000,
]);
const PIECES: &[u32] = &[PIECE_0, PIECE_1, PIECE_2, PIECE_3, PIECE_4];
const HEIGHTS: &[usize] = &[1, 3, 3, 4, 2];

#[allow(dead_code)]
fn print_board(board: &[u8], current_piece: Option<(usize, u32)>) {
    let current_piece = current_piece.map(|(h, p)| (h, p.to_le_bytes()));
    for (height, line) in board.iter().enumerate().rev().take(board.len() - 1) {
        print!("|");
        for x in (0..7).rev() {
            if 1 << x & line == 0 {
                match current_piece {
                    Some((h, mask)) => {
                        if (height >= h && height < h + 4) && (1 << x & mask[height - h] != 0) {
                            print!("@");
                        } else {
                            print!(".");
                        }
                    }
                    _ => print!("."),
                }
            } else {
                print!("#")
            }
        }
        println!("| {height}")
    }
    println!("+-------+")
}

fn run_fall(moves: Parsed, amount: usize) -> usize {
    let mut pieces = PIECES.iter().zip(HEIGHTS).enumerate().cycle().take(amount);

    let mut board = vec![255];
    let mut highest_point = 1;

    let mut moves = moves.iter().copied().enumerate().cycle().peekable();

    let mut seen = HashMap::new();

    let mut cycled_highest_point = None;

    // print_board(&board, None);

    let mut rock_count = 0;

    while rock_count < amount {
        let (piece_idx, (&piece, &piece_height)) = pieces.next().unwrap();
        let free_height = board.len() - highest_point;
        let height_needed = HEIGHT_OFFSET + 4;
        if free_height < height_needed {
            board.extend(std::iter::repeat(0).take(height_needed - free_height));
        }

        let mut height = highest_point + HEIGHT_OFFSET;
        let mut mask = piece;

        fn mask_collision(mask: u32, board: &[u8], height: usize) -> bool {
            let [a, b, c, d] = board[height..height + 4] else { unreachable!() };
            let tower_mask = u32::from_be_bytes([d, c, b, a]);

            mask & tower_mask != 0
        }

        let &(move_idx, _) = moves.peek().unwrap();
        if highest_point > 8 {
            let skyline =
                u64::from_ne_bytes(board[highest_point - 8..highest_point].try_into().unwrap());

            let state = (skyline, move_idx, piece_idx);

            match seen.entry(state) {
                Entry::Occupied(e) if cycled_highest_point.is_none() => {
                    let &(old_count, old_highest_point) = e.get();
                    let cycle_len = rock_count - old_count;
                    let remaining_rocks = amount - rock_count;
                    let cycles_required = remaining_rocks / cycle_len;
                    let cycle_height = highest_point - old_highest_point;
                    cycled_highest_point = Some(cycles_required * cycle_height);

                    rock_count += cycles_required * cycle_len;
                }
                Entry::Occupied(_) => (),
                Entry::Vacant(e) => {
                    e.insert((rock_count, highest_point));
                }
            }
        }

        // print_board(&board, Some((height, mask)));

        for (_, push) in moves.by_ref() {
            match push {
                Push::Left => {
                    if mask & u32::from_be_bytes([1 << 6; 4]) == 0 {
                        mask <<= 1;
                    }

                    if mask_collision(mask, &board, height) {
                        mask >>= 1;
                    }
                }
                Push::Right => {
                    if mask & u32::from_be_bytes([1; 4]) == 0 {
                        mask >>= 1;
                    }

                    if mask_collision(mask, &board, height) {
                        mask <<= 1;
                    }
                }
            }
            // print_board(&board, Some((height, mask)));

            if mask_collision(mask, &board, height - 1) {
                let current_piece_height = height + piece_height;
                if current_piece_height > highest_point {
                    highest_point = current_piece_height;
                }
                mask.to_le_bytes()
                    .iter()
                    .zip(&mut board[height..height + 4])
                    .for_each(|(m, b)| *b |= m);
                // print_board(&board, Some((height, mask)));
                rock_count += 1;
                break;
            }

            height -= 1;
        }
    }

    match cycled_highest_point {
        None => highest_point - 1,
        Some(cycle) => cycle + highest_point - 1,
    }
}

pub fn part1(input: Parsed) {
    print_res!("Highest point after 2022 turns: {}", run_fall(input, 2022));
}

pub fn part2(input: Parsed) {
    print_res!(
        "Highest point after 1000000000000 turns: {}",
        run_fall(input, 1000000000000)
    );
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
