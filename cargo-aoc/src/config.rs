//! Manage config stored in a `.aoc.toml` file in the project root.
use anyhow::Context;
use cargo::{
    ops::{NewOptions, VersionControl},
    util::important_paths::find_root_manifest_for_wd,
};
use qu::ick_use::*;
use serde::{Deserialize, Serialize};
use std::{
    env, fs,
    io::{self, prelude::*},
    path::{Path, PathBuf},
};

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
        let project_root = find_project_dir()?;
        let config_path = project_root.join(CONFIG_PATH);
        let file = AocConfig::load(&config_path)?;
        Ok(Config {
            project_root,
            config_path,
            file,
        })
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
        Self { cookie: None }
    }
}

trait IoResultExt<T> {
    fn optional(self) -> io::Result<Option<T>>;
}

impl<T> IoResultExt<T> for io::Result<T> {
    fn optional(self) -> io::Result<Option<T>> {
        match self {
            Ok(v) => Ok(Some(v)),
            Err(e) if matches!(e.kind(), io::ErrorKind::NotFound) => Ok(None),
            Err(e) => Err(e),
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
