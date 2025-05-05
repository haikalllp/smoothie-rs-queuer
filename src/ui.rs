use crate::config::{self, SmoothieConfig};
use crate::queue::{QueueManager, TaskStatus, VideoTask};
use crate::worker::{self, UpdateMessage};
use eframe::egui;
use rfd::FileDialog;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, mpsc};
use std::thread;

pub struct SmoothieQueueApp {
    queue_manager: Arc<Mutex<QueueManager>>,
    config: Option<SmoothieConfig>,
    output_folder: Option<PathBuf>,
    recipe_path: PathBuf,
    worker_running: bool,
    last_id: usize,
    files_dropped: bool,
    available_recipes: Vec<PathBuf>,
    worker_tx: mpsc::Sender<UpdateMessage>,
    worker_rx: mpsc::Receiver<UpdateMessage>,
}

impl SmoothieQueueApp {
    pub fn new(cc: &eframe::CreationContext<'_>, initial_config: Option<SmoothieConfig>) -> Self {
        cc.egui_ctx.set_visuals(egui::Visuals::dark());

        let (worker_tx, worker_rx) = mpsc::channel();

        let initial_recipe_path = initial_config.as_ref().map_or_else(
            || {
                log::warn!("No initial config found, using fallback 'recipe.ini'");
                PathBuf::from("recipe.ini")
            },
            |cfg| cfg.recipe_path.clone(),
        );

        let available_recipes = initial_config
            .as_ref()
            .and_then(|cfg| cfg.executable_path.parent()?.parent())
            .map_or_else(
                || {
                    if initial_config.is_some() {
                        log::warn!("Could not determine base directory from executable path");
                    } else {
                        log::warn!("No initial config, cannot scan for recipes.");
                    }
                    Vec::new()
                },
                config::find_recipe_files,
            );

        Self {
            queue_manager: Arc::new(Mutex::new(QueueManager::new())),
            config: initial_config,
            output_folder: None,
            recipe_path: initial_recipe_path,
            worker_running: false,
            last_id: 0,
            files_dropped: false,
            worker_tx,
            worker_rx,
            available_recipes,
        }
    }
}

impl eframe::App for SmoothieQueueApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        while let Ok(update) = self.worker_rx.try_recv() {
            match update {
                UpdateMessage::TaskStarted(id) => {
                    let mut manager = self.queue_manager.lock()
                        .expect("Failed to lock queue manager");
                    manager.mark_as_running(id);
                }
                UpdateMessage::TaskCompleted(id) => {
                    let mut manager = self.queue_manager.lock()
                        .expect("Failed to lock queue manager");
                    manager.mark_as_completed(id);
                }
                UpdateMessage::TaskFailed(id, err_msg) => {
                    let mut manager = self.queue_manager.lock()
                        .expect("Failed to lock queue manager");
                    manager.mark_as_failed(id, err_msg);
                }
                UpdateMessage::TaskCancelled(id) => {
                    let mut manager = self.queue_manager.lock()
                        .expect("Failed to lock queue manager");
                    manager.mark_as_cancelled(id);
                }
                UpdateMessage::WorkerFinished => {
                    self.worker_running = false;
                }
            }
        }

        if let Some(config) = &self.config {
            egui::CentralPanel::default().show(ctx, |ui| {
                let has_tasks = {
                    let manager = self.queue_manager.lock()
                        .expect("Failed to lock queue manager");
                    !manager.tasks.is_empty()
                };

                if !has_tasks && !self.files_dropped {
                    ui.vertical_centered_justified(|ui| {
                        let drop_frame = egui::Frame::none()
                            .fill(ui.visuals().widgets.noninteractive.bg_fill)
                            .stroke(ui.visuals().widgets.noninteractive.fg_stroke)
                            .rounding(12.0)
                            .inner_margin(egui::Margin::same(20.0));
                        
                        drop_frame.show(ui, |ui| {
                            ui.vertical_centered(|ui| {
                                ui.label(
                                    egui::RichText::new("Drop video files here")
                                        .heading()
                                        .color(ui.visuals().text_color())
                                );
                                ui.label(
                                    egui::RichText::new("Supported formats: mp4, mkv, mov, avi, webm")
                                        .color(ui.visuals().weak_text_color())
                                );
                            });
                        });
                    });
                } else {
                    ui.heading("Smoothie Queuer");

                    // Output Folder Selector
                    ui.horizontal(|ui| {
                        if ui.button("Select Output Folder").clicked() {
                            if let Some(path) = FileDialog::new().pick_folder() {
                                self.output_folder = Some(path.clone());
                                let mut manager = self.queue_manager.lock()
                                    .expect("Failed to lock queue manager");
                                manager.update_pending_output_dirs(path);
                            }
                        }
                        let folder_text = self.output_folder.as_ref()
                            .map_or("Default (next to input video)".to_string(), 
                                   |p| p.display().to_string());
                        ui.label(format!("Output: {}", folder_text));
                    });

                    // Recipe ComboBox
                    ui.horizontal(|ui| {
                        ui.label("Recipe:");
                        let previous_recipe_path = self.recipe_path.clone();
                        let selected_recipe_filename = self.recipe_path.file_name()
                            .map_or_else(|| "Invalid".into(), |f| f.to_string_lossy());

                        egui::ComboBox::from_id_source("recipe_select")
                            .selected_text(selected_recipe_filename)
                            .show_ui(ui, |ui| {
                                for recipe in &self.available_recipes {
                                    let filename = recipe.file_name()
                                        .map_or_else(|| "Invalid Path".into(), |f| f.to_string_lossy());
                                    ui.selectable_value(&mut self.recipe_path, recipe.clone(), filename);
                                }
                            });
                        
                        if self.recipe_path != previous_recipe_path {
                            let mut manager = self.queue_manager.lock()
                                .expect("Failed to lock queue manager");
                            manager.update_pending_recipes(self.recipe_path.clone());
                        }
                    });

                    // Open Root Folder Button
                    ui.horizontal(|ui| {
                        if ui.button("Open Smoothie Folder").clicked() {
                            if let Some(exe_dir) = config.executable_path.parent() {
                                if let Some(root_dir) = exe_dir.parent() {
                                    let _ = opener::open(root_dir);
                                }
                            }
                        }
                    });

                    // Control Buttons
                    ui.horizontal(|ui| {
                        // Start Queue Button
                        let start_button = ui.add_enabled(!self.worker_running, egui::Button::new("Start Queue"));
                        if start_button.clicked() {
                            self.worker_running = true;
                            let queue_manager_clone = Arc::clone(&self.queue_manager);
                            let tx_clone = self.worker_tx.clone();
                            let executable_path_clone = config.executable_path.clone();
                            
                            let mut manager = self.queue_manager.lock()
                                .expect("Failed to lock queue manager");
                            manager.clear_stop_request();
                            manager.clear_force_stop();

                            thread::spawn(move || {
                                worker::run_worker(queue_manager_clone, tx_clone, executable_path_clone);
                            });
                        }

                        // Pause Queue Button
                        let is_paused = {
                            let manager = self.queue_manager.lock()
                                .expect("Failed to lock queue manager");
                            manager.is_stop_requested()
                        };
                        
                        let button_text = if is_paused { "Pause Queue (Paused)" } else { "Pause Queue" };
                        let stop_button = ui.add_enabled(self.worker_running, egui::Button::new(button_text));
                        if stop_button.clicked() {
                            let mut manager = self.queue_manager.lock()
                                .expect("Failed to lock queue manager");
                            if manager.is_stop_requested() {
                                manager.clear_stop_request();
                            } else {
                                manager.request_stop();
                            }
                        }

                        // Force Stop Task Button
                        let force_stop_button = ui.add_enabled(self.worker_running, egui::Button::new("Force Stop Task"));
                        if force_stop_button.clicked() {
                            let mut manager = self.queue_manager.lock()
                                .expect("Failed to lock queue manager");
                            manager.request_force_stop();
                            manager.request_stop();
                        }

                        // Clear Queue Button
                        let queue_empty = {
                            let manager = self.queue_manager.lock()
                                .expect("Failed to lock queue manager");
                            manager.tasks.is_empty()
                        };
                        let clear_button = ui.add_enabled(!self.worker_running && !queue_empty, 
                            egui::Button::new("Clear Queue"));
                        if clear_button.clicked() {
                            let mut manager = self.queue_manager.lock()
                                .expect("Failed to lock queue manager");
                            manager.clear_all_tasks();
                        }
                    });

                    ui.separator();

                    let is_paused = {
                        let manager = self.queue_manager.lock()
                            .expect("Failed to lock queue manager");
                        manager.is_stop_requested()
                    };
                    if is_paused {
                        ui.colored_label(egui::Color32::YELLOW, "Queue Paused - will stop after current task");
                    }
                    ui.separator();

                    // Task List Display
                    ui.heading("Task Queue");
                    let mut task_to_remove: Option<usize> = None;
                    egui::ScrollArea::vertical().auto_shrink([false; 2]).show(ui, |ui| {
                        let manager = self.queue_manager.lock()
                            .expect("Failed to lock queue manager");
                        if manager.tasks.is_empty() {
                            ui.label("(No tasks added yet)");
                        } else {
                            for task in manager.tasks.iter() {
                                ui.horizontal(|ui| {
                                    // Remove Button
                                    let remove_button = ui.add_enabled(
                                        task.status == TaskStatus::Pending,
                                        egui::Button::new("❌").small()
                                    );
                                    if remove_button.clicked() {
                                        task_to_remove = Some(task.id);
                                    }

                                    let filename = task.input_path.file_name()
                                        .map_or_else(|| "Invalid Path".to_string(), 
                                                   |name| name.to_string_lossy().to_string());
                                    let (status_text, status_color, error_msg) = match &task.status {
                                        TaskStatus::Pending => ("Pending", ui.visuals().text_color(), None),
                                        TaskStatus::Running => ("Running", egui::Color32::YELLOW, None),
                                        TaskStatus::Completed => ("Completed", egui::Color32::GREEN, None),
                                        TaskStatus::Failed(err) => ("Failed", egui::Color32::RED, Some(err.clone())),
                                        TaskStatus::Cancelled => ("Cancelled", egui::Color32::LIGHT_RED, None),
                                    };
                                    let response = ui.label(format!("{}: ", filename));
                                    ui.colored_label(status_color, status_text);
                                    if let Some(err) = error_msg {
                                        response.on_hover_text(&err);
                                    }
                                });
                                ui.separator();
                            }
                        }
                    });

                    if let Some(id_to_remove) = task_to_remove {
                        let mut manager = self.queue_manager.lock()
                            .expect("Failed to lock queue manager");
                        manager.remove_task(id_to_remove);
                    }
                }

                // Handle file drops
                let dropped_files = ctx.input(|i| i.raw.dropped_files.clone());
                if !dropped_files.is_empty() {
                    self.files_dropped = true;
                    let allowed_extensions = ["mp4", "mkv", "mov", "avi", "webm"];
                    
                    for file in dropped_files {
                        if let Some(path) = file.path {
                            if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                                if allowed_extensions.contains(&ext.to_lowercase().as_str()) {
                                    self.last_id += 1;
                                    let task = VideoTask {
                                        id: self.last_id,
                                        input_path: path.clone(),
                                        output_dir: self.output_folder.clone()
                                            .unwrap_or_else(|| PathBuf::from(path.parent().unwrap_or(Path::new(".")))),
                                        recipe_path: self.recipe_path.clone(),
                                        status: TaskStatus::Pending,
                                    };
                                    
                                    let mut manager = self.queue_manager.lock()
                                        .expect("Failed to lock queue manager");
                                    manager.add_task(task);
                                }
                            }
                        }
                    }
                    ctx.request_repaint();
                }
            });
        } else {
            egui::CentralPanel::default().show(ctx, |ui| {
                ui.heading("Smoothie Queuer");
                ui.colored_label(egui::Color32::RED, 
                    "Configuration Error: Could not automatically find smoothie-rs.");
                ui.label("Please locate the main 'Smoothie' installation folder.");
                
                if ui.button("Locate Smoothie Folder...").clicked() {
                    if let Some(folder_path) = FileDialog::new().pick_folder() {
                        match config::find_smoothie_config_in_dir(&folder_path) {
                            Ok(found_config) => {
                                self.config = Some(found_config.clone());
                                self.recipe_path = found_config.recipe_path;
                            },
                            Err(e) => log::error!("Failed to find valid config: {}", e),
                        }
                    }
                }
            });
        }

        if self.worker_running {
            ctx.request_repaint();
        }
    }
}
