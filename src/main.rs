use config::ConfigError;
use eframe::egui;
use std::sync::Arc;
use ui::SmoothieQueueApp;

mod config;
mod queue;
mod ui;
mod worker;

fn main() {
    env_logger::init();
    log::info!("Starting Smoothie Queuer application");

    // --- Find Configuration ---
    let initial_config: Option<config::SmoothieConfig> = match config::find_smoothie_config_auto() {
        Ok(cfg) => {
            log::info!("Automatic configuration successful.");
            Some(cfg)
        }
        Err(ConfigError::ExecutableNotFound) => {
            log::warn!(
                "Automatic configuration failed: {}",
                ConfigError::ExecutableNotFound
            );
            None
        }
    };
    // --- Initial Config Attempt Finished ---

    // Try loading embedded PNG first
    let icon_png = include_bytes!("../assets/icon.png");
    let icon = image::load_from_memory(icon_png)
        .map_err(|e| log::warn!("Failed to load PNG icon: {}", e))
        .ok()
        .and_then(|image| {
            let image = image.to_rgba8();
            let (width, height) = image.dimensions();
            if width == height && (width == 32 || width == 64 || width == 128) {
                let rgba = image.into_raw();
                Some(egui::IconData {
                    rgba,
                    width,
                    height,
                })
            } else {
                log::warn!("Icon dimensions must be square (32x32, 64x64 or 128x128)");
                None
            }
        });

    // Fallback to ICO if PNG fails
    let icon = if icon.is_none() {
        log::info!("Trying ICO fallback");
        include_bytes!("../assets/icon.ico")
            .get(..)
            .and_then(|data| image::load_from_memory(data).ok())
            .map(|image| {
                let image = image.to_rgba8();
                let (width, height) = image.dimensions();
                let rgba = image.into_raw();
                egui::IconData {
                    rgba,
                    width,
                    height,
                }
            })
    } else {
        icon
    };

    let viewport_builder = egui::ViewportBuilder::default()
        .with_inner_size([600.0, 400.0]);
    
    let viewport_builder = if let Some(icon_data) = icon {
        viewport_builder.with_icon(Arc::new(icon_data))
    } else {
        viewport_builder
    };
    
    let options = eframe::NativeOptions {
        viewport: viewport_builder,
        ..Default::default()
    };

    if let Some(ref icon) = options.viewport.icon {
        log::info!("Successfully loaded application icon ({}x{})", icon.width, icon.height);
    } else {
        log::warn!("Could not load any application icon - using default");
    }

    eframe::run_native(
        "Smoothie Queuer",
        options,
        Box::new(move |cc| Box::new(SmoothieQueueApp::new(cc, initial_config))),
    )
    .expect("Failed to run eframe application");
}
