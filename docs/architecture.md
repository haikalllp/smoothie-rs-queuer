# Smoothie Queue Application Architecture

## 1. Overall Structure

The Smoothie Queue application is a Rust-based GUI tool that manages a queue of video tasks to be processed by the `smoothie-rs` executable. The application is structured into five primary modules:

```
smoothie-queue/
├── src/
│   ├── main.rs      # Application entry point
│   ├── config.rs    # Configuration handling
│   ├── queue.rs     # Queue management
│   ├── ui.rs        # GUI implementation
│   └── worker.rs    # Background task processing
```

## 2. Module Responsibilities

### main.rs
- Application entry point
- Initializes logging system
- Handles configuration discovery process using `find_smoothie_config_auto`
- Sets up and runs the eframe application loop
- Provides initial error handling for configuration issues
- Passes `Option<SmoothieConfig>` to UI

### config.rs
- Defines `SmoothieConfig` structure for application configuration
- Implements `ConfigError` for handling configuration-related errors
- Provides functions for locating `smoothie-rs` executable and recipes:
  - `find_smoothie_config_auto`: Automatic configuration discovery
  - `find_smoothie_config_in_dir`: Directory-specific configuration search
  - `find_recipe_files`: Discovers available recipe files
- Handles path resolution (PATH environment, relative paths, user-selected directories)
- Manages default recipe location

### queue.rs
- Defines core data models:
  - `TaskStatus`: Represents the state of video processing tasks
  - `VideoTask`: Contains task-specific information
- Implements `QueueManager` for task management:
  - Task addition
  - Finding next pending task
  - Status updates
  - `clear_all_tasks`: Clears the entire task queue
  - `remove_task`: Removes specific task by index
  - `stop_requested` flag and methods for graceful termination
- Utilizes `serde` for potential serialization support

### ui.rs
- Implements `eframe::App` trait via `SmoothieQueueApp`
- Manages UI state, including:
  - Optional `SmoothieConfig` (handles both present and missing cases)
  - `available_recipes` list for ComboBox selection
  - Thread-safe queue access (`Arc<Mutex<QueueManager>>`)
  - Channel communication (`mpsc`)
- Handles UI rendering:
  - Drag-and-drop interface
  - File/folder selection
  - Task list display with removal buttons
  - Recipe selection via ComboBox
  - Control buttons:
    - Start processing
    - "Clear Queue"
    - "Stop After Current"
    - "Open Root Folder" (using `opener` crate)
  - Config failure state with user prompt
- Manages worker thread spawning
- Processes status updates from worker thread

### worker.rs
- Defines `UpdateMessage` enum for thread communication
- Implements `run_worker` function for background processing:
  - Runs in dedicated thread
  - Processes tasks sequentially
  - Checks `stop_requested` flag for graceful termination
  - Communicates status via channel
- Contains `process_next_task` function:
  - Constructs `smoothie-rs` commands
  - Executes external process
  - Handles execution errors
  - Uses configured executable path

## 3. Configuration Handling

The configuration system follows a two-stage hierarchical flow:

1. Initial Discovery (main.rs):
   - `find_smoothie_config_auto` attempts automatic configuration
   - Returns `Option<SmoothieConfig>` to UI

2. Fallback Handling (ui.rs):
   - If config is None, shows prompt to user
   - Uses `find_smoothie_config_in_dir` for manual selection
   - Also discovers available recipes via `find_recipe_files`

State Management:
- `SmoothieQueueApp` stores `Option<SmoothieConfig>`
- UI adapts behavior based on config presence:
  - With config: Full functionality enabled
  - Without config: Limited functionality with config prompt

## 4. State Management

### Application State
- Core state managed by enhanced `QueueManager` in `queue.rs`
- New methods for queue manipulation:
  - `clear_all_tasks`: Complete queue reset
  - `remove_task`: Selective task removal
  - `stop_requested` flag for graceful termination
- Thread-safe access via `Arc<Mutex<QueueManager>>`
- UI state in `SmoothieQueueApp`:
  - Configuration status (Option<SmoothieConfig>)
  - Available recipes list
  - Queue reference
  - Channel endpoints

### Concurrency Protection
- `Arc<Mutex>` ensures thread-safe queue access
- Multiple threads can safely modify queue state
- `stop_requested` flag coordination between UI and worker
- Prevents race conditions during task processing

## 5. Threading and Communication

```
UI Thread (ui.rs)                 Worker Thread (worker.rs)
┌────────────────┐               ┌────────────────┐
│ SmoothieQueue  │               │   run_worker   │
│      App       │◄──────────────┤     loop      │
└────────┬───────┘ UpdateMessage └────────┬───────┘
         │                                │
         │ Arc<Mutex<QueueManager>>       │
         └────────────────────────────────┘
```

### Enhanced Thread Structure
- UI Thread:
  - Runs main application interface
  - Manages enhanced application state
  - Processes new UI controls (stop, clear, remove)
  - Updates display based on worker messages
  - Handles `opener` crate integration for folder access

- Worker Thread:
  - Processes tasks sequentially
  - Checks `stop_requested` flag for graceful exit
  - Sends status updates via channel
  - Manages external process execution

### Communication
- Uses `std::sync::mpsc` channel
- `UpdateMessage` enum defines message types
- Worker -> UI communication for status updates
- Shared state access via `Arc<Mutex>`
- `stop_requested` flag coordination

## 6. UI Implementation Details

### New Controls
- Recipe `ComboBox`: Select from available recipes
- Task `Remove` buttons: Individual task removal
- `Clear Queue` button: Complete queue reset
- `Stop After Current` button: Graceful termination
- `Open Root Folder` button: Uses `opener` crate

### Configuration States
1. Valid Configuration:
   - Full UI functionality available
   - Recipe selection enabled
   - All controls active

2. Missing Configuration:
   - Limited functionality
   - Prominent configuration prompt
   - Basic queue viewing only

### External Integration
- `opener` crate used for system folder access
- Maintains cross-platform compatibility
- Provides familiar system dialogs