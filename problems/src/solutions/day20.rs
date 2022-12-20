use std::time::Instant;

use crate::{load, print_res};
use bstr::{BString, ByteSlice};

type Parsed = Vec<(i64, u16)>;

pub fn parsing(input: &BString) -> color_eyre::Result<Parsed> {
    input
        .lines()
        .enumerate()
        .map(|(i, n)| -> color_eyre::Result<_> { Ok((n.to_str()?.parse()?, i as u16)) })
        .collect()
}

fn reoder_round(input: &mut Parsed, mut i: usize, current_num: u16) -> Option<usize> {
    if input[i].1 != current_num {
        return None;
    }

    let (current, idx) = input.remove(i);

    let mut next_pos = (i as i64 + current) % input.len() as i64;
    if next_pos <= 0 {
        next_pos += input.len() as i64;
    }
    let next_pos = next_pos as usize;

    if next_pos > i {
        // TODO ?
    } else {
        i += 1;
    }

    input.insert(next_pos, (current, idx));
    Some(i)
}

fn reorder(mut input: Parsed) -> Parsed {
    let mut moved = 0;
    let mut i = 0;
    while moved < input.len() as u16 {
        let next_i = match reoder_round(&mut input, i, moved) {
            None => i + 1,
            Some(i) => {
                moved += 1;
                i
            }
        };
        i = next_i % input.len();
    }

    input
}

pub fn part1(input: Parsed) {
    let decoded = reorder(input);
    let (zero_pos, _) = decoded.iter().enumerate().find(|(_, &x)| x.0 == 0).unwrap();
    let coordinates: i64 = [1000, 2000, 3000]
        .iter()
        .map(|p| (zero_pos + p) % decoded.len())
        .map(|i| decoded[i].0)
        .sum();
    print_res!("Coordinates sum: {coordinates}");
}

const DECODING_KEY: i64 = 811589153;

pub fn part2(mut input: Parsed) {
    input.iter_mut().for_each(|(i, _)| *i *= DECODING_KEY);
    for _ in 0..10 {
        input = reorder(input);
        // No idea where this rotate right comes from
        input.rotate_right(1);
    }

    let (zero_pos, _) = input.iter().enumerate().find(|(_, &x)| x.0 == 0).unwrap();
    let coordinates: i64 = [1000, 2000, 3000]
        .iter()
        .map(|p| (zero_pos + p) % input.len())
        .map(|i| input[i].0)
        .sum();
    print_res!("Coordinates sum: {coordinates}");
}

#[cfg(test)]
mod test {
    use itertools::Itertools;

    #[test]
    fn part1() {
        let steps = [
            [1, 2, -3, 3, -2, 0, 4],
            [2, 1, -3, 3, -2, 0, 4],
            [1, -3, 2, 3, -2, 0, 4],
            [1, 2, 3, -2, -3, 0, 4],
            [1, 2, -2, -3, 0, 3, 4],
            [1, 2, -3, 0, 3, 4, -2],
            [1, 2, -3, 0, 3, 4, -2],
            [1, 2, -3, 4, 0, 3, -2],
        ];

        let assert_step = |s: usize, input: &[(i64, _)]| {
            assert_eq!(
                steps[s]
                    .iter()
                    .zip(input)
                    .enumerate()
                    .find(|(_, (&a, b))| a != b.0),
                None
            )
        };

        let mut input: Vec<_> = steps[0]
            .into_iter()
            .enumerate()
            .map(|(i, n)| (n, i as u16))
            .collect();

        let mut moved = 0;
        let mut i = 0;
        while moved < input.len() as u16 {
            let next_i = match super::reoder_round(&mut input, i, moved) {
                None => i + 1,
                Some(i) => {
                    moved += 1;
                    assert_step(moved as usize, &input);
                    i
                }
            };
            i = next_i;
        }
    }

    #[test]
    fn part2() {
        #[rustfmt::skip]
        let steps = [
            [ 811589153i64,  1623178306, -2434767459,  2434767459, -1623178306,           0,  3246356612 ],
            [ 0,            -2434767459,  3246356612, -1623178306,  2434767459,  1623178306,   811589153 ],
            [ 0,             2434767459,  1623178306,  3246356612, -2434767459, -1623178306,   811589153 ],
            [ 0,              811589153,  2434767459,  3246356612,  1623178306, -1623178306, -2434767459 ],
            [ 0,             1623178306, -2434767459,   811589153,  2434767459,  3246356612, -1623178306 ],
            [ 0,              811589153, -1623178306,  1623178306, -2434767459,  3246356612,  2434767459 ],
            [ 0,              811589153, -1623178306,  3246356612, -2434767459,  1623178306,  2434767459 ],
            [ 0,            -2434767459,  2434767459,  1623178306, -1623178306,   811589153,  3246356612 ],
            [ 0,             1623178306,  3246356612,   811589153, -2434767459,  2434767459, -1623178306 ],
            [ 0,              811589153,  1623178306, -2434767459,  3246356612,  2434767459, -1623178306 ],
            [ 0,            -2434767459,  1623178306,  3246356612, -1623178306,  2434767459,   811589153 ],
        ];

        let assert_step = |s: usize, input: &[(i64, _)]| {
            if steps[s].iter().zip(input).any(|(&a, b)| a != b.0) {
                eprintln!("Step {s} mismatch");
                eprintln!("expected: {:?}", steps[s]);
                eprintln!("got:      [{}]", input.iter().map(|(x, _)| x).join(", "));
                panic!()
            }
        };

        let mut input: Vec<_> = steps[0]
            .into_iter()
            .enumerate()
            .map(|(i, n)| (n, i as u16))
            .collect();

        for x in 0..10 {
            assert_step(x, &input);
            input = super::reorder(input);
            input.rotate_right(1);
        }

        assert_step(10, &input);
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
