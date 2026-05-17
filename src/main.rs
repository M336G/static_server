use axum::{http::Method, Router, routing::get};
use dotenv::dotenv;
use std::{env, path::PathBuf, sync::Arc, process};
use tokio::net::TcpListener;
use tower_http::{compression::CompressionLayer, cors::{self, CorsLayer}};

mod cache;
mod endpoints;

use cache::FileCache;

// Stores some "global variables" which will be used across the whole program
#[derive(Clone)]
struct AppState {
    storage_directory: Arc<PathBuf>,
    disable_mime_guessing: bool,
    cache: Option<Arc<FileCache>>
}

#[tokio::main]
async fn main() {
    // Loading environment variables from the .env file if it's there
    dotenv().ok();

    // Port on which the server will be running
    let server_port: u16 = env::var("PORT")
        .ok()
        .and_then(|port| port.parse().ok())
        .unwrap_or(6784);

    // The directory from which the files will be served
    let storage_directory: Arc<PathBuf> = match env::var("STORAGE_DIRECTORY") {
        Ok(path) => Arc::new(path.into()),
        Err(_) => {
            eprintln!("STORAGE_DIRECTORY environment variable must be set!");
            process::exit(1);
        }
    };
    if !storage_directory.is_dir() {
        eprintln!("STORAGE_DIRECTORY is not a directory or does not exist!");
        process::exit(1);
    }

    // Whether to disable guessing files' mime type when serving them
    let disable_mime_guessing: bool = env::var("DISABLE_MIME_GUESSING")
        .ok()
        .and_then(|boolean| boolean.parse().ok())
        .unwrap_or(false);
    if disable_mime_guessing {
        println!("File mime guessing disabled!")
    }

    // The maximum size of the file cache (in memory)
    let max_cache_size: Option<u16> = env::var("MAX_CACHE_SIZE")
        .ok()
        .and_then(|size| size.parse().ok());
    if let Some(size) = &max_cache_size {
        println!("Max file cache size set to {size} MB!");
    } else {
        println!("File cache disabled!");
    }

    let cache = max_cache_size.map(|mb| Arc::new(FileCache::new(mb)));

    let state = AppState {
        storage_directory,
        disable_mime_guessing,
        cache
    };

    let app = Router::new()
        .route("/", get(endpoints::health_check))
        .route("/{*file_name}", get(endpoints::get_static_file))
        .layer(CompressionLayer::new())
        .layer(
            CorsLayer::new()
                .allow_origin(cors::Any)
                .allow_methods([Method::GET])
        )
        .with_state(state);

    println!("Server running on http://0.0.0.0:{server_port}/");
    axum::serve(
        TcpListener::bind(format!("0.0.0.0:{server_port}"))
            .await
            .unwrap(),
        app,
    )
    .await
    .unwrap();
}
