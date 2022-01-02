use cargo_toml::{Dependency, DepsSet, Manifest};
use qu::ick_use::*;
use std::{env, fs, ops::RangeBounds, path::PathBuf};

mod config;
mod file_gen;
mod input;
mod util;
mod web;

use crate::{config::Config, util::IoResultExt};

#[derive(StructOpt)]
struct Opt {
    #[structopt(long, short, parse(from_os_str))]
    working_dir: Option<PathBuf>,
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
    /// Set the cookie used to download puzzle input
    SetCookie { cookie: String },
    /// Shows the cookie used to download puzzle input
    ShowCookie,
    /// Download all the input available for the given year that you don't already have.
    ///
    /// The year defaults to the latest one there is data for.
    Download { year: Option<u16> },
    /// For development
    Test,
}

#[qu::ick]
fn main(opt: Opt) -> Result {
    if let Some(dir) = &opt.working_dir {
        env::set_current_dir(dir)?;
    }
    match opt.cmd {
        Cmd::New => new(),
        Cmd::SetCookie { cookie } => set_cookie(cookie),
        Cmd::ShowCookie => show_cookie(),
        Cmd::Download { year } => download(year, ..),
        Cmd::Test => test(),
    }
}

fn new() -> Result {
    let config = Config::create_project()?;
    // Add dependencies
    let cargo_toml_path = config.project_root.join("Cargo.toml");
    let mut manifest = Manifest::from_path(&cargo_toml_path)?;
    manifest.bin.clear();
    ensure_dep("structopt", &mut manifest.dependencies);
    ensure_dep("qu", &mut manifest.dependencies);
    fs::write(cargo_toml_path, toml::to_vec(&manifest)?)?;
    // Write files
    file_gen::build_main_rs(&config)?;
    config.save()?;
    Ok(())
}

fn set_cookie(cookie: String) -> Result {
    let mut config = Config::load().context("cannot load aoc config")?;
    log::info!("setting aoc cookie to {:?}", cookie);
    config.file.cookie = Some(cookie);
    config.save().context("cannot save aoc config")?;
    Ok(())
}

fn show_cookie() -> Result {
    let config = Config::load().context("cannot load aoc config")?;
    match &config.file.cookie {
        Some(cookie) => log::info!("cookie: {:?}", cookie),
        None => log::info!("cookie: <unset>"),
    }
    Ok(())
}

/// The download command.
///
/// Defaults to most recent year, and all available days.
fn download(year: Option<u16>, days: impl RangeBounds<u8>) -> Result {
    let config = Config::load().context("cannot load aoc config")?;
    let cookie = match config.file.cookie.as_ref() {
        Some(cookie) => cookie.as_str(),
        None => {
            return Err(format_err!(
                "you need to store your cookie before you can download input"
            ))
        }
    };

    let year = year.unwrap_or(current_year());

    fs::create_dir_all(config.year_folder(year))?;
    for res in input::missing_input(&config, year, days) {
        let (year, day) = res?;
        let content = web::get_day(cookie, year, day)?;
        input::save_input(&config, year, day, content)?;
    }
    for res in input::all_for_year(&config, year) {
        let day = res?;
        log::info!("Generating source file for year {}, day {}", year, day);
        file_gen::build_day_src(&config, year, day)?;
    }
    log::info!("Generating mod file for year {}", year);
    file_gen::build_mod_file(&config, year)?;
    log::info!("Generating main.rs");
    file_gen::build_main_rs(&config)?;
    Ok(())
}

fn test() -> Result {
    let _config = Config::load()?;
    Ok(())
}

// If we're into december, then the current year, else the previous one.
fn current_year() -> u16 {
    use chrono::Datelike;

    let now = chrono::Utc::now();
    let year = now.year().try_into().expect("integer conversion");
    if now.month() == 12 {
        year
    } else {
        year - 1
    }
}

fn ensure_dep(name: &str, set: &mut DepsSet) {
    if !contains_dep(name, set) {
        set.insert(name.into(), Dependency::Simple("*".into()));
    }
}

fn contains_dep(name: &str, set: &DepsSet) -> bool {
    set.iter().any(|(dep_name, _)| dep_name == name)
}
