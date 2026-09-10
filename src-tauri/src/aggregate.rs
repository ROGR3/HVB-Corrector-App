use std::collections::BTreeMap;

use serde::Serialize;

use crate::csv_ingest::{Group, Row, Sex};
use crate::ve_math::Cell;

#[derive(Debug, Serialize)]
pub struct Dataset {
    pub periods: Vec<String>,
    pub sexes: Vec<Sex>,
    pub strata: Vec<Stratum>,
}

#[derive(Debug, Serialize)]
pub struct Stratum {
    pub name: String,
    pub by_sex: BTreeMap<Sex, Vec<Cell>>,
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

    let mut sexes: Vec<Sex> = rows.iter().map(|r| r.sex).collect();
    sexes.sort();
    sexes.dedup();

    let mut stratum_names: Vec<String> = rows.iter().map(|r| r.stratum.clone()).collect();
    stratum_names.sort();
    stratum_names.dedup();

    let mut strata: Vec<Stratum> = stratum_names
        .into_iter()
        .map(|name| Stratum {
            name,
            by_sex: BTreeMap::new(),
        })
        .collect();
    let stratum_index: BTreeMap<String, usize> = strata
        .iter()
        .enumerate()
        .map(|(i, s)| (s.name.clone(), i))
        .collect();

    for row in rows {
        let stratum = &mut strata[stratum_index[&row.stratum]];
        let cells = stratum
            .by_sex
            .entry(row.sex)
            .or_insert_with(|| vec![Cell::default(); periods.len()]);
        let cell = &mut cells[period_index[row.period.as_str()]];
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

    Dataset {
        periods,
        sexes,
        strata,
    }
}

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
        sex: Sex,
        group: Group,
        target: u64,
        reference: u64,
        population: Option<u64>,
    ) -> Row {
        Row {
            period: period.to_string(),
            stratum: stratum.to_string(),
            sex,
            group,
            target_events: target,
            reference_events: reference,
            population,
        }
    }

    #[test]
    fn groups_by_stratum_and_period_and_orders_periods() {
        let rows = vec![
            row("2021-04", "60-79", Sex::F, Group::Exposed, 1, 2, Some(100)),
            row("2021-03", "60-79", Sex::F, Group::Exposed, 3, 4, Some(200)),
            row("2021-03", "60-79", Sex::F, Group::Unexposed, 5, 6, Some(300)),
            row("2021-03", "80+", Sex::F, Group::Exposed, 7, 8, None),
        ];
        let dataset = build_dataset(&rows);
        assert_eq!(dataset.periods, vec!["2021-03", "2021-04"]);
        assert_eq!(dataset.sexes, vec![Sex::F]);
        assert_eq!(dataset.strata.len(), 2);

        let s6079 = dataset.strata.iter().find(|s| s.name == "60-79").unwrap();
        let cells = &s6079.by_sex[&Sex::F];
        assert_eq!(cells[0].target_exposed, 3);
        assert_eq!(cells[0].target_unexposed, 5);
        assert_eq!(cells[1].target_exposed, 1);
        assert_eq!(cells[1].population_unexposed, None);
    }

    #[test]
    fn sums_multiple_rows_for_the_same_cell() {
        let rows = vec![
            row("2021-03", "all", Sex::F, Group::Exposed, 1, 1, Some(10)),
            row("2021-03", "all", Sex::F, Group::Exposed, 2, 2, Some(20)),
        ];
        let dataset = build_dataset(&rows);
        let cell = &dataset.strata[0].by_sex[&Sex::F][0];
        assert_eq!(cell.target_exposed, 3);
        assert_eq!(cell.population_exposed, Some(30));
    }

    #[test]
    fn keeps_female_and_male_cells_separate() {
        let rows = vec![
            row("2021-03", "60-79", Sex::F, Group::Exposed, 1, 2, Some(10)),
            row("2021-03", "60-79", Sex::M, Group::Exposed, 8, 9, Some(40)),
        ];
        let dataset = build_dataset(&rows);
        assert_eq!(dataset.sexes, vec![Sex::F, Sex::M]);
        let stratum = &dataset.strata[0];
        assert_eq!(stratum.by_sex[&Sex::F][0].target_exposed, 1);
        assert_eq!(stratum.by_sex[&Sex::M][0].target_exposed, 8);
    }

    #[test]
    fn serializes_sex_keys_as_f_and_m() {
        let rows = vec![row("2021-03", "all", Sex::F, Group::Exposed, 1, 1, None)];
        let json = serde_json::to_value(build_dataset(&rows)).unwrap();
        assert_eq!(json["sexes"], serde_json::json!(["F"]));
        assert!(json["strata"][0]["by_sex"].get("F").is_some());
    }
}
