use std::path::Path;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
pub enum Sex {
    F,
    M,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Group {
    Exposed,
    Unexposed,
}

#[derive(Debug, Clone)]
pub struct Row {
    pub period: String,
    pub stratum: String,
    pub sex: Sex,
    pub group: Group,
    pub target_events: u64,
    pub reference_events: u64,
    pub population: Option<u64>,
}

#[derive(Debug, thiserror::Error)]
pub enum IngestError {
    #[error("could not open the file: {0}")]
    Io(#[from] std::io::Error),
    #[error("could not read the CSV header: {0}")]
    Header(csv::Error),
    #[error(
        "missing required column(s): {0}. Expected: period,stratum,sex,group,target_events,reference_events[,population]"
    )]
    MissingColumns(String),
    #[error("{} row(s) failed validation", .0.len())]
    InvalidRows(Vec<RowError>),
}

#[derive(Debug, Clone)]
pub struct RowError {
    pub row: usize,
    pub message: String,
}

impl std::fmt::Display for RowError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "row {}: {}", self.row, self.message)
    }
}

#[derive(Debug, Deserialize)]
struct RawRow {
    period: String,
    stratum: String,
    sex: String,
    group: String,
    target_events: String,
    reference_events: String,
    #[serde(default)]
    population: String,
}

const REQUIRED_COLUMNS: [&str; 6] = [
    "period",
    "stratum",
    "sex",
    "group",
    "target_events",
    "reference_events",
];

pub fn load(path: &Path) -> Result<Vec<Row>, IngestError> {
    let reader = csv::ReaderBuilder::new()
        .has_headers(true)
        .from_path(path)
        .map_err(IngestError::Header)?;
    load_from_reader(reader)
}

pub fn load_from_str(contents: &str) -> Result<Vec<Row>, IngestError> {
    let reader = csv::ReaderBuilder::new()
        .has_headers(true)
        .from_reader(contents.as_bytes());
    load_from_reader(reader)
}

fn load_from_reader<R: std::io::Read>(mut reader: csv::Reader<R>) -> Result<Vec<Row>, IngestError> {
    let headers: Vec<String> = reader
        .headers()
        .map_err(IngestError::Header)?
        .iter()
        .map(str::to_string)
        .collect();
    let missing: Vec<&str> = REQUIRED_COLUMNS
        .iter()
        .filter(|col| !headers.iter().any(|h| h == *col))
        .copied()
        .collect();
    if !missing.is_empty() {
        return Err(IngestError::MissingColumns(missing.join(", ")));
    }

    let mut rows = Vec::new();
    let mut errors = Vec::new();

    for (i, result) in reader.deserialize::<RawRow>().enumerate() {
        let row_num = i + 2;
        let raw = match result {
            Ok(raw) => raw,
            Err(e) => {
                errors.push(RowError {
                    row: row_num,
                    message: format!("could not parse row: {e}"),
                });
                continue;
            }
        };
        match validate_row(&raw) {
            Ok(row) => rows.push(row),
            Err(messages) => {
                for message in messages {
                    errors.push(RowError {
                        row: row_num,
                        message,
                    });
                }
            }
        }
    }

    if !errors.is_empty() {
        return Err(IngestError::InvalidRows(errors));
    }
    Ok(rows)
}

fn validate_row(raw: &RawRow) -> Result<Row, Vec<String>> {
    let mut errors = Vec::new();

    if !is_valid_period(&raw.period) {
        errors.push(format!("period '{}' is not in YYYY-MM format", raw.period));
    }
    if raw.stratum.trim().is_empty() {
        errors.push("stratum is empty".to_string());
    }
    let sex = parse_sex(&raw.sex).or_else(|| {
        errors.push(format!("sex '{}' must be exactly 'F' or 'M'", raw.sex.trim()));
        None
    });
    let group = match raw.group.trim() {
        "exposed" => Some(Group::Exposed),
        "unexposed" => Some(Group::Unexposed),
        other => {
            errors.push(format!(
                "group '{other}' must be exactly 'exposed' or 'unexposed'"
            ));
            None
        }
    };
    let target_events = parse_count(&raw.target_events, "target_events", &mut errors);
    let reference_events = parse_count(&raw.reference_events, "reference_events", &mut errors);
    let population = parse_optional_count(&raw.population, "population", &mut errors);

    if !errors.is_empty() {
        return Err(errors);
    }

    Ok(Row {
        period: raw.period.clone(),
        stratum: raw.stratum.trim().to_string(),
        sex: sex.expect("validated above"),
        group: group.expect("validated above"),
        target_events: target_events.expect("validated above"),
        reference_events: reference_events.expect("validated above"),
        population,
    })
}

fn parse_sex(raw: &str) -> Option<Sex> {
    match raw.trim().to_ascii_uppercase().as_str() {
        "F" => Some(Sex::F),
        "M" => Some(Sex::M),
        _ => None,
    }
}

fn is_valid_period(period: &str) -> bool {
    let Some((year, month)) = period.split_once('-') else {
        return false;
    };
    if year.len() != 4 || month.len() != 2 {
        return false;
    }
    let Ok(month) = month.parse::<u32>() else {
        return false;
    };
    year.chars().all(|c| c.is_ascii_digit()) && (1..=12).contains(&month)
}

fn parse_count(raw: &str, field: &str, errors: &mut Vec<String>) -> Option<u64> {
    match raw.trim().parse::<i64>() {
        Ok(v) if v >= 0 => Some(v as u64),
        Ok(_) => {
            errors.push(format!("{field} '{raw}' must not be negative"));
            None
        }
        Err(_) => {
            errors.push(format!("{field} '{raw}' is not a whole number"));
            None
        }
    }
}

fn parse_optional_count(raw: &str, field: &str, errors: &mut Vec<String>) -> Option<u64> {
    if raw.trim().is_empty() {
        return None;
    }
    parse_count(raw, field, errors)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    const SAMPLE_CSV: &str = include_str!("../../example_data/sample.csv");

    #[test]
    fn the_shipped_sample_csv_is_valid() {
        let rows = load_from_str(SAMPLE_CSV).unwrap();
        assert!(!rows.is_empty());
        assert!(rows.iter().all(|r| r.sex == Sex::F));
    }

    fn write_csv(contents: &str) -> tempfile::NamedTempFile {
        let mut file = tempfile::NamedTempFile::new().unwrap();
        file.write_all(contents.as_bytes()).unwrap();
        file
    }

    #[test]
    fn accepts_a_well_formed_file() {
        let file = write_csv(
            "period,stratum,sex,group,target_events,reference_events,population\n\
             2021-03,60-79,F,exposed,12,340,50000\n\
             2021-03,60-79,F,unexposed,45,210,30000\n",
        );
        let rows = load(file.path()).unwrap();
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].group, Group::Exposed);
        assert_eq!(rows[0].sex, Sex::F);
        assert_eq!(rows[0].population, Some(50000));
    }

    #[test]
    fn population_column_is_optional() {
        let file = write_csv(
            "period,stratum,sex,group,target_events,reference_events\n\
             2021-03,60-79,F,exposed,12,340\n",
        );
        let rows = load(file.path()).unwrap();
        assert_eq!(rows[0].population, None);
    }

    #[test]
    fn rejects_bad_group_and_negative_counts_with_row_numbers() {
        let file = write_csv(
            "period,stratum,sex,group,target_events,reference_events\n\
             2021-03,60-79,F,vaccinated,12,340\n\
             2021-04,60-79,F,exposed,-1,340\n",
        );
        let err = load(file.path()).unwrap_err();
        let IngestError::InvalidRows(rows) = err else {
            panic!("expected InvalidRows")
        };
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].row, 2);
        assert!(rows[0].message.contains("exposed"));
        assert_eq!(rows[1].row, 3);
        assert!(rows[1].message.contains("negative"));
    }

    #[test]
    fn rejects_missing_columns() {
        let file = write_csv("period,stratum,group\n2021-03,60-79,exposed\n");
        let err = load(file.path()).unwrap_err();
        assert!(matches!(err, IngestError::MissingColumns(_)));
    }

    #[test]
    fn rejects_sex_that_is_not_f_or_m() {
        let file = write_csv(
            "period,stratum,sex,group,target_events,reference_events\n\
             2021-03,60-79,X,exposed,12,340\n",
        );
        let err = load(file.path()).unwrap_err();
        let IngestError::InvalidRows(rows) = err else {
            panic!("expected InvalidRows")
        };
        assert_eq!(rows[0].row, 2);
        assert!(rows[0].message.contains("F"));
    }

    #[test]
    fn accepts_lowercase_sex() {
        let file = write_csv(
            "period,stratum,sex,group,target_events,reference_events\n\
             2021-03,60-79,m,exposed,12,340\n",
        );
        let rows = load(file.path()).unwrap();
        assert_eq!(rows[0].sex, Sex::M);
    }
}
