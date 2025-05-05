use crate::queue::{QueueManager, VideoTask};
use std::process::Command;
#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;
use std::sync::{Arc, Mutex, mpsc::Sender};
use std::time::Duration;

#[derive(Debug, Clone)]
pub enum UpdateMessage {
    TaskStarted(usize),        // task_id
    TaskCompleted(usize),      // task_id
    TaskFailed(usize, String), // task_id, error message
    TaskCancelled(usize),      // task_id
    WorkerFinished,            // Worker has finished processing
}

pub fn process_next_task(
    task: &VideoTask,
    executable_path: &std::path::PathBuf,
    queue_manager: &Arc<Mutex<QueueManager>>,
) -> Result<(), String> {
    // Log the command invocation
    log::info!(
        "Executing {:?}: --recipe {:?} --input {:?} --outdir {:?}",
        executable_path,
        task.recipe_path,
        task.input_path,
        task.output_dir
    );

    // Check if the recipe file exists
    if !task.recipe_path.exists() {
        let err_msg = format!(
            "Task {} failed: Recipe file not found at path: {:?}",
            task.id, task.recipe_path
        );
        log::error!("{}", err_msg);
        return Err(err_msg);
    }

    // Convert output_dir to absolute path if it's relative
    let output_dir = if task.output_dir.is_relative() {
        if let Ok(abs_path) = std::env::current_dir() {
            abs_path.join(&task.output_dir)
        } else {
            task.output_dir.clone()
        }
    } else {
        task.output_dir.clone()
    };

    let mut command = Command::new(executable_path);
    command.arg("--recipe");
    command.arg(&task.recipe_path);
    command.arg("--input");
    command.arg(&task.input_path);
    command.arg("--outdir");
    command.arg(&output_dir);

    log::debug!("Full command being executed: {:?}", command);

    // On Windows, set creation flags to suppress error dialogs
    #[cfg(target_os = "windows")]
    {
        command.creation_flags(0x08000000); // CREATE_NO_WINDOW
    }

    // Spawn the process
    match command.spawn() {
        Ok(mut child) => {
            // Check for force stop every 100ms
            loop {
                // Check if force stop was requested
                {
                    let manager = queue_manager.lock()
                        .expect("Failed to lock queue manager");
                    if manager.is_force_stop_requested() {
                        if let Err(e) = child.kill() {
                            log::error!("Failed to kill process: {}", e);
                        }
                        return Err("Task force stopped by user".to_string());
                    }
                }

                match child.try_wait() {
                    Ok(Some(status)) => {
                        if status.success() {
                            log::info!("Task {} completed successfully", task.id);
                            return Ok(());
                        } else {
                            let err_msg = format!(
                                "Task {} failed with status: {}",
                                task.id, status
                            );
                            log::error!("{}", err_msg);
                            return Err(err_msg);
                        }
                    }
                    Ok(None) => {
                        std::thread::sleep(Duration::from_millis(100));
                        continue;
                    }
                    Err(e) => {
                        let err_msg = format!(
                            "Task {} failed while waiting: {}",
                            task.id, e
                        );
                        log::error!("{}", err_msg);
                        return Err(err_msg);
                    }
                }
            }
        }
        Err(e) => {
            let err_msg = format!(
                "Task {} failed to spawn: {}. Is 'smoothie-rs' in PATH?",
                task.id, e
            );
            log::error!("{}", err_msg);
            Err(err_msg)
        }
    }
}

pub fn run_worker(
    queue_manager: Arc<Mutex<QueueManager>>,
    tx: Sender<UpdateMessage>,
    executable_path: std::path::PathBuf,
) {
    println!("Worker thread started.");
    
    loop {
        // Check if stop was requested
        {
            let manager = queue_manager.lock()
                .expect("Failed to lock queue manager");
            if manager.is_stop_requested() {
                log::info!("Worker received stop request. Exiting loop.");
                break;
            }
        }

        // Get next task
        let task_option = {
            let mut manager = queue_manager.lock()
                .expect("Failed to lock queue manager");
            manager.next_pending_task()
                .map(|task| (task.id, task.clone()))
        };

        if let Some((task_id, task_data)) = task_option {
            println!("Worker found pending task: {}", task_id);

            // Mark task as running and clear any force stop flag
            {
                let mut manager = queue_manager.lock()
                    .expect("Failed to lock queue manager");
                manager.mark_as_running(task_id);
                manager.clear_force_stop();
            }
            
            if let Err(e) = tx.send(UpdateMessage::TaskStarted(task_id)) {
                eprintln!("Failed to send TaskStarted message: {}", e);
            }

            // Process the task
            let result = process_next_task(&task_data, &executable_path, &queue_manager);

            // Update task status
            {
                let mut manager = queue_manager.lock()
                    .expect("Failed to lock queue manager");
                    
                match result {
                    Ok(_) => {
                        manager.mark_as_completed(task_id);
                        if let Err(e) = tx.send(UpdateMessage::TaskCompleted(task_id)) {
                            eprintln!("Failed to send TaskCompleted message: {}", e);
                        }
                    }
                    Err(err_msg) => {
                        if err_msg == "Task force stopped by user" {
                            manager.mark_as_cancelled(task_id);
                            if let Err(e) = tx.send(UpdateMessage::TaskCancelled(task_id)) {
                                eprintln!("Failed to send TaskCancelled message: {}", e);
                            }
                        } else {
                            manager.mark_as_failed(task_id, err_msg.clone());
                            if let Err(e) = tx.send(UpdateMessage::TaskFailed(task_id, err_msg)) {
                                eprintln!("Failed to send TaskFailed message: {}", e);
                            }
                        }
                    }
                }
            }

            // Check if we should continue processing
            {
                let manager = queue_manager.lock()
                    .expect("Failed to lock queue manager");
                if manager.is_stop_requested() {
                    println!("Stop requested. Exiting loop.");
                    break;
                }
            }
        } else {
            println!("No more pending tasks. Exiting loop.");
            break;
        }
    }

    println!("Worker sending WorkerFinished message.");
    if let Err(e) = tx.send(UpdateMessage::WorkerFinished) {
        eprintln!("Failed to send WorkerFinished message: {}", e);
    }
    println!("Worker thread finished.");
}
