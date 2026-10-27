<!-- wisent-banner:start -->
<p align="center">
  <img src="assets/readme-banner.webp" alt="humans-cli by Wisent" width="100%">
</p>
<!-- wisent-banner:end -->

<!-- wisent-readme-signals:start -->
[![Source](https://img.shields.io/badge/GitHub-Source-181717?logo=github)](https://github.com/wisent-ai/humans-cli) [![Issues](https://img.shields.io/badge/GitHub-Issues-181717?logo=github)](https://github.com/wisent-ai/humans-cli/issues) [![Wisent](https://img.shields.io/badge/Wisent-Website-0B0B0B)](https://wisent.com) [![Discord](https://img.shields.io/badge/Discord-Join-5865F2?logo=discord&logoColor=white)](https://discord.gg/qRjpkthq54) [![LinkedIn](https://img.shields.io/badge/LinkedIn-Follow-0A66C2?logo=linkedin&logoColor=white)](https://www.linkedin.com/company/wisent-ai/) [![X](https://img.shields.io/badge/X-Follow-000000?logo=x&logoColor=white)](https://x.com/wisentai) [![Enterprise](https://img.shields.io/badge/Enterprise-Book%20a%20call-0B0B0B?logo=calendly)](https://calendly.com/lbartoszcze)
<!-- wisent-readme-signals:end -->

# humans-cli

Local-first normalization, identity resolution and segmentation of human
records. Commands read local JSON and contact no remote system. Every
command prints deterministic JSON, or with `--text` one `path: value` line
per field from the same document.

| Command | What it prints |
|---|---|
| `humans normalize --human <human.json>` | the record in canonical form |
| `humans duplicates --humans <humans.json>` | groups of records that share an identity |
| `humans merge --humans <humans.json> [--target-id <id>]` | one merged record (the first is the base; `--target-id` renames it) and the fields that conflicted |
| `humans segment --humans <humans.json> --segment <segment.json>` | the records the segment selects |
| `humans summarize --humans <humans.json>` | counts of records, identities, duplicate groups, identity types, tags and consent |
| `humans consolidate --humans <humans.json>` | the set with every group of records sharing an identity, at any remove (A and B share an email, B and C a phone), merged as `merge` merges with the record first in the file as the base: `humans`, the deduplicated set in input order, and `merged`, per person the `id`, the `sourceIds` merged into it and the fields that `conflicts` |
| `humans tag --humans <humans.json> --segment <segment.json> (--add <tag> \| --remove <tag>)` | the whole set with the tag put on, or taken off, every record the segment selects (`{}` selects all), tags lowercased as `normalize` keeps them: `tag`, `added`, `changedIds`, the records whose tags changed, and `humans`; refused without exactly one of `--add` and `--remove`, and for an empty tag |
| `humans csv --humans <humans.json>` | not JSON: the records, normalised, as one CSV sheet (RFC 4180, every cell quoted), a row per record in input order; the columns are the record's own fields as its JSON names them, nested ones joined with a dot (`consent.email`, `attributes.plan`), the union over all records by name; a list of plain values (tags) is written separated by `;`, a list of objects (identities) as compact JSON. `humans csv --humans people.json > people.csv` |
| `humans from-csv --csv <humans.csv>` | the inverse of `csv`: a sheet it wrote, edited in a spreadsheet or not, back into records, normalised, in row order. Lists and nested objects are told apart by the record's own shape: a list cell starting with `[` is JSON (identities), else values separated by `;` (tags); a nested cell (`consent.email`, `attributes.plan`) that reads as a JSON boolean or number is that value, else text; an empty cell is a field the row does not set. Refused, naming the line: a row with more or fewer cells than the header, a list that is not JSON, a row that is not a record, an unclosed quote |
| `humans diff --before <earlier.json> --after <later.json>` | what changed between two snapshots of a set, both normalised first: `added` and `removed`, the ids only one side holds; `changed`, per id in both, every field whose value differs, nested ones named with a dot (`consent.email`), as `{before, after}`, `null` on the side that lacks it; and `unchanged`, how many are the same. Refused when one snapshot holds two records with one id, naming the id |
| `humans audience --humans <humans.json> --channel email\|sms [--verified-only]` | who may be reached on the channel, every record normalised first: `recipients`, in input order, each `id`, `displayName`, the `address` and whether it is `verified` (a verified `email` identity for email, `phone` for sms, taken before an unverified one; with `--verified-only` only verified ones), each address once; and the ids left out, `withoutConsent` (no `consent.email` or `consent.sms`), `withoutAddress` and `sharedAddress` (an address someone earlier in the file already holds). A channel other than email or sms exits 2 |
| `humans stale --humans <humans.json> --before <RFC 3339>` | the records untouched since `--before`, normalised first: `stale`, the longest untouched first, each `id`, `displayName`, `lastTouched` (its `updatedAt`, or `createdAt` when never updated), `daysUntouched` to now, and `consentUpdatedAt` (`null` when the record never says); `undated`, the ids stating neither date, listed apart rather than guessed; and how many are `fresh`. A `--before` that is not RFC 3339 exits 2; a record date that is not exits 1 naming the record |
| `humans vcard --humans <humans.json>` | not JSON: the records, normalised, as one `.vcf` file of vCards (RFC 6350, version 4.0), in input order: `UID` the record's id, `FN` its display name (else given and family name, else the id when it has an address), `N`, every `email` identity as `EMAIL` and every `phone` as `TEL` (a verified one marked `PREF=1`), and the tags as `CATEGORIES`. A record with neither a name nor an email or phone is skipped and named on stderr. `humans vcard --humans people.json > people.vcf` |

`--help` works on the program and on every command. Exit 2: the invocation
is wrong (unknown command or flag, missing argument); exit 1: an input file
could not be read, is not valid JSON, or was refused, with the reason on
stderr.
