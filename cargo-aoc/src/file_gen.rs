use crate::{config::Config, input, util, Result};
use qu::ick_use::*;
use quote::{format_ident, quote};
use regex::Regex;
use rustfmt_wrapper::rustfmt;
use std::{collections::BTreeSet, fmt, fs};

pub fn build_main_rs(config: &Config) -> Result {
    let years: Vec<_> = input::get_years(config)?.collect();
    let year_idents = years
        .iter()
        .map(|year| format_ident!("_{}", year))
        .collect::<Vec<_>>();

    let content = auto_file(quote!(
        use qu::ick_use::*;

        #(mod #year_idents;)*

        #[derive(StructOpt)]
        struct Opt {
            /// Specify the year you want to run.
            ///
            /// Defaults to all years
            #[structopt(long, short)]
            year: Option<u16>,
            /// Specify the day you want to run.
            ///
            /// Defaults to all the days.
            #[structopt(long, short)]
            day: Option<u8>,
        }

        #[qu::ick]
        fn main(opt: Opt) {
            match opt.year {
                Some(year) => {
                    #(
                        if #years == year {
                            crate::#year_idents::run(opt.day)?;
                        }
                    )*
                },
                None => {
                    log::info!("Running all available solutions");
                    #(crate::#year_idents::run(opt.day)?;)*
                }
            }
            Ok(())
        }
    ));

    fs::write(config.project_root.join("src/main.rs"), content)?;

    Ok(())
}

pub fn build_mod_file(config: &Config, year: u16) -> Result {
    let mut days_present = BTreeSet::new();
    let regex = Regex::new(r"day(\d+)\.rs$").unwrap();
    let year_folder = config.year_folder(year);
    for file in fs::read_dir(&year_folder)? {
        let file = file?;
        if let Some(caps) = regex.captures(file.path().to_str().context("found non-utf8 file")?) {
            days_present.insert(caps.get(1).unwrap().as_str().parse::<u8>()?);
        }
    }

    let days = days_present.iter().collect::<Vec<_>>();
    let days_mods = days_present
        .iter()
        .map(|day| format_ident!("day{}", day))
        .collect::<Vec<_>>();

    let input_paths = days_present
        .iter()
        .map(|day| {
            config
                .input_path(year, *day)
                .to_str()
                .map(ToOwned::to_owned)
                .context("non-utf8 path")
        })
        .collect::<Result<Vec<_>>>()?;

    let content = auto_file(quote!(
        use qu::ick_use::*;

        #(mod #days_mods;)*

        pub fn run(day: Option<u8>) -> Result {
            match day {
                Some(day) => {
                    #(
                        if day == #days {
                            log::info!("Running day {}", #days);
                            let parsed = include_str!(#input_paths).lines().map(#days_mods::parse).collect::<Result<Vec<_>>>()?;
                            log::info!("  Part 1 result: {}", #days_mods::part1(&parsed));
                            log::info!("  Part 2 result: {}", #days_mods::part2(&parsed));
                        }
                    )*
                },
                None => {
                    #(
                        log::info!("Running day {}", #days);
                        let parsed = include_str!(#input_paths).lines().map(#days_mods::parse).collect::<Result<Vec<_>>>()?;
                        log::info!("  Part 1 result: {}", #days_mods::part1(&parsed));
                        log::info!("  Part 2 result: {}", #days_mods::part2(&parsed));
                    )*
                }
            }
            Ok(())
        }
    ));

    fs::write(year_folder.join("mod.rs"), content)?;

    Ok(())
}

/// Create a file for the given year and day, only if it is not already present.
pub fn build_day_src(config: &Config, year: u16, day: u8) -> Result {
    let filename = config.day_source(year, day);
    if util::path_exists(&filename)? {
        log::info!(
            "skipping already existing source file for year {}, day {}",
            year,
            day
        );
        return Ok(());
    }

    let content = gen_file(quote!(
        use qu::ick_use::*;
        use std::fmt;

        // TODO rename and change fields to something useful
        #[derive(Debug)]
        pub struct MyType;

        pub fn parse(_input: &str) -> Result<MyType> {
            Ok(MyType)
        }

        pub fn part1(_input: &[MyType]) -> impl fmt::Display {
            "<todo>"
        }

        pub fn part2(_input: &[MyType]) -> impl fmt::Display {
            "<todo>"
        }
    ));

    fs::write(&filename, content)?;
    Ok(())
}

// Helpers
// -------

fn auto_file(content: impl fmt::Display) -> String {
    const MSG: &str = "// NOTE: This file is auto-generated. `cargo-aoc` will overwrite any changes you make to it.\n\n";
    if let Ok(mut fmt) = rustfmt(&content) {
        fmt.insert_str(0, MSG);
        fmt
    } else {
        format!("{}{}", MSG, content)
    }
}

fn gen_file(content: impl fmt::Display) -> String {
    const MSG: &str = "\n\n// NOTE: This file was generated by `cargo-aoc`, and will not be overwritten. Feel free to delete this message.";
    if let Ok(mut fmt) = rustfmt(&content) {
        fmt.push_str(MSG);
        fmt
    } else {
        format!("{}{}", content, MSG)
    }
}
