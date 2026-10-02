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

`--help` works on the program and on every command. Exit 2: the invocation
is wrong (unknown command or flag, missing argument); exit 1: an input file
could not be read, is not valid JSON, or was refused, with the reason on
stderr.
