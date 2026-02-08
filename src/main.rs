mod ai;
mod game;
mod i18n;
mod storage;
mod ui;

use gtk4::prelude::*;
use gtk4::Application;

fn main() {
    let app = Application::builder()
        .application_id("io.github.laserlicht.TowerOops")
        .build();

    app.connect_activate(|app| {
        let res_dir = find_resources_dir();
        ui::app::build_ui(app, &res_dir);
    });

    app.run();
}

/// Locate the `resources/` directory.
fn find_resources_dir() -> String {
    let candidates = [
        // cargo run from project root
        std::env::current_dir().ok().map(|d| d.join("resources")),
        // next to executable
        std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|d| d.join("resources"))),
    ];

    for candidate in candidates.iter().flatten() {
        if candidate.is_dir() {
            return candidate.to_string_lossy().to_string();
        }
    }

    "resources".to_string()
}
