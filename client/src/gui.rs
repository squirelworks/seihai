/// Pick a level file. Returns the chosen path, if any.
pub fn pick_level(start: &std::path::Path) -> Option<std::path::PathBuf> {
    use std::sync::{Arc, Mutex};
    let chosen = Arc::new(Mutex::new(None));
    let chosen2 = chosen.clone();
    struct Picker {
        dir: std::path::PathBuf,
        entries: Vec<String>,
        out: Arc<Mutex<Option<std::path::PathBuf>>>,
    }
    impl Picker {
        fn refresh(&mut self) {
            self.entries.clear();
            if let Ok(rd) = std::fs::read_dir(&self.dir) {
                let mut dirs = Vec::new();
                let mut files = Vec::new();
                for e in rd.flatten() {
                    let name = e.file_name().to_string_lossy().into_owned();
                    if e.path().is_dir() {
                        dirs.push(format!("{name}/"));
                    } else if name.ends_with(".nw") || name.ends_with(".graal") {
                        files.push(name);
                    }
                }
                dirs.sort();
                files.sort();
                self.entries.extend(dirs);
                self.entries.extend(files);
            }
        }
    }
    impl eframe::App for Picker {
        fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
            egui::CentralPanel::default().show(ctx, |ui| {
                ui.heading("Pick a level");
                ui.label(self.dir.display().to_string());
                if ui.button("..").clicked() {
                    if let Some(p) = self.dir.parent() {
                        self.dir = p.to_path_buf();
                        self.refresh();
                    }
                }
                for name in self.entries.clone() {
                    if ui.button(&name).clicked() {
                        let path = self.dir.join(name.trim_end_matches('/'));
                        if path.is_dir() {
                            self.dir = path;
                            self.refresh();
                        } else {
                            *self.out.lock().unwrap() = Some(path);
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    }
                }
            });
        }
    }
    let mut picker = Picker {
        dir: start.to_path_buf(),
        entries: Vec::new(),
        out: chosen2,
    };
    picker.refresh();
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([400.0, 480.0]),
        ..Default::default()
    };
    let _ = eframe::run_native("Pick a level", options, Box::new(|_| Ok(Box::new(picker))));
    chosen.lock().unwrap().take()
}