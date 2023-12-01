use std::path::PathBuf;

use clap::{command, Parser, Subcommand};
use serde::{Deserialize, Serialize};

/// Tool to automate downloading and submitting advent of code problems.
///
/// It requires the AOC_SESSION environment variable to be defined.
/// You can get the session value from the cookies on the advent of code page.
///
/// You also need to define a `./template` folder which gets copied then creating a
/// problem folder.
#[derive(Debug, Clone, Parser)]
#[command()]
pub struct Args {
    #[command(subcommand)]
    pub command: Command,
    /// The path of the configuration file.
    /// If defined you don't need to define the year and day in the commands.
    /// Example:
    /// ```toml
    /// year: 2020
    /// day: 1
    /// ```
    /// It can contain the fields "year" and "day".
    #[arg(short, long, default_value = "./aoc.toml")]
    pub config: PathBuf,
}

#[derive(Debug, Clone, Subcommand)]
pub enum Command {
    /// Creates a new folder and downloads the problem into it
    New {
        /// The folder to download the files into. `day_??` is the default value.
        output: Option<PathBuf>,
        #[arg(short, long)]
        year: Option<u32>,
        #[arg(short, long)]
        day: u32,
        #[arg(short, long, default_value = "./template")]
        template: PathBuf,
    },
    /// Download the problem statement, input and example
    Download {
        #[arg(short, long)]
        example: bool,
        #[arg(short, long)]
        year: Option<u32>,
        #[arg(short, long)]
        day: Option<u32>,
    },
    /// Submit your result
    Submit {
        result: String,
        #[arg(short, long)]
        year: Option<u32>,
        #[arg(short, long)]
        day: Option<u32>,
        #[arg(short, long)]
        level: Option<u32>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub year: u32,
    pub day: u32,
}
