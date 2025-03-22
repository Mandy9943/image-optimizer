// DOM Elements
const dropArea = document.getElementById("drop-area");
const fileInput = document.getElementById("file-input");
const optimizeBtn = document.getElementById("optimize-btn");
const clearBtn = document.getElementById("clear-all");
const previewContainer = document.getElementById("preview-container");
const previewGrid = document.getElementById("preview-grid");
const imageCount = document.getElementById("image-count");
const resultsContainer = document.getElementById("results");
const resultsGrid = document.getElementById("results-grid");
const downloadAllBtn = document.getElementById("download-all");
let downloadZipBtn; // Will be created dynamically
const loadingOverlay = document.getElementById("loading-overlay");

// Templates
const previewTemplate = document.getElementById("preview-template");
const resultTemplate = document.getElementById("result-template");

// Selected files storage
let selectedFiles = [];
// Optimized images results
let optimizedResults = [];

// Initialize the application
function init() {
  setupEventListeners();
}

// Set up all event listeners
function setupEventListeners() {
  // Drag and drop events
  ["dragenter", "dragover", "dragleave", "drop"].forEach((eventName) => {
    dropArea.addEventListener(eventName, preventDefaults, false);
  });

  ["dragenter", "dragover"].forEach((eventName) => {
    dropArea.addEventListener(eventName, highlight, false);
  });

  ["dragleave", "drop"].forEach((eventName) => {
    dropArea.addEventListener(eventName, unhighlight, false);
  });

  // Handle file drop
  dropArea.addEventListener("drop", handleDrop, false);

  // File input change
  fileInput.addEventListener("change", handleFileInput, false);

  // Button clicks
  optimizeBtn.addEventListener("click", optimizeImages, false);
  clearBtn.addEventListener("click", clearAllFiles, false);
  downloadAllBtn.addEventListener("click", downloadAllImages, false);
}

// Prevent default behaviors for drag and drop
function preventDefaults(e) {
  e.preventDefault();
  e.stopPropagation();
}

// Highlight drop area when dragging over
function highlight() {
  dropArea.classList.add("active");
}

// Remove highlight when dragging leaves
function unhighlight() {
  dropArea.classList.remove("active");
}

// Handle dropped files
function handleDrop(e) {
  const dt = e.dataTransfer;
  const files = dt.files;

  handleFiles(files);
}

// Handle files from file input
function handleFileInput() {
  const files = fileInput.files;
  handleFiles(files);
}

// Process files
function handleFiles(files) {
  if (files.length === 0) return;

  // Convert FileList to Array and filter for images
  Array.from(files)
    .filter((file) => file.type.startsWith("image/"))
    .forEach(addFileToPreview);

  // Update UI
  updateUI();
}

// Add a file to the preview grid
function addFileToPreview(file) {
  // Check if the file is already in the selected files list
  const isDuplicate = selectedFiles.some(
    (existingFile) =>
      existingFile.name === file.name &&
      existingFile.size === file.size &&
      existingFile.lastModified === file.lastModified
  );

  if (isDuplicate) {
    console.log(`File ${file.name} is already selected.`);
    return;
  }

  // Add to selected files
  selectedFiles.push(file);

  // Create preview item from template
  const previewItem = document
    .importNode(previewTemplate.content, true)
    .querySelector(".preview-item");
  const previewId = `preview-${Date.now()}-${Math.random()
    .toString(36)
    .substring(2, 9)}`;
  previewItem.setAttribute("data-id", previewId);

  // Set preview image
  const img = previewItem.querySelector("img");
  img.src = URL.createObjectURL(file);
  img.alt = file.name;

  // Set file details
  previewItem.querySelector(".preview-filename").textContent = file.name;
  previewItem.querySelector(".preview-size").textContent = formatFileSize(
    file.size
  );

  // Set up remove button
  const removeBtn = previewItem.querySelector(".remove-btn");
  removeBtn.addEventListener("click", () => removeFile(previewId, file));

  // Add to grid
  previewGrid.appendChild(previewItem);
}

// Remove a file from the selected files
function removeFile(id, file) {
  // Remove from array
  selectedFiles = selectedFiles.filter((f) => f !== file);

  // Remove from DOM
  const item = previewGrid.querySelector(`[data-id="${id}"]`);
  if (item) {
    previewGrid.removeChild(item);
  }

  // Update UI
  updateUI();
}

// Clear all selected files
function clearAllFiles() {
  selectedFiles = [];
  previewGrid.innerHTML = "";
  updateUI();
}

// Update UI state based on selected files
function updateUI() {
  const hasFiles = selectedFiles.length > 0;

  // Update count
  imageCount.textContent = selectedFiles.length.toString();

  // Show/hide preview container
  previewContainer.style.display = hasFiles ? "block" : "none";

  // Enable/disable optimize button
  optimizeBtn.disabled = !hasFiles;
}

// Optimize the selected images
async function optimizeImages() {
  if (selectedFiles.length === 0) return;

  showLoading(true);

  try {
    // Process one file at a time to avoid multipart parsing issues
    const totalFiles = selectedFiles.length;
    let allResults = [];

    // Create a batch message to show progress
    const processingMessage = document.createElement("div");
    processingMessage.className = "batch-message";
    processingMessage.innerHTML = `
      <p>Processing ${totalFiles} images one by one...</p>
      <p class="progress">Image 1/${totalFiles}</p>
    `;
    document.querySelector(".loading-overlay").appendChild(processingMessage);

    // Process each image individually
    for (let i = 0; i < totalFiles; i++) {
      const file = selectedFiles[i];

      // Update progress message
      document.querySelector(".batch-message .progress").textContent = `Image ${
        i + 1
      }/${totalFiles}`;

      // Verify file is an image
      if (!file.type.startsWith("image/")) {
        console.warn(`Skipping file "${file.name}" - not an image`);
        continue;
      }

      // Check file size (max 15MB)
      const MAX_FILE_SIZE = 15 * 1024 * 1024; // 15MB
      if (file.size > MAX_FILE_SIZE) {
        console.warn(
          `Skipping file "${file.name}" - too large (${formatFileSize(
            file.size
          )} > ${formatFileSize(MAX_FILE_SIZE)})`
        );
        continue;
      }

      // Create a separate form data for each file
      const formData = new FormData();
      formData.append("file", file);

      try {
        // Send to the backend
        const response = await fetch("/api/optimize", {
          method: "POST",
          body: formData,
        });

        if (!response.ok) {
          console.error(
            `Error optimizing file "${file.name}": HTTP ${response.status}`
          );
          continue; // Skip to next file on error
        }

        // Parse the results
        const results = await response.json();
        if (results && results.length > 0) {
          allResults = [...allResults, ...results];
        }
      } catch (fileError) {
        console.error(`Error processing file "${file.name}":`, fileError);
        // Continue with next file
      }

      // Short delay between requests to avoid overwhelming the server
      if (i < totalFiles - 1) {
        await new Promise((resolve) => setTimeout(resolve, 200));
      }
    }

    // Remove the progress message
    const batchMessage = document.querySelector(".batch-message");
    if (batchMessage) {
      batchMessage.remove();
    }

    if (allResults.length === 0) {
      throw new Error("No images were successfully processed");
    }

    // Display results
    displayResults(allResults);
  } catch (error) {
    console.error("Error optimizing images:", error);
    alert("Error optimizing images: " + error.message);
  } finally {
    showLoading(false);
  }
}

// Display optimization results
function displayResults(results) {
  // Store the results for later use with ZIP download
  optimizedResults = results;

  // Clear previous results
  resultsGrid.innerHTML = "";

  // Add each result
  results.forEach((result) => {
    const resultItem = document
      .importNode(resultTemplate.content, true)
      .querySelector(".result-item");

    // Set image
    const img = resultItem.querySelector("img");
    img.src = result.download_url;
    img.alt = result.filename;

    // Set details
    resultItem.querySelector(".result-filename").textContent = result.filename;
    resultItem.querySelector(".original-size").textContent = formatFileSize(
      result.original_size
    );
    resultItem.querySelector(".optimized-size").textContent = formatFileSize(
      result.optimized_size
    );
    resultItem.querySelector(
      ".compression-ratio"
    ).textContent = `${result.compression_ratio.toFixed(2)}%`;

    // Set download link
    const downloadLink = resultItem.querySelector(".download-btn");
    downloadLink.href = result.download_url;
    downloadLink.download = result.filename;

    // Add to grid
    resultsGrid.appendChild(resultItem);
  });

  // Show results container
  resultsContainer.style.display = "block";

  // Add the download as ZIP button
  addZipDownloadButton();

  // Scroll to results
  resultsContainer.scrollIntoView({ behavior: "smooth" });
}

// Add a "Download as ZIP" button to the results section
function addZipDownloadButton() {
  // Remove the button if it already exists
  if (downloadZipBtn) {
    downloadZipBtn.remove();
  }

  // Create the new button
  downloadZipBtn = document.createElement("button");
  downloadZipBtn.className = "secondary-btn zip-download-btn";
  downloadZipBtn.innerHTML =
    '<i class="fas fa-file-archive"></i> Download as ZIP';
  downloadZipBtn.addEventListener("click", downloadAllAsZip);

  // Insert after the download all button
  downloadAllBtn.parentNode.insertBefore(
    downloadZipBtn,
    downloadAllBtn.nextSibling
  );

  // Add some space between buttons
  downloadZipBtn.style.marginLeft = "10px";
}

// Download all optimized images as a ZIP file
function downloadAllAsZip() {
  if (optimizedResults.length === 0) return;

  // Show a loading indicator
  showLoading(true);

  // Get all the file names
  const filenames = optimizedResults.map((result) => result.filename).join(",");

  // Request the ZIP file from the server
  const zipUrl = `/api/download-zip?files=${encodeURIComponent(filenames)}`;

  // Create a download link
  const downloadLink = document.createElement("a");
  downloadLink.href = zipUrl;
  downloadLink.download = "optimized-images.zip";

  // Add the link to the document and click it
  document.body.appendChild(downloadLink);
  downloadLink.click();

  // Clean up
  document.body.removeChild(downloadLink);
  showLoading(false);
}

// Download all optimized images individually
function downloadAllImages() {
  const downloadLinks = resultsGrid.querySelectorAll(".download-btn");

  if (downloadLinks.length === 0) return;

  // Click each download link programmatically
  downloadLinks.forEach((link, index) => {
    // Add a small delay between downloads to avoid browser blocking
    setTimeout(() => {
      link.click();
    }, index * 300);
  });
}

// Show or hide the loading overlay
function showLoading(show) {
  loadingOverlay.style.display = show ? "flex" : "none";
}

// Format a file size in bytes to a human-readable string
function formatFileSize(bytes) {
  if (bytes === 0) return "0 B";

  const k = 1024;
  const sizes = ["B", "KB", "MB", "GB"];
  const i = Math.floor(Math.log(bytes) / Math.log(k));

  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + " " + sizes[i];
}

// Initialize when the DOM is loaded
document.addEventListener("DOMContentLoaded", init);
