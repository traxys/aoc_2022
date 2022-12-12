use std::{collections::HashMap, time::Instant};

use crate::{load, print_res};
use bstr::{BString, ByteSlice};
use petgraph::{graph::NodeIndex, Graph};

#[derive(Clone, Debug)]
pub struct RiverMap {
    graph: Graph<(), ()>,
    lowest_points: Vec<NodeIndex>,
    start: NodeIndex,
    end: NodeIndex,
}

type Parsed = RiverMap;

pub fn parsing(input: &BString) -> color_eyre::Result<Parsed> {
    let Some(line_len) = input.find("\n") else {
        color_eyre::eyre::bail!("Input has no lines")
    };
    let line_count = input.iter().filter(|&&x| x == b'\n').count();

    let mut nodes = HashMap::new();
    let mut graph = Graph::new();

    for y in 0..line_count {
        for x in 0..line_len {
            nodes.insert((x, y), graph.add_node(()));
        }
    }

    let mut start = None;
    let mut end = None;

    let point_elevation = |x, y| input[x + y * line_len + y];
    let point_elevation_fixed = |x, y| match point_elevation(x, y) {
        b'S' => b'a',
        b'E' => b'z',
        normal => normal,
    };

    let mut lowest_points = Vec::new();

    for y in 0..line_count {
        for x in 0..line_len {
            let elevation = point_elevation(x, y);
            let &self_idx = nodes.get(&(x, y)).unwrap();
            let elevation = match elevation {
                b'S' => {
                    start = Some(self_idx);
                    b'a'
                }
                b'E' => {
                    end = Some(self_idx);
                    b'z'
                }
                _ => elevation,
            } as i16;

            if elevation == b'a'.into() {
                lowest_points.push(self_idx);
            };

            if y != 0 {
                let neigh_elev = point_elevation_fixed(x, y - 1) as i16;
                if neigh_elev - elevation <= 1 {
                    graph.add_edge(self_idx, nodes.get(&(x, y - 1)).copied().unwrap(), ());
                }
            }
            if y != line_count - 1 {
                let neigh_elev = point_elevation_fixed(x, y + 1) as i16;
                if neigh_elev - elevation <= 1 {
                    graph.add_edge(self_idx, nodes.get(&(x, y + 1)).copied().unwrap(), ());
                }
            }
            if x != 0 {
                let neigh_elev = point_elevation_fixed(x - 1, y) as i16;
                if neigh_elev - elevation <= 1 {
                    graph.add_edge(self_idx, nodes.get(&(x - 1, y)).copied().unwrap(), ());
                }
            }
            if x != line_len - 1 {
                let neigh_elev = point_elevation_fixed(x + 1, y) as i16;
                if neigh_elev - elevation <= 1 {
                    graph.add_edge(self_idx, nodes.get(&(x + 1, y)).copied().unwrap(), ());
                }
            }
        }
    }

    Ok(RiverMap {
        graph,
        lowest_points,
        start: start.ok_or_else(|| color_eyre::eyre::eyre!("No start position"))?,
        end: end.ok_or_else(|| color_eyre::eyre::eyre!("No end position"))?,
    })
}

pub fn part1(input: Parsed) {
    let paths_len = petgraph::algo::dijkstra(&input.graph, input.start, Some(input.end), |_| 1);

    let end_len = paths_len.get(&input.end).unwrap();

    print_res!("Shortest path is of length: {end_len}");
}

pub fn part2(mut input: Parsed) {
    input.graph.reverse();
    let shortest_paths = petgraph::algo::dijkstra(&input.graph, input.end, None, |_| 1);
    let shortest_from_any = shortest_paths
        .iter()
        .filter(|(start, _)| input.lowest_points.contains(start))
        .min_by_key(|(_, &len)| len)
        .unwrap()
        .1;
    print_res!("Shortest path from any is: {shortest_from_any}");
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
