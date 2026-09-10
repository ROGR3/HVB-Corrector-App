mod aggregate;
mod csv_ingest;
mod ve_math;

use std::path::PathBuf;

use serde::Serialize;
use tauri_plugin_dialog::DialogExt;

use aggregate::Dataset;
use csv_ingest::IngestError;

#[derive(Debug, Serialize)]
#[serde(tag = "kind", content = "detail")]
enum LoadOutcome {
    Dataset(Dataset),
    MissingColumns(String),
    RowErrors(Vec<String>),
    Io(String),
}

impl From<IngestError> for LoadOutcome {
    fn from(err: IngestError) -> Self {
        match err {
            IngestError::MissingColumns(cols) => LoadOutcome::MissingColumns(cols),
            IngestError::InvalidRows(rows) => {
                LoadOutcome::RowErrors(rows.iter().map(ToString::to_string).collect())
            }
            IngestError::Io(e) => LoadOutcome::Io(e.to_string()),
            IngestError::Header(e) => LoadOutcome::Io(e.to_string()),
        }
    }
}

fn load_path(path: PathBuf) -> LoadOutcome {
    match csv_ingest::load(&path) {
        Ok(rows) => LoadOutcome::Dataset(aggregate::build_dataset(&rows)),
        Err(err) => err.into(),
    }
}

#[tauri::command]
fn load_dataset_from_path(path: String) -> LoadOutcome {
    load_path(PathBuf::from(path))
}

#[tauri::command]
fn load_dataset_from_contents(contents: String) -> LoadOutcome {
    match csv_ingest::load_from_str(&contents) {
        Ok(rows) => LoadOutcome::Dataset(aggregate::build_dataset(&rows)),
        Err(err) => err.into(),
    }
}

#[tauri::command]
async fn pick_and_load_csv(app: tauri::AppHandle) -> Option<LoadOutcome> {
    let file = app
        .dialog()
        .file()
        .add_filter("CSV", &["csv"])
        .blocking_pick_file()?;
    let path = file.into_path().ok()?;
    Some(load_path(path))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            load_dataset_from_path,
            load_dataset_from_contents,
            pick_and_load_csv
        ])
        .run(tauri::generate_context!())
        .expect("error while running the HVB Corrector app");
}
