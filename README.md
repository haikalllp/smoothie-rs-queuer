<h1 align="center">
    <!-- yup if i put a line break they're not actually centered =( -->
    <img src="https://github.com/user-attachments/assets/3f112864-60a0-4fd1-aaf2-255951446e5e" width=100 /> Smoothie Queuer
</h1>
<p align="center">
    Smoothie Queuer is an add-on GUI for smoothie-rs.
</p>
<div align="center">
    <img src="https://github.com/user-attachments/assets/ce38e4f5-850c-4dd0-a308-5e1e4c6b9574" width="44.69%" style="margin-right: 10px" />
    <img src="https://github.com/user-attachments/assets/43557e65-ed8a-4c77-b4f9-35d5bb3509b0" width="45%" style="margin-left: 10px" />
</div>

# Smoothie Queuer

A graphical user interface for managing video processing tasks with the `smoothie-rs` CLI tool.

## Overview

Smoothie Queuer provides a user-friendly GUI for queuing and processing multiple video files using `smoothie-rs`. It streamlines the workflow by allowing drag-and-drop video management, recipe selection, and queue control, all within a visual interface.

https://github.com/user-attachments/assets/6e396c9a-bc84-448a-884d-8954cf0932e1

## Features

- **File Management**
  - Drag and drop videos into the queue
  - Choose an output folder and a recipe from the dropdown

- **Queue Management**
  - Monitor task status: Pending, Running, Completed, Failed
  - Start / Pause queue, force stop running task or clear the queue
  - Remove individual tasks

- **Smoothie Integration**
  - Auto-detects `smoothie-rs` and recipes
  - Manual selection of the Smoothie folder if not found
  - Quick access to the Smoothie folder

## Installation & Setup

### Prerequisites

- `smoothie-rs` must be installed on your system (DOWNLOAD HERE - [smoothie-rs](https://github.com/couleur-tweak-tips/smoothie-rs))
  - The application will look for the executable in your system PATH
  - Alternatively, you can manually select where you installed smoothie and select the folder

### Getting Started

1. Download the latest release version of `smoothie-queuer` here [smoothie-queuer](https://github.com/ShrewdyChan/smoothie-queuer/releases)
2. Run the Smoothie Queuer application
3. On first launch, the application will:
  - Automatically detect your `smoothie-rs` installation
  - If not found, prompt you to select your main Smoothie folder

### Expected Smoothie Folder Structure

```
Smoothie/
├── bin/
│   └── smoothie-rs.exe
├── recipe.ini
├── defaults.ini
└── encoding_presets.ini
```

### Smoothie Queuer Setup and Installation Video

[]()

## Usage

### Basic Operation

1. **Launch the Application**
   - Double-click the Smoothie Queuer executable

2. **Add Videos**
   - Drag and drop video files directly into the queue area
   - Files will appear in the queue with "Pending" status

3. **Configure Processing**
   - Select an output folder for processed videos
   - Choose a recipe from the dropdown menu
     - The dropdown automatically lists available recipes
     - `encoding_presets.ini` is excluded from the list

4. **Queue Management**
   - Click "Start Queue" to begin processing the queue
   - Use "Pause Queue" to pause the current queue
   - Use "Force Stop" to force stop the currently running task
   - Click "Clear Queue" to remove all pending tasks
   - Remove individual pending tasks using their remove button `X`
   - Monitor task status through the queue display

5. **Quick Access**
   - Use the "Open Root Folder" button to access the Smoothie directory

## For Developers

### Technology Stack

Built with Rust using the egui framework. For detailed architectural information, see `docs/architecture.md`.

### Building from Source

1. Clone the repository
    ```bash
    git clone https://github.com/ShrewdyChan/smoothie-queuer.git
    cd smoothie-queuer
    ```

2. Build for release
    ```bash
    cargo build --release
    ```

3. Find the executable in `target/release/`

### Development Workflow

#### Building the source code

```bash
cargo build
```

#### Running in Development Mode

```bash
cargo run
```

#### Linting

Check your code with Clippy:
```bash
cargo clippy
```

For strict linting with warnings as errors:
```bash
cargo clippy -- -D warnings
```

To fix linting issues automatically:
```bash
cargo clippy --fix -- -D warnings
```

### Key Dependencies

- `eframe`: GUI framework
- `rfd`: File dialogs
- `log`: Logging infrastructure
- `image`: Image handling (for icons)
- `regex`: Regular expressions for parsing
- `env_logger`: Logger configuration
- `serde`: Serialization/deserialization
## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.


<br>
<p align="center">
  <a href="https://github.com/couleur-tweak-tips/smoothie-rs">
    <img src="https://img.shields.io/badge/Made%20with%20%E2%9D%A4%EF%B8%8F%20for-smoothie--rs-ff69b4?style=for-the-badge&logo=rust" alt="Made with ❤️ for smoothie-rs">
  </a>
</p>
