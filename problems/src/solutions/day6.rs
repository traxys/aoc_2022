use std::time::Instant;

use crate::{load, print_res};
use bstr::{BString, ByteSlice};

type Parsed = Vec<u8>;

pub fn parsing(input: &BString) -> color_eyre::Result<Parsed> {
    Ok(input.trim().iter().map(|a| a - b'a').collect())
}

fn all_unique(slice: &[u8]) -> bool {
    let mut occ = [0; 26];

    slice.iter().for_each(|&i| occ[i as usize] += 1);

    occ.iter().all(|&i| i < 2)
}

fn marker_idx(signal: &[u8], window_size: usize) -> usize {
    signal
        .windows(window_size)
        .enumerate()
        .find(|(_, w)| all_unique(w))
        .expect("No marker")
        .0
        + window_size
}

pub fn part1(input: Parsed) {
    print_res!("start of packet marker idx: {}", marker_idx(&input, 4))
}

pub fn part2(input: Parsed) {
    print_res!("start of message marker idx: {}", marker_idx(&input, 14))
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

#[cfg(test)]
mod test {
    use super::{marker_idx, parsing};

    #[test]
    fn example1() {
        let input = parsing(&b"bvwbjplbgvbhsrlpgdmjqwftvncz".to_vec().into()).unwrap();

        assert_eq!(marker_idx(&input, 4), 5);
        assert_eq!(marker_idx(&input, 14), 23);
    }

    #[test]
    fn example2() {
        let input = parsing(&b"nppdvjthqldpwncqszvftbrmjlhg".to_vec().into()).unwrap();

        assert_eq!(marker_idx(&input, 4), 6);
        assert_eq!(marker_idx(&input, 14), 23);
    }

    #[test]
    fn example3() {
        let input = parsing(&b"nznrnfrfntjfmvfwmzdfjlvtqnbhcprsg".to_vec().into()).unwrap();

        assert_eq!(marker_idx(&input, 4), 10);
        assert_eq!(marker_idx(&input, 14), 29);
    }

    #[test]
    fn example4() {
        let input = parsing(&b"zcfzfwzzqfrljwzlrfnpqdbhtmscgvjw".to_vec().into()).unwrap();

        assert_eq!(marker_idx(&input, 4), 11);
        assert_eq!(marker_idx(&input, 14), 26);
    }
}
