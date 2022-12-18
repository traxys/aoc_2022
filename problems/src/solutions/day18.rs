use std::{
    collections::{HashMap, HashSet},
    time::Instant,
};

use crate::{load, print_res};
use bstr::{BString, ByteSlice};
use itertools::Itertools;
use petgraph::{
    prelude::UnGraph,
    unionfind::UnionFind,
    visit::{EdgeRef, NodeIndexable},
};

type Parsed = HashSet<(i16, i16, i16)>;

pub fn parsing(input: &BString) -> color_eyre::Result<Parsed> {
    input
        .to_str()?
        .lines()
        .map(|l| {
            let Some((x,y,z)) = l.split(',').map(|l| l.parse()).collect_tuple() else {
                color_eyre::eyre::bail!("could not collect into tuple")
            };

            Ok((x?, y?, z?))
        })
        .collect()
}

fn offsets() -> Vec<[i16; 3]> {
    (0..3)
        .flat_map(move |i| {
            [-1, 1].into_iter().map(move |o| {
                let mut arr = [0; 3];
                arr[i] = o;
                arr
            })
        })
        .collect()
}

pub fn part1(input: Parsed) {
    let offsets = offsets();
    let mut face_count = 0;
    for (x, y, z) in &input {
        for [ox, oy, oz] in &offsets {
            if !input.contains(&(x + ox, y + oy, z + oz)) {
                face_count += 1;
            }
        }
    }
    print_res!("Number of exposed faces: {face_count}");
}

pub fn part2(input: Parsed) {
    let (mut lx, mut gx) = (i16::MAX, i16::MIN);
    let (mut ly, mut gy) = (i16::MAX, i16::MIN);
    let (mut lz, mut gz) = (i16::MAX, i16::MIN);
    for &(x, y, z) in &input {
        lx = x.min(lx);
        ly = y.min(ly);
        lz = z.min(lz);

        gx = x.max(gx);
        gy = y.max(gy);
        gz = z.max(gz);
    }

    let mut graph = UnGraph::new_undirected();
    let mut nodes = HashMap::new();
    for x in lx - 1..=gx + 1 {
        for y in ly - 1..=gy + 1 {
            for z in lz - 1..=gz + 1 {
                nodes.insert((x, y, z), graph.add_node(()));
            }
        }
    }

    let offsets = offsets();
    for (&(x, y, z), &idx) in &nodes {
        let inside = input.contains(&(x, y, z));

        for [ox, oy, oz] in &offsets {
            let neighbour = &(x + ox, y + oy, z + oz);
            let n_inside = input.contains(neighbour);
            if inside == n_inside {
                if let Some(&n_idx) = nodes.get(neighbour) {
                    graph.add_edge(idx, n_idx, ());
                }
            }
        }
    }

    let mut components = UnionFind::new(graph.node_bound());
    for edge in graph.edge_references() {
        let (a, b) = (edge.source(), edge.target());
        components.union(graph.to_index(a), graph.to_index(b));
    }

    let outside_component =
        components.find(graph.to_index(*nodes.get(&(lx - 1, ly - 1, lz - 1)).unwrap()));

    let mut outside_faces = 0;

    for &(x, y, z) in &input {
        for [ox, oy, oz] in &offsets {
            let &neigh_idx = nodes.get(&(x + ox, y + oy, z + oz)).unwrap();
            let neigh_component = components.find(graph.to_index(neigh_idx));

            if neigh_component == outside_component {
                outside_faces += 1;
            }
        }
    }

    print_res!("Number of outside faces: {outside_faces}");
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
