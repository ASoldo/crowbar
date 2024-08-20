use std::fs;
use syn::{parse_file, visit_mut::VisitMut, Expr, File, Ident};

fn main() {
    // Read a sample Rust source file
    let code = fs::read_to_string("sample.rs").expect("Failed to read file");

    // Parse the code into an AST
    match parse_rust_code(&code) {
        Ok(mut ast) => {
            println!("Successfully parsed AST!");

            // Rename a variable in the AST
            let old_name = "x";
            let new_name = "y";
            rename_variable(&mut ast, old_name, new_name);

            // Pretty print the modified AST
            print_ast(&ast);
        }
        Err(e) => eprintln!("Failed to parse code: {}", e),
    }
}

fn parse_rust_code(code: &str) -> Result<File, syn::Error> {
    parse_file(code)
}

fn print_ast(ast: &File) {
    let formatted_code = prettyplease::unparse(ast);
    println!("{}", formatted_code);
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

    // Override to handle expressions, skipping macro expansions
    fn visit_expr_mut(&mut self, expr: &mut Expr) {
        match expr {
            Expr::Macro(_) => {
                // If it's a macro, you can choose to skip or handle it
            }
            _ => {
                // For other expressions, proceed with the default behavior
                syn::visit_mut::visit_expr_mut(self, expr);
            }
        }
    }
}
