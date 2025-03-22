use std::fs::File;
use std::io::{Cursor, Write};
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use axum::{
    body::Full,
    extract::{Multipart, Query, State},
    http::{header, HeaderMap, HeaderValue, Method, StatusCode},
    response::{Html, IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use bytes::Bytes;
use serde::{Deserialize, Serialize};
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::ServeDir;
use tracing::{debug, info, Level};
use tracing_subscriber::FmtSubscriber;
use uuid::Uuid;

mod optimizer;
mod utils;

// App state shared between routes
struct AppState {
    temp_dir: PathBuf,
    optimized_dir: PathBuf,
    rename_counter: AtomicUsize,
}

#[derive(Debug, Serialize, Deserialize)]
struct OptimizedImage {
    id: String,
    filename: String,
    original_size: u64,
    optimized_size: u64,
    compression_ratio: f64,
    download_url: String,
}

#[derive(Debug, Deserialize)]
struct ZipQuery {
    files: Option<String>,
}

#[tokio::main]
async fn main() {
    // Initialize logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)
        .expect("Failed to set global default subscriber");

    info!("Starting Image Optimizer Server");

    // Create necessary directories
    let temp_dir = std::env::temp_dir().join("images-optimizer-temp");
    let optimized_dir = std::env::current_dir()
        .expect("Failed to get current directory")
        .join("static")
        .join("optimized");

    std::fs::create_dir_all(&temp_dir).expect("Failed to create temp directory");
    std::fs::create_dir_all(&optimized_dir).expect("Failed to create optimized directory");

    info!("Temp directory: {:?}", temp_dir);
    info!("Optimized directory: {:?}", optimized_dir);

    // Create static directory for the frontend
    let static_dir = std::env::current_dir()
        .expect("Failed to get current directory")
        .join("static");
    std::fs::create_dir_all(&static_dir).expect("Failed to create static directory");

    // Write index.html
    let index_html = include_str!("../static/index.html");
    std::fs::write(static_dir.join("index.html"), index_html).expect("Failed to write index.html");

    // Write style.css
    let style_css = include_str!("../static/style.css");
    std::fs::write(static_dir.join("style.css"), style_css).expect("Failed to write style.css");

    // Write script.js
    let script_js = include_str!("../static/script.js");
    std::fs::write(static_dir.join("script.js"), script_js).expect("Failed to write script.js");

    // Create shared state
    let optimized_dir_for_state = optimized_dir.clone();
    let state = Arc::new(AppState {
        temp_dir,
        optimized_dir: optimized_dir_for_state,
        rename_counter: AtomicUsize::new(0),
    });

    // Configure CORS
    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST])
        .allow_headers(Any)
        .allow_origin(Any);

    // Create router
    let app = Router::new()
        .route("/", get(index_handler))
        .route("/api/optimize", post(optimize_handler))
        .route("/api/rename", post(rename_handler))
        .route("/api/download-zip", get(download_zip_handler))
        .nest_service("/static", ServeDir::new(static_dir))
        .nest_service("/optimized", ServeDir::new(optimized_dir))
        .layer(cors)
        .with_state(state);

    // Run server
    let addr = SocketAddr::from(([0, 0, 0, 0], 3655));
    info!("Listening on {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .expect("Failed to start server");
}

// Serve index.html
async fn index_handler() -> impl IntoResponse {
    let html = include_str!("../static/index.html");
    Html(html)
}

// Handle image optimization
async fn optimize_handler(
    State(state): State<Arc<AppState>>,
    mut multipart: Multipart,
) -> Result<Json<Vec<OptimizedImage>>, (StatusCode, String)> {
    let mut results = Vec::new();

    info!("Starting to process multipart form data");

    // Process each field individually using a more resilient approach
    while let Ok(Some(field)) = multipart.next_field().await {
        // Process one field at a time, with separate error handling
        info!("Processing a new field from multipart form");

        if let Some(optimized_image) = process_field(field, &state).await {
            info!(
                "Successfully optimized image: {:?}",
                optimized_image.filename
            );
            results.push(optimized_image);
        }
    }

    info!(
        "Completed multipart processing, optimized {} images",
        results.len()
    );

    if results.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            "No images were successfully processed".to_string(),
        ));
    }

    Ok(Json(results))
}

// Process a single field from the multipart form
async fn process_field(
    field: axum::extract::multipart::Field<'_>,
    state: &Arc<AppState>,
) -> Option<OptimizedImage> {
    // Maximum file size (15MB)
    const MAX_FILE_SIZE: usize = 15 * 1024 * 1024;

    // 1. Get field name
    let field_name = match field.name() {
        Some(name) => name.to_string(),
        None => {
            info!("Field has no name, skipping");
            return None;
        }
    };
    info!("Processing field: {}", field_name);

    // We expect files to be sent with field name "file" or "files"
    if field_name != "file" && field_name != "files" {
        info!("Skipping field with unexpected name: {}", field_name);
        return None;
    }

    // 2. Get filename
    let filename = match field.file_name() {
        Some(name) => {
            info!("Field has filename: {}", name);
            name.to_string()
        }
        None => {
            info!("Missing filename for field: {}", field_name);
            return None;
        }
    };

    // 3. Check if the file is an image by extension
    let extension = std::path::Path::new(&filename)
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.to_lowercase());

    match extension {
        Some(ext)
            if ["jpg", "jpeg", "png", "gif", "webp", "bmp", "tiff"].contains(&ext.as_str()) =>
        {
            // Valid image file extension
            info!("Valid image extension: {}", ext);
        }
        _ => {
            info!("Skipping file with unsupported extension: {:?}", extension);
            return None;
        }
    }

    // 4. Read the file data
    let data = match field.bytes().await {
        Ok(bytes) => {
            let len = bytes.len();
            if len > MAX_FILE_SIZE {
                info!(
                    "File too large: {} bytes (max: {} bytes)",
                    len, MAX_FILE_SIZE
                );
                return None;
            }
            info!("Read {} bytes of data", len);
            bytes
        }
        Err(e) => {
            info!("Failed to read file data: {}", e);
            return None;
        }
    };

    // 5. Quick validation of image format
    let format = match image::guess_format(&data) {
        Ok(format) => {
            info!("Detected image format: {:?}", format);
            format
        }
        Err(e) => {
            info!("Invalid image data - not a recognized image format: {}", e);
            return None;
        }
    };

    // 6. Generate a unique ID for the image
    let id = Uuid::new_v4().to_string();
    info!("Processing file: {} (ID: {})", filename, id);

    // 7. Create a temporary file for the uploaded image
    let temp_path = state.temp_dir.join(&id);
    if let Err(e) = tokio::fs::write(&temp_path, &data).await {
        info!("Failed to write temp file: {}", e);
        return None;
    }

    // 8. Determine optimized filename
    let file_stem = std::path::Path::new(&filename)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("image");

    let optimized_filename = format!("{}-optimized.webp", file_stem);
    let output_path = state.optimized_dir.join(&optimized_filename);

    // 9. Get original file size
    let original_size = data.len() as u64;

    // 10. Optimize the image
    info!("Starting optimization for image ID: {}", id);
    match optimizer::optimize_image(&temp_path, &output_path).await {
        Ok(_) => {
            info!("Optimization successful for image ID: {}", id);

            // 11. Get optimized file size
            let optimized_size = match tokio::fs::metadata(&output_path).await {
                Ok(metadata) => metadata.len(),
                Err(e) => {
                    info!("Failed to get metadata for optimized file: {}", e);
                    0
                }
            };

            let compression_ratio = if original_size > 0 {
                (1.0 - (optimized_size as f64 / original_size as f64)) * 100.0
            } else {
                0.0
            };

            let download_url = format!("/optimized/{}", optimized_filename);

            // 12. Remove temporary file
            if let Err(e) = tokio::fs::remove_file(&temp_path).await {
                info!("Failed to remove temp file: {}", e);
                // Continue processing anyway
            }

            Some(OptimizedImage {
                id,
                filename: optimized_filename,
                original_size,
                optimized_size,
                compression_ratio,
                download_url,
            })
        }
        Err(e) => {
            info!("Failed to optimize image: {}", e);
            let _ = tokio::fs::remove_file(&temp_path).await;
            None
        }
    }
}

// Add this new handler for renaming images without optimization
async fn rename_handler(
    State(state): State<Arc<AppState>>,
    mut multipart: Multipart,
) -> Result<Json<Vec<OptimizedImage>>, (StatusCode, String)> {
    let mut results = Vec::new();
    let mut base_name = String::from("image");
    let mut image_fields = Vec::new();

    info!("Starting to process rename multipart form data");

    // First pass: extract all fields and process the base name
    while let Ok(Some(field)) = multipart.next_field().await {
        let field_name = field.name().unwrap_or("unnamed");

        if field_name == "baseName" {
            // This is the base name field
            if let Ok(name_value) = field.text().await {
                if !name_value.trim().is_empty() {
                    base_name = name_value.trim().to_string();
                    info!("Using base name: {}", base_name);
                }
            }
        } else if field_name == "file" || field_name == "files" {
            // Collect image files for the second pass
            if let Some(filename) = field.file_name() {
                // Clone the filename before consuming the field
                let filename_clone = filename.to_string();
                info!("Collected file for renaming: {}", filename_clone);

                // Read the file data immediately to avoid issues with field lifetime
                let data = match field.bytes().await {
                    Ok(bytes) => bytes,
                    Err(e) => {
                        info!("Failed to read file data for {}: {}", filename_clone, e);
                        continue;
                    }
                };

                // Store the filename and data for later processing
                image_fields.push((filename_clone, data));
            }
        }
    }

    // Second pass: process all collected image fields
    for (filename, data) in image_fields {
        info!("Processing file for renaming: {}", filename);

        // Get file extension from original filename
        let extension = std::path::Path::new(&filename)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("jpg")
            .to_lowercase();

        // Get the next counter value atomically
        let counter_value = state.rename_counter.fetch_add(1, Ordering::SeqCst);

        // Generate new filename with counter - using the current counter value
        let new_filename = format!("{}-{}.{}", base_name, counter_value, extension);

        info!(
            "Generated new filename: {} (global counter now: {})",
            new_filename,
            counter_value + 1
        );

        // Create a unique ID for this file
        let id = Uuid::new_v4().to_string();

        // Write the file to the optimized directory with the new name
        let output_path = state.optimized_dir.join(&new_filename);
        if let Err(e) = tokio::fs::write(&output_path, &data).await {
            info!("Failed to write renamed file: {}", e);
            continue;
        }

        let file_size = data.len() as u64;

        results.push(OptimizedImage {
            id,
            filename: new_filename.clone(), // Clone here to prevent move
            original_size: file_size,
            optimized_size: file_size, // Same as original for rename only
            compression_ratio: 0.0,    // No compression for rename only
            download_url: format!("/optimized/{}", new_filename),
        });
    }

    info!(
        "Completed rename processing, renamed {} images",
        results.len()
    );

    if results.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            "No images were successfully renamed".to_string(),
        ));
    }

    Ok(Json(results))
}

// Handler for downloading all optimized images as a ZIP file
async fn download_zip_handler(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ZipQuery>,
) -> impl IntoResponse {
    info!("Received request to download images as ZIP");

    // Create a buffer to store the ZIP file
    let cursor = Cursor::new(Vec::new());
    let mut zip = zip::ZipWriter::new(cursor);

    // Options for ZIP files (compression level, etc.)
    let options = zip::write::FileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .unix_permissions(0o755);

    let mut file_count = 0;
    let mut included_filenames = Vec::new();

    // If specific files are requested, parse them
    let requested_files: Vec<String> = match &params.files {
        Some(files) => {
            info!("Requested specific files for ZIP: {}", files);
            files.split(',').map(|s| s.to_string()).collect()
        }
        None => {
            // If no files specified, include all files in the optimized directory
            info!("No specific files requested, will include all files in the optimized directory");
            Vec::new()
        }
    };

    // Read the optimized directory
    let optimized_dir = &state.optimized_dir;
    let read_dir = match tokio::fs::read_dir(optimized_dir).await {
        Ok(dir) => dir,
        Err(e) => {
            info!("Failed to read images directory: {}", e);
            return create_error_response("Failed to read images directory".to_string());
        }
    };

    // Collect all files to be included in the ZIP
    let mut entries = Vec::new();

    let mut read_dir = read_dir;
    while let Ok(Some(entry)) = read_dir.next_entry().await {
        if let Ok(metadata) = entry.metadata().await {
            if metadata.is_file() {
                let filename = entry.file_name().to_string_lossy().to_string();

                // Include only requested files if any were specified
                if !requested_files.is_empty() && !requested_files.contains(&filename) {
                    debug!("Skipping file not in requested list: {}", filename);
                    continue;
                }

                entries.push(entry.path());
                included_filenames.push(filename.clone());
            }
        }
    }

    if entries.is_empty() {
        info!("No matching files found for ZIP creation");
        return create_error_response(
            "No matching files found for the specified criteria".to_string(),
        );
    }

    info!(
        "Found {} files to include in ZIP: {:?}",
        entries.len(),
        included_filenames
    );

    // Add each file to the ZIP
    for path in entries {
        let filename = path.file_name().unwrap().to_string_lossy().to_string();
        info!("Adding file to ZIP: {}", filename);

        // Read the file contents
        let file_content = match tokio::fs::read(&path).await {
            Ok(content) => content,
            Err(e) => {
                info!("Failed to read file {}: {}", filename, e);
                continue; // Skip this file but continue with others
            }
        };

        // Add file to ZIP
        if let Err(e) = zip.start_file(&filename, options) {
            info!("Failed to start file in ZIP: {}", e);
            continue;
        }

        if let Err(e) = zip.write_all(&file_content) {
            info!("Failed to write file content to ZIP: {}", e);
            continue;
        }

        file_count += 1;
    }

    // Finalize the ZIP file
    if file_count == 0 {
        info!("No files were added to the ZIP");
        return create_error_response(
            "No images were successfully added to the ZIP file".to_string(),
        );
    }

    let zip_data = match zip.finish() {
        Ok(cursor) => cursor.into_inner(),
        Err(e) => {
            info!("Failed to finalize ZIP file: {}", e);
            return create_error_response("Failed to finalize ZIP file".to_string());
        }
    };

    info!("Successfully created ZIP file with {} images", file_count);

    // Determine if these are renamed files based on filenames (pattern: name-1.ext)
    let are_renamed = !included_filenames.is_empty()
        && included_filenames[0].contains('-')
        && !included_filenames[0].contains("optimized");

    // Create appropriate content disposition based on whether these are optimized or renamed images
    let zip_filename = if are_renamed {
        "renamed-images.zip"
    } else {
        "optimized-images.zip"
    };

    let content_disposition = format!("attachment; filename=\"{}\"", zip_filename);
    info!("Setting ZIP filename to: {}", zip_filename);

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/zip")
        .header(header::CONTENT_DISPOSITION, content_disposition)
        .body(Full::from(zip_data))
        .unwrap_or_else(|_| create_error_response("Failed to create response".to_string()))
}

// Helper function to create error responses
fn create_error_response(message: String) -> Response<Full<Bytes>> {
    Response::builder()
        .status(StatusCode::INTERNAL_SERVER_ERROR)
        .header(header::CONTENT_TYPE, "text/plain")
        .body(Full::from(message))
        .unwrap()
}
