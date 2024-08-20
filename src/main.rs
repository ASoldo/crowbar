use eframe::egui;
use egui_file::FileDialog;
use std::path::PathBuf;
use syn::{parse_file, visit_mut::VisitMut, File, Ident};
use syntect::easy::HighlightLines;
use syntect::highlighting::{Theme, ThemeSet};
use syntect::parsing::SyntaxSet;

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Rust Code Editor with AST Manipulation",
        options,
        Box::new(|_cc| Ok(Box::new(MyApp::new()))),
    )
}

#[derive(Default)]
struct MyApp {
    code: String,
    old_name: String,
    new_name: String,
    syntax_set: SyntaxSet,
    theme: Theme,
    opened_file: Option<PathBuf>,
    open_file_dialog: Option<FileDialog>,
}

impl MyApp {
    fn new() -> Self {
        let syntax_set = SyntaxSet::load_defaults_newlines();
        let theme = ThemeSet::load_defaults().themes["base16-ocean.dark"].clone();

        Self {
            syntax_set,
            theme,
            ..Default::default()
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Rust Code Editor");

            // Load File Button
            if ui.button("Load File").clicked() {
                let mut dialog = FileDialog::open_file(self.opened_file.clone());
                dialog.open();
                self.open_file_dialog = Some(dialog);
            }

            // Handle File Dialog
            if let Some(dialog) = &mut self.open_file_dialog {
                if dialog.show(ctx).selected() {
                    if let Some(file) = dialog.path() {
                        self.opened_file = Some(file.to_path_buf());
                        if let Ok(content) = std::fs::read_to_string(&file) {
                            self.code = content;
                        }
                    }
                }
            }

            // Display current file path
            if let Some(path) = &self.opened_file {
                ui.label(format!("Current File: {:?}", path.display()));
            }

            // Inputs for renaming variables
            ui.horizontal(|ui| {
                ui.label("Old Name:");
                ui.text_edit_singleline(&mut self.old_name);
                ui.label("New Name:");
                ui.text_edit_singleline(&mut self.new_name);
            });

            if ui.button("Rename Variable").clicked() {
                if let Ok(mut ast) = parse_rust_code(&self.code) {
                    rename_variable(&mut ast, &self.old_name, &self.new_name);
                    self.code = prettyplease::unparse(&ast);
                } else {
                    ui.label("Failed to parse code.");
                }
            }

            // Syntax highlighting for the code
            let mut layouter = |ui: &egui::Ui, string: &str, wrap_width: f32| {
                let mut h = HighlightLines::new(
                    self.syntax_set.find_syntax_by_extension("rs").unwrap(),
                    &self.theme,
                );
                let ranges: Vec<(syntect::highlighting::Style, &str)> =
                    h.highlight(string, &self.syntax_set);
                let mut job = egui::text::LayoutJob::default();
                for (style, text) in ranges {
                    let color = egui::Color32::from_rgb(
                        style.foreground.r,
                        style.foreground.g,
                        style.foreground.b,
                    );
                    job.append(
                        text,
                        0.0,
                        egui::TextFormat {
                            color,
                            ..Default::default()
                        },
                    );
                }
                job.wrap.max_width = wrap_width;
                ui.fonts(|f| f.layout_job(job))
            };

            // Code Editor with syntax highlighting
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.add(
                    egui::TextEdit::multiline(&mut self.code)
                        .font(egui::TextStyle::Monospace) // for cursor height
                        .code_editor()
                        .desired_rows(10)
                        .lock_focus(true)
                        .desired_width(f32::INFINITY)
                        .layouter(&mut layouter), // Pass the closure as mutable
                );
            });
        });
    }
}

fn parse_rust_code(code: &str) -> Result<File, syn::Error> {
    parse_file(code)
}

// Function to rename a variable within the AST
fn rename_variable(ast: &mut File, old_name: &str, new_name: &str) {
    let mut renamer = Renamer {
        old_name: Ident::new(old_name, proc_macro2::Span::call_site()),
        new_name: Ident::new(new_name, proc_macro2::Span::call_site()),
    };
    renamer.visit_file_mut(ast);
}

struct Renamer {
    old_name: Ident,
    new_name: Ident,
}

impl VisitMut for Renamer {
    fn visit_ident_mut(&mut self, ident: &mut Ident) {
        if *ident == self.old_name {
            *ident = self.new_name.clone();
        }
    }
}
