use eframe::egui;
use egui_file::FileDialog;
use std::path::PathBuf;
use syn::visit_mut::VisitMut;
use syn::{parse_file, visit::Visit, File as SynFile, Ident, Pat, PatType, Type};
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
    opened_file: Option<PathBuf>,
    open_file_dialog: Option<FileDialog>,
    variables: Vec<Variable>,
    syntax_set: SyntaxSet,
    theme: Theme,
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

    fn parse_variables(&mut self) {
        if let Ok(ast) = parse_rust_code(&self.code) {
            let mut visitor = VariableVisitor::new();
            visitor.visit_file(&ast);
            self.variables = visitor.variables;
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
                            self.parse_variables();
                        }
                    }
                }
            }

            // Display current file path
            if let Some(path) = &self.opened_file {
                ui.label(format!("Current File: {:?}", path.display()));
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
                        .font(egui::TextStyle::Monospace)
                        .code_editor()
                        .desired_rows(10)
                        .lock_focus(true)
                        .desired_width(f32::INFINITY)
                        .layouter(&mut layouter),
                );
            });

            ui.separator();

            // Show Variables Panel below the code editor
            ui.vertical(|ui| {
                if self.variables.is_empty() {
                    ui.label("No variables found.");
                } else {
                    for variable in &mut self.variables {
                        ui.label(format!(
                            "Variable: {} of type {}",
                            variable.name, variable.var_type
                        ));
                        match variable.var_type.as_str() {
                            "i32" => {
                                ui.add(
                                    egui::DragValue::new(&mut variable.value)
                                        .speed(1)
                                        .clamp_range(0..=100),
                                );
                            }
                            "f32" => {
                                ui.add(
                                    egui::DragValue::new(&mut variable.value)
                                        .speed(0.1)
                                        .clamp_range(0.0..=100.0),
                                );
                            }
                            _ => {
                                ui.label("Unsupported type for input");
                            }
                        }
                    }
                }
            });
        });
    }
}

fn parse_rust_code(code: &str) -> Result<SynFile, syn::Error> {
    parse_file(code)
}

struct Variable {
    name: String,
    var_type: String,
    value: f64, // Store value as f64 to allow both i32 and f32 inputs
}

struct VariableVisitor {
    variables: Vec<Variable>,
}

impl VariableVisitor {
    fn new() -> Self {
        Self {
            variables: Vec::new(),
        }
    }
}

impl<'ast> Visit<'ast> for VariableVisitor {
    fn visit_local(&mut self, local: &'ast syn::Local) {
        if let Pat::Type(PatType { pat, ty, .. }) = &local.pat {
            if let Pat::Ident(ident) = &**pat {
                let var_name = ident.ident.to_string();
                let var_type = extract_type(&**ty);
                self.variables.push(Variable {
                    name: var_name,
                    var_type,
                    value: 0.0, // Default value
                });
            }
        }
        syn::visit::visit_local(self, local);
    }
}

fn extract_type(ty: &Type) -> String {
    match ty {
        Type::Path(ref typepath) => {
            if let Some(ident) = typepath.path.get_ident() {
                ident.to_string()
            } else {
                "Unknown".to_string()
            }
        }
        _ => "Unsupported".to_string(),
    }
}

fn rename_variable(ast: &mut SynFile, old_name: &str, new_name: &str) {
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
