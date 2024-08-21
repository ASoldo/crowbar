use eframe::egui;
use egui_file::FileDialog;
use std::path::PathBuf;
use std::process::Command;
use syn::{parse_file, visit::Visit, File as SynFile, Pat, PatType, Type};
use syntect::easy::HighlightLines;
use syntect::highlighting::{Theme, ThemeSet};
use syntect::parsing::SyntaxSet;

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Crowbar",
        options,
        Box::new(|_cc| Ok(Box::new(MyApp::new()))),
    )
}

enum VariableValue {
    Int(i64),
    Float(f64),
    Bool(bool),
    Str(String),
    Unknown,
}

#[derive(Default)]
struct MyApp {
    code: String,
    opened_file: Option<PathBuf>,
    open_file_dialog: Option<FileDialog>,
    variables: Vec<Variable>,
    syntax_set: SyntaxSet,
    theme: Theme,
    output: String,
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

    fn update_code_with_variables(&mut self) {
        for variable in &self.variables {
            let search_patterns = vec![
                format!("let {}: {} = ", variable.name, variable.var_type),
                format!("let mut {}: {} = ", variable.name, variable.var_type),
            ];

            let mut code_replaced = String::new();
            let mut last_pos = 0;

            for search_str in search_patterns {
                while let Some(pos) = self.code[last_pos..].find(&search_str) {
                    let actual_pos = last_pos + pos;
                    let end_pos = self.code[actual_pos..].find(';').unwrap() + actual_pos + 1;
                    let new_value_str = match &variable.value {
                        VariableValue::Int(val) => format!("{};", val),
                        VariableValue::Float(val) => format!("{};", val),
                        VariableValue::Bool(val) => format!("{};", val),
                        VariableValue::Str(val) => {
                            if variable.var_type == "String" {
                                format!("\"{}\".to_string();", val)
                            } else {
                                format!("\"{}\";", val)
                            }
                        }
                        VariableValue::Unknown => continue,
                    };
                    code_replaced.push_str(&self.code[last_pos..actual_pos + search_str.len()]);
                    code_replaced.push_str(&new_value_str);
                    last_pos = end_pos;
                }
            }

            code_replaced.push_str(&self.code[last_pos..]);
            self.code = code_replaced;
        }
    }

    fn run_code(&mut self) {
        let temp_file_path = "temp_code.rs";
        if let Err(e) = std::fs::write(temp_file_path, &self.code) {
            self.output = format!("Failed to write code to file: {}", e);
            return;
        }

        let output = Command::new("rustc")
            .arg(temp_file_path)
            .arg("-o")
            .arg("temp_executable")
            .output();

        match output {
            Ok(output) => {
                if !output.stderr.is_empty() {
                    self.output = format!(
                        "Compilation error:\n{}",
                        String::from_utf8_lossy(&output.stderr)
                    );
                } else {
                    match Command::new("./temp_executable").output() {
                        Ok(run_output) => {
                            self.output = String::from_utf8_lossy(&run_output.stdout).to_string();
                        }
                        Err(e) => {
                            self.output = format!("Failed to run the code: {}", e);
                        }
                    }
                }
            }
            Err(e) => {
                self.output = format!("Failed to compile the code: {}", e);
            }
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Crowbar");

            ui.separator();

            // Align "Load File" and "Run Code" buttons on the same line
            ui.horizontal(|ui| {
                if ui.button("Load File").clicked() {
                    let mut dialog = FileDialog::open_file(self.opened_file.clone());
                    dialog.open();
                    self.open_file_dialog = Some(dialog);
                }

                if ui.button("Run Code").clicked() {
                    self.update_code_with_variables();
                    self.run_code();
                }
            });

            ui.separator();
            ui.add_space(10.0);

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

            if let Some(path) = &self.opened_file {
                ui.label(format!("Current File: {:?}", path.display()));
            }

            let line_count = self.code.lines().count();
            let line_numbers = (1..=line_count)
                .map(|i| format!("{}\n", i))
                .collect::<String>();

            // Make the ScrollArea bigger
            egui::ScrollArea::vertical()
                .id_source("code_scroll_area")
                .max_height(800.0) // Increase the height of the scroll area
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        let mut line_number_layouter =
                            |ui: &egui::Ui, string: &str, wrap_width: f32| {
                                let mut job = egui::text::LayoutJob::default();
                                job.append(
                                    string,
                                    0.0,
                                    egui::TextFormat {
                                        font_id: egui::TextStyle::Monospace.resolve(ui.style()),
                                        color: egui::Color32::GRAY, // Make the line numbers a different color if desired
                                        line_height: Some(16.0),
                                        ..Default::default()
                                    },
                                );
                                job.wrap.max_width = wrap_width;
                                ui.fonts(|f| f.layout_job(job))
                            };

                        ui.add(
                            egui::TextEdit::multiline(&mut line_numbers.clone())
                                .font(egui::TextStyle::Monospace)
                                .code_editor()
                                .lock_focus(true)
                                .interactive(false)
                                .desired_width(30.0)
                                .desired_rows(30)
                                .layouter(&mut line_number_layouter),
                        );

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

                        ui.add(
                            egui::TextEdit::multiline(&mut self.code)
                                .font(egui::TextStyle::Monospace)
                                .code_editor()
                                .lock_focus(true)
                                .desired_rows(30) // Set the height by the number of rows
                                .desired_width(f32::INFINITY)
                                .layouter(&mut layouter),
                        );
                    });
                });

            ui.add_space(10.0); // Add space between code editor and separator
            ui.separator();

            egui::ScrollArea::vertical()
                .id_source("variables_scroll_area")
                .max_height(150.0) // Limit height for scrolling
                .show(ui, |ui| {
                    ui.vertical(|ui| {
                        if self.variables.is_empty() {
                            ui.label("No variables found.");
                        } else {
                            for variable in &mut self.variables {
                                ui.label(format!(
                                    "Variable: {} of type {}",
                                    variable.name, variable.var_type
                                ));
                                match &mut variable.value {
                                    VariableValue::Int(val) => {
                                        ui.add(
                                            egui::DragValue::new(val)
                                                .speed(1)
                                                .clamp_range(i64::MIN..=i64::MAX),
                                        );
                                    }
                                    VariableValue::Float(val) => {
                                        ui.add(
                                            egui::DragValue::new(val)
                                                .speed(0.1)
                                                .clamp_range(f64::MIN..=f64::MAX),
                                        );
                                    }
                                    VariableValue::Bool(val) => {
                                        ui.checkbox(val, "Value");
                                    }
                                    VariableValue::Str(val) => {
                                        ui.text_edit_singleline(val);
                                    }
                                    VariableValue::Unknown => {
                                        ui.label("Unsupported type for input");
                                    }
                                }
                            }
                        }
                    });
                });

            ui.separator();

            egui::ScrollArea::vertical()
                .id_source("output_scroll_area")
                .max_height(150.0) // Limit height for scrolling
                .show(ui, |ui| {
                    ui.collapsing("Output", |ui| {
                        ui.with_layout(
                            egui::Layout::top_down(egui::Align::Min).with_main_wrap(false),
                            |ui| {
                                ui.label(&self.output);
                            },
                        );
                    });
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
    value: VariableValue,
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

                // Initialize value based on type
                let value = match var_type.as_str() {
                    "i32" | "i64" => VariableValue::Int(0),
                    "f32" | "f64" => VariableValue::Float(0.0),
                    "bool" => VariableValue::Bool(false),
                    "&str" | "String" => VariableValue::Str(String::new()), // Handle string slices and Strings
                    _ => VariableValue::Unknown,
                };

                // Add variable to the list
                self.variables.push(Variable {
                    name: var_name,
                    var_type,
                    value,
                });

                // Handle initialization if present
                // Handle initialization if present
                if let Some(local_init) = &local.init {
                    if let Some(variable) = self.variables.last_mut() {
                        match &*local_init.expr {
                            syn::Expr::Lit(syn::ExprLit {
                                lit: syn::Lit::Str(lit_str),
                                ..
                            }) => {
                                variable.value = VariableValue::Str(lit_str.value());
                            }
                            syn::Expr::Call(syn::ExprCall { func, args, .. }) => {
                                if let syn::Expr::Path(ref expr_path) = **func {
                                    if expr_path.path.is_ident("String::from") {
                                        if let Some(syn::Expr::Lit(syn::ExprLit {
                                            lit: syn::Lit::Str(lit_str),
                                            ..
                                        })) = args.first()
                                        {
                                            variable.value = VariableValue::Str(lit_str.value());
                                        }
                                    }
                                }
                            }
                            syn::Expr::Lit(syn::ExprLit {
                                lit: syn::Lit::Int(lit_int),
                                ..
                            }) => {
                                variable.value =
                                    VariableValue::Int(lit_int.base10_parse().unwrap_or(0));
                            }
                            syn::Expr::Lit(syn::ExprLit {
                                lit: syn::Lit::Float(lit_float),
                                ..
                            }) => {
                                variable.value =
                                    VariableValue::Float(lit_float.base10_parse().unwrap_or(0.0));
                            }
                            syn::Expr::Lit(syn::ExprLit {
                                lit: syn::Lit::Bool(lit_bool),
                                ..
                            }) => {
                                variable.value = VariableValue::Bool(lit_bool.value);
                            }
                            _ => {}
                        }
                    }
                }
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
        Type::Reference(_) => "&str".to_string(), // Handle string slices
        _ => "Unsupported".to_string(),
    }
}
