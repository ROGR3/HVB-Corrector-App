# CSV format

One row per `(period, stratum, sex, group)`.

| Column | Required | Meaning |
| --- | --- | --- |
| `period` | yes | Time bucket, e.g. `2021-01` |
| `stratum` | yes | Any grouping label, e.g. age band `60-69` |
| `sex` | yes | `F` or `M` |
| `group` | yes | `exposed` or `unexposed` |
| `target_events` | yes | Events of interest (non-negative integer) |
| `reference_events` | yes | Control / non-target events (non-negative integer) |
| `population` | no | Optional person-counts for that cell |

`sample.csv` is aggregated Czech open data for **women only** (`sex=F`).
