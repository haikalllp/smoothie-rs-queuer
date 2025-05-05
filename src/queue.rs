use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TaskStatus {
    Pending,
    Running,
    Completed,
    Failed(String),
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoTask {
    pub id: usize,
    pub input_path: PathBuf,
    pub output_dir: PathBuf,
    pub recipe_path: PathBuf,
    pub status: TaskStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueManager {
    pub tasks: Vec<VideoTask>,
    pub next_task_index: usize,
    pub stop_requested: bool,
    pub force_stop_requested: bool,
}

impl QueueManager {
    pub fn new() -> Self {
        Self {
            tasks: Vec::new(),
            next_task_index: 0,
            stop_requested: false,
            force_stop_requested: false,
        }
    }

    pub fn add_task(&mut self, task: VideoTask) {
        self.tasks.push(task);
    }

    pub fn next_pending_task(&mut self) -> Option<&mut VideoTask> {
        for i in self.next_task_index..self.tasks.len() {
            if self.tasks[i].status == TaskStatus::Pending {
                self.next_task_index = i;
                return Some(&mut self.tasks[i]);
            }
        }
        self.next_task_index = self.tasks.len();
        None
    }

    pub fn mark_as_running(&mut self, task_id: usize) {
        if let Some(task) = self.tasks.iter_mut().find(|t| t.id == task_id) {
            task.status = TaskStatus::Running;
        }
    }

    pub fn mark_as_completed(&mut self, task_id: usize) {
        if let Some(task) = self.tasks.iter_mut().find(|t| t.id == task_id) {
            task.status = TaskStatus::Completed;
            if self.next_task_index < self.tasks.len() 
                && self.tasks[self.next_task_index].id == task_id {
                self.next_task_index += 1;
            }
        }
    }

    pub fn mark_as_failed(&mut self, task_id: usize, err_msg: String) {
        if let Some(task) = self.tasks.iter_mut().find(|t| t.id == task_id) {
            task.status = TaskStatus::Failed(err_msg);
            if self.next_task_index < self.tasks.len() 
                && self.tasks[self.next_task_index].id == task_id {
                self.next_task_index += 1;
            }
        }
    }

    pub fn mark_as_cancelled(&mut self, task_id: usize) {
        if let Some(task) = self.tasks.iter_mut().find(|t| t.id == task_id) {
            task.status = TaskStatus::Cancelled;
            if self.next_task_index < self.tasks.len() 
                && self.tasks[self.next_task_index].id == task_id {
                self.next_task_index += 1;
            }
        }
    }

    pub fn clear_all_tasks(&mut self) {
        self.tasks.clear();
        self.next_task_index = 0;
        self.stop_requested = false;
        self.force_stop_requested = false;
    }

    pub fn remove_task(&mut self, task_id: usize) {
        let initial_len = self.tasks.len();
        self.tasks.retain(|task| task.id != task_id);
        if self.tasks.len() < initial_len {
            self.next_task_index = 0;
        }
    }

    pub fn request_stop(&mut self) {
        self.stop_requested = true;
    }

    pub fn request_force_stop(&mut self) {
        self.force_stop_requested = true;
    }

    pub fn clear_stop_request(&mut self) {
        self.stop_requested = false;
    }

    pub fn clear_force_stop(&mut self) {
        self.force_stop_requested = false;
    }

    pub fn is_stop_requested(&self) -> bool {
        self.stop_requested
    }

    pub fn is_force_stop_requested(&self) -> bool {
        self.force_stop_requested
    }

    pub fn update_pending_recipes(&mut self, new_recipe_path: PathBuf) {
        for task in &mut self.tasks {
            if task.status == TaskStatus::Pending {
                task.recipe_path = new_recipe_path.clone();
            }
        }
    }

    pub fn update_pending_output_dirs(&mut self, new_output_dir: PathBuf) {
        for task in &mut self.tasks {
            if task.status == TaskStatus::Pending {
                task.output_dir = new_output_dir.clone();
            }
        }
    }
}
