use std::{
    collections::{BinaryHeap, HashMap},
    time::Instant,
};

use crate::{load, print_res};
use bstr::{BStr, BString, ByteSlice};
use im::{vector, Vector};
use itertools::Itertools;
use petgraph::Graph;

#[derive(Debug, Clone)]
pub struct Valve<'a> {
    name: &'a BStr,
    flow: u64,
    neighbours: Vec<usize>,
}

type Parsed<'a> = Vec<Valve<'a>>;

pub fn parsing(input: &BString) -> color_eyre::Result<Parsed> {
    let mut valves = HashMap::with_capacity(input.lines().count());
    let mut ordered = Vec::with_capacity(valves.capacity());

    for line in input.lines() {
        let Some((valve, neighbours)) = line.split_once_str(";") else {
            color_eyre::eyre::bail!("No semicolon");
        };

        let Some((name, flow)) = valve.split_once_str(" has flow ") else {
            color_eyre::eyre::bail!("Malformed valve section")
        };

        let Some(name) = name.strip_prefix(b"Valve ") else {
            color_eyre::eyre::bail!("Malformed name")
        };
        let name = name.trim().as_bstr();

        let Some(flow) = flow.strip_prefix(b"rate=") else {
            color_eyre::eyre::bail!("Malformed flow rate");
        };
        let flow: u64 = flow.to_str()?.parse()?;

        let neighbours = match neighbours.strip_prefix(b" tunnel leads to valve ") {
            None => match neighbours.strip_prefix(b" tunnels lead to valves ") {
                None => color_eyre::eyre::bail!("Malformed neighbours"),
                Some(v) => v,
            },
            Some(v) => v,
        };

        let neighbours: Vec<_> = neighbours
            .split_str(",")
            .map(|v| v.trim().as_bstr())
            .collect();

        valves.insert(name, (ordered.len(), flow, neighbours));
        ordered.push(name);
    }

    ordered
        .iter()
        .map(|name| -> color_eyre::Result<_> {
            let Some((_, flow, neighbours)) = valves.get(name) else {
                color_eyre::eyre::bail!("Can't get name")
            };
            let neighbours: Vec<_> = neighbours
                .iter()
                .map(|n| -> color_eyre::Result<_> {
                    let Some(&(index, _, _)) = valves.get(n) else {
                        color_eyre::eyre::bail!("Can't get neighbour")
                    };
                    Ok(index)
                })
                .collect::<Result<_, _>>()?;

            Ok(Valve {
                name,
                flow: *flow,
                neighbours,
            })
        })
        .collect()
}

fn layout(input: &Parsed) -> (Vec<usize>, impl Fn(usize, usize) -> usize) {
    let mut graph_index = Vec::new();

    let mut graph = Graph::<(), ()>::new();
    for _ in 0..input.len() {
        graph_index.push(graph.add_node(()));
    }

    for (valve, &gi) in input.iter().zip(&graph_index) {
        for &neighbour in &valve.neighbours {
            graph.add_edge(gi, graph_index[neighbour], ());
        }
    }

    let shortest_paths = petgraph::algo::floyd_warshall(&graph, |_| 1).unwrap();
    let distance = move |from: usize, to: usize| {
        *shortest_paths
            .get(&(graph_index[from], graph_index[to]))
            .unwrap()
    };

    let non_zero_flow: Vec<_> = input
        .iter()
        .enumerate()
        .filter(|(_, v)| v.flow != 0)
        .map(|(i, _)| i)
        .collect();

    (non_zero_flow, distance)
}

pub fn part1(input: Parsed) {
    let (non_zero_flow, distance) = layout(&input);

    #[derive(Debug, Clone, Eq)]
    struct Path {
        total_len: usize,
        visited: im::HashSet<usize>,
        path: Vector<usize>,
        relief: u64,
    }

    impl PartialEq for Path {
        fn eq(&self, other: &Self) -> bool {
            self.total_len == other.total_len && self.path == other.path
        }
    }

    impl Ord for Path {
        fn cmp(&self, other: &Self) -> std::cmp::Ordering {
            self.relief.cmp(&other.relief)
        }
    }

    impl PartialOrd for Path {
        fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
            Some(self.cmp(other))
        }
    }

    let (start, _) = input
        .iter()
        .enumerate()
        .find(|(_, path)| path.name == b"AA".as_bstr())
        .unwrap();
    assert_eq!(input[start].flow, 0);

    let mut finished_paths: BinaryHeap<Path> = BinaryHeap::new();
    let mut paths = BinaryHeap::new();
    paths.push(Path {
        path: vector![start],
        visited: im::HashSet::new(),
        total_len: 0,
        relief: 0,
    });

    let path_score = |path: &Path| {
        let mut time = 0;
        let mut total = 0;
        let mut current_pressure = 0;

        for (&source, &dest) in path.path.iter().tuple_windows() {
            let time_step = (distance(source, dest) + 1) as u64;
            time += time_step;
            total += time_step * current_pressure;
            current_pressure += input[dest].flow;
        }

        total + (30 - time) * current_pressure
    };

    while let Some(path) = paths.pop() {
        let mut added = false;

        for &non_zero in &non_zero_flow {
            if path.visited.contains(&non_zero) {
                continue;
            }

            let distance = distance(*path.path.last().unwrap(), non_zero);
            if path.total_len + distance + 1 < 30 {
                added = true;
                let mut new_path = path.clone();
                new_path.total_len += distance + 1;
                new_path.visited.insert(non_zero);
                new_path.path.push_back(non_zero);
                new_path.relief = path_score(&new_path);

                let should_insert = match finished_paths.peek() {
                    None => true,
                    Some(v) => {
                        (new_path.relief as f32) / (v.relief as f32) > 0.80
                            || new_path.total_len <= 10
                    }
                };

                if should_insert {
                    paths.push(new_path);
                }
            }
        }

        if !added {
            finished_paths.push(path);
        }
    }

    let best_path = finished_paths.pop().unwrap().relief;

    print_res!("Total pressure relief: {best_path}")
}

pub fn part2(input: Parsed) {
    let (non_zero_flow, distance) = layout(&input);

    #[derive(Debug, Clone, Eq)]
    struct Path {
        total_len: usize,
        path: Vector<usize>,
        elephant_len: usize,
        elephant_path: Vector<usize>,
        visited: im::HashSet<usize>,
        relief: u64,
    }

    impl PartialEq for Path {
        fn eq(&self, other: &Self) -> bool {
            self.total_len == other.total_len
                && self.path == other.path
                && self.elephant_len == other.elephant_len
                && self.elephant_path == other.elephant_path
        }
    }

    impl Ord for Path {
        fn cmp(&self, other: &Self) -> std::cmp::Ordering {
            self.relief.cmp(&other.relief)
        }
    }

    impl PartialOrd for Path {
        fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
            Some(self.cmp(other))
        }
    }

    let (start, _) = input
        .iter()
        .enumerate()
        .find(|(_, path)| path.name == b"AA".as_bstr())
        .unwrap();
    assert_eq!(input[start].flow, 0);

    let mut finished_paths: BinaryHeap<Path> = BinaryHeap::new();
    let mut paths = BinaryHeap::new();
    paths.push(Path {
        path: vector![start],
        elephant_path: vector![start],
        visited: im::HashSet::new(),
        total_len: 0,
        elephant_len: 0,
        relief: 0,
    });

    let path_score = |path: &Path| {
        let mut valves_opening = [(None, None); 26];

        let mut time = 0;
        for (&source, &dest) in path.path.iter().tuple_windows() {
            let time_step = distance(source, dest) + 1;
            time += time_step;
            valves_opening[time - 1].0 = Some(dest);
        }

        let mut time = 0;
        for (&source, &dest) in path.elephant_path.iter().tuple_windows() {
            let time_step = distance(source, dest) + 1;
            time += time_step;
            valves_opening[time - 1].1 = Some(dest);
        }

        let mut current_pressure = 0;
        let mut total = 0;

        for (v1, v2) in valves_opening {
            total += current_pressure;
            if let Some(v1) = v1 {
                current_pressure += input[v1].flow;
            }
            if let Some(v2) = v2 {
                current_pressure += input[v2].flow;
            }
        }

        total
    };

    while let Some(path) = paths.pop() {
        let mut added = false;

        for &non_zero in &non_zero_flow {
            if path.visited.contains(&non_zero) {
                continue;
            }

            let my_distance = distance(*path.path.last().unwrap(), non_zero);
            let elephant_distance = distance(*path.elephant_path.last().unwrap(), non_zero);

            let new_path = if my_distance < elephant_distance {
                if path.total_len + my_distance + 1 < 26 {
                    added = true;
                    let mut new_path = path.clone();
                    new_path.total_len += my_distance + 1;
                    new_path.visited.insert(non_zero);
                    new_path.path.push_back(non_zero);
                    new_path.relief = path_score(&new_path);

                    Some(new_path)
                } else if path.elephant_len + elephant_distance + 1 < 26 {
                    added = true;
                    let mut new_path = path.clone();
                    new_path.elephant_len += elephant_distance + 1;
                    new_path.visited.insert(non_zero);
                    new_path.elephant_path.push_back(non_zero);
                    new_path.relief = path_score(&new_path);

                    Some(new_path)
                } else {
                    None
                }
            } else if path.elephant_len + elephant_distance + 1 < 26 {
                added = true;
                let mut new_path = path.clone();
                new_path.elephant_len += elephant_distance + 1;
                new_path.visited.insert(non_zero);
                new_path.elephant_path.push_back(non_zero);
                new_path.relief = path_score(&new_path);

                Some(new_path)
            } else if path.total_len + my_distance + 1 < 26 {
                added = true;
                let mut new_path = path.clone();
                new_path.total_len += my_distance + 1;
                new_path.visited.insert(non_zero);
                new_path.path.push_back(non_zero);
                new_path.relief = path_score(&new_path);

                Some(new_path)
            } else {
                None
            };

            if let Some(new_path) = new_path {
                let should_insert = match finished_paths.peek() {
                    None => true,
                    Some(v) => {
                        (new_path.relief as f32) / (v.relief as f32) > 0.80
                            || (new_path.total_len + new_path.elephant_len) / 2 <= 12
                    }
                };

                if should_insert {
                    paths.push(new_path);
                }
            }
        }

        if !added {
            finished_paths.push(path);
        }
    }

    let best_path = finished_paths.pop().unwrap().relief;

    print_res!("Total pressure relief with elephant: {best_path}")
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
