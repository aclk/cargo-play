use std::ffi::{OsStr, OsString};
use std::iter::FromIterator;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::vec::Vec;
use sha1::{Sha1, Digest};
use structopt::StructOpt;

use crate::errors::CargoPlayError;

#[derive(Debug, Clone)]
pub enum RustEdition {
    E2015,
    E2018,
	E2021,
}

impl FromStr for RustEdition {
    type Err = CargoPlayError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
		if s == "2015" {
            Ok(RustEdition::E2015)
        } else if s == "2018" {
            Ok(RustEdition::E2018)
        } else if s == "2021" {
            Ok(RustEdition::E2021)
        } else {
            Err(CargoPlayError::InvalidEdition(s.into()))
        }
    }
}

impl Into<String> for RustEdition {
    fn into(self) -> String {
        match self {
            RustEdition::E2015 => "2015".into(),
            RustEdition::E2018 => "2018".into(),
            RustEdition::E2021 => "2021".into(),
        }
    }
}

impl Default for RustEdition {
    fn default() -> Self {
        RustEdition::E2021
    }
}

#[derive(Debug, StructOpt, Default)]
#[structopt(
    name = "cargo-play",
    about = "Run your Rust program without Cargo.toml"
)]
pub struct Options {
    #[structopt(short = "d", long = "debug", hidden = true)]
    pub debug: bool,

    #[structopt(short = "c", long = "clean")]
    /// Rebuild the Cargo project without the cache from previous run
    pub clean: bool,

    #[structopt(
        short = "m",
        long = "mode",
        group = "modegroup",
    )]
    /// Specify subcommand to use when calling Cargo [default: run]
    pub mode: Option<String>,

    /// Run test code in your code (alias to `--mode test`)
    #[structopt(long = "test", group = "modegroup")]
    pub test: bool,

    /// Check errors in your code (alias to `--mode check`)
    #[structopt(long = "check", group = "modegroup")]
    pub check: bool,

    /// Expand macro in your code (alias to `--mode expand`, requires
    /// `cargo-expand`)
    #[structopt(long = "expand", group = "modegroup")]
    pub expand: bool,

    #[structopt(short = "t", long = "toolchain", hidden = true)]
    pub toolchain: Option<String>,

    #[structopt(
        parse(try_from_os_str = osstr_to_abspath),
        required_unless = "stdin",
        validator = file_exist
    )]
    /// Paths to your source code files
    pub src: Vec<PathBuf>,

    #[structopt(
        short = "e",
        long = "edition",
        default_value = "2021",
        possible_values = &["2015", "2018", "2021"]
    )]
    /// Specify Rust edition
    pub edition: RustEdition,

    #[structopt(long = "release")]
    /// Build program in release mode
    pub release: bool,

    #[structopt(long = "cached", hidden = true)]
    pub cached: bool,

    #[structopt(long = "quiet", short = "q")]
    /// Disable output from Cargo (equivlant to `cargo run --quiet`)
    pub quiet: bool,

    #[structopt(long = "verbose", short = "v", parse(from_occurrences))]
    /// Set Cargo verbose level
    pub verbose: u16,

    #[structopt(long = "stdin")]
    /// Use stdin as main.rs
    pub stdin: bool,

    #[structopt(long = "cargo-option")]
    /// Customize flags passing to Cargo
    pub cargo_option: Option<String>,

    #[structopt(long = "save")]
    /// Generate a Cargo project based on inputs
    pub save: Option<PathBuf>,

    /// [experimental] Automatically infers crate dependency
    #[structopt(long = "infer", short = "i")]
    pub infer: bool,

    #[structopt(multiple = true, last = true)]
    /// Arguments passed to the underlying program
    pub args: Vec<String>,
}

impl Options {
    #[allow(unused)]
    /// Convenient constructor for testing
    pub fn with_files<I: AsRef<Path>>(src: Vec<I>) -> Self {
        Self {
            src: src
                .into_iter()
                .filter_map(|x| std::fs::canonicalize(x).ok())
                .collect(),
            ..Default::default()
        }
    }

    /// Generate a string of hash based on the path passed in
    pub fn src_hash(&self) -> String {
        let mut hash = Sha1::new();
        let mut srcs = self.src.clone();

        srcs.sort();

        for file in srcs.into_iter() {
            hash.update(file.to_string_lossy().as_bytes());
        }

        bs58::encode(hash.finalize()).into_string()
    }

    pub fn temp_dirname(&self) -> PathBuf {
        format!("cargo-play.{}", self.src_hash()).into()
    }

    fn with_toolchain(mut self, toolchain: Option<String>) -> Self {
        self.toolchain = toolchain;
        self
    }

    pub fn parse(args: Vec<String>) -> Result<Self, ()> {
        if args.len() < 2 {
            Self::clap().print_help().unwrap_or(());
            return Err(());
        }

        let with_cargo = args[1] == "play";
        let mut args = args.into_iter();

        if with_cargo {
            args.next();
        }

        let toolchain = args
            .clone()
            .find(|x| x.starts_with('+'))
            .map(|s| String::from_iter(s.chars().skip(1)));

        Ok(Self::from_iter(args.filter(|x| !x.starts_with('+'))).with_toolchain(toolchain))
    }
}

/// Convert `std::ffi::OsStr` to an absolute `std::path::PathBuf`
fn osstr_to_abspath(v: &OsStr) -> Result<PathBuf, OsString> {
    if let Ok(r) = PathBuf::from(v).canonicalize() {
        Ok(r)
    } else {
        Err(v.into())
    }
}

/// structopt compataible function to check whether a file exists
fn file_exist(v: String) -> Result<(), String> {
    let p = PathBuf::from(v);
    if !p.is_file() {
        Err(format!("input file does not exist: {:?}", p))
    } else {
        Ok(())
    }
}
