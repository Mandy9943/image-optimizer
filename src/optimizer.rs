use anyhow::{Context, Result};
use image::{GenericImageView, ImageFormat};
use oxipng::{optimize_from_memory, Options as PngOptions};
use rayon::prelude::*;
use std::io::Cursor;
use std::path::Path;
use std::time::Instant;
use tracing::{debug, error, info, warn};
use webp::Encoder;

// Maximum dimensions for optimization - increased for faster processing
const MAX_WIDTH: u32 = 2048;
const MAX_HEIGHT: u32 = 2048;

// Quality settings - adjust for faster processing
const WEBP_QUALITY: f32 = 75.0; // Slightly lower quality for faster encoding
const PNG_OPTIMIZATION_LEVEL: u8 = 2; // Lower optimization level for faster processing

/// Optimize an image based on its type
pub async fn optimize_image<P: AsRef<Path>>(input_path: P, output_path: P) -> Result<()> {
    let start = Instant::now();
    let input_path = input_path.as_ref();
    let output_path = output_path.as_ref();

    // Read the file to memory
    let image_data = tokio::fs::read(input_path)
        .await
        .with_context(|| format!("Failed to read image file: {:?}", input_path))?;

    // Detect image format
    let format = detect_image_format(&image_data)
        .with_context(|| format!("Failed to detect image format for: {:?}", input_path))?;

    info!("Detected format: {:?} for {:?}", format, input_path);

    // Perform the optimization - simplified to just convert everything to WebP
    // This provides good compression while being fast to process
    let optimized_data = match format {
        // For all formats, we'll just resize and convert to WebP for speed
        _ => convert_to_webp(image_data.clone()).await?,
    };

    // Write the optimized image
    tokio::fs::write(output_path, optimized_data)
        .await
        .with_context(|| format!("Failed to write optimized image to: {:?}", output_path))?;

    let duration = start.elapsed();
    info!(
        "Image optimized in {:.2}s: {:?} -> {:?}",
        duration.as_secs_f64(),
        input_path,
        output_path
    );

    Ok(())
}

/// Detect the format of an image from its bytes
fn detect_image_format(data: &[u8]) -> Result<ImageFormat> {
    image::guess_format(data).with_context(|| "Failed to guess image format")
}

/// Convert an image to WebP format with fast settings
async fn convert_to_webp(data: Vec<u8>) -> Result<Vec<u8>> {
    tokio::task::spawn_blocking(move || {
        let img = image::load_from_memory(&data)?;

        // Resize if necessary
        let img = resize_if_needed(img, MAX_WIDTH, MAX_HEIGHT);

        // Convert to WebP with our quality settings
        convert_to_webp_from_image(&img)
    })
    .await
    .with_context(|| "WebP conversion task failed")?
}

/// Resize an image if it exceeds the maximum dimensions
fn resize_if_needed(
    img: image::DynamicImage,
    max_width: u32,
    max_height: u32,
) -> image::DynamicImage {
    let width = img.width();
    let height = img.height();

    // Check if resize is needed
    if width <= max_width && height <= max_height {
        return img;
    }

    // Calculate new dimensions while preserving aspect ratio
    let ratio = f64::min(
        max_width as f64 / width as f64,
        max_height as f64 / height as f64,
    );

    let new_width = (width as f64 * ratio).round() as u32;
    let new_height = (height as f64 * ratio).round() as u32;

    debug!(
        "Resizing image from {}x{} to {}x{}",
        width, height, new_width, new_height
    );

    // Use a faster resize algorithm for speed
    img.resize(new_width, new_height, image::imageops::FilterType::Triangle)
}

/// Convert an image::DynamicImage to WebP format
fn convert_to_webp_from_image(img: &image::DynamicImage) -> Result<Vec<u8>> {
    // Get RGBA data
    let rgba = img.to_rgba8();

    // Create WebP encoder
    let encoder = Encoder::from_rgba(rgba.as_raw(), img.width(), img.height());

    // Set quality and encode
    let encoded = encoder.encode(WEBP_QUALITY);

    // Convert to Vec<u8>
    Ok(encoded.to_vec())
}
