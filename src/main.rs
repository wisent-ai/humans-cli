use std::path::PathBuf;

use anyhow::Result;
use clap::{Parser, Subcommand};
use humans_cli::{
    Channel, Human, Segment, audience, consolidate_humans, find_duplicates, merge_humans, normalize_human, segment_humans,
    summarize_humans, tag_segment, humans_sheet, humans_from_sheet, diff_humans, humans_vcards, stale_humans,
};

/// An RFC 3339 moment from the command line, refused naming what it read.
fn parse_moment(text: &str) -> Result<chrono::DateTime<chrono::Utc>, String> {
    chrono::DateTime::parse_from_rfc3339(text)
        .map(|moment| moment.with_timezone(&chrono::Utc))
        .map_err(|error| format!("{text:?} is not an RFC 3339 moment such as 2026-01-31T00:00:00Z: {error}"))
}
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
    /// Merge every group of records that share an identity, at any remove,
    /// and print the deduplicated set with what was merged.
    Consolidate {
        /// A JSON array of human records.
        #[arg(long)]
        humans: PathBuf,
    },
    /// Put a tag on, or take it off, every record a segment selects, and
    /// print the whole set with the ids that changed.
    Tag {
        /// A JSON array of human records.
        #[arg(long)]
        humans: PathBuf,
        /// The segment definition JSON file; `{}` selects every record.
        #[arg(long)]
        segment: PathBuf,
        /// The tag to put on.
        #[arg(long, conflicts_with = "remove")]
        add: Option<String>,
        /// The tag to take off.
        #[arg(long)]
        remove: Option<String>,
    },
    /// Print the records as one CSV sheet for a spreadsheet, a row per record.
    Csv {
        /// A JSON array of human records.
        #[arg(long)]
        humans: PathBuf,
    },
    /// Compare two snapshots of a set: the ids added and removed, and per
    /// id in both, every field whose normalised value changed.
    Diff {
        /// The earlier JSON array of human records.
        #[arg(long)]
        before: PathBuf,
        /// The later JSON array of human records.
        #[arg(long)]
        after: PathBuf,
    },
    /// The people who may be reached on a channel: consented to it and with
    /// an address for it, each address once, and who was left out and why.
    Audience {
        /// A JSON array of human records.
        #[arg(long)]
        humans: PathBuf,
        /// The channel: email or sms.
        #[arg(long, value_enum)]
        channel: Channel,
        /// Take only verified addresses.
        #[arg(long)]
        verified_only: bool,
    },
    /// The records untouched since a moment: not updated (or, never updated,
    /// created) since --before, the longest untouched first.
    Stale {
        /// A JSON array of human records.
        #[arg(long)]
        humans: PathBuf,
        /// The moment, RFC 3339: a record last touched before it is stale.
        #[arg(long, value_parser = parse_moment)]
        before: chrono::DateTime<chrono::Utc>,
    },
    /// Print the records as vCards for an address book, one per person.
    Vcard {
        /// A JSON array of human records.
        #[arg(long)]
        humans: PathBuf,
    },
    /// Read a CSV sheet `humans csv` wrote, edited or not, back into
    /// records, normalised, in row order.
    FromCsv {
        /// The CSV sheet.
        #[arg(long)]
        csv: PathBuf,
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
        Command::Consolidate { humans } => {
            output(&consolidate_humans(read_json::<Vec<Human>>(&humans)?)?, text)
        }
        Command::Tag { humans, segment, add, remove } => {
            let (tag, added) = match (add, remove) {
                (Some(tag), None) => (tag, true),
                (None, Some(tag)) => (tag, false),
                _ => anyhow::bail!("tag needs exactly one of --add <tag> or --remove <tag>"),
            };
            output(&tag_segment(read_json::<Vec<Human>>(&humans)?, &read_json::<Segment>(&segment)?, &tag, added)?, text)
        }
        Command::Csv { humans } => {
            // The sheet is the answer itself, not a JSON document about it.
            print!("{}", humans_sheet(read_json::<Vec<Human>>(&humans)?)?);
            Ok(())
        }
        Command::Diff { before, after } => {
            output(&diff_humans(read_json::<Vec<Human>>(&before)?, read_json::<Vec<Human>>(&after)?)?, text)
        }
        Command::Audience { humans, channel, verified_only } => {
            output(&audience(read_json::<Vec<Human>>(&humans)?, channel, verified_only)?, text)
        }
        Command::Stale { humans, before } => {
            output(&stale_humans(read_json::<Vec<Human>>(&humans)?, before, chrono::Utc::now())?, text)
        }
        Command::Vcard { humans } => {
            let cards = humans_vcards(read_json::<Vec<Human>>(&humans)?)?;
            // The cards are the answer itself; what was left out, and why, goes to stderr.
            for id in &cards.skipped {
                eprintln!("skipped {id}: no name and no email or phone to put on a card");
            }
            print!("{}", cards.vcf);
            Ok(())
        }
        Command::FromCsv { csv } => {
            let sheet = std::fs::read_to_string(&csv).map_err(|error| anyhow::anyhow!("cannot read the sheet {}: {error}", csv.display()))?;
            output(&humans_from_sheet(&sheet)?, text)
        }
    }
}

fn main() -> std::process::ExitCode {
    match run() {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error:#}");
            std::process::ExitCode::FAILURE
        }
    }
}
