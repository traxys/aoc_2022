use std::{collections::VecDeque, time::Instant};

use crate::{load, print_res};
use bstr::{BString, ByteSlice};

#[derive(Clone, Copy, Debug)]
pub struct Blueprint {
    ore: u8,
    clay: u8,
    obsidian: (u8, u8),
    geode: (u8, u8),
}

type Parsed = Vec<Blueprint>;

pub fn parsing(input: &BString) -> color_eyre::Result<Parsed> {
    input
        .lines()
        .map(|line| -> color_eyre::Result<_> {
            let Some((_, parts)) = line.split_once_str(":") else {
                color_eyre::eyre::bail!("No colon in blueprint")
            };
            let Ok([ore, clay, obsidian, geode]): Result<[&[u8]; 4],_> = parts
                .split_str(".").take(4)
                .map(|part| {
                    part
                        .split_once_str("costs")
                        .ok_or(color_eyre::eyre::eyre!("No cost for robot"))
                        .map(|c| c.1)
                })
                .collect::<Result<Vec<_>, _>>()?
                .try_into()
            else {
                color_eyre::eyre::bail!("Could not split robots descriptions")
            };

            let nums = |cost: &[u8], amount: usize| {
                cost.trim()
                    .split_str(" ")
                    .filter(|x| x[0].is_ascii_digit())
                    .map(|x| -> color_eyre::Result<_> { Ok(x.to_str()?.parse()?) })
                    .take(amount)
                    .collect::<Result<Vec<_>, _>>()
            };
            let &[ore, ..] = &nums(ore, 1)?[..] else { color_eyre::eyre::bail!("can't parse ore") };
            let &[clay, ..] = &nums(clay, 1)?[..] else { color_eyre::eyre::bail!("can't parse clay") };
            let &[obsidian_0, obsidian_1, ..] = &nums(obsidian, 2)?[..] else {
                color_eyre::eyre::bail!("can't parse obsidian")
            };
            let &[geode_0, geode_1, ..] = &nums(geode, 2)?[..] else {
                color_eyre::eyre::bail!("can't parse geode")
            };

            Ok(Blueprint {
                ore,
                clay,
                obsidian: (obsidian_0, obsidian_1),
                geode: (geode_0, geode_1),
            })
        })
        .collect()
}

#[derive(Default, Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct Robots {
    ore: u8,
    clay: u8,
    obsidian: u8,
    geodes: u8,
}

#[derive(Default, Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct Res {
    ore: u8,
    clay: u8,
    obsidian: u8,
}

#[derive(Default, Clone, Copy, Debug)]
struct State {
    geodes: u8,
    res: Res,
    robots: Robots,

    // dbg: [u8; DURATION as usize],
    time: u8,
}

impl State {
    fn geode_potential(&self, duration: u8) -> u16 {
        (self.robots.geodes as u16) * (duration - self.time) as u16 + self.geodes as u16
    }
}

fn blueprint_count(blueprint: &Blueprint, duration: u8, trim_branch: u16) -> u8 {
    let mut initial_state = State::default();
    initial_state.robots.ore = 1;

    let max_ore = blueprint
        .ore
        .max(blueprint.clay)
        .max(blueprint.obsidian.0)
        .max(blueprint.geode.0) as u8;
    let max_clay = blueprint.obsidian.1 as u8;

    //let mut seen = HashMap::new();

    let mut best_geode = 0;
    let mut best_potential = 0;
    //let mut best_state = initial_state;
    //let mut best_potential = 0;
    let mut states = VecDeque::new();
    states.push_back(initial_state);
    while let Some(state) = states.pop_front() {
        let mut next_state = state;
        next_state.res.ore += state.robots.ore;
        next_state.res.clay += state.robots.clay;
        next_state.res.obsidian += state.robots.obsidian;
        next_state.geodes += state.robots.geodes;
        next_state.time += 1;

        if next_state.geodes > best_geode && next_state.time <= duration {
            best_geode = next_state.geodes;
            //best_state = next_state;
        }

        best_potential = next_state.geode_potential(duration).max(best_potential);

        // let cache_key = (next_state.res, next_state.robots);
        //
        // match seen.get_mut(&cache_key) {
        //     None => {
        //         seen.insert(cache_key, next_state.geodes);
        //     }
        //     Some(ge) => {
        //         if *ge > next_state.geodes {
        //             continue;
        //         } else {
        //             *ge = next_state.geodes;
        //         }
        //     }
        // }

        if next_state.time < duration
            && (best_potential < 4
                || best_potential - next_state.geode_potential(duration) < trim_branch)
        {
            states.push_back(next_state);

            if state.res.ore >= blueprint.ore && state.robots.ore < max_ore {
                let mut new_state = next_state;
                new_state.res.ore -= blueprint.ore;
                new_state.robots.ore += 1;
                // new_state.dbg[(state.time - 1) as usize] |= 1;

                states.push_back(new_state);
            }

            if state.res.ore >= blueprint.clay && state.robots.clay < max_clay {
                let mut new_state = next_state;
                new_state.res.ore -= blueprint.clay;
                new_state.robots.clay += 1;

                // new_state.dbg[(state.time - 1) as usize] |= 2;

                states.push_back(new_state);
            }

            if state.res.ore >= blueprint.obsidian.0 && state.res.clay >= blueprint.obsidian.1 {
                let mut new_state = next_state;
                new_state.res.ore -= blueprint.obsidian.0;
                new_state.res.clay -= blueprint.obsidian.1;
                new_state.robots.obsidian += 1;

                // new_state.dbg[(state.time - 1) as usize] |= 4;

                states.push_back(new_state);
            }

            if state.res.ore >= blueprint.geode.0 && state.res.obsidian >= blueprint.geode.1 {
                let mut new_state = next_state;
                new_state.res.ore -= blueprint.geode.0;
                new_state.res.obsidian -= blueprint.geode.1;
                new_state.robots.geodes += 1;

                // new_state.dbg[(state.time - 1) as usize] |= 8;

                states.push_back(new_state);
            }
        }
    }

    best_geode
}

pub fn part1(input: Parsed) {
    let quality_levels: usize = input
        .iter()
        .enumerate()
        .inspect(|(i, _)| println!("Running blueprint {}", i + 1))
        .map(|(i, b)| (i + 1) * blueprint_count(b, 24, 2) as usize)
        .sum();
    print_res!("Quality level sum: {quality_levels}")
}

pub fn part2(input: Parsed) {
    let geode_amount_product: usize = input
        .iter()
        .take(3)
        .map(|b| blueprint_count(b, 32, 3) as usize)
        .product();
    print_res!("Product of geode amounts: {geode_amount_product}");
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
