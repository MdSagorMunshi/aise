use eframe::egui;
use std::fs::File;
use std::io::Read;
use aise_core::sponge::{aise_hash, aise_xof};
use rfd::FileDialog;

#[derive(PartialEq)]
enum InputMode {
    Text,
    File,
}

#[derive(PartialEq)]
enum HashMode {
    Hash,
    Xof,
}

struct AiseApp {
    input_mode: InputMode,
    hash_mode: HashMode,
    text_input: String,
    file_path: Option<String>,
    output_len: usize,
    output_hex: String,
    status_msg: String,
}

impl Default for AiseApp {
    fn default() -> Self {
        Self {
            input_mode: InputMode::Text,
            hash_mode: HashMode::Hash,
            text_input: String::new(),
            file_path: None,
            output_len: 64,
            output_hex: String::new(),
            status_msg: String::from("Ready."),
        }
    }
}

impl eframe::App for AiseApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("AEGIS-Ω (AISE) Toolkit");
            ui.separator();

            ui.horizontal(|ui| {
                ui.label("Input Mode:");
                ui.radio_value(&mut self.input_mode, InputMode::Text, "Text");
                ui.radio_value(&mut self.input_mode, InputMode::File, "File");
            });

            ui.add_space(10.0);

            if self.input_mode == InputMode::Text {
                ui.label("Text Input:");
                ui.text_edit_multiline(&mut self.text_input);
            } else {
                ui.horizontal(|ui| {
                    if ui.button("Select File").clicked() {
                        if let Some(path) = FileDialog::new().pick_file() {
                            self.file_path = Some(path.display().to_string());
                        }
                    }
                    if let Some(ref path) = self.file_path {
                        ui.label(path);
                    } else {
                        ui.label("No file selected.");
                    }
                });
            }

            ui.add_space(10.0);

            ui.horizontal(|ui| {
                ui.label("Hash Mode:");
                ui.radio_value(&mut self.hash_mode, HashMode::Hash, "AISE-HASH");
                ui.radio_value(&mut self.hash_mode, HashMode::Xof, "AISE-XOF");
            });

            ui.horizontal(|ui| {
                ui.label("Output Length (bytes):");
                ui.add(egui::DragValue::new(&mut self.output_len).speed(1));
            });

            ui.add_space(10.0);

            if ui.button("EXECUTE").clicked() {
                self.execute_hash();
            }

            ui.add_space(10.0);
            ui.label(&self.status_msg);
            
            ui.separator();
            ui.label("Output Hex:");
            
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.add(
                    egui::TextEdit::multiline(&mut self.output_hex)
                        .font(egui::TextStyle::Monospace)
                        .desired_rows(10)
                        .desired_width(f32::INFINITY)
                        .interactive(true), // allow copying
                );
            });
        });
    }
}

impl AiseApp {
    fn execute_hash(&mut self) {
        let mut data = Vec::new();

        if self.input_mode == InputMode::Text {
            data = self.text_input.as_bytes().to_vec();
        } else {
            if let Some(ref path) = self.file_path {
                match File::open(path) {
                    Ok(mut file) => {
                        if let Err(e) = file.read_to_end(&mut data) {
                            self.status_msg = format!("Error reading file: {}", e);
                            return;
                        }
                    }
                    Err(e) => {
                        self.status_msg = format!("Error opening file: {}", e);
                        return;
                    }
                }
            } else {
                self.status_msg = "Please select a file first.".to_string();
                return;
            }
        }

        self.status_msg = "Hashing...".to_string();

        let result = if self.hash_mode == HashMode::Hash {
            aise_hash(&data, self.output_len)
        } else {
            aise_xof(&data, self.output_len)
        };

        self.output_hex = result.iter().map(|b| format!("{:02x}", b)).collect::<String>();
        self.status_msg = "Success.".to_string();
    }
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([600.0, 500.0]),
        ..Default::default()
    };
    eframe::run_native(
        "AEGIS-Ω",
        options,
        Box::new(|cc| {
            cc.egui_ctx.set_visuals(egui::Visuals::dark()); // Omega-level dark mode
            Box::new(AiseApp::default()) as Box<dyn eframe::App>
        }),
    )
}
