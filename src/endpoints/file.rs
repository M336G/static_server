use axum::{body::Body, extract::{Path, State}, http::{StatusCode, header}, response::{IntoResponse, Response}};
use std::io::ErrorKind;
use tokio::fs::{self, File};
use tokio_util::io::ReaderStream;

use crate::AppState;

fn build_response(status: StatusCode, content_type: &'static str, body: Body) -> Response {
    let mut response = (
        status,
        [
            (header::CACHE_CONTROL, "public, max-age=86400"),
            (header::CONTENT_TYPE, content_type)
        ],
        body
    ).into_response();

    if content_type != "application/octet-stream" {
        response.headers_mut().insert(
            header::X_CONTENT_TYPE_OPTIONS,
            "nosniff".parse().unwrap()
        );
    }

    response
}

pub async fn get_static_file(State(state): State<AppState>, Path(file_name): Path<String>) -> Response {
    // Make sure there is no path traversal attempt
    if file_name.contains("..") {
        return (StatusCode::BAD_REQUEST, "Invalid file name").into_response();
    }

    let file_path = state.storage_directory.join(&file_name);

    let content_type = if state.disable_mime_guessing {
        "application/octet-stream"
    } else {
        mime_guess::from_path(&file_path)
            .first_raw()
            .unwrap_or("application/octet-stream")
    };

    // If the file is cached, return it from the cache directly
    if let Some(cache) = &state.cache {
        if let Some((data, content_type)) = cache.get(&file_name) {
            return build_response(
                StatusCode::OK,
                content_type,
                Body::from((*data).clone())
            );
        }
    }

    // If the file isn't cached, check whether it fits in the per entry size limit
    // and try to cache it, otherwise just stream it directly from the disk
    if let Some(cache) = &state.cache {
        match fs::metadata(&file_path).await {
            // Cache the file and return it
            Ok(metadata) if metadata.len() as usize <= cache.get_max_size_per_entry() => {
                match fs::read(&file_path).await {
                    Ok(data) => {
                        cache.insert(file_name, data.clone(), content_type);
                        
                        return build_response(
                            StatusCode::OK,
                            content_type,
                            Body::from(data)
                        );
                    }

                    Err(error) if error.kind() == ErrorKind::NotFound => {
                        return (StatusCode::NOT_FOUND, "File not found").into_response();
                    }

                    Err(error) => eprintln!("Failed to read file for caching: {error:?}"),
                }
            }

            // Skip caching as the file's simply too big
            Ok(_) => {}
            
            Err(error) if error.kind() == ErrorKind::NotFound => {
                return (StatusCode::NOT_FOUND, "File not found").into_response();
            }

            Err(error) => eprintln!("Failed to read metadata: {error:?}"),
        }
    }

    // If the file is too large or caching is disabled, stream it directly
    let file = match File::open(&file_path).await {
        Ok(file) => file,
        Err(error) => match error.kind() {
            ErrorKind::NotFound => {
                return (StatusCode::NOT_FOUND, "File not found").into_response();
            }
            _ => {
                eprintln!("Failed to open file: {error:?}");
                return (StatusCode::INTERNAL_SERVER_ERROR, "Failed reading file").into_response();
            }
        }
    };

    let stream = ReaderStream::new(file);
    
    build_response(
        StatusCode::OK,
        content_type,
        Body::from_stream(stream)
    )
}