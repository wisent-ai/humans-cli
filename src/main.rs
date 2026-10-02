use std::{fs, path::PathBuf};

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use humans_cli::{
    Human, Segment, find_duplicates, merge_humans, normalize_human, segment_humans,
    summarize_humans,
};
use serde::{Serialize, de::DeserializeOwned};
use serde_json::Value;

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

fn read_json<T: DeserializeOwned>(path: &PathBuf) -> Result<T> {
    let input = fs::read(path).with_context(|| format!("failed to read {}", path.display()))?;
    serde_json::from_slice(&input).with_context(|| format!("invalid JSON in {}", path.display()))
}

/// JSON, or with `--text` the same document as one `path: value` line per
/// field (cli.md rule 13).
fn output<T: Serialize>(value: &T, text: bool) -> Result<()> {
    let document = serde_json::to_value(value)?;
    if !text {
        println!("{}", serde_json::to_string_pretty(&document)?);
        return Ok(());
    }
    let mut lines = Vec::new();
    text_lines(&document, String::new(), &mut lines);
    println!("{}", lines.join("\n"));
    Ok(())
}

fn text_lines(node: &Value, path: String, lines: &mut Vec<String>) {
    let child = |key: &str| if path.is_empty() { key.to_owned() } else { format!("{path}.{key}") };
    match node {
        Value::Array(items) if !items.is_empty() => {
            for (index, item) in items.iter().enumerate() {
                text_lines(item, format!("{path}[{index}]"), lines);
            }
        }
        Value::Object(fields) if !fields.is_empty() => {
            for (key, item) in fields {
                text_lines(item, child(key), lines);
            }
        }
        Value::String(value) => lines.push(format!("{path}: {value}")),
        Value::Null | Value::Array(_) | Value::Object(_) => lines.push(format!("{path}: -")),
        other => lines.push(format!("{path}: {other}")),
    }
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
