#!/bin/bash

echo "Image Optimizer Docker Preparation Script"
echo "========================================"
echo ""
echo "This script will prepare a Docker image for deployment to a server."
echo ""

# Function to check if Docker is installed
check_docker() {
  if ! command -v docker &> /dev/null; then
    echo "Error: Docker is not installed or not in your PATH."
    echo "Please install Docker before continuing."
    exit 1
  fi
}

# Function to build and save the image
build_image() {
  echo "Building Docker image..."
  sudo docker build -t images-optimizer .
  if [ $? -ne 0 ]; then
    echo "Error: Failed to build the image."
    return 1
  fi
  
  echo "Saving image to images-optimizer.tar..."
  sudo docker save -o images-optimizer.tar images-optimizer
  if [ $? -ne 0 ]; then
    echo "Error: Failed to save the image."
    return 1
  fi
  
  echo "Image built and saved successfully!"
  echo "File: images-optimizer.tar"
  echo "Size: $(du -h images-optimizer.tar | cut -f1)"
  return 0
}

# Check if Docker is installed
check_docker

# Ask if they want to proceed
read -p "Do you want to build the Docker image? (y/n): " choice

case $choice in
  [Yy]*)
    build_image
    ;;
  *)
    echo "Build canceled. Exiting."
    exit 0
    ;;
esac

echo ""
echo "Next steps:"
echo "1. Transfer the Docker image file to your server using scp:"
echo "   scp images-optimizer.tar user@your-server-ip:/path/to/destination/"
echo ""
echo "2. Follow the instructions in DOCKER_SERVER_DEPLOYMENT.md for deploying on your server."
echo ""
echo "Done!" 