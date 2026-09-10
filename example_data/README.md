# CSV format

One row per `(period, stratum, group)`.

| Column | Required | Meaning |
| --- | --- | --- |
| `period` | yes | Time bucket, e.g. `2021-01` |
| `stratum` | yes | Any grouping label, e.g. age band `60-69` |
| `group` | yes | `exposed` or `unexposed` |
| `target_events` | yes | Events of interest (non-negative integer) |
| `reference_events` | yes | Control / non-target events (non-negative integer) |
| `population` | no | Optional person-counts for that cell |

`sample.csv` is aggregated Czech open data in this layout.
