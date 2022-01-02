//! Functionality around fetching and storing puzzle input.

use crate::{util::path_exists, Config};
use qu::ick_use::*;
use regex::Regex;
use std::{
    fs, io,
    ops::{Bound, RangeBounds},
};

pub fn missing_input(
    config: &Config,
    year: u16,
    days: impl RangeBounds<u8>,
) -> impl Iterator<Item = Result<(u16, u8)>> + '_ {
    let days_start = match days.start_bound() {
        Bound::Unbounded => 0,
        Bound::Excluded(bound) => (*bound + 1),
        Bound::Included(bound) => (*bound),
    }
    .max(1);
    let days_end = match days.end_bound() {
        Bound::Unbounded => u8::MAX,
        Bound::Excluded(bound) => (*bound - 1),
        Bound::Included(bound) => (*bound),
    }
    .min(25);
    MissingInput::new(config, year, days_start, days_end)
}

struct MissingInput<'a> {
    config: &'a Config,
    year: u16,
    days_current: u8,
    days_end: u8,
}

impl<'a> MissingInput<'a> {
    fn new(config: &'a Config, year: u16, days_start: u8, days_end: u8) -> Self {
        Self {
            config,
            year,
            days_current: days_start,
            days_end,
        }
    }
}

impl<'a> Iterator for MissingInput<'a> {
    type Item = Result<(u16, u8)>;

    fn next(&mut self) -> Option<Self::Item> {
        fn inner(this: &mut MissingInput<'_>) -> Result<Option<(u16, u8)>> {
            while input_exists(this.config, this.year, this.days_current)? {
                log::info!(
                    "Skipping already-present input for year {}, day {}",
                    this.year,
                    this.days_current
                );
                this.days_current += 1;
            }
            Ok(if this.days_current > this.days_end {
                None
            } else {
                let ans = Some((this.year, this.days_current));
                this.days_current += 1;
                ans
            })
        }
        inner(self).transpose()
    }
}

/// Gets an iterator over all days in a given year where there is input
pub fn all_for_year(config: &Config, year: u16) -> impl Iterator<Item = Result<u8>> + '_ {
    AllForYear::new(config, year)
}

struct AllForYear<'a> {
    config: &'a Config,
    year: u16,
    days_current: u8,
}

impl<'a> AllForYear<'a> {
    fn new(config: &'a Config, year: u16) -> Self {
        Self {
            config,
            year,
            days_current: 1,
        }
    }
}

impl<'a> Iterator for AllForYear<'a> {
    type Item = Result<u8>;

    fn next(&mut self) -> Option<Self::Item> {
        fn inner(this: &mut AllForYear<'_>) -> Result<Option<u8>> {
            if this.days_current > 25 {
                return Ok(None);
            }
            while !input_exists(this.config, this.year, this.days_current)? {
                log::warn!(
                    "Skipping missing input for year {}, day {}",
                    this.year,
                    this.days_current
                );
                this.days_current += 1;
                if this.days_current > 25 {
                    return Ok(None);
                }
            }
            let ans = this.days_current;
            this.days_current += 1;
            Ok(Some(ans))
        }
        inner(self).transpose()
    }
}

pub fn get_years(config: &Config) -> Result<impl Iterator<Item = u16>> {
    let mut years_remaining = vec![];
    let regex = Regex::new(r"_(\d+)$").unwrap();
    for entry in fs::read_dir(config.project_root.join("src"))? {
        let entry = entry?;
        if entry.metadata()?.is_dir() {
            if let Some(caps) = regex.captures(entry.path().to_str().context("non-utf8 path")?) {
                years_remaining.push(
                    caps.get(1)
                        .unwrap()
                        .as_str()
                        .parse::<u16>()
                        .context("cannot parse year from folder name")?,
                );
            }
        }
    }
    Ok(years_remaining.into_iter())
}

pub fn input_exists(config: &Config, year: u16, day: u8) -> io::Result<bool> {
    let path = config.input_path(year, day);
    path_exists(&path)
}

/// Returns `true` if there was already a file there (that file was overwritten)
pub fn save_input(config: &Config, year: u16, day: u8, input: impl AsRef<str>) -> io::Result<bool> {
    let path = config.input_path(year, day);
    fs::create_dir_all(path.parent().unwrap())?;
    // Possible race condition here (file could be deleted in between) but I don't think it matters.
    // Just making a note.
    let exists = path_exists(&path)?;

    fs::write(&path, input.as_ref())?;
    Ok(exists)
}
