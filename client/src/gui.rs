/// Run the launcher with an error message from a failed connection.
pub fn run_with_error(error: String) -> Option<String> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([420.0, 520.0]),
        ..Default::default()
    };
    let _ = eframe::run_native(
        "Seihai",
        options,
        Box::new(move |_cc| Ok(Box::new(Launcher::with_error(Some(error))))),
    );
    Settings::load().last_server
}