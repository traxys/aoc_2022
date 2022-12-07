use std::{collections::HashMap, time::Instant};

use crate::{load, print_res};
use bstr::{BStr, BString, ByteSlice};

#[derive(Debug, Clone, Copy)]
pub enum Dest<'a> {
    Up,
    Root,
    Dir(&'a BStr),
}

#[derive(Debug, Clone, Copy)]
pub enum EntryInfo {
    File { size: u64 },
    Dir,
}

#[derive(Debug, Clone, Copy)]
pub struct Entry<'a> {
    name: &'a BStr,
    info: EntryInfo,
}

#[derive(Debug, Clone)]
pub enum Command<'a> {
    Cd(Dest<'a>),
    Ls(Vec<Entry<'a>>),
}

type Parsed<'a> = Vec<Command<'a>>;

pub fn parsing(input: &BString) -> color_eyre::Result<Parsed> {
    input
        .split_str("$ ")
        .skip(1)
        .map(|command| -> color_eyre::Result<_> {
            let mut lines = command.lines();
            let Some(cmd) = lines.next() else {
                color_eyre::eyre::bail!("No command in {}", command.as_bstr())
            };

            if let Some(dest) = cmd.strip_prefix(b"cd ") {
                let dest = match dest {
                    b"/" => Dest::Root,
                    b".." => Dest::Up,
                    dest => Dest::Dir(dest.as_bstr()),
                };
                Ok(Command::Cd(dest))
            } else if cmd == b"ls" {
                let entries: Result<_, _> = lines
                    .map(|entry| -> color_eyre::Result<_> {
                        let Some((info, name)) = entry.split_once_str(" ") else {
                            color_eyre::eyre::bail!("Malformed entry {}", entry.as_bstr());
                        };
                        let info = if info == b"dir" {
                            EntryInfo::Dir
                        } else {
                            let size = info.to_str()?.parse()?;
                            EntryInfo::File { size }
                        };
                        Ok(Entry {
                            name: name.as_bstr(),
                            info,
                        })
                    })
                    .collect();

                Ok(Command::Ls(entries?))
            } else {
                color_eyre::eyre::bail!("unknown command {}", cmd.as_bstr())
            }
        })
        .collect()
}

#[derive(Default, Debug)]
struct Tree<'a> {
    files: HashMap<&'a BStr, u64>,
    dirs: HashMap<&'a BStr, Tree<'a>>,
}

impl<'a> Tree<'a> {
    fn populate<'t, 'c>(&'t mut self, commands: &'c [Command<'a>]) -> &'c [Command<'a>] {
        if commands.is_empty() {
            return commands;
        }

        let commands_rest = &commands[1..];

        match &commands[0] {
            Command::Cd(dest) => match dest {
                Dest::Up => commands_rest,
                Dest::Root => panic!("Can't go back to root"),
                Dest::Dir(d) => {
                    let dir = self.dirs.get_mut(d).expect("cd into unvisited dir");
                    let commands_rest = dir.populate(commands_rest);

                    self.populate(commands_rest)
                }
            },
            Command::Ls(entries) => {
                for entry in entries {
                    match entry.info {
                        EntryInfo::File { size } => {
                            assert!(self.files.insert(entry.name, size).is_none());
                        }
                        EntryInfo::Dir => {
                            assert!(self.dirs.insert(entry.name, Tree::default()).is_none());
                        }
                    }
                }

                self.populate(commands_rest)
            }
        }
    }
}

#[derive(Default, Debug)]
struct SizedTree<'a> {
    files: HashMap<&'a BStr, u64>,
    dirs: HashMap<&'a BStr, SizedTree<'a>>,
    total_size: u64,
}

impl<'a> SizedTree<'a> {
    fn from_tree(tree: Tree<'a>) -> Self {
        let dirs = tree
            .dirs
            .into_iter()
            .map(|(n, d)| (n, Self::from_tree(d)))
            .collect();
        let mut sized_tree = SizedTree {
            dirs,
            files: tree.files,
            total_size: 0,
        };

        let total_size = sized_tree.files.values().sum::<u64>()
            + sized_tree.dirs.values().map(|t| t.total_size).sum::<u64>();
        sized_tree.total_size = total_size;

        sized_tree
    }

    fn size_less_than(&self, threshold: u64) -> u64 {
        let children_size: u64 = self
            .dirs
            .values()
            .map(|t| t.size_less_than(threshold))
            .sum();

        if self.total_size <= threshold {
            children_size + self.total_size
        } else {
            children_size
        }
    }

    fn find_smallest_at_least(&self, size: u64) -> Option<u64> {
        match self
            .dirs
            .values()
            .filter_map(|t| t.find_smallest_at_least(size))
            .min()
        {
            None => {
                if self.total_size >= size {
                    Some(self.total_size)
                } else {
                    None
                }
            }
            Some(v) => Some(v),
        }
    }
}

pub fn part1(input: Parsed) {
    std::assert!(matches!(input[0], Command::Cd(Dest::Root)));
    let mut fs = Tree::default();
    assert!(fs.populate(&input[1..]).is_empty());
    // We now have a populated file system

    let sized_fs = SizedTree::from_tree(fs);

    print_res!(
        "Total size of directories: {}",
        sized_fs.size_less_than(100000)
    );
}

pub fn part2(input: Parsed) {
    std::assert!(matches!(input[0], Command::Cd(Dest::Root)));
    let mut fs = Tree::default();
    assert!(fs.populate(&input[1..]).is_empty());
    // We now have a populated file system

    let sized_fs = SizedTree::from_tree(fs);

    let fs_total = 70000000;
    let fs_free = fs_total - sized_fs.total_size;
    let update_size = 30000000;
    let missing_space = update_size - fs_free;

    let smallest_delete_size = sized_fs
        .find_smallest_at_least(missing_space)
        .expect("no directory is large enough");

    print_res!("Smallest dir size that allows update: {smallest_delete_size}");
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
