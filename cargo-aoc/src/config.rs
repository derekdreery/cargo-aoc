//! Manage config stored in a `.aoc.toml` file in the project root.
use anyhow::Context;
use cargo::{
    ops::{NewOptions, VersionControl},
    util::important_paths::find_root_manifest_for_wd,
};
use qu::ick_use::*;
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    env, fs,
    io::prelude::*,
    path::{Path, PathBuf},
};

use crate::IoResultExt;

const PROJECT_NAME: &str = "aoc";
const CONFIG_PATH: &str = ".aoc.toml";

#[derive(Debug)]
pub struct Config {
    pub project_root: PathBuf,
    pub config_path: PathBuf,
    /// The contents of the `.aoc.toml` config file.
    pub file: AocConfig,
}

impl Config {
    /// Create a new project and return the config
    pub fn create_project() -> Result<Self> {
        fn inner() -> Result<Config> {
            let project_root = env::current_dir()?.join(PROJECT_NAME);

            let meta = project_root.metadata().optional()?;
            if meta.is_some() {
                bail!("directory \"{}\" already exists", project_root.display());
            }

            let config = cargo::Config::default()?;
            let opts = NewOptions::new(
                Some(VersionControl::NoVcs),
                true,
                false,
                project_root.clone(),
                None,
                None,
                config.default_registry()?,
            )?;
            cargo::ops::new(&opts, &config)?;

            let config_path = project_root.join(CONFIG_PATH);
            let file = AocConfig::create(&config_path)?;
            Ok(Config {
                project_root,
                config_path,
                file,
            })
        }
        inner().context("cannot create new project")
    }

    pub fn load() -> Result<Self> {
        let project_root =
            find_project_dir().context("cannot load project at the current location")?;
        let config_path = project_root.join(CONFIG_PATH);
        let file =
            AocConfig::load(&config_path).context("cannot load project at the current location")?;
        Ok(Config {
            project_root,
            config_path,
            file,
        })
    }

    /// The folder for the source files for year given.
    pub fn year_folder(&self, year: u16) -> PathBuf {
        self.project_root.join(format!("src/_{}", year))
    }

    /// The source file for the given year and day
    pub fn day_source(&self, year: u16, day: u8) -> PathBuf {
        self.project_root
            .join(format!("src/_{}/day{}.rs", year, day))
    }

    pub fn input_path(&self, year: u16, day: u8) -> PathBuf {
        self.project_root
            .join(format!("input/{}/input{}.txt", year, day))
    }

    pub fn save(self) -> Result {
        self.file.save(&*self.config_path)
    }
}

impl Drop for Config {
    fn drop(&mut self) {
        // We can't handle errors in `Drop`.
        let _ = self.file.save(&*self.config_path);
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AocConfig {
    pub cookie: Option<String>,
    /// Keeps track of the years we have downloaded for.
    pub years: BTreeMap<u16, BTreeSet<u8>>,
}

impl AocConfig {
    fn create(config_path: impl AsRef<Path>) -> Result<Self> {
        fn inner(config_path: &Path) -> Result<AocConfig> {
            let mut file = fs::OpenOptions::new()
                .create_new(true)
                .write(true)
                .open(config_path)?;
            let config = AocConfig::default();
            let config_raw = toml::to_vec(&config)?;
            file.write_all(&config_raw)?;
            Ok(config)
        }
        inner(config_path.as_ref()).context("cannot create config file")
    }

    fn load(config_path: impl AsRef<Path>) -> Result<Self> {
        fn inner(config_path: &Path) -> Result<AocConfig> {
            let bytes = fs::read(config_path)?;
            Ok(toml::from_slice(&bytes)?)
        }
        inner(config_path.as_ref()).context("could not load config file")
    }

    fn save(&self, path: impl AsRef<Path>) -> Result {
        fn inner(this: &AocConfig, path: &Path) -> Result {
            let bytes = toml::to_vec(this)?;
            fs::write(path, &bytes).map_err(Into::into)
        }
        inner(self, path.as_ref()).context("could not save config file")
    }
}

impl Default for AocConfig {
    fn default() -> Self {
        Self {
            cookie: None,
            years: BTreeMap::new(),
        }
    }
}

fn find_project_dir() -> Result<PathBuf> {
    let cwd = std::env::current_dir().context("cannot find project dir")?;
    let cargo_toml = find_root_manifest_for_wd(&cwd).context("cannot find project dir")?;
    cargo_toml
        .parent()
        .map(Into::into)
        .context("cannot find project dir")
}
