#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{fs::File, io::Write};

use base64::Engine;
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

#[derive(serde::Deserialize, Debug)]
struct DetailedData {
    presses: Vec<u32>,
    #[allow(dead_code)]
    #[serde(rename = "Xs")]
    xs: Vec<u32>,
    #[allow(dead_code)]
    #[serde(rename = "Ls")]
    ls: Vec<u32>,
}

#[derive(serde::Deserialize, Debug)]
struct OriginData {
    #[allow(dead_code)]
    sum_presses: u32,
    #[allow(dead_code)]
    #[serde(rename = "sum_Ls")]
    sum_ls: u32,
    #[allow(dead_code)]
    #[serde(rename = "sum_Xs")]
    sum_xs: u32,
    detailed: DetailedData,
}

#[tauri::command]
async fn get_data(path: &str, column_ids: Vec<usize>) -> Result<Vec<Vec<Data>>> {
    if column_ids.len() == 0 {
        return Err(Error::InvalidParameter("column_ids is empty".to_owned()));
    }
    let file = File::open(path)?;
    let mut reader = csv::Reader::from_reader(file);
    // let mut data = Vec::new().resize(column_ids.len(), Vec::<Data>::new());
    let records = reader.records().filter_map(|record| {
        let record = record.ok()?;
        let datas = column_ids.iter().filter_map(|id| serde_json::from_str(&record.get(*id)?).ok()).collect::<Vec<OriginData>>();
        // println!("{:?}", serde_json::from_str::<OriginData>(&record.get(column_ids[0])?));
        if datas.len() != column_ids.len() {
            return None;
        }
        Some(datas)
    }).collect::<Vec<_>>();
    let n = records.len();
    if n < 2 {
        return Err(Error::InvalidParameter("Valid data is not enough".to_owned()));
    }

    let mut result = records[0].iter().map(|data| vec![Data { avg: 0.0, se: 0.0 }; data.detailed.presses.len()]).collect::<Vec<_>>();
    records.iter().for_each(|record| {
        record.iter().enumerate().for_each(|(i, data)| {
            data.detailed.presses.iter().enumerate().for_each(|(j, press)| {
                result[i][j].avg += *press as f64 / n as f64;
            });
        });
    });

    let sqrt_n = (n as f64).sqrt();
    records.iter().for_each(|record| {
        record.iter().enumerate().for_each(|(i, data)| {
            data.detailed.presses.iter().enumerate().for_each(|(j, press)| {
                result[i][j].se += (*press as f64 - result[i][j].avg).powi(2) / (n - 1) as f64;
            });
        });
    });
    records.iter().for_each(|record| {
        record.iter().enumerate().for_each(|(i, data)| {
            data.detailed.presses.iter().enumerate().for_each(|(j, _)| {
                result[i][j].se = result[i][j].se.sqrt() / sqrt_n;
            });
        });
    });

    Ok(result)
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
