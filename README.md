# HVB Corrector

A small desktop tool for comparing two ways of estimating the effectiveness
of an intervention (a vaccine, a screening program, anything with a clear
"before/after" or "exposed/unexposed" split):

- **Apparent effectiveness** — the naive estimate you get by directly
  comparing event rates between the exposed and unexposed group.
- **Corrected effectiveness** — an estimate that corrects for a specific,
  well-documented bias (the "healthy-vaccinee" / "healthy-user" effect)
  that inflates apparent effectiveness when the exposed group is, on
  average, healthier than the unexposed group for reasons unrelated to the
  intervention itself.

The app does not do statistics for you beyond this one correction — it
takes event counts you already have, computes both estimates for each time
period and subgroup, and lets you compare them on a chart.

**No programming or command line is required for normal use.** The steps
below only ever involve downloading a file and double-clicking it.

## What it shows

After loading a CSV file (see below), you get:

- A **chart** of apparent vs. corrected effectiveness, either
  - as **bars**, one per subgroup, for a period you pick, or
  - as **lines over time**, one point per period in your data.
- A **table** underneath with the raw counts behind every number, so you
  can always see exactly what went into each estimate.
- Filters for **time window**, **sex** (F/M), and **subgroup** (e.g. age
  band), so you can zoom into just the population you care about.
- A warning flag (⚠) on any point based on very few events — those numbers
  are real, but statistically noisy, so they are marked rather than hidden.

## Installing

Go to the **[Releases page](https://github.com/ROGR3/HVB-Corrector-App/releases/latest)**
and download the file for your system. Nothing else is needed.

- **Windows:** download the `.exe` installer (e.g.
  `HVB.Corrector_0.1.1_x64-setup.exe`), double-click it, and follow the
  installer. It adds "HVB Corrector" to your Start menu.
- **Linux:**
  - **Debian/Ubuntu and derivatives:** download the `.deb` file and
    install it (double-click it, or `sudo apt install ./HVB.Corrector_*.deb`).
  - **Any other Linux:** download the `.AppImage` file, mark it as
    executable (right-click → Properties → Permissions → "Allow executing
    file as program", or `chmod +x HVB.Corrector_*.AppImage`), then
    double-click it to run.

Your data is **never uploaded anywhere**. The app runs entirely on your
computer and reads only the file you explicitly choose.

## Trying it without your own data

If you don't have a CSV ready yet, you can try the app on real, aggregated
Czech public-health data that is included with the source: download
[`example_data/sample.csv`](example_data/sample.csv) from this repository
and load it with the app's "Choose CSV file…" button.

## Using your own data

The app reads a single CSV file. Each row is one count for one
combination of **time period, subgroup, sex, and exposure group**.

| Column | Required | Meaning |
| --- | --- | --- |
| `period` | yes | Time bucket, format `YYYY-MM` (e.g. `2021-03`) |
| `stratum` | yes | Any subgroup label you want to compare separately, e.g. an age band like `60-79`. Use the same label consistently; if you don't want subgroups, use one label for everyone. |
| `sex` | yes | `F` or `M` |
| `group` | yes | `exposed` or `unexposed` — which side of the intervention this row's counts belong to |
| `target_events` | yes | Number of the events you're studying (e.g. deaths, infections) in this row |
| `reference_events` | yes | Number of *other* events in the same people, used only as a reference/control count for the correction (e.g. all-cause deaths from an unrelated cause, or a matched control event) |
| `population` | no | Number of people this row's counts come from. Optional — only needed for the apparent estimate; the corrected estimate doesn't use it. |

Example (women, age band 60-79, one period, both exposure groups):

```csv
period,stratum,sex,group,target_events,reference_events,population
2021-03,60-79,F,exposed,12,340,50000
2021-03,60-79,F,unexposed,45,210,30000
```

Practical notes:

- One row per `(period, stratum, sex, group)` combination. If you have
  several raw rows for the same combination, that's fine — the app sums
  them.
- `target_events` and `reference_events` must be whole, non-negative
  numbers.
- If the app rejects your file, it will tell you exactly which row and
  column is the problem — fix that row and re-load the file.

## Running from source (only if you want to modify the app)

This section is for developers, not for normal use. If you just want to
run the app, use the [Releases page](https://github.com/ROGR3/HVB-Corrector-App/releases/latest)
above instead.

Requires a Rust toolchain and, on Linux, the `webkit2gtk` development
packages (see `.github/workflows/build.yml` for the exact package list).

```bash
cd src-tauri
cargo test              # unit tests (CSV validation, aggregation, VE math)
cargo run                # launch the app in dev mode
cargo tauri build        # build an installer for your current OS
```

The frontend (`ui/`) is plain HTML/CSS/JS with a vendored Chart.js — no
Node.js or build step needed. It talks to the Rust backend over Tauri's
IPC (`window.__TAURI__.core.invoke`).

### Project layout

```
src-tauri/       Rust backend: CSV parsing/validation, aggregation, VE math
ui/              Frontend: upload screen + Chart.js dashboard
example_data/    Sample CSV (aggregated Czech open data) and its format docs
docs/            Czech translation of this README
```

### Releases

`.github/workflows/build.yml` runs the test suite on every push/PR, builds
Linux + Windows installers as workflow artifacts, and — when a commit is
tagged `v*` (e.g. `v0.1.1`) — publishes those installers to this
repository's [Releases page](https://github.com/ROGR3/HVB-Corrector-App/releases).

## Česká verze

[docs/README.cs.md](docs/README.cs.md)
