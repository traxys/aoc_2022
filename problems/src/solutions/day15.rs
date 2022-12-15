use std::{collections::HashSet, time::Instant};

use crate::{load, print_res};
use bstr::{BString, ByteSlice};
use itertools::Itertools;

#[derive(Debug, Clone, Copy)]
pub struct Sensor {
    pos: (i64, i64),
    beacon: (i64, i64),
}

type Parsed = Vec<Sensor>;

pub fn parsing(input: &BString) -> color_eyre::Result<Parsed> {
    input
        .lines()
        .map(|line| -> color_eyre::Result<_> {
            let Some((pos,beacon)) = line.split_once_str(":") else {
                color_eyre::eyre::bail!("No colon in line")
            };

            fn extract_coords(part: &[u8], prefix: &[u8]) -> color_eyre::Result<(i64, i64)> {
                let Some(part) = part.strip_prefix(prefix) else {
                    color_eyre::eyre::bail!("Part does not start with prefix");
                };

                let Some((x,y)) = part.split_once_str(",") else {
                    color_eyre::eyre::bail!("Part does not have a comma");
                };

                let Some(x) = x.strip_prefix(b"x=") else {
                    color_eyre::eyre::bail!("x does not start with 'x='");
                };

                let Some(y) = y.trim().strip_prefix(b"y=") else {
                    color_eyre::eyre::bail!("y does not start with 'y='");
                };

                Ok((x.trim().to_str()?.parse()?, y.trim().to_str()?.parse()?))
            }

            Ok(Sensor {
                pos: extract_coords(pos, b"Sensor at ")?,
                beacon: extract_coords(beacon, b" closest beacon is at ")?,
            })
        })
        .collect()
}

fn mh_distance(a: (i64, i64), b: (i64, i64)) -> u64 {
    a.0.abs_diff(b.0) + a.1.abs_diff(b.1)
}

fn covered_intervals(sensors: &[Sensor], y: i64) -> Vec<(i64, i64)> {
    let mut intervals = Vec::new();

    for sensor in sensors {
        let radius = mh_distance(sensor.beacon, sensor.pos);
        let row_factor = y.abs_diff(sensor.pos.1);
        if radius < row_factor {
            continue;
        }

        let cord = radius - row_factor;
        let start = sensor.pos.0 - (cord as i64);
        let end = sensor.pos.0 + (cord as i64);

        if sensor.beacon.1 == y {
            if start != end {
                if sensor.beacon.0 == start {
                    intervals.push((start + 1, end));
                } else {
                    intervals.push((start, end - 1));
                }
            }
        } else {
            intervals.push((start, end));
        }
    }

    intervals.sort_by(|a, b| a.0.cmp(&b.0));

    let mut merged_intervals = Vec::new();
    let mut current_start = intervals[0].0;
    let mut current_end = intervals[0].1;

    for &(start, end) in intervals.iter().skip(1) {
        if start <= current_end || start - 1 == current_end {
            if end > current_end {
                current_end = end;
            }
        } else {
            merged_intervals.push((current_start, current_end));
            current_start = start;
            current_end = end;
        }
    }
    merged_intervals.push((current_start, current_end));

    merged_intervals
}

pub fn part1(input: Parsed) {
    //let y = 10;
    let y = 2000000;

    let merged_intervals = covered_intervals(&input, y);

    let amount: i64 = merged_intervals
        .iter()
        .map(|(start, end)| end - start + 1)
        .sum();

    print_res!("Cleared squares in line y={y}: {amount}");
}

pub fn part2(input: Parsed) {
    let mut possible_positions: HashSet<(i64, i64)> = HashSet::new();

    for y in 0..=4000000 {
        let intervals = covered_intervals(&input, y);
        if intervals.len() > 1 {
            intervals
                .iter()
                .tuple_windows()
                .flat_map(|(&(_, end), &(start, _))| (end + 1)..(start))
                .for_each(|x| {
                    possible_positions.insert((x, y));
                });
        }
    }

    for sensor in &input {
        possible_positions.remove(&sensor.beacon);
    }
    let position = possible_positions.drain().next().unwrap();
    print_res!("Tuning frequency is: {}", position.0 * 4000000 + position.1);
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
