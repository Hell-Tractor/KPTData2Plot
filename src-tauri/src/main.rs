#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{cmp::min, fs::File, io::Write};

use base64::Engine;
use futures::future::try_join_all;
use tauri::InvokeError;

#[derive(thiserror::Error, Debug)]
enum Error {
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error(transparent)]
    CsvError(#[from] csv::Error),
    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),
    #[error(transparent)]
    Base64Error(#[from] base64::DecodeError),
}

impl Into<InvokeError> for Error {
    fn into(self) -> InvokeError {
        InvokeError::from(self.to_string())
    }
}

type Result<T> = std::result::Result<T, Error>;

#[tauri::command]
async fn get_excel_header(path: &str) -> Result<Vec<String>> {
    let file = File::open(path)?;
    let mut reader = csv::Reader::from_reader(file);
    Ok(reader.headers()?.iter().map(|s| s.to_string()).collect())
}

#[derive(serde::Serialize, Clone, Debug)]
struct Data {
    avg: f64,
    se: f64,
}

#[derive(serde::Deserialize, Debug, Clone)]
struct DetailedData {
    presses: Vec<u32>,
    #[allow(dead_code)]
    #[serde(rename = "Xs")]
    xs: Vec<u32>,
    #[allow(dead_code)]
    #[serde(rename = "Ls")]
    ls: Vec<u32>,
}

#[derive(serde::Deserialize, Debug, Clone)]
struct OriginData {
    #[allow(dead_code)]
    #[serde(alias = "score")]    // for compatibility
    sum_presses: u32,
    #[allow(dead_code)]
    #[serde(default, rename = "sum_Ls")]
    sum_ls: u32,
    #[allow(dead_code)]
    #[serde(default, rename = "sum_Xs")]
    sum_xs: u32,
    #[serde(alias = "data")]
    detailed: DetailedData,

    // for compatibility
    #[allow(dead_code)]
    #[serde(default)]
    seconds: u32,
    #[allow(dead_code)]
    #[serde(default, rename="colorId")]
    color_id: u32,
}

#[derive(serde::Deserialize, Debug)]
struct CurveConfig {
    column_id: usize,
    unit: usize,
    max_length: usize,
}

async fn parse_column_data(mut origin_data: Vec<OriginData>, curve: &CurveConfig) -> Result<Vec<Data>> {
    let n = origin_data.len();
    if n < 2 {
        return Err(Error::InvalidParameter(format!("Valid data is not enough for column {}", n)));
    }

    let seconds = if curve.max_length == 0 {
        origin_data[0].detailed.presses.len()
    } else {
        min(curve.max_length, origin_data[0].detailed.presses.len())
    };

    if seconds == 0 {
        return Err(Error::InvalidParameter(format!("Data length is 0 for column {}", curve.column_id)));
    }

    let length = (seconds + curve.unit - 1) / curve.unit;

    // sum each unit data
    let zipped_data: Vec<Vec<u32>> = origin_data.iter_mut().map(|data| {
        data.detailed.presses.truncate(seconds);
        data.detailed.presses.iter().enumerate().fold(vec![0; length], |mut acc, (i, press)| {
            acc[i / curve.unit] += *press;
            acc
        })
    }).collect();

    let mut result = vec![Data { avg: 0.0, se: 0.0 }; length];
    for data in &zipped_data {
        data.iter().enumerate().for_each(|(i, press)| {
            result[i].avg += *press as f64 / n as f64;
        });
    }
    for data in &zipped_data {
        data.iter().enumerate().for_each(|(i, press)| {
            result[i].se += (*press as f64 - result[i].avg).powi(2) / (n - 1) as f64;
        });
    }
    result.iter_mut().for_each(|data| {
        data.se = (data.se / n as f64).sqrt();
    });
    Ok(result)
}

#[tauri::command]
async fn get_data(path: &str, curve_configs: Vec<CurveConfig>) -> Result<Vec<Vec<Data>>> {
    if curve_configs.len() == 0 {
        return Err(Error::InvalidParameter("No curve is required".to_owned()));
    }
    let file = File::open(path)?;
    let mut reader = csv::Reader::from_reader(file);

    let mut records: Vec<Vec<OriginData>> = vec![Vec::new(); curve_configs.len()];
    for record in reader.records() {
        let record = record?;
        for (idx, id) in curve_configs.iter().enumerate() {
            let Ok(data) = serde_json::from_str(&record.get(id.column_id).ok_or(Error::InvalidParameter(format!("column id {}", id.column_id)))?) else {
                continue;
            };
            records[idx].push(data);
        }
    }
    try_join_all(records.into_iter()
        .zip(curve_configs.iter())
        .map(|(data, config)| parse_column_data(data, config))
    ).await
}

#[tauri::command]
async fn save_image(path: &str, image: &str) -> Result<()> {
    let base64 = image.split(",").nth(1).ok_or(Error::InvalidParameter("Invalid image".to_owned()))?;
    let decode_data = base64::engine::general_purpose::STANDARD.decode(base64)?;
    let mut file = File::create(path)?;
    file.write_all(&decode_data)?;
    Ok(())
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![get_excel_header, get_data, save_image])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
