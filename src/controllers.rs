use axum::{
    body::Body,
    extract::Multipart,
    http::StatusCode,
    response::{self, IntoResponse},
    routing::{get, method_routing::MethodRouter, post},
};

use futures::future;
use std::sync::{Arc, Mutex, Once};
use tokio::fs::File;
use tokio::io::{AsyncWriteExt, BufWriter};

use once_cell::sync::Lazy;
use uuid::Uuid;

use core::convert::Infallible;
use std::collections::HashMap;

use crate::process::process;
use crate::whisper_pool::WhisperPool;

static WHISPER_POOL: Lazy<WhisperPool> = Lazy::new(|| WhisperPool::new_pool());

pub fn index() -> MethodRouter<(), Infallible> {
    get(|| async { response::Html(include_str!("../templates/index.html")) })
}

pub fn upload_file() -> MethodRouter<(), Infallible> {
    post(get_file)
}

async fn get_file(mut multipart: Multipart) -> impl IntoResponse {
    let uuid_to_inputfile: Arc<Mutex<HashMap<String, String>>> =
        Arc::new(Mutex::new(HashMap::new()));
    let output_response: Arc<Mutex<HashMap<String, String>>> = Arc::new(Mutex::new(HashMap::new()));
    let mut file_ids = Vec::new();

    let process_error_string = Arc::new(Mutex::new(String::new()));
    let maybe_error_filename = Arc::new(Mutex::new(String::new()));
    let process_error_once = Arc::new(Mutex::new(Once::new()));

    let resp_builder = response::Response::builder();

    while let Some(mut field) = match multipart.next_field().await {
        Ok(field_option) => field_option,
        Err(e) => {
            return resp_builder
                .status(e.status())
                .body(Body::from(e.body_text()))
                .unwrap();
        }
    } {
        let file_name: String;
        if let Some(field_file_name) = field.file_name() {
            file_name = field_file_name.to_string();
        } else {
            continue;
        }

        let file_uuid = Uuid::new_v4().to_string();
        let tmpfile_name = to_tmpfile_path(file_uuid.clone());

        let file = File::create(tmpfile_name)
            .await
            .expect("Error creating file. Check if the directory tmpfiles exists");
        let mut file_bufwriter = BufWriter::new(file);

        while let Some(chunk) = match field.chunk().await {
            Ok(chunk_option) => chunk_option,
            Err(e) => {
                return resp_builder
                    .status(StatusCode::BAD_REQUEST)
                    .body(Body::from(e.to_string()))
                    .unwrap();
            }
        } {
            file_bufwriter
                .write(chunk.to_vec().as_slice())
                .await
                .unwrap();
        }

        uuid_to_inputfile
            .clone()
            .lock()
            .unwrap()
            .insert(file_uuid.clone(), file_name);
        file_ids.push(file_uuid);
    }

    let mut handles = Vec::new();

    for uuid in file_ids.iter() {
        let input_path = to_tmpfile_path(uuid.clone());

        let safe_uuid_to_inputfile = uuid_to_inputfile.clone();
        let safe_output_response = output_response.clone();

        let uuid_clone: String = uuid.clone();
        let safe_process_error_string = process_error_string.clone();
        let safe_process_error_once = process_error_once.clone();
        let safe_maybe_error_filename = maybe_error_filename.clone();

        let handle = tokio::spawn(async move {
            match process(input_path, WHISPER_POOL.get_state()).await {
                Ok(output) => {
                    let received_file_name = safe_uuid_to_inputfile
                        .lock()
                        .unwrap()
                        .get(&uuid_clone)
                        .unwrap()
                        .to_owned();
                    safe_output_response
                        .lock()
                        .unwrap()
                        .insert(received_file_name, output);
                }
                Err(e) => safe_process_error_once.lock().unwrap().call_once(|| {
                    *safe_process_error_string.lock().unwrap() = e.to_string();
                    *safe_maybe_error_filename.lock().unwrap() = safe_uuid_to_inputfile
                        .lock()
                        .unwrap()
                        .get(&uuid_clone)
                        .unwrap()
                        .to_owned();
                }),
            }
        });

        handles.push(handle);
    }

    future::try_join_all(handles).await.unwrap();

    let maybe_error = process_error_string.clone().lock().unwrap().to_owned();
    let maybe_error_fname = maybe_error_filename.clone().lock().unwrap().to_owned();

    if !maybe_error.is_empty() {
        return resp_builder
            .status(StatusCode::BAD_REQUEST)
            .body(Body::from(format!(
                "Possible issue with uploaded file {}; Error: {}",
                maybe_error_fname, maybe_error
            )))
            .unwrap();
    }

    return resp_builder
        .status(StatusCode::OK)
        .body(Body::from(response::Json(serde_json::json!({"status": "successfully transcibed file/files", "response": *output_response.lock().unwrap()})).to_string()))
        .unwrap();
}

// format uuid to the temporary file path for the input file
fn to_tmpfile_path(uuid: String) -> String {
    return format!("./tmpfiles/{}_blob", uuid);
}
