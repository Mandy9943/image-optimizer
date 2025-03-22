# Image Optimizer

A powerful web application built with Rust and modern web technologies to optimize images similarly to messaging platforms like Telegram and WhatsApp. The application reduces image file sizes while maintaining good visual quality.

## Features

- **Drag & Drop Interface**: Easy-to-use interface for uploading multiple images at once
- **Multi-format Support**: Handles JPEG, PNG, GIF, WebP, and other image formats
- **Smart Optimization**: Intelligently optimizes each image based on its format
- **WebP Conversion**: Converts images to WebP for maximum compression with good quality
- **Responsive Design**: Works on desktop and mobile devices
- **Batch Processing**: Process multiple images at once
- **Before/After Comparison**: See the original and optimized file sizes

## Technical Details

The application consists of two main components:

1. **Backend (Rust)**: Handles image processing and optimization using libraries like:
   - `image` - For basic image manipulation
   - `mozjpeg` - For high-quality JPEG compression
   - `webp` - For WebP encoding
   - `oxipng` - For PNG optimization
   - `axum` - For the web server framework

2. **Frontend (HTML/CSS/JS)**: Provides a modern, user-friendly interface with:
   - Drag and drop functionality
   - Image previews
   - Progress indicators
   - Responsive design

## Getting Started

### Prerequisites

- Rust and Cargo (latest stable version)
- A modern web browser

### Installation

1. Clone the repository:
   ```
   git clone https://github.com/yourusername/images-optimizer.git
   cd images-optimizer
   ```

2. Build the application:
   ```
   cargo build --release
   ```

3. Run the application:
   ```
   cargo run --release
   ```

4. Open your browser and navigate to:
   ```
   http://localhost:3000
   ```

## How It Works

1. **Upload**: Drag and drop images onto the interface or use the file picker
2. **Processing**: Images are sent to the Rust backend where they are:
   - Analyzed to determine the best optimization strategy
   - Resized if they exceed maximum dimensions (2048x2048)
   - Compressed using format-specific optimizations
   - Converted to WebP if it produces better results
3. **Results**: Optimized images are displayed with comparison metrics and can be downloaded

## Optimization Strategies

- **JPEG**: Uses mozjpeg with quality level 80 and progressive encoding
- **PNG**: Uses oxipng for lossless compression, then converts to WebP for better results
- **GIF**: Converts to WebP for better compression
- **WebP**: Re-encodes with optimal parameters if already in WebP format
- **Other formats**: Converts to WebP as a fallback

## Configuration

The application uses sensible defaults that work well for most use cases, but several parameters can be adjusted in the source code:

- Maximum image dimensions (default: 2048x2048)
- JPEG quality level (default: 80)
- WebP quality level (default: 80.0)
- PNG optimization level (default: 3)

## Performance Considerations

- Image processing is CPU-intensive, so the application uses `tokio::task::spawn_blocking` for CPU-bound operations
- For very large images, processing may take longer
- The server is configured to handle multiple requests concurrently using Tokio's async runtime

## Building for Production

For production deployment, build with optimizations:

```
cargo build --release
```

The optimized binary will be located at `target/release/images-optimizer`

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Acknowledgments

- [image](https://github.com/image-rs/image) - Rust image processing library
- [mozjpeg](https://github.com/kornelski/mozjpeg-sys) - Mozilla's JPEG encoder for Rust
- [webp](https://github.com/jaredforth/webp) - WebP encoding for Rust
- [oxipng](https://github.com/shssoichiro/oxipng) - PNG optimization tool
- [axum](https://github.com/tokio-rs/axum) - Web framework 