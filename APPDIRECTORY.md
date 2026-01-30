# AppImage Build Instructions

This document explains how to build an AppImage for Lectern.

## Prerequisites

- Ubuntu 20.04+ or Fedora (for building)
- Python 3 with pip
- appimage-builder: `pip3 install appimage-builder`
- Required system packages:
  - `patchelf`
  - `desktop-file-utils`
  - `ffmpeg` (will be bundled in the AppImage)

## Building Locally

1. Install appimage-builder:
   ```bash
   pip3 install appimage-builder
   ```

2. Build the AppImage:
   ```bash
   appimage-builder --recipe AppImageBuilder.yml
   ```

3. The resulting AppImage will be in the current directory: `Lectern-*.AppImage`

## Using GitHub Actions

The AppImage is automatically built when:
- A tag starting with `v` is pushed (e.g., `v0.1.0`)
- A release is created
- The workflow is manually triggered

The built AppImage will be:
- Uploaded as an artifact
- Attached to the release (if triggered by a tag)

### Running the workflow locally with act

You can run the same workflow on your machine using [act](https://nektosact.com/introduction.html) (run GitHub Actions locally via Docker-compatible containers). **act works with Podman**: it uses the Docker API, and Podman exposes a compatible socket, so no Docker installation is required.

1. Install act (e.g. `brew install act`).
2. Ensure Podman is running (`podman info`). If you use the rootless socket, act will typically pick it up (e.g. `unix:///run/user/$UID/podman/podman.sock`). To force it: `export DOCKER_HOST=unix:///run/user/$(id -u)/podman/podman.sock`.
3. From the repo root, list workflows and run the AppImage job:

   ```bash
   # List workflows and events
   act -l

   # Run the Build AppImage workflow (uses workflow_dispatch)
   act workflow_dispatch -W .github/workflows/build-appimage.yml
   ```

   To use the same runner image size as GitHub (`ubuntu-latest`, medium):

   ```bash
   act workflow_dispatch -W .github/workflows/build-appimage.yml -P ubuntu-latest=catthehacker/ubuntu:act-latest
   ```

4. The built AppImage will appear in the workflow’s working directory inside the container; use artifact steps or bind mounts if you need it on the host.

If `actions-rs/toolchain` or other steps fail under act, run the [local appimage-builder](#building-locally) steps instead; they produce the same result without containers.

## AppImage Contents

The AppImage bundles:
- The Lectern binary (built from Rust source)
- FFmpeg and FFprobe (required for audio processing)
- All application assets (icons, etc.)
- Desktop entry file

## Running the AppImage

1. Make it executable:
   ```bash
   chmod +x Lectern-*.AppImage
   ```

2. Run it:
   ```bash
   ./Lectern-*.AppImage
   ```

The AppImage is portable and doesn't require installation - it contains all dependencies.

## Troubleshooting

### Build fails with "FFmpeg not found"
- Ensure the apt sources in `AppImageBuilder.yml` are correct for your Ubuntu version
- The recipe uses Ubuntu 22.04 (Jammy) repositories

### Binary not found
- Check that `cargo build --release` completed successfully
- Verify the binary exists at `target/release/lectern`

### Missing assets
- Ensure the `assets/` directory exists and contains the required files
- The recipe will continue even if some assets are missing
