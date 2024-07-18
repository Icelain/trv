use axum::{
    body::Body,
    extract::Multipart,
    http::StatusCode,
    response::{self, IntoResponse, Response},
    routing::{get, method_routing::MethodRouter, post},
};

use tokio::fs::File;
use tokio::io::{AsyncWriteExt, BufWriter};
use uuid::Uuid;

use core::convert::Infallible;

pub fn index() -> MethodRouter<(), Infallible> {
    get(|| async { response::Html(include_str!("../templates/index.html")) })
}

pub fn upload_file() -> MethodRouter<(), Infallible> {
    post(get_file)
}

async fn get_file(mut multipart: Multipart) -> impl IntoResponse {
    
    let mut file_name: String;
    let mut file_ids: Vec<String> = Vec::new();
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

        if let Some(field_file_name) = field.file_name() {
            file_name = field_file_name.to_string();
        } else {
            continue;
        }

        let file_uuid = Uuid::new_v4().to_string();
        let tmpfile_name = to_tmpfile_path(file_uuid.clone());

        let file = File::create(tmpfile_name)
            .await
            .expect("Error creating file");
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

        file_ids.push(file_uuid);

    }

    return resp_builder
        .status(StatusCode::OK)
        .body(Body::from(response::Json(serde_json::json!({"status": "successfully uploaded file/files", "file_ids": file_ids})).to_string()))
        .unwrap();
}

fn to_tmpfile_path(uuid: String) -> String {

    return format!("./tmpfiles/{}.blob", uuid);

}
