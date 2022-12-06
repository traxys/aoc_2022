use std::{
    fs::OpenOptions,
    io::Write,
    os::unix::process::CommandExt,
    path::{Path, PathBuf},
};

use chrono::{Datelike, Local};
use clap::Parser;
use color_eyre::eyre;
use reqwest::header::{self, HeaderValue};

#[derive(Parser, Debug)]
pub struct Args {
    #[arg(short, long)]
    day: Option<u32>,
    #[arg(short, long)]
    part: Option<u32>,
    #[arg(short, long)]
    input: Option<PathBuf>,
    #[arg(short, long, env = "AOC_COOKIE")]
    cookie: Option<String>,
    #[arg(long)]
    release: bool,
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Parser, Debug, Clone)]
pub enum Command {
    Init,
    Edit,
    Fetch,
    Run,
}

fn fetch(day: u32, input_dir: &Path, cookie: &Option<String>) -> color_eyre::Result<()> {
    let Some(cookie) = cookie else { eyre::bail!("Must provide cookie to fetch inputs") };

    let client = reqwest::blocking::Client::new();
    let data = client
        .get(format!("https://adventofcode.com/2022/day/{day}/input"))
        .header(
            header::COOKIE,
            HeaderValue::from_str(&format!("session={cookie}"))?,
        )
        .send()?
        .bytes()?;

    let mut input_file = OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(input_dir.join(format!("day{day}")))?;

    input_file.write_all(&data)?;

    Ok(())
}

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    let args = Args::parse();
    let time = Local::now();
    let day = args.day.unwrap_or_else(|| time.day());

    let workspace = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    let inputs_dir = workspace.join("inputs");

    if !inputs_dir.exists() {
        std::fs::create_dir(&inputs_dir)?;
    }

    let day_str = format!("day{day}");
    let day_file = workspace.join(format!("problems/src/solutions/{day_str}.rs"));
    let day_bin_file = workspace.join(format!("problems/src/bin/{day_str}.rs"));

    let input = inputs_dir.join(&day_str);

    match args.command {
        Some(Command::Edit) => {
            return Err(std::process::Command::new(std::env::var("EDITOR")?)
                .arg(day_file)
                .exec()
                .into());
        }
        Some(Command::Init) => {
            let template = workspace.join("template.rs");
            let day = workspace.join(day_file);
            std::fs::copy(template, day)?;

            let bin_content = format!(
                r#"
                fn main() -> color_eyre::Result<()> {{
                    problems::solutions::{day_str}::main()
                }}
            "#
            );

            std::fs::write(day_bin_file, bin_content.as_bytes())?;

            let mut solution_mod = OpenOptions::new()
                .append(true)
                .open(workspace.join("problems/src/solutions/mod.rs"))?;
            writeln!(solution_mod, "pub mod {day_str};")?;
        }
        Some(Command::Fetch) => fetch(day, &inputs_dir, &args.cookie)?,
        Some(Command::Run) | None => {
            let input = args.input.unwrap_or(input);

            if !input.exists() {
                fetch(day, &inputs_dir, &args.cookie)?;
            }

            let day_impl = std::fs::read_to_string(day_file)?;
            let impl_part = if day_impl.contains(r#"todo!("todo part2")"#) {
                1
            } else {
                2
            };

            let part = args.part.unwrap_or(impl_part);

            println!("==> Running day {day} part {}", part);
            let mut command = std::process::Command::new(env!("CARGO"));
            command
                .current_dir(workspace)
                .args(["run", "--package", "problems", "--bin"])
                .arg(&day_str);

            if args.release {
                command.arg("--release");
            }

            command
                .args(["--", "--part"])
                .arg(part.to_string())
                .arg("--input")
                .arg(&input);

            command.spawn()?.wait()?;
        }
    }

    Ok(())
}
