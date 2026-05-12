use eframe::egui;
use std::fs;
use std::path::Path;
use gpupatch_core::patch_pe;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([600.0, 320.0]),
        ..Default::default()
    };
    eframe::run_native(
        "GPUPatch GUI",
        options,
        Box::new(|_cc| Ok(Box::<CompanionApp>::default())),
    )
}

struct CompanionApp {
    input_path: String,
    output_path: String,
    disable: bool,
    status_msg: String,
    is_error: bool,
}

impl Default for CompanionApp {
    fn default() -> Self {
        Self {
            input_path: "".to_owned(),
            output_path: "".to_owned(),
            disable: false,
            status_msg: "Select an executable to begin.".to_owned(),
            is_error: false,
        }
    }
}

impl CompanionApp {
    fn get_inferred_output(&self) -> String {
        if self.input_path.trim().is_empty() {
            return String::new();
        }
        let path = Path::new(&self.input_path);
        let file_stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("output");
        let extension = path.extension().and_then(|s| s.to_str()).map(|e| format!(".{}", e)).unwrap_or_default();
        if let Some(parent) = path.parent() {
            parent.join(format!("{}_patched{}", file_stem, extension)).to_string_lossy().into_owned()
        } else {
            format!("{}_patched{}", file_stem, extension)
        }
    }

    fn run_patch(&mut self) {
        if self.input_path.trim().is_empty() {
            self.status_msg = "❌ Error: Input path is empty!".to_owned();
            self.is_error = true;
            return;
        }

        let actual_output = if self.output_path.trim().is_empty() {
            self.get_inferred_output()
        } else {
            self.output_path.clone()
        };

        let bytes = match fs::read(&self.input_path) {
            Ok(b) => b,
            Err(e) => {
                self.status_msg = format!("❌ Failed to read file: {}", e);
                self.is_error = true;
                return;
            }
        };

        let filename = Path::new(&self.input_path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("output.exe");

        match patch_pe(&bytes, self.disable, filename) {
            Ok(patched) => {
                match fs::write(&actual_output, patched) {
                    Ok(_) => {
                        self.status_msg = format!("✨ Success: Written to {}", actual_output);
                        self.is_error = false;
                    }
                    Err(e) => {
                        self.status_msg = format!("❌ Failed to save: {}", e);
                        self.is_error = true;
                    }
                }
            }
            Err(e) => {
                self.status_msg = format!("❌ Patch Failed: {}", e);
                self.is_error = true;
            }
        }
    }
}

impl eframe::App for CompanionApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.heading("🚀 GPUPatch GUI");
            });
            
            ui.separator();
            ui.add_space(10.0);

            egui::Grid::new("inputs_grid")
                .num_columns(3)
                .spacing([10.0, 10.0])
                .min_col_width(100.0)
                .show(ui, |ui| {
                    ui.label("Input Path:");
                    ui.add(egui::TextEdit::singleline(&mut self.input_path)
                        .hint_text("/path/to/target.exe")
                        .desired_width(300.0));
                    
                    if ui.button("📁 Browse...").clicked() {
                        if let Some(path) = rfd::FileDialog::new()
                            .add_filter("Executable", &["exe"])
                            .pick_file() {
                            self.input_path = path.to_string_lossy().to_string();
                        }
                    }
                    ui.end_row();

                    ui.label("Output Path:");
                    ui.vertical(|ui| {
                        ui.add(egui::TextEdit::singleline(&mut self.output_path)
                            .hint_text("Leave empty to append _patched")
                            .desired_width(300.0));
                        
                        if self.output_path.trim().is_empty() && !self.input_path.trim().is_empty() {
                            let preview = self.get_inferred_output();
                            ui.add(egui::Label::new(egui::RichText::new(format!("→ {}", preview)).weak().size(10.0)));
                        }
                    });
                    
                    if ui.button("📁 Browse...").clicked() {
                        if let Some(path) = rfd::FileDialog::new()
                            .add_filter("Executable", &["exe"])
                            .save_file() {
                            self.output_path = path.to_string_lossy().to_string();
                        }
                    }
                    ui.end_row();
                });

            ui.add_space(10.0);
            ui.checkbox(&mut self.disable, "Disable Patch (Remove enforcement)");
            
            ui.add_space(20.0);
            
            ui.vertical_centered(|ui| {
                if ui.add_sized([120.0, 30.0], egui::Button::new("⚡ Start Patch")).clicked() {
                    self.run_patch();
                }
            });

            ui.add_space(20.0);
            ui.separator();
            ui.add_space(5.0);

            let color = if self.is_error {
                egui::Color32::LIGHT_RED
            } else if self.status_msg.starts_with("✨") {
                egui::Color32::GREEN
            } else {
                ui.visuals().text_color()
            };
            
            ui.colored_label(color, &self.status_msg);
        });
    }
}
