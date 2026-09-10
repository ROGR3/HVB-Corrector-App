# HVB Corrector

Desktop app (Tauri + Rust) that compares an apparent (naive) effectiveness
estimate against a corrected estimate for any before/after intervention where
you have event counts by period and group.

Upload a CSV, pick a time window, sex (F/M), and strata, and see apparent vs.
corrected VE as a bar chart or a time series, with the underlying counts.

## CSV format

See [example_data/README.md](example_data/README.md). Short version: one row
per `(period, stratum, sex, group)`, columns `period,stratum,sex,group,target_events,
reference_events[,population]`. `sex` is `F` or `M`; `group` is `exposed` or
`unexposed`. The sample file is women only (`F`).

A sample file is in [`example_data/sample.csv`](example_data/sample.csv).

## Development

Requires a Rust toolchain and the Linux webview dependencies (`webkit2gtk`,
see `.github/workflows/build.yml` for the exact package list).

```bash
cd src-tauri
cargo test              # unit tests (CSV validation, aggregation, VE math)
cargo run               # launch the app in dev mode
cargo tauri build       # build installers for the current OS
```

The frontend (`ui/`) is plain HTML/CSS/JS with a vendored Chart.js — no
npm build step. It talks to the Rust backend over Tauri's IPC
(`window.__TAURI__.core.invoke`, enabled via `withGlobalTauri` in
`tauri.conf.json`).

## Project layout

```
src-tauri/       Rust backend: CSV parsing/validation, aggregation, VE math
ui/              Frontend: upload screen + Chart.js dashboard
example_data/    Sample CSV (aggregated Czech open data)
```

## CI

`.github/workflows/build.yml` runs `cargo test` on every push/PR, builds
Linux + Windows bundles as workflow artifacts, and (on a `v*` tag) attaches
installers to a draft GitHub Release.
