//! Groups validated CSV rows into per-stratum, per-period 2x2 cells.

use std::collections::BTreeMap;

use serde::Serialize;

use crate::csv_ingest::{Group, Row};
use crate::ve_math::Cell;

#[derive(Debug, Serialize)]
pub struct Dataset {
    pub periods: Vec<String>,
    pub strata: Vec<Stratum>,
}

#[derive(Debug, Serialize)]
pub struct Stratum {
    pub name: String,
    /// One cell per period in `Dataset::periods`, in the same order.
    pub cells: Vec<Cell>,
}

pub fn build_dataset(rows: &[Row]) -> Dataset {
    let mut periods: Vec<String> = rows.iter().map(|r| r.period.clone()).collect();
    periods.sort();
    periods.dedup();

    let period_index: BTreeMap<&str, usize> = periods
        .iter()
        .enumerate()
        .map(|(i, p)| (p.as_str(), i))
        .collect();

    let mut stratum_names: Vec<String> = rows.iter().map(|r| r.stratum.clone()).collect();
    stratum_names.sort();
    stratum_names.dedup();

    let mut strata: Vec<Stratum> = stratum_names
        .into_iter()
        .map(|name| Stratum {
            name,
            cells: vec![Cell::default(); periods.len()],
        })
        .collect();
    let stratum_index: BTreeMap<String, usize> = strata
        .iter()
        .enumerate()
        .map(|(i, s)| (s.name.clone(), i))
        .collect();

    for row in rows {
        let stratum = &mut strata[stratum_index[&row.stratum]];
        let cell = &mut stratum.cells[period_index[row.period.as_str()]];
        match row.group {
            Group::Exposed => {
                cell.target_exposed += row.target_events;
                cell.reference_exposed += row.reference_events;
                cell.population_exposed = accumulate(cell.population_exposed, row.population);
            }
            Group::Unexposed => {
                cell.target_unexposed += row.target_events;
                cell.reference_unexposed += row.reference_events;
                cell.population_unexposed = accumulate(cell.population_unexposed, row.population);
            }
        }
    }

    Dataset { periods, strata }
}

/// Sums population across rows that share a (stratum, period, group), but a
/// missing value anywhere in that group makes the total unknown rather than
/// silently treating it as zero.
fn accumulate(existing: Option<u64>, new: Option<u64>) -> Option<u64> {
    match (existing, new) {
        (None, v) => v,
        (Some(e), Some(n)) => Some(e + n),
        (Some(_), None) => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::csv_ingest::Group;

    fn row(
        period: &str,
        stratum: &str,
        group: Group,
        target: u64,
        reference: u64,
        population: Option<u64>,
    ) -> Row {
        Row {
            period: period.to_string(),
            stratum: stratum.to_string(),
            group,
            target_events: target,
            reference_events: reference,
            population,
        }
    }

    #[test]
    fn groups_by_stratum_and_period_and_orders_periods() {
        let rows = vec![
            row("2021-04", "60-79", Group::Exposed, 1, 2, Some(100)),
            row("2021-03", "60-79", Group::Exposed, 3, 4, Some(200)),
            row("2021-03", "60-79", Group::Unexposed, 5, 6, Some(300)),
            row("2021-03", "80+", Group::Exposed, 7, 8, None),
        ];
        let dataset = build_dataset(&rows);
        assert_eq!(dataset.periods, vec!["2021-03", "2021-04"]);
        assert_eq!(dataset.strata.len(), 2);

        let s6079 = dataset.strata.iter().find(|s| s.name == "60-79").unwrap();
        assert_eq!(s6079.cells[0].target_exposed, 3);
        assert_eq!(s6079.cells[0].target_unexposed, 5);
        assert_eq!(s6079.cells[1].target_exposed, 1);
        assert_eq!(s6079.cells[1].population_unexposed, None);
    }

    #[test]
    fn sums_multiple_rows_for_the_same_cell() {
        let rows = vec![
            row("2021-03", "all", Group::Exposed, 1, 1, Some(10)),
            row("2021-03", "all", Group::Exposed, 2, 2, Some(20)),
        ];
        let dataset = build_dataset(&rows);
        let cell = &dataset.strata[0].cells[0];
        assert_eq!(cell.target_exposed, 3);
        assert_eq!(cell.population_exposed, Some(30));
    }
}
