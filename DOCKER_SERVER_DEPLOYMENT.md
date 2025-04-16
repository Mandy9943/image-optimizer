# Deploying Images Optimizer to a Server

This guide provides step-by-step instructions for deploying the Images Optimizer application on a remote server using Docker.

## Prerequisites

- A remote server with Docker installed
- SSH access to the server

## Deployment Options

There are several ways to deploy your application. Choose the option that best fits your workflow:

### Option 1: Using Docker Hub (Recommended)

This is the simplest deployment method and eliminates the need to transfer large image files.

#### 1. Create a Docker Hub Account

If you don't already have one, create an account at [Docker Hub](https://hub.docker.com/).

#### 2. Build and Push Your Image to Docker Hub

```bash
# Build your image locally
sudo docker build -t yourusername/images-optimizer:latest .

# Log in to Docker Hub
sudo docker login

# Push the image to Docker Hub
sudo docker push yourusername/images-optimizer:latest
```

#### 3. Pull and Run the Image on Your Server

```bash
# SSH into your server
ssh user@your-server-ip

# Pull the image from Docker Hub
sudo docker pull yourusername/images-optimizer:latest

# Run the container
sudo docker run -d --name images-optimizer -p 3655:3655 -v /path/on/server/optimized:/app/static/optimized yourusername/images-optimizer:latest
```

#### 4. Update the Application

When you make changes to your application:

```bash
# On your development machine
sudo docker build -t yourusername/images-optimizer:latest .
sudo docker push yourusername/images-optimizer:latest

# On your server
sudo docker pull yourusername/images-optimizer:latest
sudo docker stop images-optimizer
sudo docker rm images-optimizer
sudo docker run -d --name images-optimizer -p 3655:3655 -v /path/on/server/optimized:/app/static/optimized yourusername/images-optimizer:latest
```

### Option 2: Using a Docker Image File

This method is useful when you don't want to use a public repository or can't access Docker Hub.

## Creating the Docker Image File

Before you can deploy to a server, you need to create the Docker image file. You can do this using the provided script:

```bash
# Run the preparation script from the project root
./prepare_docker_image.sh
```

This script will build the Docker image and save it as `images-optimizer.tar` in the current directory.

Alternatively, you can create the file manually:

```bash
# Build the Docker image
sudo docker build -t images-optimizer .

# Save the Docker image to a file
sudo docker save -o images-optimizer.tar images-optimizer
```

## Deployment Steps

### 1. Transfer the Docker Image to the Server

Use SCP to transfer the Docker image to your server:

```bash
scp images-optimizer.tar user@your-server-ip:/path/to/destination/
```

### 2. Log into Your Server

```bash
ssh user@your-server-ip
```

### 3. Load the Docker Image

Navigate to the directory where you transferred the file and load the image:

```bash
cd /path/to/destination/
sudo docker load -i images-optimizer.tar
```

You should see output confirming the image was loaded.

### 4. Create a Directory for Persistent Storage (Optional)

If you want the optimized images to persist between container restarts:

```bash
sudo mkdir -p /path/on/server/optimized
sudo chmod 777 /path/on/server/optimized  # Ensure proper permissions
```

### 5. Run the Container

#### Basic Run Command:

```bash
sudo docker run -d --name images-optimizer -p 3655:3655 images-optimizer
```

#### Run with Volume for Persistent Storage:

```bash
sudo docker run -d --name images-optimizer -p 3655:3655 -v /path/on/server/optimized:/app/static/optimized images-optimizer
```

#### Run on a Different Port:

If port 3655 is already in use on your server, you can map it to a different port:

```bash
# Example using the image on port 8080
sudo docker run -d --name images-optimizer -p 8080:3655 images-optimizer
```

In this example, the application will be accessible on port 8080.

### 6. Verify the Container is Running

```bash
sudo docker ps | grep images-optimizer
```

You should see output showing that the container is running.

### 7. Check the Container Logs

```bash
sudo docker logs images-optimizer
```

You should see a message indicating that the application has started, similar to:

```
INFO images_optimizer: Starting Image Optimizer Server
INFO images_optimizer: Temp directory: "/tmp/images-optimizer-temp"
INFO images_optimizer: Optimized directory: "/app/static/optimized"
INFO images_optimizer: Listening on 0.0.0.0:3655
```

## Accessing the Application

After deployment, you can access the Image Optimizer application by opening a web browser and navigating to:

```
http://your-server-ip:3655
```

Or, if you used a different port:

```
http://your-server-ip:custom-port
```

## Managing the Container

### Stopping the Container

```bash
sudo docker stop images-optimizer
```

### Starting the Container Again

```bash
sudo docker start images-optimizer
```

### Removing the Container

```bash
sudo docker stop images-optimizer
sudo docker rm images-optimizer
```

### Viewing Container Logs

```bash
sudo docker logs images-optimizer
```

For continuous log monitoring:
```bash
sudo docker logs -f images-optimizer
```

## Updating the Application

### Standard Update (With Brief Downtime)

To update the application with minimal downtime:

1. Create an updated Docker image
2. Save it to a tar file
3. Transfer it to the server
4. Load the new image
5. Stop and remove the old container
6. Start a new container with the updated image

```bash
# On your development machine
sudo docker save -o images-optimizer-new.tar images-optimizer

# Transfer to server
scp images-optimizer-new.tar user@your-server-ip:/path/to/destination/

# On the server
sudo docker load -i images-optimizer-new.tar
sudo docker stop images-optimizer
sudo docker rm images-optimizer
sudo docker run -d --name images-optimizer -p 3655:3655 -v /path/on/server/optimized:/app/static/optimized images-optimizer
```

### Zero-Downtime Update

To update the application without any downtime, you can use the following approach:

1. Load the new Docker image on the server
2. Run a new container on a different name/port
3. Verify the new container works correctly
4. Switch the traffic to the new container (either using a reverse proxy or by stopping the old one and starting the new one on the same port)

```bash
# On your development machine
sudo docker save -o images-optimizer-new.tar images-optimizer

# Transfer to server
scp images-optimizer-new.tar user@your-server-ip:/path/to/destination/

# On the server
# 1. Load the new image
sudo docker load -i images-optimizer-new.tar

# 2. Run a new container with a different name on a temporary port
sudo docker run -d --name images-optimizer-new -p 3656:3655 -v /path/on/server/optimized:/app/static/optimized images-optimizer

# 3. Verify the new container works correctly
curl -I http://localhost:3656

# 4. If everything looks good, swap the containers
# Stop the old container
sudo docker stop images-optimizer

# Start the new container on the original port
sudo docker run -d --name images-optimizer-updated -p 3655:3655 -v /path/on/server/optimized:/app/static/optimized images-optimizer

# 5. Once confirmed working, clean up
sudo docker rm images-optimizer
sudo docker rm images-optimizer-new
sudo docker rename images-optimizer-updated images-optimizer
```

For even smoother upgrades, consider using a reverse proxy like Nginx or a load balancer to direct traffic to the appropriate container.

## Alternative Deployment Methods

### Using GitHub Actions and Docker Hub

For a completely automated deployment workflow, you can set up GitHub Actions to:
1. Automatically build your Docker image when you push to your repository
2. Push the built image to Docker Hub
3. Trigger a deployment to your server

This requires setting up a CI/CD pipeline, but once established, deployment becomes a simple `git push`.

### Using a Container Registry Service

Besides Docker Hub, there are other container registry services:
- GitHub Container Registry
- GitLab Container Registry
- Amazon ECR
- Google Container Registry
- Azure Container Registry

These services provide private repositories and often integrate well with their respective cloud platforms.

## Troubleshooting

### Port Conflicts

If you see an error about the port being already in use, check if another process is using that port:

```bash
sudo netstat -tuln | grep 3655
```

If the port is in use, you can choose a different port when starting the container.

### Permission Issues

If there are permission issues with the mounted volume, you may need to adjust the permissions:

```bash
sudo chmod -R 777 /path/on/server/optimized
```

### Docker Command Access

If you're getting permission errors when running Docker commands, you may need to add your user to the Docker group or use `sudo` with all Docker commands as shown in this guide.

## Security Considerations

- The container runs with minimal privileges but modifying the `/app/static/optimized` directory requires write permissions.
- Consider running behind a reverse proxy like Nginx for SSL termination and additional security.
- Restrict access to the application as needed for your environment.

```bash
</rewritten_file> 