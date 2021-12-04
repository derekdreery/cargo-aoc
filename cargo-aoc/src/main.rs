use std::{fs, path::Path};

use qu::ick_use::*;

mod config;
mod file_gen;
mod web;

use crate::config::Config;

#[derive(StructOpt)]
struct Opt {
    #[structopt(subcommand)]
    cmd: Cmd,
}

#[derive(StructOpt)]
enum Cmd {
    /// Create a new project for advent of code.
    ///
    /// This creates a folder with the required structure. The folder will be called
    /// `aoc` and will be in the current directory.
    New,
    /// Download all the input available for the given year that you don't already have.
    ///
    /// The year defaults to the latest one there is data for.
    Download { year: Option<u16> },
}

#[qu::ick]
fn main(opt: Opt) -> Result {
    match opt.cmd {
        Cmd::New => new(),
        Cmd::Download { year } => download(year),
    }
}

fn new() -> Result {
    let config = Config::create_project()?;
    fs::write(config.project_root.join("src/lib.rs"), file_gen::main_rs())?;
    Ok(())
}

fn download(year: Option<u16>) -> Result {
    // use this for testing for now
    web::get_year(2021)?;
    Ok(())
}
