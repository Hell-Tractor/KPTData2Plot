#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{fs::File, io::Write};

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

async fn parse_column_data(origin_data: &Vec<OriginData>, column_id: usize) -> Result<Vec<Data>> {
    let n = origin_data.len();
    if n < 2 {
        return Err(Error::InvalidParameter(format!("Valid data is not enough for column {}", n)));
    }
    let mut result = vec![Data { avg: 0.0, se: 0.0 }; origin_data[0].detailed.presses.len()];
    for data in origin_data {
        if data.detailed.presses.len() != result.len() {
            return Err(Error::InvalidParameter(format!("Data length is not equal for column {}", column_id)));
        }
        data.detailed.presses.iter().enumerate().for_each(|(i, press)| {
            result[i].avg += *press as f64 / n as f64;
        });
    }
    for data in origin_data {
        data.detailed.presses.iter().enumerate().for_each(|(i, press)| {
            result[i].se += (*press as f64 - result[i].avg).powi(2) / (n - 1) as f64;
        });
    }
    result.iter_mut().for_each(|data| {
        data.se = (data.se / n as f64).sqrt();
    });
    Ok(result)
}

#[tauri::command]
async fn get_data(path: &str, column_ids: Vec<usize>) -> Result<Vec<Vec<Data>>> {
    if column_ids.len() == 0 {
        return Err(Error::InvalidParameter("column_ids is empty".to_owned()));
    }
    let file = File::open(path)?;
    let mut reader = csv::Reader::from_reader(file);

    let mut records: Vec<Vec<OriginData>> = vec![Vec::new(); column_ids.len()];
    for record in reader.records() {
        let record = record?;
        for (idx, id) in column_ids.iter().enumerate() {
            let Ok(data) = serde_json::from_str(&record.get(*id).ok_or(Error::InvalidParameter(format!("column id {}", id)))?) else {
                continue;
            };
            records[idx].push(data);
        }
    }
    try_join_all(records.iter()
        .zip(column_ids.iter())
        .map(|(data, id)| parse_column_data(data, *id))
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
