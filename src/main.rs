use std::path::PathBuf;

use anyhow::Result;
use clap::{Parser, Subcommand};
use humans_cli::{
    Human, Segment, find_duplicates, merge_humans, normalize_human, segment_humans,
    summarize_humans,
};
use wisent_errors::cli_output::{output, read_json};

#[derive(Parser)]
#[command(
    name = "humans",
    version,
    about = "Local-first human record normalization, identity resolution, and segmentation CLI",
    after_help = "Commands read local JSON; no remote system is contacted. Every command prints deterministic JSON, or with --text one `path: value` line per field from the same data. Exit 2: the invocation is wrong; exit 1: an input could not be read or was refused."
)]
struct Cli {
    /// Print one `path: value` line per field instead of JSON.
    #[arg(long, global = true)]
    text: bool,
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Read one human record and print it in canonical form.
    Normalize {
        /// The human record JSON file.
        #[arg(long)]
        human: PathBuf,
    },
    /// Group records that share an identity.
    Duplicates {
        /// A JSON array of human records.
        #[arg(long)]
        humans: PathBuf,
    },
    /// Merge records into one, listing every field that conflicted.
    Merge {
        /// A JSON array of human records; the first is the base.
        #[arg(long)]
        humans: PathBuf,
        /// The id the merged record takes; the first record's id when omitted.
        #[arg(long)]
        target_id: Option<String>,
    },
    /// Print the records a segment definition selects.
    Segment {
        /// A JSON array of human records.
        #[arg(long)]
        humans: PathBuf,
        /// The segment definition JSON file.
        #[arg(long)]
        segment: PathBuf,
    },
    /// Count records, identities, duplicate groups, identity types, tags and consent.
    Summarize {
        /// A JSON array of human records.
        #[arg(long)]
        humans: PathBuf,
    },
}

fn run() -> Result<()> {
    let cli = Cli::parse();
    let text = cli.text;
    match cli.command {
        Command::Normalize { human } => {
            output(&normalize_human(read_json::<Human>(&human)?)?, text)
        }
        Command::Duplicates { humans } => {
            output(&find_duplicates(read_json::<Vec<Human>>(&humans)?)?, text)
        }
        Command::Merge { humans, target_id } => {
            output(&merge_humans(read_json::<Vec<Human>>(&humans)?, target_id)?, text)
        }
        Command::Segment { humans, segment } => output(&segment_humans(
            read_json::<Vec<Human>>(&humans)?,
            read_json::<Segment>(&segment)?,
        )?, text),
        Command::Summarize { humans } => {
            output(&summarize_humans(read_json::<Vec<Human>>(&humans)?)?, text)
        }
    }
}

fn main() {
    if let Err(error) = run() {
        eprintln!("{error:#}");
        std::process::exit(1);
    }
}
