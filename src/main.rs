use std::{fs, path::PathBuf};

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use humans_cli::{
    Human, Segment, find_duplicates, merge_humans, normalize_human, segment_humans,
    summarize_humans,
};
use serde::{Serialize, de::DeserializeOwned};

#[derive(Parser)]
#[command(
    name = "humans",
    version,
    about = "Local-first human record normalization, identity resolution, and segmentation CLI",
    after_help = "Commands read local JSON and print deterministic JSON. No remote system is contacted."
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    Normalize {
        #[arg(long)]
        human: PathBuf,
    },
    Duplicates {
        #[arg(long)]
        humans: PathBuf,
    },
    Merge {
        #[arg(long)]
        humans: PathBuf,
        #[arg(long)]
        target_id: Option<String>,
    },
    Segment {
        #[arg(long)]
        humans: PathBuf,
        #[arg(long)]
        segment: PathBuf,
    },
    Summarize {
        #[arg(long)]
        humans: PathBuf,
    },
}

fn read_json<T: DeserializeOwned>(path: &PathBuf) -> Result<T> {
    let input = fs::read(path).with_context(|| format!("failed to read {}", path.display()))?;
    serde_json::from_slice(&input).with_context(|| format!("invalid JSON in {}", path.display()))
}

fn output<T: Serialize>(value: &T) -> Result<()> {
    println!("{}", serde_json::to_string_pretty(value)?);
    Ok(())
}

fn run() -> Result<()> {
    match Cli::parse().command {
        Command::Normalize { human } => output(&normalize_human(read_json::<Human>(&human)?)?),
        Command::Duplicates { humans } => {
            output(&find_duplicates(read_json::<Vec<Human>>(&humans)?)?)
        }
        Command::Merge { humans, target_id } => {
            output(&merge_humans(read_json::<Vec<Human>>(&humans)?, target_id)?)
        }
        Command::Segment { humans, segment } => output(&segment_humans(
            read_json::<Vec<Human>>(&humans)?,
            read_json::<Segment>(&segment)?,
        )?),
        Command::Summarize { humans } => {
            output(&summarize_humans(read_json::<Vec<Human>>(&humans)?)?)
        }
    }
}

fn main() {
    if let Err(error) = run() {
        eprintln!("{error:#}");
        std::process::exit(1);
    }
}
