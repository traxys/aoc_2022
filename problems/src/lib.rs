use std::path::PathBuf;

use bstr::BString;
use clap::Parser;

pub mod solutions;

#[derive(Parser)]
struct Args {
    #[arg(short, long)]
    part: u32,
    #[arg(short, long)]
    input: PathBuf,
}

#[derive(Debug)]
pub struct Context {
    pub part: u32,
    pub input: BString,
}

pub fn load() -> color_eyre::Result<Context> {
    color_eyre::install()?;

    let args = Args::parse();

    let input = std::fs::read(args.input)?.into();

    Ok(Context {
        part: args.part,
        input,
    })
}
