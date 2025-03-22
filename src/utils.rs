use anyhow::{Context, Result};
use std::ffi::OsStr;
use std::fmt;
use std::path::Path;

/// Represents a file size with appropriate units
pub struct FileSize(pub u64);

impl fmt::Display for FileSize {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let bytes = self.0;

        if bytes < 1024 {
            write!(f, "{} B", bytes)
        } else if bytes < 1024 * 1024 {
            write!(f, "{:.2} KB", bytes as f64 / 1024.0)
        } else if bytes < 1024 * 1024 * 1024 {
            write!(f, "{:.2} MB", bytes as f64 / (1024.0 * 1024.0))
        } else {
            write!(f, "{:.2} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
        }
    }
}

/// Get the extension of a file
pub fn get_extension<P: AsRef<Path>>(path: P) -> Option<String> {
    path.as_ref()
        .extension()
        .and_then(OsStr::to_str)
        .map(|s| s.to_lowercase())
}

/// Check if a file is an image based on its extension
pub fn is_image_file<P: AsRef<Path>>(path: P) -> bool {
    let extensions = [
        "jpg", "jpeg", "png", "gif", "webp", "tiff", "bmp", "ico", "svg",
    ];

    if let Some(ext) = get_extension(path) {
        extensions.contains(&ext.as_str())
    } else {
        false
    }
}

/// Create a directory if it doesn't exist
pub async fn ensure_dir_exists<P: AsRef<Path>>(path: P) -> Result<()> {
    let path = path.as_ref();

    if !path.exists() {
        tokio::fs::create_dir_all(path)
            .await
            .with_context(|| format!("Failed to create directory: {:?}", path))?;
    }

    Ok(())
}

/// Format a percentage value with 2 decimal places
pub fn format_percentage(value: f64) -> String {
    format!("{:.2}%", value)
}

/// Validate a MIME type is an image
pub fn is_image_mime_type(mime_type: &str) -> bool {
    mime_type.starts_with("image/")
}

/// Get a sanitized filename from a path
pub fn sanitize_filename<P: AsRef<Path>>(path: P) -> String {
    path.as_ref()
        .file_name()
        .and_then(OsStr::to_str)
        .map(|s| {
            s.chars()
                .map(|c| {
                    if c.is_alphanumeric() || c == '.' || c == '-' || c == '_' {
                        c
                    } else {
                        '_'
                    }
                })
                .collect()
        })
        .unwrap_or_else(|| "unknown".to_string())
}

/// Generate a unique filename by adding a timestamp
pub fn generate_unique_filename<P: AsRef<Path>>(path: P) -> String {
    let path = path.as_ref();
    let timestamp = chrono::Utc::now().timestamp();

    let stem = path.file_stem().and_then(OsStr::to_str).unwrap_or("image");

    let ext = path.extension().and_then(OsStr::to_str).unwrap_or("webp");

    format!("{}-{}.{}", stem, timestamp, ext)
}
