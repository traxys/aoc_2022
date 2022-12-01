use std::path::PathBuf;

use bstr::BString;
use clap::Parser;

#[derive(Parser)]
struct Args {
    #[arg(short, long)]
    part: u32,
    #[arg(short, long)]
    input: PathBuf,
    #[arg(long)]
    show_impl_parts: bool,
}

#[derive(Debug)]
pub struct Context {
    pub part: u32,
    pub input: BString,
}

pub fn load(implemented_parts: u8) -> color_eyre::Result<Context> {
    color_eyre::install()?;

    let args = Args::parse();

    if args.show_impl_parts {
        println!("{implemented_parts}");
        std::process::exit(0)
    }

    let input = std::fs::read(args.input)?.into();

    Ok(Context {
        part: args.part,
        input,
    })
}
