![Image Optimizer App](./static/Screenshot%20from%202025-04-16%2001-11-32.png)

# Image Optimizer

A fast and efficient web application built with Rust and modern web technologies to optimize images with quality preservation. The application intelligently compresses image file sizes while maintaining good visual quality, similar to how messaging platforms like Telegram and WhatsApp handle image optimization.

## Features

- **Dual Functionality**: Optimize images or bulk rename them
- **Drag & Drop Interface**: Easy-to-use interface for uploading multiple images at once
- **Multi-format Support**: Handles JPEG, PNG, GIF, WebP, and other common image formats
- **WebP Conversion**: Optimizes by converting images to the efficient WebP format
- **Intelligent Resizing**: Automatically resizes images that exceed maximum dimensions
- **Batch Processing**: Process multiple images simultaneously
- **Session Management**: Files are organized in unique sessions for better organization
- **Bulk Download**: Download all processed images as a ZIP archive
- **Before/After Comparison**: See visual quality and file size differences
- **Responsive Design**: Works seamlessly on desktop and mobile devices

## Technical Architecture

The application follows a client-server architecture:

1. **Backend (Rust)**:
   - Built with the Axum web framework for high performance
   - Uses Tokio for asynchronous processing
   - Leverages Rayon for parallel image processing
   - Implements WebP conversion for optimal compression
   - Image processing pipeline:
     - Format detection
     - Resizing (max 2048×2048)
     - WebP conversion with quality settings
   - Session-based file organization

2. **Frontend (HTML/CSS/JavaScript)**:
   - Modern, responsive interface
   - Real-time previews and progress indicators
   - File drag & drop with browser-native APIs
   - Modal image viewer with comparison tools
   - Supports both optimization and batch renaming modes

## Getting Started

### Prerequisites

- Rust and Cargo (latest stable version)
- Modern web browser

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

## Usage

### Optimizing Images

1. Drag & drop images onto the interface or use the file picker
2. Click the "Optimize Images" button to process them
3. View the before/after comparison and compression statistics
4. Download individual optimized images or all as a ZIP archive

### Renaming Images

1. Switch to the "Rename Images" tab
2. Set a base name for your files in the input field
3. Drag & drop images to upload
4. The app will rename files sequentially (e.g., vacation-1.jpg, vacation-2.jpg)
5. Download processed files individually or as a ZIP archive

## Configuration

The project contains several configurable settings in the `optimizer.rs` file:

- `MAX_WIDTH` and `MAX_HEIGHT`: Maximum dimensions for images (default: 2048×2048)
- `WEBP_QUALITY`: Quality level for WebP conversion (default: 75.0)
- `PNG_OPTIMIZATION_LEVEL`: Level of PNG optimization (default: 2)

## Performance Considerations

- The application employs Tokio's asynchronous runtime for handling concurrent requests
- CPU-intensive operations run in separate threads via `spawn_blocking`
- Rayon is used for parallel processing when appropriate
- Images are stored in session-specific directories for organization and cleanup
- WebP conversion provides significant file size reduction while maintaining quality

## Building for Production

For production deployment:

```
cargo build --release
```

The optimized binary will be in `target/release/images-optimizer`.

Consider configuring a reverse proxy like Nginx for TLS termination and serving static files.

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Acknowledgments

- [image](https://github.com/image-rs/image) - Rust image processing library
- [webp](https://github.com/jaredforth/webp) - WebP encoding for Rust
- [oxipng](https://github.com/shssoichiro/oxipng) - PNG optimization tool
- [axum](https://github.com/tokio-rs/axum) - Web framework from the Tokio team
- [rayon](https://github.com/rayon-rs/rayon) - Data parallelism library for Rust 