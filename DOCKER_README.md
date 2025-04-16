# Docker Setup for Image Optimizer

This document provides instructions for running the Image Optimizer application using Docker.

## Prerequisites

- Docker installed on your machine
- Docker Compose installed on your machine (optional, but recommended)

## Running Locally

### Using Docker Compose (Recommended)

1. Build and start the container:
   ```bash
   sudo docker compose up -d
   ```

2. Access the application in your browser:
   ```
   http://localhost:3655
   ```

3. To stop the container:
   ```bash
   sudo docker compose down
   ```

### Using Docker Directly

1. Build the Docker image:
   ```bash
   sudo docker build -t images-optimizer .
   ```

2. Run the container:
   ```bash
   sudo docker run -p 3655:3655 -d --name images-optimizer images-optimizer
   ```

3. Access the application in your browser:
   ```
   http://localhost:3655
   ```

4. To stop the container:
   ```bash
   sudo docker stop images-optimizer
   sudo docker rm images-optimizer
   ```

## Deploying to a Remote Server

For deploying to a remote server, we provide a preparation script and detailed instructions:

1. Run the preparation script to build and save the Docker image:
   ```bash
   ./prepare_docker_image.sh
   ```

2. Follow the instructions in `DOCKER_SERVER_DEPLOYMENT.md` for deploying the image to your server.

## Data Persistence

The application stores optimized images in the `/app/static/optimized` directory inside the container. To persist this data across container restarts:

- When using Docker Compose, a named volume is already configured in the docker-compose.yml file.

- When using Docker directly, you can mount a volume:
  ```bash
  sudo docker run -p 3655:3655 -v images-optimizer-data:/app/static/optimized -d --name images-optimizer images-optimizer
  ```

## Configuration

The application listens on port 3655 by default. If you need to change this:

1. Update the `EXPOSE` directive in the Dockerfile
2. Update the port mapping in the docker-compose.yml file or the docker run command

## Troubleshooting

- If you encounter permission issues with the mounted volumes, you may need to adjust the permissions on the host:
  ```bash
  sudo chown -R <your-user-id>:<your-group-id> /path/to/mounted/volume
  ```

- To view logs from the container:
  ```bash
  sudo docker logs images-optimizer
  ```

- To check if the container is running:
  ```bash
  sudo docker ps
  ``` 