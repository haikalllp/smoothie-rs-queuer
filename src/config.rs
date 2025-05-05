use std::fs;
use std::path::{self, Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone)]
pub struct SmoothieConfig {
    pub executable_path: PathBuf,
    pub recipe_path: PathBuf, // Default recipe path found or selected by user
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigError {
    ExecutableNotFound,
    // Could add RecipeNotFound later if needed, but UI handles selection for now
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::ExecutableNotFound => {
                write!(f, "smoothie-rs executable not found automatically.")
            }
        }
    }
}

/// Attempts to find the smoothie-rs executable and default recipe automatically.
/// If executable isn't found automatically, returns Err(ExecutableNotFound).
pub fn find_smoothie_config_auto() -> Result<SmoothieConfig, ConfigError> {
    log::info!("Attempting to automatically find smoothie-rs configuration...");

    // 1. Find Executable (automatically)
    let executable_path = find_executable_auto().ok_or(ConfigError::ExecutableNotFound)?; // Return error if not found automatically
    log::info!(
        "Found smoothie-rs executable automatically at: {:?}",
        executable_path
    );

    // 2. Find Default Recipe (using found executable path)
    let recipe_path = find_default_recipe(&executable_path);
    log::info!("Using default recipe path: {:?}", recipe_path);

    Ok(SmoothieConfig {
        executable_path,
        recipe_path,
    })
}

/// Attempts to find config within a user-specified base directory.
pub fn find_smoothie_config_in_dir(base_dir: &Path) -> Result<SmoothieConfig, ConfigError> {
    log::info!(
        "Attempting to find smoothie-rs configuration in specified directory: {:?}",
        base_dir
    );
    let exe_path_in_dir = base_dir.join("bin").join("smoothie-rs.exe"); // Check specific structure
    if !exe_path_in_dir.is_file() {
        log::error!(
            "'smoothie-rs.exe' not found in specified directory's 'bin' subfolder: {:?}",
            base_dir
        );
        return Err(ConfigError::ExecutableNotFound); // Use same error type for simplicity
    }
    log::info!("Found smoothie-rs executable at: {:?}", exe_path_in_dir);

    // Find recipe relative to this specific structure
    let recipe_path_in_dir = base_dir.join("recipe.ini"); // Check root of selected folder first
    let final_recipe_path = if recipe_path_in_dir.is_file() {
        log::info!(
            "Found recipe.ini in specified directory: {:?}",
            recipe_path_in_dir
        );
        recipe_path_in_dir
    } else {
        // Fallback to checking relative to executable's parent (Smoothie folder)
        find_default_recipe(&exe_path_in_dir)
    };
    log::info!("Using recipe path: {:?}", final_recipe_path);

    Ok(SmoothieConfig {
        executable_path: exe_path_in_dir,
        recipe_path: final_recipe_path,
    })
}

/// Scans for .ini files in the base directory and a 'recipes' subdirectory.
pub fn find_recipe_files(base_dir: &Path) -> Vec<PathBuf> {
    log::debug!(
        "Scanning for recipe files in base directory: {:?}",
        base_dir
    );
    let mut recipes = Vec::new();

    // Check base directory
    if let Ok(entries) = fs::read_dir(base_dir) {
        for entry in entries.flatten() {
            // Use flatten to ignore read errors for individual entries
            let path = entry.path();
            if path.is_file() {
                if let Some(ext) = path::Path::extension(&path) {
                    if ext == "ini" {
                        if let Some(name) = path.file_name() {
                            if name != "encoding_presets.ini" {
                                log::debug!("Found recipe: {:?}", path);
                                recipes.push(path);
                            } else {
                                log::debug!("Ignoring excluded recipe file: {:?}", path);
                            }
                        }
                    }
                }
            }
        }
    } else {
        log::warn!("Could not read base directory for recipes: {:?}", base_dir);
    }

    // Check 'recipes' subdirectory
    let recipes_subdir = base_dir.join("recipes");
    if recipes_subdir.is_dir() {
        log::debug!(
            "Scanning for recipe files in subdirectory: {:?}",
            recipes_subdir
        );
        if let Ok(entries) = fs::read_dir(&recipes_subdir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    if let Some(ext) = path::Path::extension(&path) {
                        if ext == "ini" {
                            if let Some(name) = path.file_name() {
                                if name != "encoding_presets.ini" {
                                    log::debug!("Found recipe: {:?}", path);
                                    // Avoid duplicates if somehow present in both
                                    if !recipes.contains(&path) {
                                        recipes.push(path);
                                    }
                                } else {
                                    log::debug!("Ignoring excluded recipe file: {:?}", path);
                                }
                            }
                        }
                    }
                }
            }
        } else {
            log::warn!("Could not read recipes subdirectory: {:?}", recipes_subdir);
        }
    } else {
        log::debug!(
            "Recipes subdirectory not found or not a directory: {:?}",
            recipes_subdir
        );
    }

    // Sort for consistent order
    recipes.sort();

    if recipes.is_empty() {
        log::warn!(
            "No recipe (.ini) files found in {:?} or its 'recipes' subdirectory.",
            base_dir
        );
    }

    recipes
}

/// Tries to find the smoothie-rs executable automatically.
/// Order: PATH, then relative path `./Smoothie/bin/smoothie-rs.exe`.
fn find_executable_auto() -> Option<PathBuf> {
    // Try checking PATH first
    let command_name = if cfg!(windows) { "where" } else { "which" };
    let arg_name = "smoothie-rs";

    log::debug!("Running '{} {}' to check PATH", command_name, arg_name);
    if let Ok(output) = Command::new(command_name).arg(arg_name).output() {
        if output.status.success() {
            if let Ok(stdout) = String::from_utf8(output.stdout) {
                // 'where' can return multiple lines, 'which' usually one
                if let Some(first_path_str) = stdout.lines().next() {
                    let path = PathBuf::from(first_path_str.trim());
                    // Basic check if it's likely the Roaming path on Windows
                    let is_likely_roaming_path = cfg!(windows)
                        && path
                            .to_string_lossy()
                            .contains("AppData\\Roaming\\Smoothie\\bin");

                    if path.is_file() {
                        log::info!(
                            "Found '{}' via {} command: {:?}",
                            arg_name,
                            command_name,
                            path
                        );
                        return Some(path);
                    } else if is_likely_roaming_path {
                        // Sometimes 'where' might list the directory containing it? Or symlink?
                        // If it looks like the right dir structure, try appending the exe name
                        let potential_path = path.join("smoothie-rs.exe");
                        if potential_path.is_file() {
                            log::info!(
                                "Found '{}' via {} command (adjusted): {:?}",
                                arg_name,
                                command_name,
                                potential_path
                            );
                            return Some(potential_path);
                        } else {
                            log::warn!(
                                "'{} {}' succeeded but path '{}' is not a file, and adjusted path {:?} not found.",
                                command_name,
                                arg_name,
                                first_path_str,
                                potential_path
                            );
                        }
                    } else {
                        log::warn!(
                            "'{} {}' succeeded but path '{}' is not a file.",
                            command_name,
                            arg_name,
                            first_path_str
                        );
                    }
                }
            }
        } else {
            log::debug!(
                "'{} {}' command failed or returned non-zero status.",
                command_name,
                arg_name
            );
        }
    } else {
        log::warn!("Failed to execute '{} {}' command.", command_name, arg_name);
    }

    // If not found in PATH, check relative path
    let relative_path = PathBuf::from("./Smoothie/bin/smoothie-rs.exe");
    log::debug!("Checking relative path: {:?}", relative_path);
    if relative_path.is_file() {
        log::info!("Found smoothie-rs at relative path: {:?}", relative_path);
        return Some(relative_path);
    }

    log::warn!(
        "smoothie-rs executable not found automatically in PATH or at relative path './Smoothie/bin/smoothie-rs.exe'"
    );
    None
}

/// Finds the default recipe path based on standard locations relative to the executable.
fn find_default_recipe(executable_path: &Path) -> PathBuf {
    // 1. Check relative `./Smoothie/recipe.ini` (as per user feedback)
    let relative_recipe = PathBuf::from("./Smoothie/recipe.ini");
    if relative_recipe.is_file() {
        log::debug!(
            "Found default recipe at relative path: {:?}",
            relative_recipe
        );
        return relative_recipe;
    }
    log::debug!(
        "Default recipe not found at relative path: {:?}",
        relative_recipe
    );

    // 2. Check relative to executable's parent directory
    // e.g., if exe is /path/to/Smoothie/bin/smoothie-rs.exe, check /path/to/Smoothie/recipe.ini
    if let Some(exe_dir) = executable_path.parent() {
        // bin directory
        if let Some(base_dir) = exe_dir.parent() {
            // Smoothie directory
            let recipe_in_base = base_dir.join("recipe.ini");
            if recipe_in_base.is_file() {
                log::debug!(
                    "Found default recipe relative to executable's base directory: {:?}",
                    recipe_in_base
                );
                return recipe_in_base;
            }
            log::debug!(
                "Default recipe not found relative to executable's base directory: {:?}",
                recipe_in_base
            );
        }
    }

    // 3. Fallback
    let fallback_path = PathBuf::from("recipe.ini");
    log::warn!(
        "Default recipe not found in standard locations. Falling back to: {:?}",
        fallback_path
    );
    fallback_path
}
